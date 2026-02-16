use crate::error::{AuthzError, Result};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "application_type", rename_all = "lowercase")]
pub enum ApplicationType {
    Service,  // Machine-to-machine (backend services)
    WebApp,   // Confidential client (server-side web apps)
    SPA,      // Public client (single-page apps)
    Native,   // Mobile/desktop apps
}

impl std::convert::TryFrom<String> for ApplicationType {
    type Error = String;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "service" => Ok(ApplicationType::Service),
            "webapp" => Ok(ApplicationType::WebApp),
            "spa" => Ok(ApplicationType::SPA),
            "native" => Ok(ApplicationType::Native),
            _ => Err(format!("Invalid application type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Application {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub client_secret_hash: String,
    #[sqlx(try_from = "String")]
    pub application_type: ApplicationType,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    #[sqlx(json)]
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApplicationRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub application_type: ApplicationType,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationWithSecret {
    pub application: Application,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct ApplicationService {
    pool: PgPool,
}

impl ApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a secure random client ID
    fn generate_client_id() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        format!("app_{}", hex::encode(bytes))
    }

    /// Generate a secure random client secret
    fn generate_client_secret() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// Hash client secret for storage
    fn hash_secret(secret: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify client secret against hash
    pub fn verify_secret(secret: &str, hash: &str) -> bool {
        let secret_hash = Self::hash_secret(secret);
        secret_hash == hash
    }

    /// Create a new application (service principal)
    pub async fn create_application(
        &self,
        request: CreateApplicationRequest,
    ) -> Result<ApplicationWithSecret> {
        let client_id = Self::generate_client_id();
        let client_secret = Self::generate_client_secret();
        let client_secret_hash = Self::hash_secret(&client_secret);

        let metadata = request.metadata.unwrap_or_else(|| serde_json::json!({}));

        let app = sqlx::query_as::<_, Application>(
            r#"
            INSERT INTO applications
                (tenant_id, name, description, client_id, client_secret_hash, application_type,
                 redirect_uris, allowed_scopes, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(request.tenant_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&client_id)
        .bind(&client_secret_hash)
        .bind(&request.application_type)
        .bind(&request.redirect_uris)
        .bind(&request.allowed_scopes)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            "Created application: id={}, name={}, type={:?}",
            app.id,
            app.name,
            app.application_type
        );

        Ok(ApplicationWithSecret {
            application: app,
            client_secret,
        })
    }

    /// Get application by ID
    pub async fn get_application(&self, app_id: Uuid, tenant_id: Uuid) -> Result<Application> {
        sqlx::query_as::<_, Application>(
            "SELECT * FROM applications WHERE id = $1 AND tenant_id = $2",
        )
        .bind(app_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthzError::NotFound("Application not found".to_string()))
    }

    /// Get application by client ID
    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Application> {
        sqlx::query_as::<_, Application>("SELECT * FROM applications WHERE client_id = $1")
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AuthzError::NotFound("Application not found".to_string()))
    }

    /// List applications for a tenant
    pub async fn list_applications(&self, tenant_id: Uuid) -> Result<Vec<Application>> {
        Ok(
            sqlx::query_as::<_, Application>(
                "SELECT * FROM applications WHERE tenant_id = $1 ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?,
        )
    }

    /// Update application
    pub async fn update_application(
        &self,
        app_id: Uuid,
        tenant_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        redirect_uris: Option<Vec<String>>,
        allowed_scopes: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<Application> {
        // First verify the application exists
        let _ = self.get_application(app_id, tenant_id).await?;

        let app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET name = COALESCE($3, name),
                description = COALESCE($4, description),
                redirect_uris = COALESCE($5, redirect_uris),
                allowed_scopes = COALESCE($6, allowed_scopes),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#,
        )
        .bind(app_id)
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(redirect_uris)
        .bind(allowed_scopes)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Updated application: id={}, name={}", app.id, app.name);

        Ok(app)
    }

    /// Rotate client secret
    pub async fn rotate_secret(
        &self,
        app_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<ApplicationWithSecret> {
        // Verify application exists
        let app = self.get_application(app_id, tenant_id).await?;

        // Generate new secret
        let new_secret = Self::generate_client_secret();
        let new_hash = Self::hash_secret(&new_secret);

        // Update in database
        let updated_app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET client_secret_hash = $3, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#,
        )
        .bind(app_id)
        .bind(tenant_id)
        .bind(&new_hash)
        .fetch_one(&self.pool)
        .await?;

        tracing::warn!(
            "Rotated secret for application: id={}, name={}",
            app.id,
            app.name
        );

        Ok(ApplicationWithSecret {
            application: updated_app,
            client_secret: new_secret,
        })
    }

    /// Delete application
    pub async fn delete_application(&self, app_id: Uuid, tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM applications WHERE id = $1 AND tenant_id = $2")
            .bind(app_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthzError::NotFound("Application not found".to_string()));
        }

        tracing::info!("Deleted application: id={}", app_id);

        Ok(())
    }

    /// Authenticate application using client credentials
    pub async fn authenticate(&self, client_id: &str, client_secret: &str) -> Result<Application> {
        let app = self.get_by_client_id(client_id).await?;

        if !app.is_active {
            return Err(AuthzError::Unauthorized(
                "Application is not active".to_string(),
            ));
        }

        if !Self::verify_secret(client_secret, &app.client_secret_hash) {
            return Err(AuthzError::InvalidCredentials);
        }

        Ok(app)
    }
}
