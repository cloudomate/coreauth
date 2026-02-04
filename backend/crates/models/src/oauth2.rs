use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// AUTHORIZATION CODE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub nonce: Option<String>,
    pub state: Option<String>,
    pub response_type: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorizationCode {
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub nonce: Option<String>,
    pub state: Option<String>,
    pub response_type: String,
}

// ============================================================================
// REFRESH TOKEN
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub token_hash: String,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub scope: Option<String>,
    pub audience: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRefreshToken {
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub scope: Option<String>,
    pub audience: Option<String>,
    pub expires_in_seconds: Option<i64>,
}

// ============================================================================
// ACCESS TOKEN (Audit)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessTokenRecord {
    pub id: Uuid,
    pub jti: String,
    pub client_id: Uuid,
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub scope: Option<String>,
    pub audience: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// CONSENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OAuthConsent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

// ============================================================================
// LOGIN SESSION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoginSession {
    pub id: Uuid,
    pub session_token_hash: String,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub authenticated_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub mfa_verified: bool,
    pub mfa_verified_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLoginSession {
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub mfa_verified: bool,
    pub expires_in_seconds: i64,
}

// ============================================================================
// AUTHORIZATION REQUEST (Pending)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorizationRequest {
    pub id: Uuid,
    pub request_id: String,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub nonce: Option<String>,
    pub organization_id: Option<Uuid>,
    pub connection_hint: Option<String>,
    pub login_hint: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<i32>,
    pub ui_locales: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorizationRequest {
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub nonce: Option<String>,
    pub organization_id: Option<Uuid>,
    pub connection_hint: Option<String>,
    pub login_hint: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<i32>,
    pub ui_locales: Option<String>,
}

// ============================================================================
// SIGNING KEY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SigningKey {
    pub id: String,
    pub algorithm: String,
    pub public_key_pem: String,
    #[serde(skip_serializing)]
    pub private_key_pem: String,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// OIDC DISCOVERY
// ============================================================================

#[derive(Debug, Serialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub revocation_endpoint: Option<String>,
    pub introspection_endpoint: Option<String>,
    pub end_session_endpoint: Option<String>,
}

impl OidcDiscovery {
    pub fn new(issuer: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
            authorization_endpoint: format!("{}/authorize", issuer),
            token_endpoint: format!("{}/oauth/token", issuer),
            userinfo_endpoint: format!("{}/userinfo", issuer),
            jwks_uri: format!("{}/.well-known/jwks.json", issuer),
            registration_endpoint: None,
            scopes_supported: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
                "offline_access".to_string(),
            ],
            response_types_supported: vec![
                "code".to_string(),
                "token".to_string(),
                "id_token".to_string(),
                "code token".to_string(),
                "code id_token".to_string(),
                "token id_token".to_string(),
                "code token id_token".to_string(),
            ],
            response_modes_supported: vec![
                "query".to_string(),
                "fragment".to_string(),
                "form_post".to_string(),
            ],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
                "client_credentials".to_string(),
            ],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_basic".to_string(),
                "client_secret_post".to_string(),
                "none".to_string(),
            ],
            claims_supported: vec![
                "sub".to_string(),
                "iss".to_string(),
                "aud".to_string(),
                "exp".to_string(),
                "iat".to_string(),
                "auth_time".to_string(),
                "nonce".to_string(),
                "acr".to_string(),
                "amr".to_string(),
                "azp".to_string(),
                "email".to_string(),
                "email_verified".to_string(),
                "name".to_string(),
                "given_name".to_string(),
                "family_name".to_string(),
                "picture".to_string(),
                "locale".to_string(),
                "updated_at".to_string(),
                "org_id".to_string(),
                "org_name".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string(), "plain".to_string()],
            revocation_endpoint: Some(format!("{}/oauth/revoke", issuer)),
            introspection_endpoint: Some(format!("{}/oauth/introspect", issuer)),
            end_session_endpoint: Some(format!("{}/logout", issuer)),
        }
    }
}

// ============================================================================
// JWKS (JSON Web Key Set)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Serialize)]
pub struct Jwk {
    pub kty: String,       // Key type: "RSA"
    pub r#use: String,     // Usage: "sig"
    pub kid: String,       // Key ID
    pub alg: String,       // Algorithm: "RS256"
    pub n: String,         // RSA modulus (base64url)
    pub e: String,         // RSA exponent (base64url)
}

// ============================================================================
// TOKEN REQUEST / RESPONSE
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub audience: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

impl TokenError {
    pub fn new(error: &str, description: &str) -> Self {
        Self {
            error: error.to_string(),
            error_description: Some(description.to_string()),
            error_uri: None,
        }
    }

    pub fn invalid_request(description: &str) -> Self {
        Self::new("invalid_request", description)
    }

    pub fn invalid_client(description: &str) -> Self {
        Self::new("invalid_client", description)
    }

    pub fn invalid_grant(description: &str) -> Self {
        Self::new("invalid_grant", description)
    }

    pub fn unauthorized_client(description: &str) -> Self {
        Self::new("unauthorized_client", description)
    }

    pub fn unsupported_grant_type(description: &str) -> Self {
        Self::new("unsupported_grant_type", description)
    }

    pub fn invalid_scope(description: &str) -> Self {
        Self::new("invalid_scope", description)
    }
}

// ============================================================================
// USERINFO RESPONSE
// ============================================================================

#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_name: Option<String>,
}

// ============================================================================
// AUTHORIZE REQUEST (Query Parameters)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_challenge_method: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub connection: Option<String>,
    #[serde(default)]
    pub login_hint: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub max_age: Option<i32>,
    #[serde(default)]
    pub ui_locales: Option<String>,
    #[serde(default)]
    pub audience: Option<String>,
}

// ============================================================================
// INTROSPECTION
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct IntrospectionRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

// ============================================================================
// REVOCATION
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RevocationRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>,
}
