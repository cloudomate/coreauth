use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// OIDC provider configuration for external identity providers
/// This allows users to sign in with providers like Azure Entra ID, Google, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OidcProvider {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub provider_type: String, // "azure", "google", "okta", "custom"
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String, // Encrypted in production
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
    pub scopes: Vec<String>,
    #[sqlx(json)]
    pub claim_mappings: ClaimMappings,
    pub groups_claim: Option<String>, // Claim name for groups (e.g., "groups", "roles")
    #[sqlx(json)]
    pub group_role_mappings: Option<Value>, // Map OIDC groups to tenant roles
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimMappings {
    pub email: String,          // e.g., "email" or "preferred_username"
    pub first_name: Option<String>, // e.g., "given_name"
    pub last_name: Option<String>,  // e.g., "family_name"
    pub phone: Option<String>,      // e.g., "phone_number"
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewOidcProvider {
    pub tenant_id: Uuid,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 1))]
    pub provider_type: String,

    #[validate(url)]
    pub issuer: String,

    #[validate(length(min = 1))]
    pub client_id: String,

    #[validate(length(min = 1))]
    pub client_secret: String,

    #[validate(url)]
    pub authorization_endpoint: String,

    #[validate(url)]
    pub token_endpoint: String,

    pub userinfo_endpoint: Option<String>,

    #[validate(url)]
    pub jwks_uri: String,

    pub scopes: Vec<String>,

    pub claim_mappings: Option<ClaimMappings>,
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub nonce: String,
    pub provider_id: Uuid,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
}

/// ID token claims from OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nonce: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub phone_number: Option<String>,
}
