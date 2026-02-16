use axum::body::Body;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, Response, StatusCode};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::Deserialize;
use std::sync::Arc;

use crate::jwt::Claims;
use crate::session::{SessionData, TokenResponse};
use crate::ProxyState;

/// Claims from the OIDC id_token (includes email, profile).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IdTokenClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<bool>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    org_id: Option<String>,
    #[serde(default)]
    org_name: Option<String>,
}

/// Query params for /auth/login
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    /// If provided, check SSO for this email before redirecting to generic login.
    pub email: Option<String>,
    /// Where to redirect after successful login.
    pub redirect: Option<String>,
}

/// Query params for /auth/callback
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Query params for /auth/logout
#[derive(Debug, Deserialize)]
pub struct LogoutQuery {
    pub redirect: Option<String>,
}

// ============================================================================
// LOGIN
// ============================================================================

/// Handle GET /auth/login
///
/// 1. If `email` param → check SSO via CoreAuth /api/oidc/sso-check
///    - If SSO available → redirect to IdP auth URL
/// 2. Otherwise → redirect to CoreAuth /authorize for hosted login
pub async fn handle_login(
    state: Arc<ProxyState>,
    query: LoginQuery,
) -> Response<Body> {
    // Generate a random state parameter that encodes the redirect URL
    let redirect_url = query.redirect.unwrap_or_else(|| "/".to_string());
    let oauth_state = encode_state(&redirect_url);

    // Check SSO if email provided
    if let Some(ref email) = query.email {
        match check_sso(&state, email).await {
            Ok(Some(sso_url)) => {
                // SSO provider found — redirect to IdP
                tracing::info!("SSO detected for {}, redirecting to IdP", email);
                return redirect_response(&sso_url);
            }
            Ok(None) => {
                tracing::debug!("No SSO for {}, falling back to standard login", email);
            }
            Err(e) => {
                tracing::warn!("SSO check failed: {}, falling back to standard login", e);
            }
        }
    }

    // Build /authorize URL (relative — CoreAuth endpoints are proxied through us)
    let authorize_url = format!(
        "/authorize?response_type=code&client_id={}&redirect_uri={}&state={}&scope=openid+profile+email+offline_access",
        urlencoding::encode(&state.config.coreauth.client_id),
        urlencoding::encode(&state.config.coreauth.callback_url),
        urlencoding::encode(&oauth_state),
    );

    redirect_response(&authorize_url)
}

// ============================================================================
// CALLBACK
// ============================================================================

/// Handle GET /auth/callback?code=...&state=...
///
/// Exchange the authorization code for tokens, create a session cookie,
/// and redirect to the original URL.
pub async fn handle_callback(
    state: Arc<ProxyState>,
    query: CallbackQuery,
) -> Response<Body> {
    // Check for OAuth errors
    if let Some(error) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown error");
        tracing::error!("OAuth callback error: {} — {}", error, desc);
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!("Authentication failed: {}", desc),
        );
    }

    let code = match &query.code {
        Some(c) => c,
        None => {
            return error_response(StatusCode::BAD_REQUEST, "Missing authorization code");
        }
    };

    let oauth_state = query.state.as_deref().unwrap_or("");
    let redirect_url = decode_state(oauth_state);

    // Exchange code for tokens
    let token_resp = match exchange_code(&state, code).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token exchange failed: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Token exchange failed");
        }
    };

    // Decode the access token to extract claims (we trust our own CoreAuth tokens)
    let claims = match state.jwt_validator.validate(&token_resp.access_token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Access token validation failed: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid access token");
        }
    };

    // Decode id_token for email/profile claims (access token doesn't include these)
    let id_claims = token_resp.id_token.as_ref().and_then(|id_token| {
        decode_id_token_claims(id_token)
    });

    // Build session data
    let session_data = claims_to_session(&claims, &token_resp, id_claims.as_ref());

    // Create server-side session and get the cookie
    let cookie_value = match state.session_manager.create_session(session_data).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Session create failed: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed");
        }
    };

    // Redirect to original URL with session cookie
    Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, &redirect_url)
        .header(SET_COOKIE, &cookie_value)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// LOGOUT
// ============================================================================

/// Handle GET /auth/logout
///
/// Clear the session cookie and redirect to CoreAuth /logout.
pub async fn handle_logout(
    state: Arc<ProxyState>,
    headers: &HeaderMap,
    query: LogoutQuery,
) -> Response<Body> {
    // Read current session to get id_token for logout hint, then destroy it
    let session_id = state.session_manager.extract_session_id(headers);
    let session = match &session_id {
        Some(id) => state.session_manager.get_session(id).await,
        None => None,
    };

    // Destroy the server-side session
    if let Some(ref id) = session_id {
        state.session_manager.destroy_session(id).await;
    }

    let clear_cookie = state.session_manager.clear_cookie();

    // Build /logout URL (relative — CoreAuth endpoints are proxied through us)
    let post_logout_redirect = query.redirect.unwrap_or_else(|| "/".to_string());
    let mut logout_url = format!(
        "/logout?client_id={}&post_logout_redirect_uri={}",
        urlencoding::encode(&state.config.coreauth.client_id),
        urlencoding::encode(&post_logout_redirect),
    );

    // Add id_token_hint if available
    if let Some(ref session) = session {
        if let Some(ref id_token) = session.id_token {
            logout_url.push_str(&format!("&id_token_hint={}", urlencoding::encode(id_token)));
        }
    }

    Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, &logout_url)
        .header(SET_COOKIE, &clear_cookie)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// USERINFO
// ============================================================================

/// Handle GET /auth/userinfo — return current user info from session.
pub async fn handle_userinfo(
    state: Arc<ProxyState>,
    headers: &HeaderMap,
) -> Response<Body> {
    let session = match state.session_manager.extract_session_id(headers) {
        Some(id) => state.session_manager.get_session(&id).await,
        None => None,
    };

    match session {
        Some(session) => {
            let body = serde_json::json!({
                "user_id": session.user_id,
                "email": session.email,
                "tenant_id": session.tenant_id,
                "tenant_slug": session.tenant_slug,
                "role": session.role,
                "is_platform_admin": session.is_platform_admin,
                "authenticated": true,
            });
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        }
        None => {
            let body = serde_json::json!({ "authenticated": false });
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        }
    }
}

// ============================================================================
// HELPERS
// ============================================================================

/// Check SSO for an email via CoreAuth.
/// Returns the SSO authorization URL if SSO is available, None otherwise.
async fn check_sso(state: &ProxyState, email: &str) -> Result<Option<String>, String> {
    let url = format!(
        "{}/api/oidc/sso-check?email={}",
        state.config.coreauth.url,
        urlencoding::encode(email)
    );

    let resp = state.http_client.get(&url)
        .send()
        .await
        .map_err(|e| format!("SSO check request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("SSO check returned {}", resp.status()));
    }

    #[derive(Deserialize)]
    struct SsoResponse {
        has_sso: bool,
        providers: Vec<SsoProvider>,
    }

    #[derive(Deserialize)]
    struct SsoProvider {
        id: String,
        #[allow(dead_code)]
        name: String,
        #[allow(dead_code)]
        tenant_id: String,
    }

    let sso: SsoResponse = resp.json().await
        .map_err(|e| format!("SSO check parse error: {}", e))?;

    if !sso.has_sso || sso.providers.is_empty() {
        return Ok(None);
    }

    // Use the first available SSO provider
    let provider = &sso.providers[0];

    // Build the SSO login URL — redirect through CoreAuth OIDC flow
    let sso_url = format!(
        "{}/api/oidc/authorize/{}?redirect_uri={}&state={}",
        state.config.coreauth.url,
        provider.id,
        urlencoding::encode(&state.config.coreauth.callback_url),
        urlencoding::encode(&encode_state("/")),
    );

    Ok(Some(sso_url))
}

/// Exchange authorization code for tokens via CoreAuth /oauth/token.
async fn exchange_code(
    state: &ProxyState,
    code: &str,
) -> Result<TokenResponse, String> {
    let url = format!("{}/oauth/token", state.config.coreauth.url);

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &state.config.coreauth.callback_url),
        ("client_id", &state.config.coreauth.client_id),
        ("client_secret", &state.config.coreauth.client_secret),
    ];

    let resp = state.http_client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed ({}): {}", status, body));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Token exchange parse error: {}", e))
}

/// Build SessionData from JWT claims, token response, and optional id_token claims.
fn claims_to_session(claims: &Claims, tokens: &TokenResponse, id_claims: Option<&IdTokenClaims>) -> SessionData {
    let expires_at = chrono::Utc::now().timestamp() + tokens.expires_in;

    // Email comes from id_token (access token doesn't include it)
    let email = id_claims
        .and_then(|c| c.email.clone())
        .or_else(|| claims.email.clone())
        .unwrap_or_default();

    // Tenant/org ID: prefer access token, fall back to id_token
    let tenant_id = claims.tenant_id.clone()
        .or_else(|| claims.org_id.clone())
        .or_else(|| id_claims.and_then(|c| c.org_id.clone()));

    SessionData {
        user_id: claims.sub.clone(),
        email,
        tenant_id,
        tenant_slug: claims.organization_slug.clone(),
        role: claims.role.clone(),
        is_platform_admin: claims.is_platform_admin.unwrap_or(false),
        access_token: tokens.access_token.clone(),
        refresh_token: tokens.refresh_token.clone(),
        id_token: tokens.id_token.clone(),
        expires_at,
    }
}

/// Decode id_token claims without full validation (we trust it from our own token exchange).
fn decode_id_token_claims(id_token: &str) -> Option<IdTokenClaims> {
    // Split JWT and decode the payload (middle part)
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    serde_json::from_slice(&payload).ok()
}

/// Encode the redirect URL into a state parameter.
fn encode_state(redirect_url: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; 16];
    rng.fill_bytes(&mut nonce);
    let nonce_b64 = URL_SAFE_NO_PAD.encode(&nonce);

    // Format: nonce:base64(redirect_url)
    let redirect_b64 = URL_SAFE_NO_PAD.encode(redirect_url.as_bytes());
    format!("{}:{}", nonce_b64, redirect_b64)
}

/// Decode the redirect URL from a state parameter.
fn decode_state(state: &str) -> String {
    if let Some((_nonce, redirect_b64)) = state.split_once(':') {
        if let Ok(bytes) = URL_SAFE_NO_PAD.decode(redirect_b64) {
            if let Ok(url) = String::from_utf8(bytes) {
                return url;
            }
        }
    }
    "/".to_string()
}

fn redirect_response(url: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, url)
        .body(Body::empty())
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<Body> {
    let body = serde_json::json!({ "error": message });
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}
