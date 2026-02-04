use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use ciam_auth::{OAuth2Service, PasswordHasher};
use ciam_models::oauth2::{CreateAuthorizationCode, CreateLoginSession};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

// ============================================================================
// TEMPLATE STRUCTS
// ============================================================================

/// Branding settings for templates
/// Note: Using String instead of Option<String> for Askama template compatibility
#[derive(Debug, Clone)]
pub struct Branding {
    /// Empty string means no logo
    pub logo_url: String,
    pub primary_color: String,
}

impl Branding {
    pub fn has_logo(&self) -> bool {
        !self.logo_url.is_empty()
    }
}

impl Default for Branding {
    fn default() -> Self {
        Branding {
            logo_url: String::new(),
            primary_color: "#2563eb".to_string(),
        }
    }
}

/// Social connection for display
#[derive(Debug, Clone)]
pub struct SocialConnection {
    pub id: String,
    pub name: String,
    pub provider_type: String,
}

/// Scope information for consent page
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub name: String,
    pub title: String,
    pub description: String,
}

impl ScopeInfo {
    pub fn from_scope(scope: &str) -> Self {
        match scope {
            "openid" => ScopeInfo {
                name: "openid".to_string(),
                title: "Authenticate you".to_string(),
                description: "Verify your identity".to_string(),
            },
            "profile" => ScopeInfo {
                name: "profile".to_string(),
                title: "View your profile".to_string(),
                description: "Access your name and profile picture".to_string(),
            },
            "email" => ScopeInfo {
                name: "email".to_string(),
                title: "View your email".to_string(),
                description: "Access your email address".to_string(),
            },
            "offline_access" => ScopeInfo {
                name: "offline_access".to_string(),
                title: "Maintain access".to_string(),
                description: "Access your data when you're not using the app".to_string(),
            },
            _ => ScopeInfo {
                name: scope.to_string(),
                title: scope.to_string(),
                description: format!("Access to {}", scope),
            },
        }
    }
}

// ============================================================================
// TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    org_name: String,
    app_name: String,
    request_id: String,
    email: String,
    error: String,
    has_error: bool,
    branding: Branding,
    social_connections: Vec<SocialConnection>,
    allow_signup: bool,
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate {
    org_name: String,
    app_name: String,
    request_id: String,
    email: String,
    error: String,
    has_error: bool,
    branding: Branding,
    social_connections: Vec<SocialConnection>,
}

#[derive(Template)]
#[template(path = "mfa.html")]
struct MfaTemplate {
    org_name: String,
    request_id: String,
    challenge_token: String,
    error: String,
    has_error: bool,
    branding: Branding,
}

#[derive(Template)]
#[template(path = "consent.html")]
struct ConsentTemplate {
    org_name: String,
    app_name: String,
    request_id: String,
    user_email: String,
    scopes: Vec<ScopeInfo>,
    branding: Branding,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error_code: String,
    error_description: String,
    return_url: String,
    has_return_url: bool,
}

#[derive(Template)]
#[template(path = "logged_out.html")]
struct LoggedOutTemplate {}

// ============================================================================
// QUERY/FORM STRUCTS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub request_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub request_id: String,
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub request_id: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub terms: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaForm {
    pub request_id: String,
    pub challenge_token: String,
    pub method: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ConsentForm {
    pub request_id: String,
    pub action: String,
}

// ============================================================================
// LOGIN HANDLERS
// ============================================================================

/// GET /login - Show login page
pub async fn login_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LoginQuery>,
    headers: HeaderMap,
) -> Response {
    // Check if user already has a session
    if let Some(session) = get_session_from_cookie(&headers, &state).await {
        // User is already logged in
        if let Some(request_id) = &params.request_id {
            // Complete the authorization flow
            return handle_authenticated_user(&state, request_id, session.user_id, session.organization_id)
                .await;
        }
    }

    // Get auth request details if request_id provided
    let (org_name, app_name, branding, organization_id) = if let Some(request_id) = &params.request_id {
        match state.oauth2_service.get_authorization_request(request_id).await {
            Ok(auth_request) => {
                // Get application details
                if let Ok(app) = sqlx::query_as::<_, ciam_models::Application>(
                    "SELECT * FROM applications WHERE id = $1",
                )
                .bind(auth_request.client_id)
                .fetch_one(state.auth_service.db.pool())
                .await
                {
                    // Get organization details if set
                    let effective_org_id = auth_request.organization_id.or(app.organization_id);
                    let (org_name, branding) = if let Some(org_id) = effective_org_id {
                        get_org_branding(&state, org_id).await
                    } else {
                        ("CoreAuth".to_string(), Branding::default())
                    };

                    (org_name, app.name.clone(), branding, effective_org_id)
                } else {
                    ("CoreAuth".to_string(), "Application".to_string(), Branding::default(), None)
                }
            }
            Err(_) => {
                // Invalid request_id - show error
                return render_error("invalid_request", "The authorization request has expired or is invalid.", None);
            }
        }
    } else {
        ("CoreAuth".to_string(), "CoreAuth".to_string(), Branding::default(), None)
    };

    // Load social connections
    let social_connections = get_social_connections(&state, organization_id).await;

    // Render login page
    let template = LoginTemplate {
        org_name,
        app_name,
        request_id: params.request_id.unwrap_or_default(),
        email: String::new(),
        error: params.error.clone().unwrap_or_default(),
        has_error: params.error.is_some(),
        branding,
        social_connections,
        allow_signup: true,         // TODO: Check org settings
    };

    Html(template.render().unwrap_or_else(|e| {
        error!("Template render error: {}", e);
        "Error rendering page".to_string()
    }))
    .into_response()
}

/// POST /login - Handle login form submission
pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> Response {
    // Get the authorization request
    let auth_request = match state.oauth2_service.get_authorization_request(&form.request_id).await {
        Ok(req) => req,
        Err(_) => {
            return render_error(
                "invalid_request",
                "The authorization request has expired or is invalid.",
                None,
            );
        }
    };

    // Get the organization ID from the auth request or application
    let organization_id = if let Some(org_id) = auth_request.organization_id {
        Some(org_id)
    } else {
        // Get from application
        if let Ok(app) = sqlx::query_as::<_, ciam_models::Application>(
            "SELECT * FROM applications WHERE id = $1",
        )
        .bind(auth_request.client_id)
        .fetch_one(state.auth_service.db.pool())
        .await
        {
            app.organization_id
        } else {
            None
        }
    };

    // Find user by email
    let user = match sqlx::query_as::<_, ciam_models::User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true",
    )
    .bind(&form.email)
    .fetch_optional(state.auth_service.db.pool())
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return redirect_with_error(&form.request_id, "Invalid email or password");
        }
        Err(e) => {
            error!("Database error: {}", e);
            return redirect_with_error(&form.request_id, "An error occurred. Please try again.");
        }
    };

    // If organization is specified, verify user membership
    if let Some(org_id) = organization_id {
        let is_member = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM organization_members WHERE user_id = $1 AND organization_id = $2)",
        )
        .bind(user.id)
        .bind(org_id)
        .fetch_one(state.auth_service.db.pool())
        .await
        .unwrap_or(false);

        if !is_member && !user.is_platform_admin {
            return redirect_with_error(&form.request_id, "You don't have access to this organization");
        }
    }

    // Verify password
    let password_hash = match &user.password_hash {
        Some(hash) => hash,
        None => {
            return redirect_with_error(&form.request_id, "Invalid email or password");
        }
    };

    let is_valid = PasswordHasher::verify(&form.password, password_hash).unwrap_or(false);

    if !is_valid {
        return redirect_with_error(&form.request_id, "Invalid email or password");
    }

    // Check if MFA is required
    if user.mfa_enabled {
        // Create MFA challenge and redirect to MFA page
        let challenge_token = uuid::Uuid::new_v4().to_string();

        // Store challenge
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        if let Err(e) = sqlx::query(
            "INSERT INTO mfa_challenges (user_id, challenge_token, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user.id)
        .bind(&challenge_token)
        .bind(expires_at)
        .execute(state.auth_service.db.pool())
        .await
        {
            error!("Failed to create MFA challenge: {}", e);
            return redirect_with_error(&form.request_id, "An error occurred. Please try again.");
        }

        return Redirect::temporary(&format!(
            "/mfa?request_id={}&challenge_token={}",
            form.request_id, challenge_token
        ))
        .into_response();
    }

    // Create session and complete authorization
    handle_authenticated_user(&state, &form.request_id, user.id, organization_id).await
}

// ============================================================================
// SIGNUP HANDLERS
// ============================================================================

/// GET /signup - Show signup page
pub async fn signup_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LoginQuery>,
) -> Response {
    let (org_name, app_name, branding, organization_id) = if let Some(request_id) = &params.request_id {
        match state.oauth2_service.get_authorization_request(request_id).await {
            Ok(auth_request) => {
                if let Ok(app) = sqlx::query_as::<_, ciam_models::Application>(
                    "SELECT * FROM applications WHERE id = $1",
                )
                .bind(auth_request.client_id)
                .fetch_one(state.auth_service.db.pool())
                .await
                {
                    let effective_org_id = auth_request.organization_id.or(app.organization_id);
                    let (org_name, branding) = if let Some(org_id) = effective_org_id {
                        get_org_branding(&state, org_id).await
                    } else {
                        ("CoreAuth".to_string(), Branding::default())
                    };
                    (org_name, app.name.clone(), branding, effective_org_id)
                } else {
                    ("CoreAuth".to_string(), "Application".to_string(), Branding::default(), None)
                }
            }
            Err(_) => {
                return render_error("invalid_request", "The authorization request has expired or is invalid.", None);
            }
        }
    } else {
        ("CoreAuth".to_string(), "CoreAuth".to_string(), Branding::default(), None)
    };

    // Load social connections
    let social_connections = get_social_connections(&state, organization_id).await;

    let template = SignupTemplate {
        org_name,
        app_name,
        request_id: params.request_id.unwrap_or_default(),
        email: String::new(),
        error: params.error.clone().unwrap_or_default(),
        has_error: params.error.is_some(),
        branding,
        social_connections,
    };

    Html(template.render().unwrap_or_else(|e| {
        error!("Template render error: {}", e);
        "Error rendering page".to_string()
    }))
    .into_response()
}

/// POST /signup - Handle signup form submission
pub async fn signup_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SignupForm>,
) -> Response {
    // Validate passwords match
    if form.password != form.password_confirm {
        return redirect_to_signup_with_error(&form.request_id, "Passwords do not match");
    }

    // Validate password length
    if form.password.len() < 8 {
        return redirect_to_signup_with_error(&form.request_id, "Password must be at least 8 characters");
    }

    // Validate terms accepted
    if form.terms.is_none() {
        return redirect_to_signup_with_error(&form.request_id, "You must accept the terms of service");
    }

    // Get authorization request
    let auth_request = match state.oauth2_service.get_authorization_request(&form.request_id).await {
        Ok(req) => req,
        Err(_) => {
            return render_error("invalid_request", "The authorization request has expired.", None);
        }
    };

    // Get organization ID
    let organization_id = if let Some(org_id) = auth_request.organization_id {
        Some(org_id)
    } else {
        if let Ok(app) = sqlx::query_as::<_, ciam_models::Application>(
            "SELECT * FROM applications WHERE id = $1",
        )
        .bind(auth_request.client_id)
        .fetch_one(state.auth_service.db.pool())
        .await
        {
            app.organization_id
        } else {
            None
        }
    };

    // Check if email already exists
    let existing_user = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
    )
    .bind(&form.email)
    .fetch_one(state.auth_service.db.pool())
    .await
    .unwrap_or(false);

    if existing_user {
        return redirect_to_signup_with_error(&form.request_id, "An account with this email already exists");
    }

    // Hash password
    let password_hash = match PasswordHasher::hash(&form.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return redirect_to_signup_with_error(&form.request_id, "An error occurred. Please try again.");
        }
    };

    // Create user
    let user_id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, default_organization_id, metadata, is_active)
        VALUES ($1, $2, $3, $4, '{}', true)
        "#,
    )
    .bind(user_id)
    .bind(&form.email)
    .bind(&password_hash)
    .bind(organization_id)
    .execute(state.auth_service.db.pool())
    .await
    {
        error!("Failed to create user: {}", e);
        return redirect_to_signup_with_error(&form.request_id, "An error occurred. Please try again.");
    }

    // Add to organization if specified
    if let Some(org_id) = organization_id {
        let _ = sqlx::query(
            "INSERT INTO organization_members (user_id, organization_id, role) VALUES ($1, $2, 'member')",
        )
        .bind(user_id)
        .bind(org_id)
        .execute(state.auth_service.db.pool())
        .await;
    }

    info!(user_id = %user_id, email = %form.email, "New user registered via Universal Login");

    // Complete authorization
    handle_authenticated_user(&state, &form.request_id, user_id, organization_id).await
}

// ============================================================================
// MFA HANDLERS
// ============================================================================

/// GET /mfa - Show MFA page
pub async fn mfa_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MfaQuery>,
) -> Response {
    let branding = Branding::default();

    let template = MfaTemplate {
        org_name: "CoreAuth".to_string(),
        request_id: params.request_id.unwrap_or_default(),
        challenge_token: params.challenge_token.unwrap_or_default(),
        error: params.error.clone().unwrap_or_default(),
        has_error: params.error.is_some(),
        branding,
    };

    Html(template.render().unwrap_or_else(|e| {
        error!("Template render error: {}", e);
        "Error rendering page".to_string()
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct MfaQuery {
    pub request_id: Option<String>,
    pub challenge_token: Option<String>,
    pub error: Option<String>,
}

/// POST /mfa/verify - Handle MFA verification
pub async fn mfa_verify(
    State(state): State<Arc<AppState>>,
    Form(form): Form<MfaForm>,
) -> Response {
    // Validate challenge token and get user
    let challenge = match sqlx::query_as::<_, (Uuid, chrono::DateTime<chrono::Utc>)>(
        "SELECT user_id, expires_at FROM mfa_challenges WHERE challenge_token = $1",
    )
    .bind(&form.challenge_token)
    .fetch_optional(state.auth_service.db.pool())
    .await
    {
        Ok(Some((user_id, expires_at))) => {
            if expires_at < chrono::Utc::now() {
                return redirect_to_mfa_with_error(
                    &form.request_id,
                    &form.challenge_token,
                    "Challenge expired. Please try again.",
                );
            }
            user_id
        }
        Ok(None) => {
            return render_error("invalid_request", "Invalid challenge token.", None);
        }
        Err(e) => {
            error!("Database error: {}", e);
            return redirect_to_mfa_with_error(
                &form.request_id,
                &form.challenge_token,
                "An error occurred. Please try again.",
            );
        }
    };

    // Get user's TOTP secret
    let totp_secret = match sqlx::query_scalar::<_, String>(
        "SELECT secret FROM mfa_methods WHERE user_id = $1 AND method_type = 'totp' AND verified = true",
    )
    .bind(challenge)
    .fetch_optional(state.auth_service.db.pool())
    .await
    {
        Ok(Some(secret)) => secret,
        Ok(None) => {
            return redirect_to_mfa_with_error(
                &form.request_id,
                &form.challenge_token,
                "MFA not configured properly.",
            );
        }
        Err(e) => {
            error!("Database error: {}", e);
            return redirect_to_mfa_with_error(
                &form.request_id,
                &form.challenge_token,
                "An error occurred. Please try again.",
            );
        }
    };

    // Verify TOTP code
    let is_valid = match ciam_auth::verify_totp(&totp_secret, &form.code) {
        Ok(valid) => valid,
        Err(e) => {
            error!("TOTP verification error: {}", e);
            return redirect_to_mfa_with_error(
                &form.request_id,
                &form.challenge_token,
                "An error occurred during verification.",
            );
        }
    };

    if !is_valid {
        return redirect_to_mfa_with_error(
            &form.request_id,
            &form.challenge_token,
            "Invalid verification code.",
        );
    }

    // Delete the challenge
    let _ = sqlx::query("DELETE FROM mfa_challenges WHERE challenge_token = $1")
        .bind(&form.challenge_token)
        .execute(state.auth_service.db.pool())
        .await;

    // Get user's organization
    let user = sqlx::query_as::<_, ciam_models::User>("SELECT * FROM users WHERE id = $1")
        .bind(challenge)
        .fetch_one(state.auth_service.db.pool())
        .await
        .ok();

    let organization_id = user.as_ref().and_then(|u| u.default_organization_id);

    // Complete authorization
    handle_authenticated_user(&state, &form.request_id, challenge, organization_id).await
}

// ============================================================================
// CONSENT HANDLERS
// ============================================================================

/// GET /consent - Show consent page
pub async fn consent_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConsentQuery>,
) -> Response {
    // This is called when user needs to explicitly consent to scope access
    // For first-party apps, this is skipped

    let template = ConsentTemplate {
        org_name: "CoreAuth".to_string(),
        app_name: "Application".to_string(),
        request_id: params.request_id.unwrap_or_default(),
        user_email: params.email.unwrap_or_default(),
        scopes: vec![
            ScopeInfo::from_scope("openid"),
            ScopeInfo::from_scope("profile"),
            ScopeInfo::from_scope("email"),
        ],
        branding: Branding::default(),
    };

    Html(template.render().unwrap_or_else(|e| {
        error!("Template render error: {}", e);
        "Error rendering page".to_string()
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ConsentQuery {
    pub request_id: Option<String>,
    pub email: Option<String>,
}

/// POST /consent - Handle consent form
pub async fn consent_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ConsentForm>,
) -> Response {
    if form.action == "deny" {
        // Get auth request for redirect_uri
        if let Ok(auth_request) = state.oauth2_service.get_authorization_request(&form.request_id).await {
            let error_url = format!(
                "{}?error=access_denied&error_description=User%20denied%20access&state={}",
                auth_request.redirect_uri,
                auth_request.state.as_deref().unwrap_or("")
            );
            return Redirect::temporary(&error_url).into_response();
        }
        return render_error("access_denied", "You denied the authorization request.", None);
    }

    // Action is "allow" - this should be handled by completing the flow
    // The actual consent granting happens in handle_authenticated_user
    render_error("invalid_request", "Invalid consent flow.", None)
}

// ============================================================================
// STATIC PAGES
// ============================================================================

/// GET /logged-out - Show logged out page
pub async fn logged_out_page() -> Html<String> {
    let template = LoggedOutTemplate {};
    Html(template.render().unwrap_or_else(|e| {
        error!("Template render error: {}", e);
        "You have been logged out.".to_string()
    }))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn get_session_from_cookie(
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Option<ciam_models::oauth2::LoginSession> {
    let cookie = headers.get(header::COOKIE)?;
    let cookie_str = cookie.to_str().ok()?;

    for part in cookie_str.split(';') {
        let part = part.trim();
        if part.starts_with("coreauth_session=") {
            let session_token = &part[17..];
            if let Ok(session) = state.oauth2_service.validate_login_session(session_token).await {
                return Some(session);
            }
        }
    }
    None
}

async fn get_org_branding(state: &Arc<AppState>, org_id: Uuid) -> (String, Branding) {
    if let Ok(org) = sqlx::query_as::<_, ciam_models::Organization>(
        "SELECT * FROM organizations WHERE id = $1",
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    {
        let branding = Branding {
            logo_url: org.settings.branding.logo_url.clone().unwrap_or_default(),
            primary_color: org.settings.branding.primary_color.clone().unwrap_or_else(|| "#2563eb".to_string()),
        };
        (org.name, branding)
    } else {
        ("CoreAuth".to_string(), Branding::default())
    }
}

/// Load enabled social connections for the login page
async fn get_social_connections(state: &Arc<AppState>, organization_id: Option<Uuid>) -> Vec<SocialConnection> {
    // Load platform-level social connections
    let platform_connections = sqlx::query_as::<_, ciam_models::Connection>(
        "SELECT * FROM connections WHERE type = 'social' AND scope = 'platform' AND is_enabled = true"
    )
    .fetch_all(state.auth_service.db.pool())
    .await
    .unwrap_or_default();

    // Load organization-level social connections if applicable
    let org_connections = if let Some(org_id) = organization_id {
        sqlx::query_as::<_, ciam_models::Connection>(
            "SELECT * FROM connections WHERE type = 'social' AND scope = 'organization' AND organization_id = $1 AND is_enabled = true"
        )
        .bind(org_id)
        .fetch_all(state.auth_service.db.pool())
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };

    // Convert to display format
    let mut social_connections = Vec::new();

    for conn in platform_connections.into_iter().chain(org_connections) {
        // Try to parse the config to get the provider type
        if let Ok(config) = serde_json::from_value::<ciam_models::SocialConnectionConfig>(conn.config.clone()) {
            social_connections.push(SocialConnection {
                id: conn.id.to_string(),
                name: conn.name.clone(),
                provider_type: config.provider.to_string(),
            });
        }
    }

    social_connections
}

async fn handle_authenticated_user(
    state: &Arc<AppState>,
    request_id: &str,
    user_id: Uuid,
    organization_id: Option<Uuid>,
) -> Response {
    // Get the authorization request
    let auth_request = match state.oauth2_service.get_authorization_request(request_id).await {
        Ok(req) => req,
        Err(_) => {
            return render_error("invalid_request", "The authorization request has expired.", None);
        }
    };

    // Create authorization code
    let code = match state
        .oauth2_service
        .create_authorization_code(CreateAuthorizationCode {
            client_id: auth_request.client_id,
            user_id,
            organization_id: organization_id.or(auth_request.organization_id),
            redirect_uri: auth_request.redirect_uri.clone(),
            scope: auth_request.scope.clone(),
            code_challenge: auth_request.code_challenge.clone(),
            code_challenge_method: auth_request.code_challenge_method.clone(),
            nonce: auth_request.nonce.clone(),
            state: auth_request.state.clone(),
            response_type: auth_request.response_type.clone(),
        })
        .await
    {
        Ok(code) => code,
        Err(e) => {
            error!("Failed to create authorization code: {}", e);
            return render_error("server_error", "Failed to complete authorization.", None);
        }
    };

    // Delete the authorization request
    let _ = state.oauth2_service.delete_authorization_request(request_id).await;

    // Create login session cookie
    let session_token = match state
        .oauth2_service
        .create_login_session(CreateLoginSession {
            user_id,
            organization_id,
            ip_address: None,
            user_agent: None,
            mfa_verified: false,
            expires_in_seconds: 86400 * 7, // 7 days
        })
        .await
    {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to create login session: {}", e);
            // Continue without session - just redirect
            let redirect_url = format!(
                "{}?code={}&state={}",
                auth_request.redirect_uri,
                code,
                auth_request.state.as_deref().unwrap_or("")
            );
            return Redirect::temporary(&redirect_url).into_response();
        }
    };

    // Build redirect URL with code
    let redirect_url = format!(
        "{}?code={}&state={}",
        auth_request.redirect_uri,
        code,
        auth_request.state.as_deref().unwrap_or("")
    );

    // Create response with session cookie
    let mut response = Redirect::temporary(&redirect_url).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        format!(
            "coreauth_session={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            session_token,
            86400 * 7
        )
        .parse()
        .unwrap(),
    );

    response
}

fn redirect_with_error(request_id: &str, error: &str) -> Response {
    Redirect::temporary(&format!(
        "/login?request_id={}&error={}",
        request_id,
        urlencoding::encode(error)
    ))
    .into_response()
}

fn redirect_to_signup_with_error(request_id: &str, error: &str) -> Response {
    Redirect::temporary(&format!(
        "/signup?request_id={}&error={}",
        request_id,
        urlencoding::encode(error)
    ))
    .into_response()
}

fn redirect_to_mfa_with_error(request_id: &str, challenge_token: &str, error: &str) -> Response {
    Redirect::temporary(&format!(
        "/mfa?request_id={}&challenge_token={}&error={}",
        request_id,
        challenge_token,
        urlencoding::encode(error)
    ))
    .into_response()
}

fn render_error(error_code: &str, error_description: &str, return_url: Option<&str>) -> Response {
    let template = ErrorTemplate {
        error_code: error_code.to_string(),
        error_description: error_description.to_string(),
        return_url: return_url.map(String::from).unwrap_or_default(),
        has_return_url: return_url.is_some(),
    };

    (
        StatusCode::BAD_REQUEST,
        Html(template.render().unwrap_or_else(|e| {
            error!("Template render error: {}", e);
            format!("Error: {}", error_description)
        })),
    )
        .into_response()
}
