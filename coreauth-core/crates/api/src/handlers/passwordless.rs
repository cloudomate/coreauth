//! Passwordless authentication API handlers
//!
//! Provides magic link and OTP endpoints for headless IAM.

use crate::AppState;
use crate::handlers::ErrorResponse;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use ciam_models::{
    PasswordlessStartRequest, PasswordlessStartResponse,
    PasswordlessVerifyRequest, PasswordlessVerifyResponse,
    UpdateRateLimitRequest, RateLimitsResponse, TenantRateLimit,
};
use ciam_auth::PasswordlessService;

/// Start passwordless authentication
pub async fn start_passwordless(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<PasswordlessStartRequest>,
) -> Result<Json<PasswordlessStartResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let email_service = state.email_service.clone();

    let service = PasswordlessService::new(
        state.db.pool().clone(),
        state.jwt_service.clone(),
        email_service,
        base_url,
    );

    match service.start(tenant_id, request, ip_address, user_agent).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Passwordless start failed: {}", e);
            let (status, code) = match e {
                ciam_auth::AuthError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
                ciam_auth::AuthError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "invalid_input"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };
            Err((status, Json(ErrorResponse::new(code, &e.to_string()))))
        }
    }
}

/// Verify passwordless token (magic link or OTP)
pub async fn verify_passwordless(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<PasswordlessVerifyRequest>,
) -> Result<Json<PasswordlessVerifyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let email_service = state.email_service.clone();

    let service = PasswordlessService::new(
        state.db.pool().clone(),
        state.jwt_service.clone(),
        email_service,
        base_url,
    );

    match service.verify(tenant_id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Passwordless verify failed: {}", e);
            let (status, code) = match e {
                ciam_auth::AuthError::InvalidCredentials(_) => (StatusCode::UNAUTHORIZED, "invalid_code"),
                ciam_auth::AuthError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };
            Err((status, Json(ErrorResponse::new(code, &e.to_string()))))
        }
    }
}

/// Resend passwordless token
#[derive(Debug, Deserialize)]
pub struct ResendRequest {
    pub token_id: Uuid,
}

pub async fn resend_passwordless(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<ResendRequest>,
) -> Result<Json<PasswordlessStartResponse>, (StatusCode, Json<ErrorResponse>)> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let email_service = state.email_service.clone();

    let service = PasswordlessService::new(
        state.db.pool().clone(),
        state.jwt_service.clone(),
        email_service,
        base_url,
    );

    match service.resend(tenant_id, request.token_id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Passwordless resend failed: {}", e);
            let (status, code) = match e {
                ciam_auth::AuthError::InvalidCredentials(_) => (StatusCode::NOT_FOUND, "token_not_found"),
                ciam_auth::AuthError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };
            Err((status, Json(ErrorResponse::new(code, &e.to_string()))))
        }
    }
}

/// Get rate limits for a tenant
pub async fn get_rate_limits(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<RateLimitsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

    let service = PasswordlessService::new(
        state.db.pool().clone(),
        state.jwt_service.clone(),
        None,
        base_url,
    );

    match service.get_rate_limits(tenant_id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Get rate limits failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            ))
        }
    }
}

/// Update rate limit configuration
pub async fn update_rate_limit(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<UpdateRateLimitRequest>,
) -> Result<Json<TenantRateLimit>, (StatusCode, Json<ErrorResponse>)> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

    let service = PasswordlessService::new(
        state.db.pool().clone(),
        state.jwt_service.clone(),
        None,
        base_url,
    );

    match service.update_rate_limit(tenant_id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Update rate limit failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Token Customization API
// ============================================================================

/// Request to update application token claims
#[derive(Debug, Deserialize)]
pub struct UpdateTokenClaimsRequest {
    /// Custom claims to include in tokens (static key-value pairs)
    pub custom_claims: Option<serde_json::Value>,
    /// Claims to include in ID tokens
    pub id_token_claims: Option<Vec<String>>,
    /// Claims to include in access tokens
    pub access_token_claims: Option<Vec<String>>,
}

/// Response with updated token configuration
#[derive(Debug, Serialize)]
pub struct TokenClaimsResponse {
    pub custom_claims: serde_json::Value,
    pub id_token_claims: Vec<String>,
    pub access_token_claims: Vec<String>,
}

/// Get token claims configuration for an application
pub async fn get_token_claims(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TokenClaimsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result: Option<(serde_json::Value, Vec<String>, Vec<String>)> = sqlx::query_as(
        r#"
        SELECT custom_claims, id_token_claims, access_token_claims
        FROM applications
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(app_id)
    .bind(tenant_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    match result {
        Some((custom_claims, id_token_claims, access_token_claims)) => {
            Ok(Json(TokenClaimsResponse {
                custom_claims,
                id_token_claims,
                access_token_claims,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Application not found")),
        )),
    }
}

/// Update token claims configuration for an application
pub async fn update_token_claims(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, app_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateTokenClaimsRequest>,
) -> Result<Json<TokenClaimsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Build dynamic update query
    let result: Option<(serde_json::Value, Vec<String>, Vec<String>)> = sqlx::query_as(
        r#"
        UPDATE applications
        SET
            custom_claims = COALESCE($3, custom_claims),
            id_token_claims = COALESCE($4, id_token_claims),
            access_token_claims = COALESCE($5, access_token_claims),
            updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING custom_claims, id_token_claims, access_token_claims
        "#,
    )
    .bind(app_id)
    .bind(tenant_id)
    .bind(&request.custom_claims)
    .bind(&request.id_token_claims)
    .bind(&request.access_token_claims)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    match result {
        Some((custom_claims, id_token_claims, access_token_claims)) => {
            Ok(Json(TokenClaimsResponse {
                custom_claims,
                id_token_claims,
                access_token_claims,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Application not found")),
        )),
    }
}
