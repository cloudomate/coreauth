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
        let access_token_lifetime = request.access_token_lifetime_seconds.unwrap_or(3600);
        let refresh_token_lifetime = request.refresh_token_lifetime_seconds.unwrap_or(2592000); // 30 days
        let grant_types = request.grant_types.clone().unwrap_or_else(|| {
            vec!["authorization_code".to_string(), "refresh_token".to_string()]
        });
        let allowed_scopes = request.allowed_scopes.clone().unwrap_or_else(|| {
            vec!["openid".to_string(), "profile".to_string(), "email".to_string()]
        });

        let app = sqlx::query_as::<_, Application>(
            r#"
            INSERT INTO applications (
                tenant_id, name, slug, description, logo_url, app_type,
                client_id, client_secret_hash, callback_urls, allowed_logout_urls, allowed_web_origins,
                access_token_ttl_seconds, refresh_token_ttl_seconds,
                grant_types, allowed_scopes, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(&request.organization_id)
        .bind(&request.name)
        .bind(&request.slug)
        .bind(&request.description)
        .bind(&request.logo_url)
        .bind(&request.app_type)
        .bind(&client_id)
        .bind(&client_secret_hash)
        .bind(&request.callback_urls)
        .bind(&logout_urls)
        .bind(&web_origins)
        .bind(access_token_lifetime)
        .bind(refresh_token_lifetime)
        .bind(&grant_types)
        .bind(&allowed_scopes)
        .bind(true) // is_active
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
    /// Note: Uses tenant_id column but the parameter is organization_id for API consistency
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Application>> {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE tenant_id = $1
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
            WHERE tenant_id IS NULL
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
        let logo_url = request.logo_url.as_ref().or(current_app.logo_url.as_ref());
        let callback_urls = request.callback_urls.as_ref().unwrap_or(&current_app.callback_urls);
        let logout_urls = request.logout_urls.as_ref().unwrap_or(&current_app.logout_urls);
        let web_origins = request.web_origins.as_ref().unwrap_or(&current_app.web_origins);
        let access_token_lifetime = request.access_token_lifetime_seconds.unwrap_or(current_app.access_token_lifetime_seconds);
        let refresh_token_lifetime = request.refresh_token_lifetime_seconds.unwrap_or(current_app.refresh_token_lifetime_seconds);
        let grant_types = request.grant_types.as_ref().unwrap_or(&current_app.grant_types);
        let allowed_scopes = request.allowed_scopes.as_ref().unwrap_or(&current_app.allowed_scopes);
        let is_active = request.is_enabled.unwrap_or(current_app.is_enabled);

        let app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET name = $1, description = $2, logo_url = $3, callback_urls = $4, allowed_logout_urls = $5,
                allowed_web_origins = $6,
                access_token_ttl_seconds = $7, refresh_token_ttl_seconds = $8,
                grant_types = $9, allowed_scopes = $10, is_active = $11, updated_at = NOW()
            WHERE id = $12
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(logo_url)
        .bind(callback_urls)
        .bind(logout_urls)
        .bind(web_origins)
        .bind(access_token_lifetime)
        .bind(refresh_token_lifetime)
        .bind(grant_types)
        .bind(allowed_scopes)
        .bind(is_active)
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
            SET client_secret_hash = $1, updated_at = NOW()
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
            "SELECT COUNT(*) FROM applications WHERE tenant_id = $1",
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
