use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Authentication connection (database, OIDC, SAML)
/// Can be platform-level (available to all) or organization-level (specific org only)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connection {
    pub id: Uuid,
    pub name: String,

    #[sqlx(rename = "type")]
    pub connection_type: String,  // 'database', 'oidc', 'saml', 'oauth2'

    pub scope: ConnectionScope,
    pub organization_id: Option<Uuid>,  // NULL for platform-level, UUID for org-level

    #[sqlx(json)]
    pub config: serde_json::Value,

    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Connection scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionScope {
    Platform,      // Available to all users (e.g., username/password)
    Organization,  // Only for specific organization (e.g., customer's SSO)
}

impl sqlx::Type<sqlx::Postgres> for ConnectionScope {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("text")
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ConnectionScope {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "platform" => Ok(ConnectionScope::Platform),
            "organization" => Ok(ConnectionScope::Organization),
            _ => Err(format!("Invalid connection scope: {}", s).into()),
        }
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ConnectionScope {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            ConnectionScope::Platform => "platform",
            ConnectionScope::Organization => "organization",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

/// Request to create a new connection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateConnection {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 1))]
    pub connection_type: String,  // 'database', 'oidc', 'saml', 'oauth2'

    pub scope: ConnectionScope,
    pub organization_id: Option<Uuid>,

    pub config: serde_json::Value,
}

/// Request to update a connection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateConnection {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub config: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
}

/// OIDC connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConnectionConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
    pub scopes: Vec<String>,
    pub claim_mappings: ClaimMappings,
    pub groups_claim: Option<String>,
    pub group_role_mappings: Option<serde_json::Value>,
}

/// SAML connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConnectionConfig {
    pub sso_url: String,
    pub entity_id: String,
    pub certificate: String,
    pub sign_requests: bool,
    pub signature_algorithm: String,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimMappings {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

impl Default for ClaimMappings {
    fn default() -> Self {
        Self {
            email: "email".to_string(),
            first_name: Some("given_name".to_string()),
            last_name: Some("family_name".to_string()),
            phone: Some("phone_number".to_string()),
        }
    }
}

/// Connection types
pub mod types {
    pub const DATABASE: &str = "database";
    pub const OIDC: &str = "oidc";
    pub const SAML: &str = "saml";
    pub const OAUTH2: &str = "oauth2";
}
