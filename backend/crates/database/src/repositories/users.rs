use crate::error::{DatabaseError, Result};
use ciam_models::user::{NewUser, UpdateUser, UserProfile};
use ciam_models::User;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create(&self, new_user: &NewUser, password_hash: &str) -> Result<User> {
        // Use default metadata if none provided
        let metadata = new_user.metadata.as_ref()
            .cloned()
            .unwrap_or_default();
        let metadata_json = sqlx::types::Json(&metadata);

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (tenant_id, email, password_hash, phone, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&new_user.tenant_id)
        .bind(&new_user.email)
        .bind(password_hash)
        .bind(&new_user.phone)
        .bind(metadata_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("User", &id.to_string()))?;

        Ok(user)
    }

    /// Find user by email and tenant
    pub async fn find_by_email(&self, tenant_id: Uuid, email: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND email = $2",
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("User", email))?;

        Ok(user)
    }

    /// Find user by phone and tenant
    pub async fn find_by_phone(&self, tenant_id: Uuid, phone: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND phone = $2",
        )
        .bind(tenant_id)
        .bind(phone)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("User", phone))?;

        Ok(user)
    }

    /// List all users for a tenant (paginated)
    pub async fn list(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(users.into_iter().map(UserProfile::from).collect())
    }

    /// Count users for a tenant
    pub async fn count(&self, tenant_id: Uuid) -> Result<i64> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count.0)
    }

    /// Update user
    pub async fn update(&self, id: Uuid, update: &UpdateUser) -> Result<User> {
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE users SET updated_at = NOW()");

        let mut has_updates = false;

        if let Some(ref email) = update.email {
            query_builder.push(", email = ");
            query_builder.push_bind(email);
            has_updates = true;
        }

        if let Some(ref phone) = update.phone {
            query_builder.push(", phone = ");
            query_builder.push_bind(phone);
            has_updates = true;
        }

        if let Some(ref metadata) = update.metadata {
            query_builder.push(", metadata = ");
            query_builder.push_bind(sqlx::types::Json(metadata));
            has_updates = true;
        }

        if let Some(is_active) = update.is_active {
            query_builder.push(", is_active = ");
            query_builder.push_bind(is_active);
            has_updates = true;
        }

        if !has_updates {
            return self.find_by_id(id).await;
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let user = query_builder
            .build_query_as::<User>()
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    /// Update password hash
    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Mark email as verified
    pub async fn verify_email(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Mark phone as verified
    pub async fn verify_phone(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET phone_verified = true, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Soft delete user (deactivate)
    pub async fn deactivate(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Hard delete user
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
