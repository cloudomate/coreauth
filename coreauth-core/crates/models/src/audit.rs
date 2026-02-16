use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "audit_event_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditEventCategory {
    Authentication,
    Authorization,
    UserManagement,
    TenantManagement,
    Security,
    Admin,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuditStatus {
    Success,
    Failure,
    Error,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditStatus::Success => write!(f, "success"),
            AuditStatus::Failure => write!(f, "failure"),
            AuditStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Uuid,

    // Event information
    pub event_type: String,
    pub event_category: AuditEventCategory,
    pub event_action: String,

    // Actor
    pub actor_type: Option<String>,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub actor_ip_address: Option<String>,
    pub actor_user_agent: Option<String>,

    // Target
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,

    // Details
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,

    // Result
    pub status: String,
    pub error_message: Option<String>,

    // Context
    pub request_id: Option<String>,
    pub session_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLog {
    pub tenant_id: Uuid,

    // Event information
    pub event_type: String,
    pub event_category: AuditEventCategory,
    pub event_action: String,

    // Actor
    pub actor_type: Option<String>,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub actor_ip_address: Option<IpAddr>,
    pub actor_user_agent: Option<String>,

    // Target
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,

    // Details
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,

    // Result
    pub status: AuditStatus,
    pub error_message: Option<String>,

    // Context
    pub request_id: Option<String>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogQuery {
    pub tenant_id: Uuid,
    pub event_types: Option<Vec<String>>,
    pub event_categories: Option<Vec<AuditEventCategory>>,
    pub actor_id: Option<String>,
    pub target_id: Option<String>,
    pub status: Option<AuditStatus>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            tenant_id: Uuid::nil(),
            event_types: None,
            event_categories: None,
            actor_id: None,
            target_id: None,
            status: None,
            from_date: None,
            to_date: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

// Helper builders for common audit events
pub struct AuditLogBuilder {
    log: CreateAuditLog,
}

impl AuditLogBuilder {
    pub fn new(tenant_id: Uuid, event_type: impl Into<String>, category: AuditEventCategory) -> Self {
        let event_type = event_type.into();
        let parts: Vec<&str> = event_type.split('.').collect();
        let action = parts.last().unwrap_or(&"unknown").to_string();

        Self {
            log: CreateAuditLog {
                tenant_id,
                event_type,
                event_category: category,
                event_action: action,
                actor_type: None,
                actor_id: None,
                actor_name: None,
                actor_ip_address: None,
                actor_user_agent: None,
                target_type: None,
                target_id: None,
                target_name: None,
                description: None,
                metadata: None,
                status: AuditStatus::Success,
                error_message: None,
                request_id: None,
                session_id: None,
            },
        }
    }

    pub fn actor(mut self, actor_type: impl Into<String>, actor_id: impl Into<String>) -> Self {
        self.log.actor_type = Some(actor_type.into());
        self.log.actor_id = Some(actor_id.into());
        self
    }

    pub fn actor_name(mut self, name: impl Into<String>) -> Self {
        self.log.actor_name = Some(name.into());
        self
    }

    pub fn actor_ip(mut self, ip: IpAddr) -> Self {
        self.log.actor_ip_address = Some(ip);
        self
    }

    pub fn actor_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.log.actor_user_agent = Some(ua.into());
        self
    }

    pub fn target(mut self, target_type: impl Into<String>, target_id: impl Into<String>) -> Self {
        self.log.target_type = Some(target_type.into());
        self.log.target_id = Some(target_id.into());
        self
    }

    pub fn target_name(mut self, name: impl Into<String>) -> Self {
        self.log.target_name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.log.description = Some(desc.into());
        self
    }

    pub fn metadata(mut self, data: serde_json::Value) -> Self {
        self.log.metadata = Some(data);
        self
    }

    pub fn status(mut self, status: AuditStatus) -> Self {
        self.log.status = status;
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.log.status = AuditStatus::Error;
        self.log.error_message = Some(error.into());
        self
    }

    pub fn failure(mut self, reason: impl Into<String>) -> Self {
        self.log.status = AuditStatus::Failure;
        self.log.error_message = Some(reason.into());
        self
    }

    pub fn request_id(mut self, id: impl Into<String>) -> Self {
        self.log.request_id = Some(id.into());
        self
    }

    pub fn session_id(mut self, id: Uuid) -> Self {
        self.log.session_id = Some(id);
        self
    }

    pub fn build(self) -> CreateAuditLog {
        self.log
    }
}

// Common event type constants
pub mod events {
    // Authentication events
    pub const USER_LOGIN: &str = "user.login";
    pub const USER_LOGOUT: &str = "user.logout";
    pub const USER_LOGIN_FAILED: &str = "user.login.failed";
    pub const USER_REGISTERED: &str = "user.registered";
    pub const PASSWORD_RESET_REQUESTED: &str = "password.reset.requested";
    pub const PASSWORD_RESET_COMPLETED: &str = "password.reset.completed";
    pub const PASSWORD_CHANGED: &str = "password.changed";
    pub const EMAIL_VERIFIED: &str = "email.verified";

    // MFA events
    pub const MFA_ENROLLED: &str = "mfa.enrolled";
    pub const MFA_VERIFIED: &str = "mfa.verified";
    pub const MFA_VERIFICATION_FAILED: &str = "mfa.verification.failed";
    pub const MFA_DISABLED: &str = "mfa.disabled";
    pub const MFA_BACKUP_CODES_GENERATED: &str = "mfa.backup_codes.generated";

    // User management events
    pub const USER_CREATED: &str = "user.created";
    pub const USER_UPDATED: &str = "user.updated";
    pub const USER_DELETED: &str = "user.deleted";
    pub const USER_SUSPENDED: &str = "user.suspended";
    pub const USER_ACTIVATED: &str = "user.activated";
    pub const USER_INVITED: &str = "user.invited";
    pub const INVITATION_ACCEPTED: &str = "invitation.accepted";

    // Role/Permission events
    pub const ROLE_CREATED: &str = "role.created";
    pub const ROLE_UPDATED: &str = "role.updated";
    pub const ROLE_DELETED: &str = "role.deleted";
    pub const ROLE_ASSIGNED: &str = "role.assigned";
    pub const ROLE_REVOKED: &str = "role.revoked";

    // Authorization events
    pub const PERMISSION_CHECKED: &str = "permission.checked";
    pub const PERMISSION_GRANTED: &str = "permission.granted";
    pub const PERMISSION_DENIED: &str = "permission.denied";
    pub const TUPLE_CREATED: &str = "tuple.created";
    pub const TUPLE_DELETED: &str = "tuple.deleted";

    // Application events
    pub const APPLICATION_CREATED: &str = "application.created";
    pub const APPLICATION_UPDATED: &str = "application.updated";
    pub const APPLICATION_DELETED: &str = "application.deleted";
    pub const APPLICATION_SECRET_ROTATED: &str = "application.secret.rotated";
    pub const APPLICATION_AUTHENTICATED: &str = "application.authenticated";

    // Tenant events
    pub const TENANT_CREATED: &str = "tenant.created";
    pub const TENANT_UPDATED: &str = "tenant.updated";
    pub const TENANT_DELETED: &str = "tenant.deleted";
    pub const TENANT_SETTINGS_CHANGED: &str = "tenant.settings.changed";

    // Security events
    pub const ACCOUNT_LOCKED: &str = "account.locked";
    pub const ACCOUNT_UNLOCKED: &str = "account.unlocked";
    pub const SUSPICIOUS_ACTIVITY: &str = "security.suspicious_activity";
    pub const INVALID_TOKEN: &str = "security.invalid_token";
    pub const TOKEN_EXPIRED: &str = "security.token_expired";
}
