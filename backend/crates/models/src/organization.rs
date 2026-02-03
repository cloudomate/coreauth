use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Organization (tenant) with hierarchical support
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub isolation_mode: IsolationMode,
    pub custom_domain: Option<String>,

    #[sqlx(json)]
    pub settings: OrganizationSettings,

    // Hierarchy fields
    pub parent_organization_id: Option<Uuid>,
    pub hierarchy_level: i32,
    pub hierarchy_path: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create new organization request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrganization {
    #[validate(length(min = 3, max = 63))]
    pub slug: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub parent_organization_id: Option<Uuid>,

    pub isolation_mode: Option<IsolationMode>,

    #[validate(url)]
    pub custom_domain: Option<String>,

    pub settings: Option<OrganizationSettings>,
}

/// Update organization request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateOrganization {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub isolation_mode: Option<IsolationMode>,

    #[validate(url)]
    pub custom_domain: Option<String>,

    pub settings: Option<OrganizationSettings>,

    pub parent_organization_id: Option<Uuid>,
}

/// Isolation mode for multi-tenancy
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum IsolationMode {
    Pool,  // Shared database schema
    Silo,  // Dedicated database schema
}

impl Default for IsolationMode {
    fn default() -> Self {
        IsolationMode::Pool
    }
}

/// Organization settings (JSON stored in database)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationSettings {
    #[serde(default)]
    pub branding: BrandingSettings,

    #[serde(default)]
    pub security: SecuritySettings,

    #[serde(default)]
    pub features: FeatureFlags,

    /// Primary SSO connection for this organization
    pub sso_connection_id: Option<Uuid>,
}

/// Branding customization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandingSettings {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub favicon_url: Option<String>,
    pub custom_css: Option<String>,
}

/// Security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    #[serde(default = "default_mfa_required")]
    pub mfa_required: bool,

    #[serde(default)]
    pub mfa_enforcement_date: Option<DateTime<Utc>>,

    #[serde(default = "default_mfa_grace_period_days")]
    pub mfa_grace_period_days: i32,

    #[serde(default)]
    pub allowed_mfa_methods: Vec<String>,

    #[serde(default = "default_password_min_length")]
    pub password_min_length: usize,

    #[serde(default = "default_session_timeout_hours")]
    pub session_timeout_hours: i64,

    #[serde(default)]
    pub password_require_uppercase: bool,

    #[serde(default)]
    pub password_require_lowercase: bool,

    #[serde(default)]
    pub password_require_number: bool,

    #[serde(default)]
    pub password_require_special: bool,

    #[serde(default = "default_max_login_attempts")]
    pub max_login_attempts: i32,

    #[serde(default = "default_lockout_duration_minutes")]
    pub lockout_duration_minutes: i32,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            mfa_required: false,
            mfa_enforcement_date: None,
            mfa_grace_period_days: 7,
            allowed_mfa_methods: vec!["totp".to_string(), "sms".to_string()],
            password_min_length: 8,
            session_timeout_hours: 24,
            password_require_uppercase: false,
            password_require_lowercase: false,
            password_require_number: false,
            password_require_special: false,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
        }
    }
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    #[serde(default)]
    pub sso_enabled: bool,

    #[serde(default)]
    pub api_access_enabled: bool,

    #[serde(default)]
    pub webhooks_enabled: bool,

    #[serde(default)]
    pub actions_enabled: bool,
}

// Default functions for serde
fn default_mfa_required() -> bool {
    false
}

fn default_mfa_grace_period_days() -> i32 {
    7
}

fn default_password_min_length() -> usize {
    8
}

fn default_session_timeout_hours() -> i64 {
    24
}

fn default_max_login_attempts() -> i32 {
    5
}

fn default_lockout_duration_minutes() -> i32 {
    15
}

// Type aliases for backward compatibility
pub type Tenant = Organization;
pub type NewTenant = CreateOrganization;
pub type TenantSettings = OrganizationSettings;
