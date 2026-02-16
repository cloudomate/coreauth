use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// OAuth application/client
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,

    /// Tenant that owns this application (NULL = platform-level)
    /// Note: DB column is tenant_id, but API uses organization_id for consistency
    #[sqlx(rename = "tenant_id")]
    pub organization_id: Option<Uuid>,

    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,

    pub app_type: ApplicationType,

    pub client_id: String,

    /// Hashed client secret (only shown plaintext on create/rotate)
    #[sqlx(rename = "client_secret_hash")]
    pub client_secret: Option<String>,

    #[sqlx(default)]
    pub callback_urls: Vec<String>,

    #[sqlx(rename = "allowed_logout_urls", default)]
    pub logout_urls: Vec<String>,

    #[sqlx(rename = "allowed_web_origins", default)]
    pub web_origins: Vec<String>,

    /// Allowed OAuth2 grant types (authorization_code, refresh_token, client_credentials)
    #[sqlx(default)]
    pub grant_types: Vec<String>,

    /// Response types
    #[sqlx(default)]
    pub response_types: Vec<String>,

    /// Allowed scopes for this application
    #[sqlx(default)]
    pub allowed_scopes: Vec<String>,

    pub token_endpoint_auth_method: Option<String>,

    #[sqlx(rename = "access_token_ttl_seconds")]
    pub access_token_lifetime_seconds: i32,

    #[sqlx(rename = "refresh_token_ttl_seconds")]
    pub refresh_token_lifetime_seconds: i32,

    pub id_token_ttl_seconds: i32,

    #[sqlx(rename = "is_active")]
    pub is_enabled: bool,

    pub is_first_party: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Application with plaintext client secret (only returned on create/rotate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationWithSecret {
    #[serde(flatten)]
    pub application: Application,

    /// Plaintext client secret - store this securely!
    pub client_secret_plain: String,
}

/// Create new application request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateApplication {
    pub organization_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 3, max = 63), regex(path = *SLUG_REGEX))]
    pub slug: String,

    pub description: Option<String>,

    pub logo_url: Option<String>,

    pub app_type: ApplicationType,

    #[validate(length(min = 1))]
    pub callback_urls: Vec<String>,

    pub logout_urls: Option<Vec<String>>,

    pub web_origins: Option<Vec<String>>,

    pub access_token_lifetime_seconds: Option<i32>,

    pub refresh_token_lifetime_seconds: Option<i32>,

    /// Allowed OAuth2 grant types
    pub grant_types: Option<Vec<String>>,

    /// Allowed scopes for this application
    pub allowed_scopes: Option<Vec<String>>,
}

/// Update application request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateApplication {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub logo_url: Option<String>,

    pub callback_urls: Option<Vec<String>>,

    pub logout_urls: Option<Vec<String>>,

    pub web_origins: Option<Vec<String>>,

    pub access_token_lifetime_seconds: Option<i32>,

    pub refresh_token_lifetime_seconds: Option<i32>,

    /// Allowed OAuth2 grant types
    pub grant_types: Option<Vec<String>>,

    /// Allowed scopes for this application
    pub allowed_scopes: Option<Vec<String>>,

    pub is_enabled: Option<bool>,
}

/// Application type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationType {
    /// Web application (server-side)
    #[serde(rename = "webapp")]
    Web,

    /// Single Page Application (browser-based)
    Spa,

    /// Native/mobile application
    Native,

    /// Machine-to-machine API
    #[serde(rename = "m2m")]
    Api,
}

impl std::fmt::Display for ApplicationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationType::Web => write!(f, "webapp"),
            ApplicationType::Spa => write!(f, "spa"),
            ApplicationType::Native => write!(f, "native"),
            ApplicationType::Api => write!(f, "m2m"),
        }
    }
}

// SQLx implementation for ApplicationType
impl sqlx::Type<sqlx::Postgres> for ApplicationType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("application_type")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApplicationType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "webapp" | "web" => Ok(ApplicationType::Web),
            "spa" => Ok(ApplicationType::Spa),
            "native" => Ok(ApplicationType::Native),
            "m2m" | "api" => Ok(ApplicationType::Api),
            _ => Err(format!("Invalid application type: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ApplicationType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s.as_str(), buf)
    }
}

// Slug validation regex
lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}
