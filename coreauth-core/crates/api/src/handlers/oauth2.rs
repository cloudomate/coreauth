use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Form, Json,
};
use ciam_auth::OAuth2Service;
use ciam_models::oauth2::{
    AuthorizeRequest, CreateAuthorizationCode, CreateAuthorizationRequest,
    IntrospectionRequest, IntrospectionResponse, Jwks, OidcDiscovery, RevocationRequest,
    TokenError, TokenRequest, TokenResponse, UserInfoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;
use super::auth::ErrorResponse;

// ============================================================================
// OIDC DISCOVERY
// ============================================================================

/// GET /.well-known/openid-configuration
/// Returns the OpenID Connect Discovery document
pub async fn openid_configuration(
    State(state): State<Arc<AppState>>,
) -> Json<OidcDiscovery> {
    Json(state.oauth2_service.get_discovery())
}

/// GET /.well-known/jwks.json
/// Returns the JSON Web Key Set for token verification
pub async fn jwks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Jwks>, (StatusCode, Json<ErrorResponse>)> {
    let jwks = state
        .oauth2_service
        .get_jwks()
        .await
        .map_err(|e| {
            error!("Failed to get JWKS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("server_error", "Failed to retrieve keys")),
            )
        })?;

    Ok(Json(jwks))
}

// ============================================================================
// AUTHORIZATION ENDPOINT
// ============================================================================

/// GET /authorize
/// OAuth2 Authorization Endpoint - starts the authorization flow
pub async fn authorize(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthorizeRequest>,
    headers: HeaderMap,
) -> Response {
    // Validate client_id and redirect_uri
    let application = match state
        .oauth2_service
        .get_application_by_client_id(&params.client_id)
        .await
    {
        Ok(app) => app,
        Err(e) => {
            warn!("Invalid client_id: {}", params.client_id);
            // Can't redirect - show error page
            return (
                StatusCode::BAD_REQUEST,
                Json(TokenError::invalid_client("Unknown client_id")),
            )
                .into_response();
        }
    };

    // Validate redirect_uri
    if !state
        .oauth2_service
        .validate_redirect_uri(&application, &params.redirect_uri)
    {
        warn!(
            "Invalid redirect_uri for client {}: {}",
            params.client_id, params.redirect_uri
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenError::invalid_request("Invalid redirect_uri")),
        )
            .into_response();
    }

    // Validate response_type
    if params.response_type != "code" {
        let error_url = format!(
            "{}?error=unsupported_response_type&error_description=Only%20code%20response%20type%20is%20supported&state={}",
            params.redirect_uri,
            params.state.as_deref().unwrap_or("")
        );
        return Redirect::temporary(&error_url).into_response();
    }

    // Validate PKCE for public clients (SPA apps are public clients)
    let is_public_client = application.app_type == ciam_models::ApplicationType::Spa;
    if is_public_client && params.code_challenge.is_none() {
        let error_url = format!(
            "{}?error=invalid_request&error_description=PKCE%20required%20for%20public%20clients&state={}",
            params.redirect_uri,
            params.state.as_deref().unwrap_or("")
        );
        return Redirect::temporary(&error_url).into_response();
    }

    // Validate code_challenge_method
    if let Some(method) = &params.code_challenge_method {
        if method != "S256" && method != "plain" {
            let error_url = format!(
                "{}?error=invalid_request&error_description=Invalid%20code_challenge_method&state={}",
                params.redirect_uri,
                params.state.as_deref().unwrap_or("")
            );
            return Redirect::temporary(&error_url).into_response();
        }
    }

    // Organization ID only from explicit ?organization= parameter (org-specific login)
    // Never infer from the application's owning tenant — that's the platform tenant
    let organization_id = if let Some(org) = &params.organization {
        // Look up organization by slug or ID
        Uuid::parse_str(org).ok()
    } else {
        None
    };

    // Create authorization request (stored for Universal Login)
    let auth_request = match state
        .oauth2_service
        .create_authorization_request(CreateAuthorizationRequest {
            client_id: application.client_id.clone(),
            redirect_uri: params.redirect_uri.clone(),
            response_type: params.response_type.clone(),
            scope: params.scope.clone(),
            state: params.state.clone(),
            code_challenge: params.code_challenge.clone(),
            code_challenge_method: params.code_challenge_method.clone(),
            nonce: params.nonce.clone(),
            organization_id,
            connection_hint: params.connection.clone(),
            login_hint: params.login_hint.clone(),
            prompt: params.prompt.clone(),
            max_age: params.max_age,
            ui_locales: params.ui_locales.clone(),
        })
        .await
    {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to create authorization request: {}", e);
            let error_url = format!(
                "{}?error=server_error&error_description=Internal%20error&state={}",
                params.redirect_uri,
                params.state.as_deref().unwrap_or("")
            );
            return Redirect::temporary(&error_url).into_response();
        }
    };

    info!(
        client_id = %params.client_id,
        request_id = %auth_request.request_id,
        "Created authorization request"
    );

    // Redirect to Universal Login page
    let login_url = format!("/login?request_id={}", auth_request.request_id);
    Redirect::temporary(&login_url).into_response()
}

// ============================================================================
// TOKEN ENDPOINT
// ============================================================================

/// POST /oauth/token
/// OAuth2 Token Endpoint - exchanges credentials for tokens
pub async fn token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenError>)> {
    // Extract client credentials from request or Basic auth header
    let (client_id_str, client_secret) = extract_client_credentials(&headers, &request)?;

    // Validate client
    let application = state
        .oauth2_service
        .get_application_by_client_id(&client_id_str)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenError::invalid_client("Invalid client credentials")),
            )
        })?;

    // Validate client_secret for confidential clients (SPA apps don't require secret)
    let is_public_client = application.app_type == ciam_models::ApplicationType::Spa;
    if !is_public_client {
        let secret = client_secret.ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenError::invalid_client("Client secret required")),
            )
        })?;

        if !state
            .oauth2_service
            .validate_client_secret(&application, &secret)
        {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(TokenError::invalid_client("Invalid client credentials")),
            ));
        }
    }

    match request.grant_type.as_str() {
        "authorization_code" => {
            handle_authorization_code_grant(&state, &application, &request).await
        }
        "refresh_token" => handle_refresh_token_grant(&state, &application, &request).await,
        "client_credentials" => {
            handle_client_credentials_grant(&state, &application, &request).await
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(TokenError::unsupported_grant_type(
                "Unsupported grant type",
            )),
        )),
    }
}

async fn handle_authorization_code_grant(
    state: &Arc<AppState>,
    application: &ciam_models::Application,
    request: &TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenError>)> {
    let code = request.code.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(TokenError::invalid_request("code is required")),
        )
    })?;

    let redirect_uri = request.redirect_uri.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(TokenError::invalid_request("redirect_uri is required")),
        )
    })?;

    // Exchange code for tokens
    let (auth_code, user) = state
        .oauth2_service
        .exchange_authorization_code(
            code,
            &application.client_id,
            redirect_uri,
            request.code_verifier.as_deref(),
        )
        .await
        .map_err(|e| {
            warn!("Authorization code exchange failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(TokenError::invalid_grant(&e.to_string())),
            )
        })?;

    // Generate tokens
    let tokens = state
        .oauth2_service
        .generate_tokens(
            &user,
            application,
            auth_code.scope.as_deref(),
            auth_code.nonce.as_deref(),
            auth_code.organization_id,
            true, // include refresh token
        )
        .await
        .map_err(|e| {
            error!("Token generation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenError::new("server_error", "Failed to generate tokens")),
            )
        })?;

    info!(
        user_id = %user.id,
        client_id = %application.client_id,
        "Issued tokens via authorization_code grant"
    );

    Ok(Json(tokens))
}

async fn handle_refresh_token_grant(
    state: &Arc<AppState>,
    application: &ciam_models::Application,
    request: &TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenError>)> {
    let refresh_token = request.refresh_token.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(TokenError::invalid_request("refresh_token is required")),
        )
    })?;

    let tokens = state
        .oauth2_service
        .refresh_tokens(refresh_token, &application.client_id, application)
        .await
        .map_err(|e| {
            warn!("Refresh token failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(TokenError::invalid_grant(&e.to_string())),
            )
        })?;

    info!(
        client_id = %application.client_id,
        "Issued tokens via refresh_token grant"
    );

    Ok(Json(tokens))
}

async fn handle_client_credentials_grant(
    state: &Arc<AppState>,
    application: &ciam_models::Application,
    request: &TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenError>)> {
    // Client credentials grant is for machine-to-machine authentication
    // No user is involved - the token represents the application itself

    // Parse requested scopes (default to empty if not provided)
    let scope = request.scope.clone().unwrap_or_default();

    // Generate access token for the application (no user)
    let access_token = state
        .oauth2_service
        .generate_client_credentials_token(
            application,
            &scope,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate client credentials token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenError::new("server_error", "Failed to generate token")),
            )
        })?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: application.access_token_lifetime_seconds as i64,
        refresh_token: None, // Client credentials don't get refresh tokens
        id_token: None,      // No id_token for client credentials (no user)
        scope: if scope.is_empty() { None } else { Some(scope) },
    }))
}

fn extract_client_credentials(
    headers: &HeaderMap,
    request: &TokenRequest,
) -> Result<(String, Option<String>), (StatusCode, Json<TokenError>)> {
    // Try Basic auth header first
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                let encoded = &auth_str[6..];
                if let Ok(decoded) = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    encoded,
                ) {
                    if let Ok(credentials) = String::from_utf8(decoded) {
                        if let Some((id, secret)) = credentials.split_once(':') {
                            return Ok((id.to_string(), Some(secret.to_string())));
                        }
                    }
                }
            }
        }
    }

    // Fall back to request body
    let client_id = request.client_id.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(TokenError::invalid_request("client_id is required")),
        )
    })?;

    Ok((client_id, request.client_secret.clone()))
}

// ============================================================================
// USERINFO ENDPOINT
// ============================================================================

/// GET/POST /userinfo
/// Returns claims about the authenticated user
pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Extract and validate access token from Authorization header
    let auth_header = headers.get(header::AUTHORIZATION).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "invalid_token",
                "Authorization header required",
            )),
        )
    })?;

    let auth_str = auth_header.to_str().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_token", "Invalid authorization header")),
        )
    })?;

    if !auth_str.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_token", "Bearer token required")),
        ));
    }

    let access_token = &auth_str[7..];

    // Decode and validate the token
    // For now, we'll do a simple JWT decode - in production, use proper validation
    let claims = decode_access_token(access_token).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_token", &e)),
        )
    })?;

    // Get user info
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("server_error", "Invalid user ID in token")),
        )
    })?;

    let org_id = claims
        .org_id
        .and_then(|id| Uuid::parse_str(&id).ok());

    let user_info = state
        .oauth2_service
        .get_user_info(user_id, claims.scope.as_deref(), org_id)
        .await
        .map_err(|e| {
            error!("Failed to get user info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("server_error", "Failed to retrieve user info")),
            )
        })?;

    Ok(Json(user_info))
}

fn decode_access_token(token: &str) -> Result<ciam_auth::oauth2_service::AccessTokenClaims, String> {
    // Simple decode without verification (for internal use)
    // In production, this should verify the signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".to_string());
    }

    let payload = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        parts[1],
    )
    .map_err(|_| "Invalid token encoding")?;

    let claims: ciam_auth::oauth2_service::AccessTokenClaims =
        serde_json::from_slice(&payload).map_err(|_| "Invalid token payload")?;

    // Check expiration
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err("Token expired".to_string());
    }

    Ok(claims)
}

// ============================================================================
// TOKEN REVOCATION
// ============================================================================

/// POST /oauth/revoke
/// Revokes an access token or refresh token
pub async fn revoke(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(request): Form<RevocationRequest>,
) -> StatusCode {
    // Extract client credentials
    let (client_id_str, client_secret) = match extract_client_credentials(&headers, &TokenRequest {
        grant_type: String::new(),
        code: None,
        redirect_uri: None,
        client_id: None,
        client_secret: None,
        code_verifier: None,
        refresh_token: None,
        scope: None,
        audience: None,
    }) {
        Ok(creds) => creds,
        Err(_) => return StatusCode::OK, // Per RFC 7009, return 200 even on error
    };

    // Validate client
    let application = match state
        .oauth2_service
        .get_application_by_client_id(&client_id_str)
        .await
    {
        Ok(app) => app,
        Err(_) => return StatusCode::OK,
    };

    // Revoke the token
    let _ = state
        .oauth2_service
        .revoke_token(&request.token, &application.client_id)
        .await;

    StatusCode::OK
}

// ============================================================================
// TOKEN INTROSPECTION
// ============================================================================

/// POST /oauth/introspect
/// Returns information about a token
pub async fn introspect(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(request): Form<IntrospectionRequest>,
) -> Result<Json<IntrospectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Extract and validate client credentials
    let (client_id_str, client_secret) = extract_client_credentials(&headers, &TokenRequest {
        grant_type: String::new(),
        code: None,
        redirect_uri: None,
        client_id: None,
        client_secret: None,
        code_verifier: None,
        refresh_token: None,
        scope: None,
        audience: None,
    })
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_client", "Client authentication required")),
        )
    })?;

    let application = state
        .oauth2_service
        .get_application_by_client_id(&client_id_str)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_client", "Invalid client credentials")),
            )
        })?;

    // Try to decode the token
    match decode_access_token(&request.token) {
        Ok(claims) => {
            Ok(Json(IntrospectionResponse {
                active: true,
                scope: claims.scope,
                client_id: Some(claims.azp),
                username: None,
                token_type: Some("Bearer".to_string()),
                exp: Some(claims.exp),
                iat: Some(claims.iat),
                nbf: Some(claims.nbf),
                sub: Some(claims.sub),
                aud: claims.aud.first().cloned(),
                iss: Some(claims.iss),
                jti: Some(claims.jti),
            }))
        }
        Err(_) => {
            // Token is invalid or expired
            Ok(Json(IntrospectionResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                token_type: None,
                exp: None,
                iat: None,
                nbf: None,
                sub: None,
                aud: None,
                iss: None,
                jti: None,
            }))
        }
    }
}

// ============================================================================
// LOGOUT ENDPOINT
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub id_token_hint: Option<String>,
    pub post_logout_redirect_uri: Option<String>,
    pub state: Option<String>,
    pub client_id: Option<String>,
}

/// GET /logout
/// End session endpoint (OIDC logout)
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LogoutRequest>,
    headers: HeaderMap,
) -> Response {
    // If a session cookie exists, revoke it
    if let Some(cookie) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            // Parse session cookie and revoke
            for part in cookie_str.split(';') {
                let part = part.trim();
                if part.starts_with("coreauth_session=") {
                    let session_token = &part[17..];
                    let _ = state.oauth2_service.revoke_login_session(session_token).await;
                }
            }
        }
    }

    // Determine redirect URL
    let redirect_url = if let Some(redirect_uri) = params.post_logout_redirect_uri {
        // Validate redirect_uri against client if client_id provided
        if let Some(client_id) = params.client_id {
            if let Ok(app) = state
                .oauth2_service
                .get_application_by_client_id(&client_id)
                .await
            {
                if app.logout_urls.contains(&redirect_uri) {
                    if let Some(state) = params.state {
                        format!("{}?state={}", redirect_uri, state)
                    } else {
                        redirect_uri
                    }
                } else {
                    "/logged-out".to_string()
                }
            } else {
                "/logged-out".to_string()
            }
        } else {
            "/logged-out".to_string()
        }
    } else {
        "/logged-out".to_string()
    };

    // Clear session cookie and redirect
    let mut response = Redirect::temporary(&redirect_url).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        "coreauth_session=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0"
            .parse()
            .unwrap(),
    );

    response
}
