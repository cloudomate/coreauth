use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// OAuth application/client
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,

    /// Organization that owns this application (NULL = platform-level)
    pub organization_id: Option<Uuid>,

    pub name: String,
    pub slug: String,
    pub description: Option<String>,

    #[sqlx(rename = "type")]
    pub app_type: ApplicationType,

    pub client_id: String,

    /// Hashed client secret (only shown plaintext on create/rotate)
    pub client_secret: Option<String>,

    #[sqlx(json)]
    pub callback_urls: Vec<String>,

    #[sqlx(json)]
    pub logout_urls: Vec<String>,

    #[sqlx(json)]
    pub web_origins: Vec<String>,

    #[sqlx(json)]
    pub allowed_connections: Vec<Uuid>,

    pub require_organization: bool,
    pub platform_admin_only: bool,

    pub access_token_lifetime_seconds: i32,
    pub refresh_token_lifetime_seconds: i32,

    pub is_enabled: bool,

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

    pub app_type: ApplicationType,

    #[validate(length(min = 1))]
    pub callback_urls: Vec<String>,

    pub logout_urls: Option<Vec<String>>,

    pub web_origins: Option<Vec<String>>,

    pub allowed_connections: Option<Vec<Uuid>>,

    pub require_organization: Option<bool>,

    pub access_token_lifetime_seconds: Option<i32>,

    pub refresh_token_lifetime_seconds: Option<i32>,
}

/// Update application request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateApplication {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub callback_urls: Option<Vec<String>>,

    pub logout_urls: Option<Vec<String>>,

    pub web_origins: Option<Vec<String>>,

    pub allowed_connections: Option<Vec<Uuid>>,

    pub require_organization: Option<bool>,

    pub access_token_lifetime_seconds: Option<i32>,

    pub refresh_token_lifetime_seconds: Option<i32>,

    pub is_enabled: Option<bool>,
}

/// Application type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationType {
    /// Web application (server-side)
    Web,

    /// Single Page Application (browser-based)
    Spa,

    /// Native/mobile application
    Native,

    /// Machine-to-machine API
    Api,
}

impl std::fmt::Display for ApplicationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationType::Web => write!(f, "web"),
            ApplicationType::Spa => write!(f, "spa"),
            ApplicationType::Native => write!(f, "native"),
            ApplicationType::Api => write!(f, "api"),
        }
    }
}

// SQLx implementation for ApplicationType
impl sqlx::Type<sqlx::Postgres> for ApplicationType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("TEXT")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApplicationType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "web" => Ok(ApplicationType::Web),
            "spa" => Ok(ApplicationType::Spa),
            "native" => Ok(ApplicationType::Native),
            "api" => Ok(ApplicationType::Api),
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
