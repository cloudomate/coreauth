use ciam_database::repositories::audit::AuditRepository;
use ciam_models::{AuditLog, CreateAuditLog, AuditLogQuery, AuditLogBuilder, AuditEventCategory, AuditStatus};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    repository: AuditRepository,
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: AuditRepository::new(pool),
        }
    }

    /// Log an audit event
    pub async fn log(&self, log: CreateAuditLog) -> Result<AuditLog, crate::error::AuthError> {
        self.repository
            .create(log)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to create audit log: {}", e)))
    }

    /// Log an audit event with builder pattern (convenience method)
    pub async fn log_event(
        &self,
        tenant_id: Uuid,
        event_type: impl Into<String>,
        category: AuditEventCategory,
    ) -> AuditLogBuilder {
        AuditLogBuilder::new(tenant_id, event_type, category)
    }

    /// Query audit logs
    pub async fn query(&self, query: AuditLogQuery) -> Result<Vec<AuditLog>, crate::error::AuthError> {
        self.repository
            .query(query)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to query audit logs: {}", e)))
    }

    /// Get audit log by ID
    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<AuditLog>, crate::error::AuthError> {
        self.repository
            .get_by_id(tenant_id, id)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to get audit log: {}", e)))
    }

    /// Get audit log count for tenant
    pub async fn count(&self, tenant_id: Uuid) -> Result<i64, crate::error::AuthError> {
        self.repository
            .count(tenant_id)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to count audit logs: {}", e)))
    }

    /// Get recent security events
    pub async fn get_security_events(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLog>, crate::error::AuthError> {
        self.repository
            .get_security_events(tenant_id, limit)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to get security events: {}", e)))
    }

    /// Get failed login attempts for a user
    pub async fn get_failed_logins(
        &self,
        tenant_id: Uuid,
        user_id: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditLog>, crate::error::AuthError> {
        self.repository
            .get_failed_logins(tenant_id, user_id, since)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to get failed logins: {}", e)))
    }

    /// Export audit logs for a date range (for compliance/archival)
    pub async fn export(
        &self,
        tenant_id: Uuid,
        from_date: chrono::DateTime<chrono::Utc>,
        to_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditLog>, crate::error::AuthError> {
        self.repository
            .export(tenant_id, from_date, to_date)
            .await
            .map_err(|e| crate::error::AuthError::Internal(format!("Failed to export audit logs: {}", e)))
    }

    /// Helper method to log a successful authentication event
    pub async fn log_login_success(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        user_email: &str,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<(), crate::error::AuthError> {
        let log = AuditLogBuilder::new(tenant_id, ciam_models::audit::events::USER_LOGIN, AuditEventCategory::Authentication)
            .actor("user", user_id.to_string())
            .actor_name(user_email)
            .status(AuditStatus::Success)
            .description(format!("User {} logged in successfully", user_email));

        let log = if let Some(ip) = ip_address {
            log.actor_ip(ip).build()
        } else {
            log.build()
        };

        self.log(log).await?;
        Ok(())
    }

    /// Helper method to log a failed authentication event
    pub async fn log_login_failure(
        &self,
        tenant_id: Uuid,
        email: &str,
        reason: &str,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<(), crate::error::AuthError> {
        let log = AuditLogBuilder::new(tenant_id, ciam_models::audit::events::USER_LOGIN_FAILED, AuditEventCategory::Security)
            .actor("user", email)
            .failure(reason)
            .description(format!("Login failed for {}: {}", email, reason));

        let log = if let Some(ip) = ip_address {
            log.actor_ip(ip).build()
        } else {
            log.build()
        };

        self.log(log).await?;
        Ok(())
    }

    /// Helper method to log user registration
    pub async fn log_registration(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        user_email: &str,
        ip_address: Option<std::net::IpAddr>,
    ) -> Result<(), crate::error::AuthError> {
        let log = AuditLogBuilder::new(tenant_id, ciam_models::audit::events::USER_REGISTERED, AuditEventCategory::UserManagement)
            .actor("user", user_id.to_string())
            .actor_name(user_email)
            .target("user", user_id.to_string())
            .target_name(user_email)
            .status(AuditStatus::Success)
            .description(format!("User {} registered", user_email));

        let log = if let Some(ip) = ip_address {
            log.actor_ip(ip).build()
        } else {
            log.build()
        };

        self.log(log).await?;
        Ok(())
    }
}
