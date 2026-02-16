//! Passwordless authentication models
//!
//! For headless IAM where clients build their own login UIs using
//! magic links or one-time passwords (OTP).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// Database Models
// ============================================================================

/// Passwordless token stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordlessToken {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub user_id: Option<Uuid>,
    pub token_type: String,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Token type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasswordlessTokenType {
    MagicLink,
    Otp,
}

impl std::fmt::Display for PasswordlessTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MagicLink => write!(f, "magic_link"),
            Self::Otp => write!(f, "otp"),
        }
    }
}

impl std::str::FromStr for PasswordlessTokenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "magic_link" => Ok(Self::MagicLink),
            "otp" => Ok(Self::Otp),
            _ => Err(format!("Unknown token type: {}", s)),
        }
    }
}

// ============================================================================
// API Request/Response Models
// ============================================================================

/// Request to start passwordless authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordlessStartRequest {
    /// User's email address
    pub email: String,

    /// Token type: "magic_link" or "otp"
    #[serde(default = "default_token_type")]
    pub token_type: PasswordlessTokenType,

    /// Optional: Where to redirect after magic link click (for magic_link type)
    pub redirect_uri: Option<String>,

    /// Optional: State to preserve through the flow
    pub state: Option<String>,

    /// Optional: Send code via SMS instead of email (if phone available)
    #[serde(default)]
    pub send_sms: bool,
}

fn default_token_type() -> PasswordlessTokenType {
    PasswordlessTokenType::MagicLink
}

/// Response after starting passwordless auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordlessStartResponse {
    pub success: bool,
    pub message: String,

    /// For OTP: a hint about where the code was sent
    pub delivery_method: String,

    /// Masked destination (e.g., "j***@example.com")
    pub masked_destination: String,

    /// Token ID for verification (used in verify request)
    pub token_id: Uuid,

    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

/// Request to verify passwordless token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordlessVerifyRequest {
    /// Token ID from start response
    pub token_id: Uuid,

    /// The token/code to verify
    /// For magic_link: the full token from the URL
    /// For OTP: the 6-digit code
    pub code: String,
}

/// Response after successful verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordlessVerifyResponse {
    pub success: bool,

    /// Access token for API calls
    pub access_token: String,

    /// Refresh token for getting new access tokens
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiry in seconds
    pub expires_in: i64,

    /// ID token (if OIDC scopes requested)
    pub id_token: Option<String>,

    /// User info
    pub user: PasswordlessUserInfo,

    /// Original state if provided
    pub state: Option<String>,
}

/// User info returned after passwordless auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordlessUserInfo {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub tenant_id: Uuid,
    pub is_new_user: bool,
}

// ============================================================================
// WebAuthn Models
// ============================================================================

/// WebAuthn credential stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub sign_count: i64,
    pub name: Option<String>,
    pub aaguid: Option<Vec<u8>>,
    #[sqlx(default)]
    pub transports: Vec<String>,
    pub device_type: Option<String>,
    pub backed_up: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// WebAuthn challenge for registration/authentication
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnChallenge {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub challenge: Vec<u8>,
    pub challenge_type: String,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Request to start WebAuthn registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnRegisterStartRequest {
    /// Optional: Friendly name for this credential
    pub name: Option<String>,
}

/// Response with WebAuthn registration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnRegisterStartResponse {
    /// Challenge ID for the verification step
    pub challenge_id: Uuid,

    /// PublicKeyCredentialCreationOptions for navigator.credentials.create()
    pub options: serde_json::Value,
}

/// Request to complete WebAuthn registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnRegisterFinishRequest {
    pub challenge_id: Uuid,
    pub credential: serde_json::Value,
    pub name: Option<String>,
}

/// Request to start WebAuthn authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnAuthStartRequest {
    /// Optional: email for user-specific credentials
    pub email: Option<String>,
}

/// Response with WebAuthn authentication options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnAuthStartResponse {
    pub challenge_id: Uuid,
    pub options: serde_json::Value,
}

/// Request to complete WebAuthn authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnAuthFinishRequest {
    pub challenge_id: Uuid,
    pub credential: serde_json::Value,
}

// ============================================================================
// Rate Limiting Models
// ============================================================================

/// Tenant rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TenantRateLimit {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub endpoint_category: String,
    pub requests_per_minute: i32,
    pub requests_per_hour: i32,
    pub burst_limit: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRateLimitRequest {
    pub endpoint_category: String,
    pub requests_per_minute: Option<i32>,
    pub requests_per_hour: Option<i32>,
    pub burst_limit: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// Response with all rate limits for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitsResponse {
    pub rate_limits: Vec<TenantRateLimit>,
}
