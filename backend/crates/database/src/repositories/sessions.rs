use crate::error::{DatabaseError, Result};
use ciam_models::{NewSession, Session};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new session
    pub async fn create(&self, new_session: &NewSession) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (
                user_id, token_hash, refresh_token_hash,
                device_fingerprint, ip_address, user_agent, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&new_session.user_id)
        .bind(&new_session.token_hash)
        .bind(&new_session.refresh_token_hash)
        .bind(&new_session.device_fingerprint)
        .bind(&new_session.ip_address)
        .bind(&new_session.user_agent)
        .bind(&new_session.expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// Find session by token hash
    pub async fn find_by_token(&self, token_hash: &str) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::NotFound("Session not found or expired".to_string()))?;

        Ok(session)
    }

    /// Find session by refresh token hash
    pub async fn find_by_refresh_token(&self, refresh_token_hash: &str) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE refresh_token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::NotFound("Session not found or expired".to_string()))?;

        Ok(session)
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    /// Delete a session (logout)
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete session by token hash
    pub async fn delete_by_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete all sessions for a user (logout from all devices)
    pub async fn delete_all_user_sessions(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
