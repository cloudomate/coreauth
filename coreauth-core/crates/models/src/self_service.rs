use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Flow Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlowType {
    Login,
    Registration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethod {
    Browser,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlowState {
    Active,
    RequiresMfa,
    RequiresEmailVerification,
    Completed,
}

// ── Flow ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfServiceFlow {
    pub id: Uuid,
    pub flow_type: FlowType,
    #[serde(rename = "type")]
    pub delivery_method: DeliveryMethod,
    pub state: FlowState,
    pub request_url: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,

    /// Links to an OAuth2 authorization request (for code flow integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_request_id: Option<String>,

    /// Client ID from the linked authorization request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Tenant/org context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<Uuid>,

    /// CSRF token (browser flows only, not exposed to clients)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,

    /// Set after successful password authentication (internal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authenticated_user_id: Option<Uuid>,

    /// Methods completed so far: ["password"], ["password", "totp"] (internal)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authentication_methods: Vec<String>,

    /// MFA challenge token (for TOTP/SMS verification, internal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfa_challenge_token: Option<String>,

    /// UI description for rendering the form
    pub ui: FlowUi,
}

// ── UI Model ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowUi {
    /// Endpoint to submit the form to
    pub action: String,
    /// HTTP method (always "POST")
    pub method: String,
    /// Form fields, buttons, hidden inputs
    pub nodes: Vec<UiNode>,
    /// Top-level messages (errors, info)
    pub messages: Vec<UiMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNode {
    /// "input", "text", "img", "script"
    #[serde(rename = "type")]
    pub node_type: String,
    /// "default", "password", "oidc", "totp", "code"
    pub group: String,
    /// HTML attributes
    pub attributes: UiNodeAttributes,
    /// Node-level messages (field validation errors)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<UiMessage>,
    /// Metadata (labels, etc.)
    pub meta: UiNodeMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNodeAttributes {
    pub name: String,
    /// "text", "email", "password", "hidden", "submit", "number"
    #[serde(rename = "type")]
    pub input_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autocomplete: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxlength: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiMessage {
    /// Numeric code (e.g., 4010001 = invalid credentials)
    pub id: i64,
    pub text: String,
    /// "error", "info", "success"
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNodeMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<UiLabel>,
    /// For social login buttons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLabel {
    pub text: String,
}

// ── Submit Payloads ─────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct LoginFlowSubmit {
    pub method: String,
    #[serde(default)]
    pub csrf_token: Option<String>,
    // password method
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    // oidc method
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub connection_id: Option<Uuid>,
    // totp method
    #[serde(default)]
    pub totp_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationFlowSubmit {
    pub method: String,
    #[serde(default)]
    pub csrf_token: Option<String>,
    // password method
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    // oidc method
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub connection_id: Option<Uuid>,
}

// ── Response Types ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FlowResponse {
    /// On success: the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionResponse>,
    /// API flows: session token for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    /// OAuth2 integration: redirect with authorization code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_browser_to: Option<String>,
    /// On error or intermediate state: the updated flow
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub flow: Option<SelfServiceFlow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub identity: IdentityResponse,
    pub authenticated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub authentication_methods: Vec<AuthMethodRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityResponse {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthMethodRef {
    pub method: String,
    pub completed_at: DateTime<Utc>,
}

// ── Query Params ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FlowQuery {
    pub flow: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlowQuery {
    #[serde(default)]
    pub organization_id: Option<Uuid>,
    /// Link to an existing OAuth2 authorization request
    #[serde(default)]
    pub request_id: Option<String>,
}

// ── Message IDs ─────────────────────────────────────────────
// Following Ory's convention: xyyzzzz where x=type, yy=module, zzzz=message

pub mod message_ids {
    // 1xx = info
    pub const LOGIN_IDENTIFIER_LABEL: i64 = 1010001;
    pub const LOGIN_PASSWORD_LABEL: i64 = 1010002;
    pub const LOGIN_SUBMIT_LABEL: i64 = 1010003;
    pub const REGISTRATION_EMAIL_LABEL: i64 = 1040001;
    pub const REGISTRATION_PASSWORD_LABEL: i64 = 1040002;
    pub const REGISTRATION_SUBMIT_LABEL: i64 = 1040003;
    pub const REGISTRATION_NAME_LABEL: i64 = 1040004;
    pub const MFA_TOTP_LABEL: i64 = 1060001;
    pub const MFA_SUBMIT_LABEL: i64 = 1060002;

    // 4xx = validation error
    pub const INVALID_CREDENTIALS: i64 = 4010001;
    pub const ACCOUNT_LOCKED: i64 = 4010002;
    pub const EMAIL_NOT_VERIFIED: i64 = 4010003;
    pub const FIELD_REQUIRED: i64 = 4000001;
    pub const EMAIL_ALREADY_EXISTS: i64 = 4040001;
    pub const PASSWORD_TOO_SHORT: i64 = 4040002;
    pub const INVALID_TOTP_CODE: i64 = 4060001;

    // 5xx = system error
    pub const FLOW_EXPIRED: i64 = 5010001;
    pub const FLOW_NOT_FOUND: i64 = 5010002;
    pub const CSRF_MISMATCH: i64 = 5010003;
    pub const INTERNAL_ERROR: i64 = 5000001;
}
