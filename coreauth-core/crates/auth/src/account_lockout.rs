use crate::error::{AuthError, Result};
use ciam_database::Database;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub struct AccountLockoutService {
    db: Database,
}

impl AccountLockoutService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Check if account is currently locked
    pub async fn is_locked(&self, user_id: Uuid) -> Result<Option<DateTime<Utc>>> {
        let locked_until: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT locked_until
            FROM account_lockouts
            WHERE user_id = $1
              AND locked_until > NOW()
              AND unlocked_at IS NULL
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(locked_until)
    }

    /// Record a login attempt
    pub async fn record_attempt(
        &self,
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,  // Nullable for platform admin logins
        email: &str,
        ip_address: &str,
        user_agent: Option<&str>,
        successful: bool,
        failure_reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO login_attempts
                (user_id, tenant_id, email, ip_address, successful, failure_reason, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(email)
        .bind(ip_address)
        .bind(successful)
        .bind(failure_reason)
        .bind(user_agent)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// Get failed login attempts in the last N minutes
    pub async fn get_failed_attempts(
        &self,
        user_id: Uuid,
        minutes: i64,
    ) -> Result<i64> {
        let since = Utc::now() - Duration::minutes(minutes);

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM login_attempts
            WHERE user_id = $1
              AND successful = false
              AND attempted_at > $2
            "#,
        )
        .bind(user_id)
        .bind(since)
        .fetch_one(self.db.pool())
        .await?;

        Ok(count)
    }

    /// Lock an account due to failed attempts
    pub async fn lock_account(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,  // Nullable for platform admin lockouts
        duration_minutes: i32,
        reason: &str,
    ) -> Result<DateTime<Utc>> {
        let locked_until = Utc::now() + Duration::minutes(duration_minutes as i64);

        sqlx::query(
            r#"
            INSERT INTO account_lockouts (user_id, tenant_id, locked_until, reason)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(locked_until)
        .bind(reason)
        .execute(self.db.pool())
        .await?;

        tracing::warn!(
            "Account locked: user_id={}, tenant_id={:?}, until={}, reason={}",
            user_id,
            tenant_id,
            locked_until,
            reason
        );

        Ok(locked_until)
    }

    /// Manually unlock an account (admin action)
    pub async fn unlock_account(&self, user_id: Uuid, unlocked_by: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE account_lockouts
            SET unlocked_at = NOW(), unlocked_by = $1
            WHERE user_id = $2
              AND unlocked_at IS NULL
            "#,
        )
        .bind(unlocked_by)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// Check and handle failed login attempt (with auto-lockout)
    pub async fn handle_failed_login(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,  // Nullable for platform admin failed logins
        email: &str,
        ip_address: &str,
        user_agent: Option<&str>,
        max_attempts: i32,
        lockout_duration_minutes: i32,
    ) -> Result<()> {
        // Record failed attempt
        self.record_attempt(
            Some(user_id),
            tenant_id,
            email,
            ip_address,
            user_agent,
            false,
            Some("invalid_credentials"),
        )
        .await?;

        // Check failed attempts in last 15 minutes
        let failed_count = self.get_failed_attempts(user_id, 15).await?;

        // Lock account if threshold exceeded
        if failed_count >= max_attempts as i64 {
            self.lock_account(
                user_id,
                tenant_id,
                lockout_duration_minutes,
                &format!("Automatic lockout after {} failed attempts", failed_count),
            )
            .await?;

            return Err(AuthError::AccountLocked {
                locked_until: Utc::now() + Duration::minutes(lockout_duration_minutes as i64),
            });
        }

        Ok(())
    }

    /// Check if account is banned
    pub async fn is_banned(
        &self,
        tenant_id: Option<Uuid>,  // Nullable for platform admin ban checks
        user_id: Option<Uuid>,
        email: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<bool> {
        let mut conditions = vec!["unbanned_at IS NULL".to_string()];
        let mut param_index = 1;

        // Add tenant_id condition if provided
        if tenant_id.is_some() {
            conditions.insert(0, format!("tenant_id = ${}", param_index));
            param_index += 1;
        } else {
            conditions.insert(0, "tenant_id IS NULL".to_string());
        }

        let query = if user_id.is_some() || email.is_some() || ip_address.is_some() {
            let mut user_conditions = vec![];

            if user_id.is_some() {
                user_conditions.push(format!("user_id = ${}", param_index));
                param_index += 1;
            }
            if email.is_some() {
                user_conditions.push(format!("email = ${}", param_index));
                param_index += 1;
            }
            if ip_address.is_some() {
                user_conditions.push(format!("ip_address = ${}", param_index));
            }

            conditions.push(format!("({})", user_conditions.join(" OR ")));
            format!(
                "SELECT EXISTS(SELECT 1 FROM user_bans WHERE {})",
                conditions.join(" AND ")
            )
        } else {
            return Ok(false);
        };

        let mut query_builder = sqlx::query_scalar::<_, bool>(&query);

        if let Some(tid) = tenant_id {
            query_builder = query_builder.bind(tid);
        }

        if let Some(uid) = user_id {
            query_builder = query_builder.bind(uid);
        }
        if let Some(em) = email {
            query_builder = query_builder.bind(em);
        }
        if let Some(ip) = ip_address {
            query_builder = query_builder.bind(ip);
        }

        let is_banned = query_builder.fetch_one(self.db.pool()).await?;

        Ok(is_banned)
    }
}
