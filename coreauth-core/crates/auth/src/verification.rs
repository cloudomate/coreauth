use crate::email::{EmailMessage, EmailService};
use crate::error::{AuthError, Result};
use ciam_database::Database;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct VerificationService {
    db: Database,
    email_service: EmailService,
    base_url: String,
}

impl VerificationService {
    pub fn new(db: Database, email_service: EmailService, base_url: String) -> Self {
        Self {
            db,
            email_service,
            base_url,
        }
    }

    /// Generate a secure random token
    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(token_bytes)
    }

    /// Hash a token for secure storage
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Send email verification
    /// If `verification_base_url` is provided, it overrides the default base_url
    /// (e.g., use the application's origin so users stay on the app, not CoreAuth)
    pub async fn send_verification_email(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
        user_name: &str,
        verification_base_url: Option<&str>,
    ) -> Result<()> {
        // Generate verification token
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(24);

        // Store token in database
        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (user_id, tenant_id, email, token_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(email)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(self.db.pool())
        .await?;

        // Generate verification link — use app's URL if provided, otherwise CoreAuth's base URL
        let base = verification_base_url.unwrap_or(&self.base_url);
        let verification_link = format!("{}/verify-email?token={}", base, token);

        // Fetch tenant branding for email template
        let branding = self.get_tenant_branding(tenant_id).await;

        // Build template variables
        let mut variables = std::collections::HashMap::new();
        variables.insert("user_name".into(), user_name.to_string());
        variables.insert("verification_link".into(), verification_link.clone());
        variables.insert("expires_at".into(), expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());

        // Fetch custom template (if tenant has one)
        let custom_template = crate::email::templates::fetch_custom_template(
            self.db.pool(),
            tenant_id,
            crate::email::templates::EmailTemplateType::EmailVerification,
        ).await;

        // Render email (custom template or built-in fallback)
        let (subject, text_body, html_body) = crate::email::templates::render_email(
            crate::email::templates::EmailTemplateType::EmailVerification,
            custom_template.as_ref(),
            &variables,
            &branding,
        );

        // Send email
        let email_message = EmailMessage {
            to: email.to_string(),
            to_name: Some(user_name.to_string()),
            subject,
            text_body,
            html_body: Some(html_body),
        };

        self.email_service.send(email_message).await?;

        tracing::info!(
            "Email verification sent: user_id={}, email={}",
            user_id,
            email
        );

        Ok(())
    }

    /// Verify email token
    pub async fn verify_email(&self, token: &str) -> Result<Uuid> {
        let token_hash = Self::hash_token(token);

        // Find and validate token
        let record: Option<(Uuid, Uuid, String, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT user_id, tenant_id, email, expires_at
            FROM email_verification_tokens
            WHERE token_hash = $1
              AND verified_at IS NULL
              AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(self.db.pool())
        .await?;

        let (user_id, _tenant_id, email, _expires_at) = record.ok_or_else(|| {
            AuthError::InvalidToken("Invalid or expired verification token".to_string())
        })?;

        // Mark token as used
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET verified_at = NOW()
            WHERE token_hash = $1
            "#,
        )
        .bind(&token_hash)
        .execute(self.db.pool())
        .await?;

        // Update user's email verification status
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = true,
                email = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&email)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        tracing::info!(
            "Email verified successfully: user_id={}, email={}",
            user_id,
            email
        );

        Ok(user_id)
    }

    /// Resend verification email
    pub async fn resend_verification_email(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<()> {
        // Get user info
        let user: Option<(String, String, bool)> = sqlx::query_as(
            r#"
            SELECT email, full_name, email_verified
            FROM users
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        let (email, full_name, email_verified) = user.ok_or_else(|| {
            AuthError::NotFound("User not found".to_string())
        })?;

        if email_verified {
            return Err(AuthError::BadRequest(
                "Email already verified".to_string(),
            ));
        }

        // Invalidate old tokens
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET verified_at = NOW()
            WHERE user_id = $1 AND verified_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        // Send new verification email (no app-specific URL for API-triggered resends)
        self.send_verification_email(user_id, tenant_id, &email, &full_name, None)
            .await?;

        Ok(())
    }

    /// Fetch tenant branding for email templates
    async fn get_tenant_branding(&self, tenant_id: Uuid) -> crate::email::templates::EmailBranding {
        let settings: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT settings FROM tenants WHERE id = $1",
        )
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await
        .ok()
        .flatten();

        settings
            .and_then(|(v,)| serde_json::from_value::<ciam_models::OrganizationSettings>(v).ok())
            .map(|s| crate::email::templates::EmailBranding::from_settings(&s.branding))
            .unwrap_or_default()
    }

    /// Check if email verification is required for tenant
    pub async fn is_verification_required(&self, tenant_id: Uuid) -> Result<bool> {
        let settings_json: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT settings FROM tenants WHERE id = $1",
        )
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(settings_json
            .and_then(|(v,)| serde_json::from_value::<ciam_models::OrganizationSettings>(v).ok())
            .map(|s| s.security.require_email_verification)
            .unwrap_or(false))
    }
}
