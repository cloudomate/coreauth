use crate::error::{DatabaseError, Result};
use ciam_models::{Application, ApplicationWithSecret, CreateApplication, UpdateApplication};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ApplicationRepository {
    pool: PgPool,
}

impl ApplicationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new application and return with plaintext client secret
    pub async fn create(&self, request: &CreateApplication) -> Result<ApplicationWithSecret> {
        // Generate client_id and client_secret
        let client_id = format!("app_{}", Uuid::new_v4().to_string().replace("-", ""));
        let client_secret_plain = Self::generate_secret();
        let client_secret_hash = Self::hash_secret(&client_secret_plain)?;

        let logout_urls = request.logout_urls.clone().unwrap_or_default();
        let web_origins = request.web_origins.clone().unwrap_or_default();
        let allowed_connections = request.allowed_connections.clone().unwrap_or_default();
        let require_organization = request.require_organization.unwrap_or(false);
        let access_token_lifetime = request.access_token_lifetime_seconds.unwrap_or(3600);
        let refresh_token_lifetime = request.refresh_token_lifetime_seconds.unwrap_or(2592000); // 30 days

        let app = sqlx::query_as::<_, Application>(
            r#"
            INSERT INTO applications (
                organization_id, name, slug, description, type,
                client_id, client_secret, callback_urls, logout_urls, web_origins,
                allowed_connections, require_organization, platform_admin_only,
                access_token_lifetime_seconds, refresh_token_lifetime_seconds, is_enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(&request.organization_id)
        .bind(&request.name)
        .bind(&request.slug)
        .bind(&request.description)
        .bind(&request.app_type)
        .bind(&client_id)
        .bind(&client_secret_hash)
        .bind(sqlx::types::Json(&request.callback_urls))
        .bind(sqlx::types::Json(&logout_urls))
        .bind(sqlx::types::Json(&web_origins))
        .bind(sqlx::types::Json(&allowed_connections))
        .bind(require_organization)
        .bind(false) // platform_admin_only
        .bind(access_token_lifetime)
        .bind(refresh_token_lifetime)
        .bind(true) // is_enabled
        .fetch_one(&self.pool)
        .await?;

        Ok(ApplicationWithSecret {
            application: app,
            client_secret_plain,
        })
    }

    /// Get application by ID (secret is hashed)
    pub async fn get_by_id(&self, id: Uuid) -> Result<Application> {
        let app = sqlx::query_as::<_, Application>("SELECT * FROM applications WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Application", &id.to_string()))?;

        Ok(app)
    }

    /// Get application by client_id
    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Application> {
        let app = sqlx::query_as::<_, Application>(
            "SELECT * FROM applications WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Application", client_id))?;

        Ok(app)
    }

    /// List applications by organization (paginated)
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Application>> {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    /// List global/platform-level applications
    pub async fn list_global(&self, limit: i64, offset: i64) -> Result<Vec<Application>> {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE organization_id IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    /// Update application
    pub async fn update(&self, id: Uuid, request: &UpdateApplication) -> Result<Application> {
        // Build dynamic update query
        let current_app = self.get_by_id(id).await?;

        let name = request.name.as_ref().unwrap_or(&current_app.name);
        let description = request.description.as_ref().or(current_app.description.as_ref());
        let callback_urls = request.callback_urls.as_ref().unwrap_or(&current_app.callback_urls);
        let logout_urls = request.logout_urls.as_ref().unwrap_or(&current_app.logout_urls);
        let web_origins = request.web_origins.as_ref().unwrap_or(&current_app.web_origins);
        let allowed_connections = request.allowed_connections.as_ref().unwrap_or(&current_app.allowed_connections);
        let require_organization = request.require_organization.unwrap_or(current_app.require_organization);
        let access_token_lifetime = request.access_token_lifetime_seconds.unwrap_or(current_app.access_token_lifetime_seconds);
        let refresh_token_lifetime = request.refresh_token_lifetime_seconds.unwrap_or(current_app.refresh_token_lifetime_seconds);
        let is_enabled = request.is_enabled.unwrap_or(current_app.is_enabled);

        let app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET name = $1, description = $2, callback_urls = $3, logout_urls = $4,
                web_origins = $5, allowed_connections = $6, require_organization = $7,
                access_token_lifetime_seconds = $8, refresh_token_lifetime_seconds = $9,
                is_enabled = $10, updated_at = NOW()
            WHERE id = $11
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(sqlx::types::Json(callback_urls))
        .bind(sqlx::types::Json(logout_urls))
        .bind(sqlx::types::Json(web_origins))
        .bind(sqlx::types::Json(allowed_connections))
        .bind(require_organization)
        .bind(access_token_lifetime)
        .bind(refresh_token_lifetime)
        .bind(is_enabled)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(app)
    }

    /// Rotate client secret (returns new plaintext secret)
    pub async fn rotate_secret(&self, id: Uuid) -> Result<ApplicationWithSecret> {
        let client_secret_plain = Self::generate_secret();
        let client_secret_hash = Self::hash_secret(&client_secret_plain)?;

        let app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET client_secret = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(&client_secret_hash)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ApplicationWithSecret {
            application: app,
            client_secret_plain,
        })
    }

    /// Verify client credentials (for OAuth flows)
    pub async fn verify_credentials(&self, client_id: &str, client_secret: &str) -> Result<Application> {
        let app = self.get_by_client_id(client_id).await?;

        if let Some(hashed_secret) = &app.client_secret {
            if Self::verify_secret(client_secret, hashed_secret)? {
                return Ok(app);
            }
        }

        Err(DatabaseError::Unauthorized("Invalid client credentials".to_string()))
    }

    /// Delete application
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM applications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Count applications by organization
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM applications WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    // Helper: Generate random secret
    fn generate_secret() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const SECRET_LEN: usize = 64;

        let mut rng = rand::thread_rng();
        let secret: String = (0..SECRET_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        secret
    }

    // Helper: Hash secret with bcrypt
    fn hash_secret(secret: &str) -> Result<String> {
        bcrypt::hash(secret, bcrypt::DEFAULT_COST)
            .map_err(|e| DatabaseError::Internal(format!("Failed to hash secret: {}", e)))
    }

    // Helper: Verify secret against hash
    fn verify_secret(secret: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(secret, hash)
            .map_err(|e| DatabaseError::Internal(format!("Failed to verify secret: {}", e)))
    }
}
