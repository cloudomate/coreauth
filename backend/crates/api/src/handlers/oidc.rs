use crate::handlers::auth::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use ciam_auth::{AuthResponse, OidcProviderTemplate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct OidcLoginQuery {
    pub tenant_id: Uuid,
    pub provider_id: Uuid,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct OidcAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
}

/// Initiate OIDC login flow (tenant-scoped)
/// GET /api/oidc/login?tenant_id=xxx&provider_id=xxx&redirect_uri=http://localhost:3001/callback
pub async fn oidc_login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OidcLoginQuery>,
) -> Result<Json<OidcAuthUrlResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .oidc_service
        .get_authorization_url(params.tenant_id, params.provider_id, &params.redirect_uri)
        .await
    {
        Ok((auth_url, csrf_state)) => Ok(Json(OidcAuthUrlResponse {
            authorization_url: auth_url,
            state: csrf_state,
        })),
        Err(e) => {
            tracing::error!("OIDC login error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("oidc_login_failed", &e.to_string())),
            ))
        }
    }
}

/// Handle OIDC callback
/// GET /api/oidc/callback?code=xxx&state=xxx
/// Note: tenant_id is retrieved from the OAuth state stored during login
pub async fn oidc_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OidcCallbackQuery>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Retrieve tenant_id from the OAuth state
    let tenant_id = get_tenant_from_state(&state, &params.state).await?;

    // Handle the callback and get/create user
    let user = match state
        .oidc_service
        .handle_callback(&params.code, &params.state, tenant_id)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("OIDC callback error: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("oidc_callback_failed", &e.to_string())),
            ));
        }
    };

    // Generate access and refresh tokens for the user
    // Use default_organization_id or fallback to nil UUID
    let org_id = user.default_organization_id.unwrap_or_else(|| Uuid::nil());

    let access_token = state
        .auth_service
        .jwt
        .generate_access_token_legacy(user.id, org_id, &user.email)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("token_generation_failed", &e.to_string())),
            )
        })?;

    let refresh_token = state
        .auth_service
        .jwt
        .generate_refresh_token_legacy(user.id, org_id, &user.email)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("token_generation_failed", &e.to_string())),
            )
        })?;

    // TODO: Create session and cache user data
    // For now, just return tokens

    Ok(Json(AuthResponse::Success {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: user.into(),
    }))
}

/// Get tenant ID from OAuth state
async fn get_tenant_from_state(
    state: &Arc<AppState>,
    oauth_state: &str,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    use ciam_models::OAuthState;

    let state_key = format!("oauth:state:{}", oauth_state);
    let cached_state: OAuthState = state
        .auth_service
        .cache
        .get(&state_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("cache_error", &e.to_string())),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_state", "State not found or expired")),
            )
        })?;

    // Get tenant_id from the provider
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT tenant_id FROM oidc_providers WHERE id = $1",
    )
    .bind(cached_state.provider_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("provider_not_found", "Provider not found")),
        )
    })?;

    let tenant_id: Uuid = row.try_get("tenant_id").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    Ok(tenant_id)
}

/// List OIDC providers for a tenant
/// GET /api/oidc/providers?tenant_id=xxx
#[derive(Debug, Deserialize)]
pub struct ListProvidersQuery {
    pub tenant_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct OidcProviderInfo {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
}

pub async fn list_providers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListProvidersQuery>,
) -> Result<Json<Vec<OidcProviderInfo>>, (StatusCode, Json<ErrorResponse>)> {
    match state.oidc_service.list_providers(params.tenant_id).await {
        Ok(providers) => {
            let provider_list = providers
                .into_iter()
                .map(|p| OidcProviderInfo {
                    id: p.id,
                    name: p.name,
                    provider_type: p.provider_type,
                })
                .collect();
            Ok(Json(provider_list))
        }
        Err(e) => {
            tracing::error!("List providers error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("list_providers_failed", &e.to_string())),
            ))
        }
    }
}

/// Create OIDC provider for a tenant
/// POST /api/oidc/providers
#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
    pub scopes: Option<Vec<String>>,
    pub groups_claim: Option<String>,
    pub group_role_mappings: Option<HashMap<String, String>>,
}

pub async fn create_provider(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<Json<OidcProviderInfo>, (StatusCode, Json<ErrorResponse>)> {
    let scopes = request.scopes.unwrap_or_else(|| vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ]);

    match state
        .oidc_service
        .create_provider(
            request.tenant_id,
            request.name,
            request.provider_type,
            request.issuer,
            request.client_id,
            request.client_secret,
            request.authorization_endpoint,
            request.token_endpoint,
            request.userinfo_endpoint,
            request.jwks_uri,
            scopes,
            None,
            request.groups_claim,
            request.group_role_mappings,
        )
        .await
    {
        Ok(provider) => Ok(Json(OidcProviderInfo {
            id: provider.id,
            name: provider.name,
            provider_type: provider.provider_type,
        })),
        Err(e) => {
            tracing::error!("Create provider error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("create_provider_failed", &e.to_string())),
            ))
        }
    }
}

/// Get all available OIDC provider templates
/// GET /api/oidc/templates
pub async fn list_provider_templates() -> Json<Vec<OidcProviderTemplate>> {
    Json(ciam_auth::list_provider_templates())
}

/// Get a specific OIDC provider template by type
/// GET /api/oidc/templates/:provider_type
pub async fn get_provider_template(
    Path(provider_type): Path<String>,
) -> Result<Json<OidcProviderTemplate>, (StatusCode, Json<ErrorResponse>)> {
    match ciam_auth::get_provider_template(&provider_type) {
        Some(template) => Ok(Json(template)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "template_not_found",
                &format!("Provider template '{}' not found", provider_type),
            )),
        )),
    }
}
