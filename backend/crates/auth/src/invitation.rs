use crate::email::{EmailMessage, EmailService};
use crate::error::{AuthError, Result};
use crate::password::PasswordHasher;
use ciam_database::Database;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub invited_by: Uuid,
    pub role_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

pub struct InvitationService {
    db: Database,
    email_service: EmailService,
    base_url: String,
}

impl InvitationService {
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

    /// Create and send invitation
    pub async fn create_invitation(
        &self,
        tenant_id: Uuid,
        email: &str,
        invited_by: Uuid,
        role_id: Option<Uuid>,
        metadata: Option<serde_json::Value>,
        expires_in_days: i64,
    ) -> Result<Uuid> {
        // Check if user already exists with this email in this tenant
        let existing_user: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id
            FROM users u
            INNER JOIN organization_members om ON u.id = om.user_id
            WHERE u.email = $1 AND om.organization_id = $2
            "#,
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        if existing_user.is_some() {
            return Err(AuthError::BadRequest(
                "User with this email already exists in this tenant".to_string(),
            ));
        }

        // Check if there's already a pending invitation
        let pending_invitation: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, created_at
            FROM invitations
            WHERE email = $1 AND tenant_id = $2 AND accepted_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        // If pending invitation exists and was created less than 5 minutes ago, reject
        // Otherwise, allow resending (delete old one first)
        if let Some((old_invitation_id, created_at)) = pending_invitation {
            let age = Utc::now().signed_duration_since(created_at);
            if age.num_minutes() < 5 {
                return Err(AuthError::BadRequest(
                    "An invitation was recently sent. Please wait a few minutes before resending.".to_string(),
                ));
            } else {
                // Delete old pending invitation to allow retry
                tracing::info!(
                    "Deleting old pending invitation for email={}, tenant_id={} to allow resend",
                    email,
                    tenant_id
                );
                sqlx::query("DELETE FROM invitations WHERE id = $1")
                    .bind(old_invitation_id)
                    .execute(self.db.pool())
                    .await?;
            }
        }

        // Generate invitation token
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::days(expires_in_days);

        // Get inviter's name from metadata
        let inviter_name: String = sqlx::query_scalar(
            r#"
            SELECT COALESCE(metadata->>'full_name', email)
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(invited_by)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or_else(|_| "A team member".to_string());

        // Get tenant name
        let tenant_name: String = sqlx::query_scalar(
            r#"
            SELECT name
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or_else(|_| "the team".to_string());

        // Get role name if provided
        let role_name: Option<String> = if let Some(rid) = role_id {
            sqlx::query_scalar(
                r#"
                SELECT name
                FROM roles
                WHERE id = $1
                "#,
            )
            .bind(rid)
            .fetch_optional(self.db.pool())
            .await?
        } else {
            None
        };

        // Create invitation in database
        let invitation_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO invitations
                (tenant_id, email, token_hash, invited_by, role_id, metadata, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .bind(&token_hash)
        .bind(invited_by)
        .bind(role_id)
        .bind(&metadata)
        .bind(expires_at)
        .fetch_one(self.db.pool())
        .await?;

        // Generate invitation link
        let invitation_link = format!("{}/accept-invitation?token={}", self.base_url, token);

        // Generate email content
        let (text_body, html_body) = crate::email::templates::user_invitation(
            &inviter_name,
            &tenant_name,
            &invitation_link,
            role_name.as_deref(),
            &expires_at,
        );

        // Send email
        let email_message = EmailMessage {
            to: email.to_string(),
            to_name: None,
            subject: format!("You've been invited to join {}", tenant_name),
            text_body,
            html_body: Some(html_body),
        };

        // Try to send email - if it fails, delete the invitation
        match self.email_service.send(email_message).await {
            Ok(_) => {
                tracing::info!(
                    "Invitation created and sent: id={}, email={}, tenant_id={}",
                    invitation_id,
                    email,
                    tenant_id
                );
                Ok(invitation_id)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to send invitation email for id={}, email={}, tenant_id={}. Error: {}. Cleaning up invitation record.",
                    invitation_id,
                    email,
                    tenant_id,
                    e
                );

                // Delete the invitation since email failed
                let _ = sqlx::query("DELETE FROM invitations WHERE id = $1")
                    .bind(invitation_id)
                    .execute(self.db.pool())
                    .await;

                Err(e)
            }
        }
    }

    /// Verify invitation token
    pub async fn verify_invitation(&self, token: &str) -> Result<Invitation> {
        let token_hash = Self::hash_token(token);

        // Find and validate token
        let record: Option<(Uuid, Uuid, String, Uuid, Option<Uuid>, Option<serde_json::Value>, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, email, invited_by, role_id, metadata, expires_at, created_at
            FROM invitations
            WHERE token_hash = $1
              AND accepted_at IS NULL
              AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(self.db.pool())
        .await?;

        let (id, tenant_id, email, invited_by, role_id, metadata, expires_at, created_at) = record
            .ok_or_else(|| AuthError::InvalidToken("Invalid or expired invitation".to_string()))?;

        Ok(Invitation {
            id,
            tenant_id,
            email,
            invited_by,
            role_id,
            metadata,
            expires_at,
            created_at,
            accepted_at: None,
        })
    }

    /// Accept invitation and create user account
    pub async fn accept_invitation(
        &self,
        token: &str,
        password: &str,
        full_name: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        // Verify invitation
        let invitation = self.verify_invitation(token).await?;

        // Hash password
        let password_hash = PasswordHasher::hash(password)?;

        // Merge metadata from invitation and user input, including full_name
        let mut final_metadata = if let Some(mut inv_meta) = invitation.metadata {
            if let Some(user_meta) = metadata {
                // Merge user metadata into invitation metadata
                if let (Some(inv_obj), Some(user_obj)) =
                    (inv_meta.as_object_mut(), user_meta.as_object())
                {
                    for (key, value) in user_obj {
                        inv_obj.insert(key.clone(), value.clone());
                    }
                }
            }
            inv_meta
        } else {
            metadata.unwrap_or_else(|| serde_json::json!({}))
        };

        // Add full_name to metadata
        if let Some(obj) = final_metadata.as_object_mut() {
            obj.insert("full_name".to_string(), serde_json::json!(full_name));

            // Parse full name into first and last name
            let parts: Vec<&str> = full_name.split_whitespace().collect();
            if !parts.is_empty() {
                obj.insert("first_name".to_string(), serde_json::json!(parts[0]));
                if parts.len() > 1 {
                    obj.insert("last_name".to_string(), serde_json::json!(parts[1..].join(" ")));
                }
            }
        }

        // Create user account
        let user_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO users
                (default_organization_id, email, password_hash, email_verified, is_active, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(invitation.tenant_id)
        .bind(&invitation.email)
        .bind(&password_hash)
        .bind(true) // Email is pre-verified through invitation
        .bind(true)
        .bind(&final_metadata)
        .fetch_one(self.db.pool())
        .await?;

        // Add user to organization_members table
        sqlx::query(
            r#"
            INSERT INTO organization_members (user_id, organization_id, role)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(invitation.tenant_id)
        .bind(invitation.role_id.and_then(|_| Some("member")).unwrap_or("member"))
        .execute(self.db.pool())
        .await?;

        // Assign role if specified
        if let Some(role_id) = invitation.role_id {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                VALUES ($1, $2)
                ON CONFLICT (user_id, role_id) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(role_id)
            .execute(self.db.pool())
            .await?;
        }

        // Mark invitation as accepted
        let token_hash = Self::hash_token(token);
        sqlx::query(
            r#"
            UPDATE invitations
            SET accepted_at = NOW(), accepted_by = $1
            WHERE token_hash = $2
            "#,
        )
        .bind(user_id)
        .bind(&token_hash)
        .execute(self.db.pool())
        .await?;

        tracing::info!(
            "Invitation accepted: user_id={}, email={}, tenant_id={}",
            user_id,
            invitation.email,
            invitation.tenant_id
        );

        Ok(user_id)
    }

    /// List invitations for a tenant
    pub async fn list_invitations(
        &self,
        tenant_id: Uuid,
        include_accepted: bool,
    ) -> Result<Vec<Invitation>> {
        let query = if include_accepted {
            r#"
            SELECT id, tenant_id, email, invited_by, role_id, metadata, expires_at, created_at, accepted_at
            FROM invitations
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, tenant_id, email, invited_by, role_id, metadata, expires_at, created_at, accepted_at
            FROM invitations
            WHERE tenant_id = $1 AND accepted_at IS NULL
            ORDER BY created_at DESC
            "#
        };

        let invitations: Vec<(Uuid, Uuid, String, Uuid, Option<Uuid>, Option<serde_json::Value>, DateTime<Utc>, DateTime<Utc>, Option<DateTime<Utc>>)> =
            sqlx::query_as(query)
                .bind(tenant_id)
                .fetch_all(self.db.pool())
                .await?;

        Ok(invitations
            .into_iter()
            .map(
                |(id, tenant_id, email, invited_by, role_id, metadata, expires_at, created_at, accepted_at)| {
                    Invitation {
                        id,
                        tenant_id,
                        email,
                        invited_by,
                        role_id,
                        metadata,
                        expires_at,
                        created_at,
                        accepted_at,
                    }
                },
            )
            .collect())
    }

    /// Revoke/cancel an invitation
    pub async fn revoke_invitation(&self, invitation_id: Uuid, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM invitations
            WHERE id = $1 AND tenant_id = $2 AND accepted_at IS NULL
            "#,
        )
        .bind(invitation_id)
        .bind(tenant_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthError::NotFound(
                "Invitation not found or already accepted".to_string(),
            ));
        }

        tracing::info!(
            "Invitation revoked: id={}, tenant_id={}",
            invitation_id,
            tenant_id
        );

        Ok(())
    }

    /// Resend invitation email
    pub async fn resend_invitation(&self, invitation_id: Uuid, tenant_id: Uuid) -> Result<()> {
        // Get invitation details
        let invitation: Option<(String, Uuid, Option<Uuid>, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT email, invited_by, role_id, expires_at
            FROM invitations
            WHERE id = $1 AND tenant_id = $2 AND accepted_at IS NULL AND expires_at > NOW()
            "#,
        )
        .bind(invitation_id)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await?;

        let (email, invited_by, role_id, expires_at) = invitation.ok_or_else(|| {
            AuthError::NotFound("Invitation not found or expired".to_string())
        })?;

        // Generate new token
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);

        // Update invitation with new token
        sqlx::query(
            r#"
            UPDATE invitations
            SET token_hash = $1
            WHERE id = $2
            "#,
        )
        .bind(&token_hash)
        .bind(invitation_id)
        .execute(self.db.pool())
        .await?;

        // Get inviter and tenant names
        let inviter_name: String = sqlx::query_scalar("SELECT COALESCE(metadata->>'full_name', email) FROM users WHERE id = $1")
            .bind(invited_by)
            .fetch_one(self.db.pool())
            .await
            .unwrap_or_else(|_| "A team member".to_string());

        let tenant_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
            .bind(tenant_id)
            .fetch_one(self.db.pool())
            .await
            .unwrap_or_else(|_| "the team".to_string());

        let role_name: Option<String> = if let Some(rid) = role_id {
            sqlx::query_scalar("SELECT name FROM roles WHERE id = $1")
                .bind(rid)
                .fetch_optional(self.db.pool())
                .await?
        } else {
            None
        };

        // Generate invitation link
        let invitation_link = format!("{}/accept-invitation?token={}", self.base_url, token);

        // Generate email content
        let (text_body, html_body) = crate::email::templates::user_invitation(
            &inviter_name,
            &tenant_name,
            &invitation_link,
            role_name.as_deref(),
            &expires_at,
        );

        // Send email
        let email_message = EmailMessage {
            to: email.clone(),
            to_name: None,
            subject: format!("Reminder: You've been invited to join {}", tenant_name),
            text_body,
            html_body: Some(html_body),
        };

        self.email_service.send(email_message).await?;

        tracing::info!(
            "Invitation resent: id={}, email={}",
            invitation_id,
            email
        );

        Ok(())
    }
}
