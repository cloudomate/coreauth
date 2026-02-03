use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,

    // Optional organization membership (NULL for platform admins)
    pub default_organization_id: Option<Uuid>,

    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub password_hash: Option<String>,

    #[sqlx(json)]
    pub metadata: UserMetadata,

    pub is_active: bool,

    // Platform admin flag (can manage entire system)
    pub is_platform_admin: bool,

    // MFA
    pub mfa_enabled: bool,
    pub mfa_enforced_at: Option<DateTime<Utc>>,

    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewUser {
    // Optional organization (NULL for platform admins)
    pub tenant_id: Option<Uuid>,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: Option<String>,

    pub phone: Option<String>,

    pub metadata: Option<UserMetadata>,

    // Create as platform admin
    #[serde(default)]
    pub is_platform_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserMetadata {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,

    #[serde(flatten)]
    pub custom: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub metadata: UserMetadata,
    pub default_organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            phone: user.phone,
            phone_verified: user.phone_verified,
            metadata: user.metadata,
            default_organization_id: user.default_organization_id,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(email)]
    pub email: Option<String>,

    pub phone: Option<String>,

    pub metadata: Option<UserMetadata>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChangePassword {
    #[validate(length(min = 8))]
    pub current_password: String,

    #[validate(length(min = 8))]
    pub new_password: String,
}
