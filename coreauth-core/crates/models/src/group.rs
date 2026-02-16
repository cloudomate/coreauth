use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User group within a tenant
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub tenant_id: Uuid,

    pub name: String,
    pub slug: String,
    pub description: Option<String>,

    pub group_type: String,
    pub default_role_id: Option<Uuid>,

    #[sqlx(default)]
    pub metadata: serde_json::Value,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Group member
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,

    pub role: String,
    pub added_at: DateTime<Utc>,
    pub added_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Group member with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberWithUser {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub added_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,

    // User details
    pub email: String,
    pub full_name: Option<String>,
}

/// Group with member count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithMemberCount {
    #[serde(flatten)]
    pub group: Group,
    pub member_count: i64,
}

/// Create group request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateGroup {
    /// Tenant ID - set from URL path, not from request body
    #[serde(default)]
    pub tenant_id: Uuid,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 1, max = 255), regex(path = *SLUG_REGEX))]
    pub slug: String,

    pub description: Option<String>,

    pub default_role_id: Option<Uuid>,

    pub metadata: Option<serde_json::Value>,
}

/// Update group request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateGroup {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub default_role_id: Option<Uuid>,

    pub metadata: Option<serde_json::Value>,

    pub is_active: Option<bool>,
}

/// Add member to group request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGroupMember {
    pub user_id: Uuid,
    pub role: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Update group member request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupMember {
    pub role: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Group role assignment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupRole {
    pub id: Uuid,
    pub group_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// Slug validation regex
lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}
