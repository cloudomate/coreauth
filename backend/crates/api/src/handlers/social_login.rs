use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use ciam_models::{Connection, SocialConnectionConfig, SocialProvider, SocialUserInfo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

// ============================================================================
// QUERY/FORM STRUCTS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SocialLoginQuery {
    pub request_id: Option<String>,
    pub signup: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SocialCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// State stored in cache during OAuth flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialOAuthState {
    pub connection_id: Uuid,
    pub request_id: String,
    pub signup: bool,
    pub nonce: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Token response from social provider
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: Option<String>,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    scope: Option<String>,
    id_token: Option<String>,
}

// ============================================================================
// SOCIAL LOGIN HANDLERS
// ============================================================================

/// GET /login/social/:connection_id - Initiate social login
pub async fn social_login(
    State(state): State<Arc<AppState>>,
    Path(connection_id): Path<Uuid>,
    Query(params): Query<SocialLoginQuery>,
) -> Response {
    // Get the connection
    let connection = match get_social_connection(&state, connection_id).await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get connection: {}", e);
            return redirect_to_login_with_error(
                params.request_id.as_deref(),
                "Invalid connection",
            );
        }
    };

    // Parse config
    let config: SocialConnectionConfig = match serde_json::from_value(connection.config.clone()) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse social connection config: {}", e);
            return redirect_to_login_with_error(
                params.request_id.as_deref(),
                "Connection misconfigured",
            );
        }
    };

    // Generate state for CSRF protection
    let nonce = Uuid::new_v4().to_string();
    let oauth_state = SocialOAuthState {
        connection_id,
        request_id: params.request_id.clone().unwrap_or_default(),
        signup: params.signup.unwrap_or(false),
        nonce: nonce.clone(),
        created_at: chrono::Utc::now(),
    };

    // Store state in cache (expires in 10 minutes)
    let state_key = format!("social_oauth_state:{}", nonce);
    if let Err(e) = state.cache.set(
        &state_key,
        &serde_json::to_string(&oauth_state).unwrap(),
        Some(600),
    ).await {
        error!("Failed to store OAuth state: {}", e);
        return redirect_to_login_with_error(
            params.request_id.as_deref(),
            "Failed to initiate login",
        );
    }

    // Build authorization URL
    let scopes = if config.scopes.is_empty() {
        config.provider.default_scopes().iter().map(|s| s.to_string()).collect::<Vec<_>>()
    } else {
        config.scopes.clone()
    };

    let redirect_uri = get_callback_url(&state);
    let scope_str = scopes.join(" ");

    let auth_url = match config.provider {
        SocialProvider::Github => {
            // GitHub uses slightly different parameter names
            format!(
                "{}?client_id={}&redirect_uri={}&scope={}&state={}",
                config.provider.authorization_url(),
                urlencoding::encode(&config.client_id),
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(&scope_str),
                urlencoding::encode(&nonce),
            )
        }
        _ => {
            // Standard OAuth2/OIDC providers
            format!(
                "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                config.provider.authorization_url(),
                urlencoding::encode(&config.client_id),
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(&scope_str),
                urlencoding::encode(&nonce),
            )
        }
    };

    info!(
        provider = %config.provider,
        connection_id = %connection_id,
        "Initiating social login"
    );

    Redirect::temporary(&auth_url).into_response()
}

/// GET /login/social/callback - Handle callback from social provider
pub async fn social_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SocialCallbackQuery>,
) -> Response {
    // Check for error from provider
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_else(|| "Login was cancelled".to_string());
        warn!(error = %error, description = %description, "Social login error from provider");
        return redirect_to_login_with_error(None, &description);
    }

    // Get authorization code
    let code = match params.code {
        Some(c) => c,
        None => {
            return redirect_to_login_with_error(None, "Missing authorization code");
        }
    };

    // Get and validate state
    let state_nonce = match params.state {
        Some(s) => s,
        None => {
            return redirect_to_login_with_error(None, "Missing state parameter");
        }
    };

    let state_key = format!("social_oauth_state:{}", state_nonce);
    let oauth_state: SocialOAuthState = match state.cache.get::<SocialOAuthState>(&state_key).await {
        Ok(Some(data)) => {
            // Delete the state to prevent replay
            let _ = state.cache.delete(&state_key).await;
            data
        }
        Ok(None) => {
            warn!("OAuth state not found or expired");
            return redirect_to_login_with_error(None, "Session expired. Please try again.");
        }
        Err(e) => {
            error!("Failed to get OAuth state: {}", e);
            return redirect_to_login_with_error(None, "Session expired. Please try again.");
        }
    };

    // Get the connection
    let connection = match get_social_connection(&state, oauth_state.connection_id).await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get connection: {}", e);
            return redirect_to_login_with_error(
                Some(&oauth_state.request_id),
                "Connection not found",
            );
        }
    };

    let config: SocialConnectionConfig = match serde_json::from_value(connection.config.clone()) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse social connection config: {}", e);
            return redirect_to_login_with_error(
                Some(&oauth_state.request_id),
                "Connection misconfigured",
            );
        }
    };

    // Exchange code for tokens
    let tokens = match exchange_code_for_tokens(&state, &config, &code).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to exchange code for tokens: {}", e);
            return redirect_to_login_with_error(
                Some(&oauth_state.request_id),
                "Failed to complete login",
            );
        }
    };

    // Get user info from provider
    let user_info = match fetch_user_info(&config, &tokens.access_token).await {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to fetch user info: {}", e);
            return redirect_to_login_with_error(
                Some(&oauth_state.request_id),
                "Failed to get user information",
            );
        }
    };

    info!(
        provider = %config.provider,
        provider_user_id = %user_info.provider_user_id,
        email = ?user_info.email,
        "Social login user info received"
    );

    // Find or create user
    let (user_id, organization_id) = match find_or_create_user(&state, &user_info, &oauth_state).await {
        Ok((uid, oid)) => (uid, oid),
        Err(e) => {
            error!("Failed to find or create user: {}", e);
            return redirect_to_login_with_error(
                Some(&oauth_state.request_id),
                &e,
            );
        }
    };

    // Complete authorization using the shared handler
    complete_social_login(&state, &oauth_state.request_id, user_id, organization_id).await
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn get_social_connection(state: &Arc<AppState>, connection_id: Uuid) -> Result<Connection, String> {
    sqlx::query_as::<_, Connection>(
        "SELECT * FROM connections WHERE id = $1 AND type = 'social' AND is_enabled = true"
    )
    .bind(connection_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Connection not found".to_string())
}

fn get_callback_url(state: &Arc<AppState>) -> String {
    // Get base URL from environment or use default
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    format!("{}/login/social/callback", base_url)
}

async fn exchange_code_for_tokens(
    state: &Arc<AppState>,
    config: &SocialConnectionConfig,
    code: &str,
) -> Result<TokenResponse, String> {
    let redirect_uri = get_callback_url(state);

    let client = reqwest::Client::new();

    let response = match config.provider {
        SocialProvider::Github => {
            // GitHub requires Accept header for JSON response
            client
                .post(config.provider.token_url())
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", config.client_id.as_str()),
                    ("client_secret", config.client_secret.as_str()),
                    ("code", code),
                    ("redirect_uri", redirect_uri.as_str()),
                ])
                .send()
                .await
                .map_err(|e| e.to_string())?
        }
        _ => {
            client
                .post(config.provider.token_url())
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", config.client_id.as_str()),
                    ("client_secret", config.client_secret.as_str()),
                    ("code", code),
                    ("redirect_uri", redirect_uri.as_str()),
                ])
                .send()
                .await
                .map_err(|e| e.to_string())?
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, body = %body, "Token exchange failed");
        return Err(format!("Token exchange failed: {}", status));
    }

    response.json::<TokenResponse>().await.map_err(|e| e.to_string())
}

async fn fetch_user_info(
    config: &SocialConnectionConfig,
    access_token: &str,
) -> Result<SocialUserInfo, String> {
    let client = reqwest::Client::new();

    let userinfo_url = config.provider.userinfo_url();
    if userinfo_url.is_empty() {
        return Err("Provider does not support userinfo endpoint".to_string());
    }

    let response = client
        .get(userinfo_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, body = %body, "Userinfo fetch failed");
        return Err(format!("Failed to get user info: {}", status));
    }

    let raw: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    // Normalize user info based on provider
    let user_info = match config.provider {
        SocialProvider::Google => normalize_google_user(&raw),
        SocialProvider::Github => normalize_github_user(&raw, access_token).await,
        SocialProvider::Microsoft => normalize_microsoft_user(&raw),
        SocialProvider::Facebook => normalize_facebook_user(&raw),
        _ => normalize_generic_user(&raw),
    };

    Ok(user_info)
}

fn normalize_google_user(raw: &serde_json::Value) -> SocialUserInfo {
    SocialUserInfo {
        provider: "google".to_string(),
        provider_user_id: raw["sub"].as_str().unwrap_or("").to_string(),
        email: raw["email"].as_str().map(String::from),
        email_verified: raw["email_verified"].as_bool(),
        name: raw["name"].as_str().map(String::from),
        first_name: raw["given_name"].as_str().map(String::from),
        last_name: raw["family_name"].as_str().map(String::from),
        picture: raw["picture"].as_str().map(String::from),
        raw: raw.clone(),
    }
}

async fn normalize_github_user(raw: &serde_json::Value, access_token: &str) -> SocialUserInfo {
    // GitHub's user endpoint doesn't return email if it's private
    // Need to fetch from /user/emails endpoint
    let email = if let Some(email) = raw["email"].as_str() {
        Some(email.to_string())
    } else {
        fetch_github_primary_email(access_token).await.ok()
    };

    let name = raw["name"].as_str().map(String::from);
    let (first_name, last_name) = if let Some(ref full_name) = name {
        let parts: Vec<&str> = full_name.splitn(2, ' ').collect();
        (
            parts.first().map(|s| s.to_string()),
            parts.get(1).map(|s| s.to_string()),
        )
    } else {
        (None, None)
    };

    SocialUserInfo {
        provider: "github".to_string(),
        provider_user_id: raw["id"].as_i64().map(|i| i.to_string()).unwrap_or_default(),
        email,
        email_verified: Some(true), // GitHub emails are verified
        name,
        first_name,
        last_name,
        picture: raw["avatar_url"].as_str().map(String::from),
        raw: raw.clone(),
    }
}

async fn fetch_github_primary_email(access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .header("User-Agent", "CoreAuth")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("Failed to fetch emails".to_string());
    }

    let emails: Vec<serde_json::Value> = response.json().await.map_err(|e| e.to_string())?;

    // Find primary email
    for email_obj in &emails {
        if email_obj["primary"].as_bool() == Some(true) {
            if let Some(email) = email_obj["email"].as_str() {
                return Ok(email.to_string());
            }
        }
    }

    // Fallback to first verified email
    for email_obj in &emails {
        if email_obj["verified"].as_bool() == Some(true) {
            if let Some(email) = email_obj["email"].as_str() {
                return Ok(email.to_string());
            }
        }
    }

    Err("No email found".to_string())
}

fn normalize_microsoft_user(raw: &serde_json::Value) -> SocialUserInfo {
    SocialUserInfo {
        provider: "microsoft".to_string(),
        provider_user_id: raw["id"].as_str().unwrap_or("").to_string(),
        email: raw["mail"].as_str().or(raw["userPrincipalName"].as_str()).map(String::from),
        email_verified: Some(true),
        name: raw["displayName"].as_str().map(String::from),
        first_name: raw["givenName"].as_str().map(String::from),
        last_name: raw["surname"].as_str().map(String::from),
        picture: None, // Microsoft requires additional API call for photo
        raw: raw.clone(),
    }
}

fn normalize_facebook_user(raw: &serde_json::Value) -> SocialUserInfo {
    let name = raw["name"].as_str().map(String::from);
    let (first_name, last_name) = (
        raw["first_name"].as_str().map(String::from),
        raw["last_name"].as_str().map(String::from),
    );

    SocialUserInfo {
        provider: "facebook".to_string(),
        provider_user_id: raw["id"].as_str().unwrap_or("").to_string(),
        email: raw["email"].as_str().map(String::from),
        email_verified: Some(true),
        name,
        first_name,
        last_name,
        picture: raw["picture"]["data"]["url"].as_str().map(String::from),
        raw: raw.clone(),
    }
}

fn normalize_generic_user(raw: &serde_json::Value) -> SocialUserInfo {
    SocialUserInfo {
        provider: "unknown".to_string(),
        provider_user_id: raw["sub"].as_str()
            .or(raw["id"].as_str())
            .unwrap_or("")
            .to_string(),
        email: raw["email"].as_str().map(String::from),
        email_verified: raw["email_verified"].as_bool(),
        name: raw["name"].as_str().map(String::from),
        first_name: raw["given_name"].as_str()
            .or(raw["first_name"].as_str())
            .map(String::from),
        last_name: raw["family_name"].as_str()
            .or(raw["last_name"].as_str())
            .map(String::from),
        picture: raw["picture"].as_str()
            .or(raw["avatar_url"].as_str())
            .map(String::from),
        raw: raw.clone(),
    }
}

async fn find_or_create_user(
    state: &Arc<AppState>,
    user_info: &SocialUserInfo,
    oauth_state: &SocialOAuthState,
) -> Result<(Uuid, Option<Uuid>), String> {
    let pool = state.auth_service.db.pool();

    // Get email - required for account linking
    let email = user_info.email.as_ref()
        .ok_or_else(|| "Email is required for social login".to_string())?;

    // Check if user exists with this email
    let existing_user = sqlx::query_as::<_, ciam_models::User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(user) = existing_user {
        // User exists - link social identity if not already linked
        link_social_identity(state, user.id, user_info).await?;

        return Ok((user.id, user.default_organization_id));
    }

    // New user - create account if signup is allowed
    if !oauth_state.signup && !oauth_state.request_id.is_empty() {
        // Check organization settings for signup
        // For now, allow signup
    }

    // Create new user
    let user_id = Uuid::new_v4();
    let metadata = serde_json::json!({
        "first_name": user_info.first_name,
        "last_name": user_info.last_name,
        "avatar_url": user_info.picture,
        "social_provider": user_info.provider,
    });

    sqlx::query(
        r#"
        INSERT INTO users (id, email, email_verified, metadata, is_active)
        VALUES ($1, $2, $3, $4, true)
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(user_info.email_verified.unwrap_or(false))
    .bind(&metadata)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create user: {}", e))?;

    // Link social identity
    link_social_identity(state, user_id, user_info).await?;

    info!(
        user_id = %user_id,
        email = %email,
        provider = %user_info.provider,
        "Created new user via social login"
    );

    Ok((user_id, None))
}

async fn link_social_identity(
    state: &Arc<AppState>,
    user_id: Uuid,
    user_info: &SocialUserInfo,
) -> Result<(), String> {
    let pool = state.auth_service.db.pool();

    // Check if identity already linked
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_identities WHERE user_id = $1 AND provider = $2 AND provider_user_id = $3)"
    )
    .bind(user_id)
    .bind(&user_info.provider)
    .bind(&user_info.provider_user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing {
        // Update last login
        sqlx::query(
            "UPDATE user_identities SET last_login_at = NOW(), raw_profile = $4 WHERE user_id = $1 AND provider = $2 AND provider_user_id = $3"
        )
        .bind(user_id)
        .bind(&user_info.provider)
        .bind(&user_info.provider_user_id)
        .bind(&user_info.raw)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        // Create new identity link
        sqlx::query(
            r#"
            INSERT INTO user_identities (user_id, provider, provider_user_id, raw_profile)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(&user_info.provider)
        .bind(&user_info.provider_user_id)
        .bind(&user_info.raw)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to link social identity: {}", e))?;

        info!(
            user_id = %user_id,
            provider = %user_info.provider,
            "Linked social identity to user"
        );
    }

    Ok(())
}

async fn complete_social_login(
    state: &Arc<AppState>,
    request_id: &str,
    user_id: Uuid,
    organization_id: Option<Uuid>,
) -> Response {
    use ciam_models::oauth2::{CreateAuthorizationCode, CreateLoginSession};

    if request_id.is_empty() {
        // No OAuth flow - just redirect to dashboard
        return Redirect::temporary("/dashboard").into_response();
    }

    // Get the authorization request
    let auth_request = match state.oauth2_service.get_authorization_request(request_id).await {
        Ok(req) => req,
        Err(_) => {
            return redirect_to_login_with_error(Some(request_id), "Authorization request expired");
        }
    };

    // Create authorization code
    let code = match state.oauth2_service.create_authorization_code(CreateAuthorizationCode {
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
    }).await {
        Ok(code) => code,
        Err(e) => {
            error!("Failed to create authorization code: {}", e);
            return redirect_to_login_with_error(Some(request_id), "Failed to complete authorization");
        }
    };

    // Delete the authorization request
    let _ = state.oauth2_service.delete_authorization_request(request_id).await;

    // Create login session
    let session_token = match state.oauth2_service.create_login_session(CreateLoginSession {
        user_id,
        organization_id,
        ip_address: None,
        user_agent: None,
        mfa_verified: false,
        expires_in_seconds: 86400 * 7, // 7 days
    }).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to create login session: {}", e);
            // Continue without session
            let redirect_url = format!(
                "{}?code={}&state={}",
                auth_request.redirect_uri,
                code,
                auth_request.state.as_deref().unwrap_or("")
            );
            return Redirect::temporary(&redirect_url).into_response();
        }
    };

    // Build redirect URL
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

fn redirect_to_login_with_error(request_id: Option<&str>, error: &str) -> Response {
    let url = if let Some(rid) = request_id {
        format!(
            "/login?request_id={}&error={}",
            rid,
            urlencoding::encode(error)
        )
    } else {
        format!("/login?error={}", urlencoding::encode(error))
    };
    Redirect::temporary(&url).into_response()
}
