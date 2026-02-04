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

/// Social provider connection configuration (Google, GitHub, Microsoft, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConnectionConfig {
    pub provider: SocialProvider,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}

/// Supported social providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SocialProvider {
    Google,
    Github,
    Microsoft,
    Facebook,
    Apple,
    LinkedIn,
}

impl SocialProvider {
    pub fn authorization_url(&self) -> &'static str {
        match self {
            SocialProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            SocialProvider::Github => "https://github.com/login/oauth/authorize",
            SocialProvider::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            SocialProvider::Facebook => "https://www.facebook.com/v18.0/dialog/oauth",
            SocialProvider::Apple => "https://appleid.apple.com/auth/authorize",
            SocialProvider::LinkedIn => "https://www.linkedin.com/oauth/v2/authorization",
        }
    }

    pub fn token_url(&self) -> &'static str {
        match self {
            SocialProvider::Google => "https://oauth2.googleapis.com/token",
            SocialProvider::Github => "https://github.com/login/oauth/access_token",
            SocialProvider::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            SocialProvider::Facebook => "https://graph.facebook.com/v18.0/oauth/access_token",
            SocialProvider::Apple => "https://appleid.apple.com/auth/token",
            SocialProvider::LinkedIn => "https://www.linkedin.com/oauth/v2/accessToken",
        }
    }

    pub fn userinfo_url(&self) -> &'static str {
        match self {
            SocialProvider::Google => "https://www.googleapis.com/oauth2/v3/userinfo",
            SocialProvider::Github => "https://api.github.com/user",
            SocialProvider::Microsoft => "https://graph.microsoft.com/v1.0/me",
            SocialProvider::Facebook => "https://graph.facebook.com/me?fields=id,name,email,picture",
            SocialProvider::Apple => "", // Apple uses ID token claims
            SocialProvider::LinkedIn => "https://api.linkedin.com/v2/userinfo",
        }
    }

    pub fn default_scopes(&self) -> Vec<&'static str> {
        match self {
            SocialProvider::Google => vec!["openid", "email", "profile"],
            SocialProvider::Github => vec!["user:email", "read:user"],
            SocialProvider::Microsoft => vec!["openid", "email", "profile", "User.Read"],
            SocialProvider::Facebook => vec!["email", "public_profile"],
            SocialProvider::Apple => vec!["name", "email"],
            SocialProvider::LinkedIn => vec!["openid", "email", "profile"],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SocialProvider::Google => "google",
            SocialProvider::Github => "github",
            SocialProvider::Microsoft => "microsoft",
            SocialProvider::Facebook => "facebook",
            SocialProvider::Apple => "apple",
            SocialProvider::LinkedIn => "linkedin",
        }
    }
}

impl std::fmt::Display for SocialProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// User info returned from social providers (normalized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialUserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub picture: Option<String>,
    pub raw: serde_json::Value,
}

/// Connection types
pub mod types {
    pub const DATABASE: &str = "database";
    pub const OIDC: &str = "oidc";
    pub const SAML: &str = "saml";
    pub const OAUTH2: &str = "oauth2";
    pub const SOCIAL: &str = "social";
}
