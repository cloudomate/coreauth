use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Organization member (many-to-many relationship between users and organizations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: String,  // Organization-scoped role: 'admin', 'member', 'viewer', etc.
    pub joined_at: DateTime<Utc>,
}

/// Request to add a user to an organization
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddOrganizationMember {
    pub user_id: Uuid,
    pub organization_id: Uuid,

    #[validate(length(min = 1, max = 50))]
    pub role: String,
}

/// Request to update a member's role
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateMemberRole {
    #[validate(length(min = 1, max = 50))]
    pub role: String,
}

/// Organization member with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMemberWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,

    // User details
    pub email: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub mfa_enabled: bool,
}

/// Common organization roles
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const MEMBER: &str = "member";
    pub const VIEWER: &str = "viewer";
    pub const BILLING: &str = "billing";
}
