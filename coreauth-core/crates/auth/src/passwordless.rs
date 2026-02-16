//! Passwordless authentication service
//!
//! Provides magic link and OTP authentication for headless IAM scenarios
//! where customers build their own UIs.

use crate::error::{AuthError, Result};
use crate::jwt::JwtService;
use crate::email::EmailService;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;

use ciam_models::{
    PasswordlessToken, PasswordlessTokenType, PasswordlessStartRequest, PasswordlessStartResponse,
    PasswordlessVerifyRequest, PasswordlessVerifyResponse, PasswordlessUserInfo,
    TenantRateLimit, UpdateRateLimitRequest, RateLimitsResponse,
};

/// Passwordless authentication service
pub struct PasswordlessService {
    pool: PgPool,
    jwt_service: JwtService,
    email_service: Option<EmailService>,
    base_url: String,
}

impl PasswordlessService {
    pub fn new(
        pool: PgPool,
        jwt_service: JwtService,
        email_service: Option<EmailService>,
        base_url: String,
    ) -> Self {
        Self {
            pool,
            jwt_service,
            email_service,
            base_url,
        }
    }

    /// Start passwordless authentication flow
    pub async fn start(
        &self,
        tenant_id: Uuid,
        request: PasswordlessStartRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<PasswordlessStartResponse> {
        // Check rate limits
        self.check_rate_limit(tenant_id, "passwordless", &ip_address).await?;

        // Generate token based on type
        let (token, token_hash) = match request.token_type {
            PasswordlessTokenType::MagicLink => {
                let token = generate_secure_token(64);
                let hash = hash_token(&token);
                (token, hash)
            }
            PasswordlessTokenType::Otp => {
                let code = generate_otp_code();
                let hash = hash_token(&code);
                (code, hash)
            }
        };

        // Check if user exists (via tenant membership or default tenant)
        let user_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT u.id FROM users u
            LEFT JOIN tenant_members tm ON tm.user_id = u.id
            WHERE u.email = $1
              AND (tm.tenant_id = $2 OR u.default_tenant_id = $2)
            "#
        )
        .bind(&request.email)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        // Set expiration based on token type
        let expires_at = match request.token_type {
            PasswordlessTokenType::MagicLink => Utc::now() + Duration::minutes(15),
            PasswordlessTokenType::Otp => Utc::now() + Duration::minutes(10),
        };

        // Create token record
        let token_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO passwordless_tokens (
                tenant_id, email, user_id, token_type, token_hash,
                ip_address, user_agent, expires_at, redirect_uri, state
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#
        )
        .bind(tenant_id)
        .bind(&request.email)
        .bind(user_id)
        .bind(request.token_type.to_string())
        .bind(&token_hash)
        .bind(&ip_address)
        .bind(&user_agent)
        .bind(expires_at)
        .bind(&request.redirect_uri)
        .bind(&request.state)
        .fetch_one(&self.pool)
        .await?;

        // Send the token via email
        let delivery_method = if request.send_sms {
            "sms" // TODO: Implement SMS delivery
        } else {
            "email"
        };

        if let Some(ref email_service) = self.email_service {
            match request.token_type {
                PasswordlessTokenType::MagicLink => {
                    let magic_link = format!(
                        "{}/auth/verify?token_id={}&code={}",
                        self.base_url, token_id, token
                    );
                    send_magic_link_email(email_service, &request.email, &magic_link).await?;
                }
                PasswordlessTokenType::Otp => {
                    send_otp_code_email(email_service, &request.email, &token).await?;
                }
            }
        }

        Ok(PasswordlessStartResponse {
            success: true,
            message: format!(
                "Verification {} sent to your email",
                match request.token_type {
                    PasswordlessTokenType::MagicLink => "link",
                    PasswordlessTokenType::Otp => "code",
                }
            ),
            delivery_method: delivery_method.to_string(),
            masked_destination: mask_email(&request.email),
            token_id,
            expires_at,
        })
    }

    /// Verify passwordless token
    pub async fn verify(
        &self,
        tenant_id: Uuid,
        request: PasswordlessVerifyRequest,
    ) -> Result<PasswordlessVerifyResponse> {
        // Get the token record
        let token: PasswordlessToken = sqlx::query_as(
            r#"
            SELECT * FROM passwordless_tokens
            WHERE id = $1 AND tenant_id = $2
            "#
        )
        .bind(request.token_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthError::InvalidCredentials("Invalid or expired token".into()))?;

        // Check if already used
        if token.used_at.is_some() {
            return Err(AuthError::InvalidCredentials("Token already used".into()));
        }

        // Check expiration
        if token.expires_at < Utc::now() {
            return Err(AuthError::InvalidCredentials("Token expired".into()));
        }

        // Check attempts for OTP
        if token.attempts >= token.max_attempts {
            return Err(AuthError::InvalidCredentials("Too many attempts".into()));
        }

        // Verify the code
        let code_hash = hash_token(&request.code);
        if code_hash != token.token_hash {
            // Increment attempts
            sqlx::query(
                "UPDATE passwordless_tokens SET attempts = attempts + 1 WHERE id = $1"
            )
            .bind(request.token_id)
            .execute(&self.pool)
            .await?;

            return Err(AuthError::InvalidCredentials("Invalid code".into()));
        }

        // Mark token as used
        sqlx::query(
            "UPDATE passwordless_tokens SET used_at = NOW() WHERE id = $1"
        )
        .bind(request.token_id)
        .execute(&self.pool)
        .await?;

        // Get or create user
        let (user_id, is_new_user) = if let Some(uid) = token.user_id {
            (uid, false)
        } else {
            // Create new user and add to tenant
            let new_user_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO users (default_tenant_id, email, email_verified, is_active, created_at, updated_at)
                VALUES ($1, $2, true, true, NOW(), NOW())
                RETURNING id
                "#
            )
            .bind(tenant_id)
            .bind(&token.email)
            .fetch_one(&self.pool)
            .await?;

            // Add user to tenant membership
            sqlx::query(
                r#"
                INSERT INTO tenant_members (tenant_id, user_id, role, created_at)
                VALUES ($1, $2, 'member', NOW())
                ON CONFLICT (tenant_id, user_id) DO NOTHING
                "#
            )
            .bind(tenant_id)
            .bind(new_user_id)
            .execute(&self.pool)
            .await?;

            (new_user_id, true)
        };

        // Mark email as verified
        sqlx::query(
            "UPDATE users SET email_verified = true WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Generate tokens
        let access_token = self.jwt_service.generate_access_token(
            user_id,
            &token.email,
            Some(tenant_id),
            None, // organization_slug
            None, // role
            false, // is_platform_admin
        )?;

        let refresh_token = generate_secure_token(64);

        // Store refresh token (simple implementation)
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (token_hash, user_id, tenant_id, family_id, expires_at)
            VALUES ($1, $2, $3, gen_random_uuid(), NOW() + INTERVAL '30 days')
            "#
        )
        .bind(hash_token(&refresh_token))
        .bind(user_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .ok(); // Ignore error if table doesn't exist yet

        Ok(PasswordlessVerifyResponse {
            success: true,
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            id_token: None, // TODO: Generate OIDC id_token if requested
            user: PasswordlessUserInfo {
                id: user_id,
                email: token.email,
                email_verified: true,
                tenant_id,
                is_new_user,
            },
            state: token.state,
        })
    }

    /// Check rate limit for an operation
    async fn check_rate_limit(
        &self,
        tenant_id: Uuid,
        endpoint_category: &str,
        ip_address: &Option<String>,
    ) -> Result<()> {
        // Get rate limit config
        let rate_limit: Option<TenantRateLimit> = sqlx::query_as(
            r#"
            SELECT * FROM tenant_rate_limits
            WHERE tenant_id = $1 AND endpoint_category = $2 AND is_enabled = true
            "#
        )
        .bind(tenant_id)
        .bind(endpoint_category)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(limit) = rate_limit {
            // Count recent requests (simplified - production would use Redis)
            let recent_count: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM passwordless_tokens
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR ip_address = $2)
                  AND created_at > NOW() - INTERVAL '1 minute'
                "#
            )
            .bind(tenant_id)
            .bind(ip_address)
            .fetch_one(&self.pool)
            .await?;

            if recent_count >= limit.requests_per_minute as i64 {
                return Err(AuthError::RateLimited(
                    "Too many requests. Please try again later.".into()
                ));
            }
        }

        Ok(())
    }

    /// Get rate limits for a tenant
    pub async fn get_rate_limits(&self, tenant_id: Uuid) -> Result<RateLimitsResponse> {
        let rate_limits: Vec<TenantRateLimit> = sqlx::query_as(
            "SELECT * FROM tenant_rate_limits WHERE tenant_id = $1 ORDER BY endpoint_category"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(RateLimitsResponse { rate_limits })
    }

    /// Update rate limit for a tenant
    pub async fn update_rate_limit(
        &self,
        tenant_id: Uuid,
        request: UpdateRateLimitRequest,
    ) -> Result<TenantRateLimit> {
        let rate_limit: TenantRateLimit = sqlx::query_as(
            r#"
            INSERT INTO tenant_rate_limits (tenant_id, endpoint_category, requests_per_minute, requests_per_hour, burst_limit, is_enabled)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id, endpoint_category)
            DO UPDATE SET
                requests_per_minute = COALESCE($3, tenant_rate_limits.requests_per_minute),
                requests_per_hour = COALESCE($4, tenant_rate_limits.requests_per_hour),
                burst_limit = COALESCE($5, tenant_rate_limits.burst_limit),
                is_enabled = COALESCE($6, tenant_rate_limits.is_enabled),
                updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(tenant_id)
        .bind(&request.endpoint_category)
        .bind(request.requests_per_minute.unwrap_or(60))
        .bind(request.requests_per_hour.unwrap_or(1000))
        .bind(request.burst_limit.unwrap_or(10))
        .bind(request.is_enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(rate_limit)
    }

    /// Resend passwordless token
    pub async fn resend(
        &self,
        tenant_id: Uuid,
        token_id: Uuid,
    ) -> Result<PasswordlessStartResponse> {
        // Get the original token
        let token: PasswordlessToken = sqlx::query_as(
            "SELECT * FROM passwordless_tokens WHERE id = $1 AND tenant_id = $2 AND used_at IS NULL"
        )
        .bind(token_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthError::InvalidCredentials("Token not found".into()))?;

        // Generate a new token and update
        let token_type: PasswordlessTokenType = token.token_type.parse()
            .map_err(|_| AuthError::InvalidCredentials("Invalid token type".into()))?;

        let (new_token, token_hash) = match token_type {
            PasswordlessTokenType::MagicLink => {
                let t = generate_secure_token(64);
                let h = hash_token(&t);
                (t, h)
            }
            PasswordlessTokenType::Otp => {
                let c = generate_otp_code();
                let h = hash_token(&c);
                (c, h)
            }
        };

        let expires_at = match token_type {
            PasswordlessTokenType::MagicLink => Utc::now() + Duration::minutes(15),
            PasswordlessTokenType::Otp => Utc::now() + Duration::minutes(10),
        };

        // Update the token
        sqlx::query(
            r#"
            UPDATE passwordless_tokens
            SET token_hash = $1, expires_at = $2, attempts = 0
            WHERE id = $3
            "#
        )
        .bind(&token_hash)
        .bind(expires_at)
        .bind(token_id)
        .execute(&self.pool)
        .await?;

        // Resend email
        if let Some(ref email_service) = self.email_service {
            match token_type {
                PasswordlessTokenType::MagicLink => {
                    let magic_link = format!(
                        "{}/auth/verify?token_id={}&code={}",
                        self.base_url, token_id, new_token
                    );
                    send_magic_link_email(email_service, &token.email, &magic_link).await?;
                }
                PasswordlessTokenType::Otp => {
                    send_otp_code_email(email_service, &token.email, &new_token).await?;
                }
            }
        }

        Ok(PasswordlessStartResponse {
            success: true,
            message: "Verification code resent".to_string(),
            delivery_method: "email".to_string(),
            masked_destination: mask_email(&token.email),
            token_id,
            expires_at,
        })
    }
}

/// Generate a cryptographically secure random token
fn generate_secure_token(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a 6-digit OTP code
fn generate_otp_code() -> String {
    let code: u32 = rand::thread_rng().gen_range(100000..1000000);
    format!("{:06}", code)
}

/// Hash a token using SHA-256
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Mask an email address for privacy
fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return "***".to_string();
    }

    let local = parts[0];
    let domain = parts[1];

    let masked_local = if local.len() <= 2 {
        "*".repeat(local.len())
    } else {
        format!("{}***{}", &local[..1], &local[local.len()-1..])
    };

    format!("{}@{}", masked_local, domain)
}

/// Send magic link email
async fn send_magic_link_email(email_service: &EmailService, to: &str, link: &str) -> Result<()> {
    let subject = "Sign in to your account";
    let body = format!(
        r#"
        <h2>Sign in to your account</h2>
        <p>Click the link below to sign in:</p>
        <p><a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #4F46E5; color: white; text-decoration: none; border-radius: 6px;">Sign In</a></p>
        <p>Or copy this link: {}</p>
        <p>This link will expire in 15 minutes.</p>
        <p>If you didn't request this, you can safely ignore this email.</p>
        "#,
        link, link
    );

    email_service.send_simple(to, subject, &body).await
}

/// Send OTP code email
async fn send_otp_code_email(email_service: &EmailService, to: &str, code: &str) -> Result<()> {
    let subject = "Your verification code";
    let body = format!(
        r#"
        <h2>Your verification code</h2>
        <p>Enter this code to sign in:</p>
        <p style="font-size: 32px; font-weight: bold; letter-spacing: 4px; color: #4F46E5;">{}</p>
        <p>This code will expire in 10 minutes.</p>
        <p>If you didn't request this, you can safely ignore this email.</p>
        "#,
        code
    );

    email_service.send_simple(to, subject, &body).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_otp_code() {
        let code = generate_otp_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("john@example.com"), "j***n@example.com");
        assert_eq!(mask_email("ab@example.com"), "**@example.com");
        assert_eq!(mask_email("a@example.com"), "*@example.com");
        assert_eq!(mask_email("invalid"), "***");
    }

    #[test]
    fn test_hash_token() {
        let token = "test_token";
        let hash = hash_token(token);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars

        // Same input produces same hash
        assert_eq!(hash, hash_token(token));

        // Different input produces different hash
        assert_ne!(hash, hash_token("different_token"));
    }
}
