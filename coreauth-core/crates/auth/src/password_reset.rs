use crate::email::{EmailMessage, EmailService};
use crate::error::{AuthError, Result};
use crate::password::PasswordHasher;
use ciam_database::Database;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct PasswordResetService {
    db: Database,
    email_service: EmailService,
    base_url: String,
}

impl PasswordResetService {
    pub fn new(
        db: Database,
        email_service: EmailService,
        base_url: String,
    ) -> Self {
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

    /// Request password reset
    pub async fn request_password_reset(
        &self,
        tenant_id: Uuid,
        email: &str,
        ip_address: &str,
        user_agent: Option<&str>,
    ) -> Result<()> {
        // Find user by email and tenant_id
        let user: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, full_name
            FROM users
            WHERE email = $1 AND tenant_id = $2 AND is_active = true
            "#,
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        // Don't reveal whether email exists (security best practice)
        if user.is_none() {
            tracing::warn!(
                "Password reset requested for non-existent email: {}",
                email
            );
            // Still return success to avoid user enumeration
            return Ok(());
        }

        let (user_id, full_name) = user.unwrap();

        // Generate reset token
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(1); // 1 hour expiration

        // Invalidate any existing reset tokens for this user
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE user_id = $1 AND used_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        // Store new token in database
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens
                (user_id, tenant_id, token_hash, expires_at, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(&token_hash)
        .bind(expires_at)
        .bind(ip_address)
        .bind(user_agent)
        .execute(self.db.pool())
        .await?;

        // Generate reset link
        let reset_link = format!("{}/reset-password?token={}", self.base_url, token);

        // Fetch tenant branding for email template
        let branding = self.get_tenant_branding(tenant_id).await;

        // Build template variables
        let mut variables = std::collections::HashMap::new();
        variables.insert("user_name".into(), full_name.clone());
        variables.insert("reset_link".into(), reset_link.clone());
        variables.insert("expires_at".into(), expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        variables.insert("ip_address".into(), ip_address.to_string());

        // Fetch custom template (if tenant has one)
        let custom_template = crate::email::templates::fetch_custom_template(
            self.db.pool(),
            tenant_id,
            crate::email::templates::EmailTemplateType::PasswordReset,
        ).await;

        // Render email (custom template or built-in fallback)
        let (subject, text_body, html_body) = crate::email::templates::render_email(
            crate::email::templates::EmailTemplateType::PasswordReset,
            custom_template.as_ref(),
            &variables,
            &branding,
        );

        // Send email
        let email_message = EmailMessage {
            to: email.to_string(),
            to_name: Some(full_name.clone()),
            subject,
            text_body,
            html_body: Some(html_body),
        };

        self.email_service.send(email_message).await?;

        tracing::info!(
            "Password reset email sent: user_id={}, email={}",
            user_id,
            email
        );

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

    /// Verify reset token and get user_id
    pub async fn verify_reset_token(&self, token: &str) -> Result<Uuid> {
        let token_hash = Self::hash_token(token);

        // Find and validate token
        let record: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id
            FROM password_reset_tokens
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(self.db.pool())
        .await?;

        let (user_id,) = record.ok_or_else(|| {
            AuthError::InvalidToken("Invalid or expired reset token".to_string())
        })?;

        Ok(user_id)
    }

    /// Reset password with token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        // Verify token and get user_id
        let user_id = self.verify_reset_token(token).await?;

        // Hash new password
        let password_hash = PasswordHasher::hash(new_password)?;

        // Update user's password
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&password_hash)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        // Mark token as used
        let token_hash = Self::hash_token(token);
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE token_hash = $1
            "#,
        )
        .bind(&token_hash)
        .execute(self.db.pool())
        .await?;

        // Invalidate all sessions for this user (force re-login)
        sqlx::query(
            r#"
            UPDATE sessions
            SET expires_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        tracing::info!("Password reset successfully: user_id={}", user_id);

        Ok(())
    }
}
