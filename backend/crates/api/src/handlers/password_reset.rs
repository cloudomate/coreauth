use crate::handlers::auth::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct RequestPasswordResetRequest {
    pub tenant_id: uuid::Uuid,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyResetTokenQuery {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
}

/// Request password reset
/// POST /api/auth/forgot-password
pub async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RequestPasswordResetRequest>,
) -> Result<Json<PasswordResetResponse>, (StatusCode, Json<ErrorResponse>)> {
    let password_reset_service = &state.password_reset_service;

    // Extract IP address from headers
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown");

    // Extract user agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok());

    password_reset_service
        .request_password_reset(
            request.tenant_id,
            &request.email,
            ip_address,
            user_agent,
        )
        .await
        .map_err(|e| {
            tracing::error!("Password reset request error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("request_failed", &e.to_string())),
            )
        })?;

    // Always return success (even if email doesn't exist) to prevent user enumeration
    Ok(Json(PasswordResetResponse {
        message: "If an account exists with that email, a password reset link has been sent"
            .to_string(),
    }))
}

/// Verify reset token (optional endpoint to check token validity)
/// GET /api/auth/verify-reset-token?token=xxx
pub async fn verify_reset_token(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyResetTokenQuery>,
) -> Result<Json<PasswordResetResponse>, (StatusCode, Json<ErrorResponse>)> {
    let password_reset_service = &state.password_reset_service;

    password_reset_service
        .verify_reset_token(&query.token)
        .await
        .map_err(|e| {
            let (status, error_code) = match &e {
                ciam_auth::AuthError::InvalidToken(_) => {
                    (StatusCode::BAD_REQUEST, "invalid_token")
                }
                ciam_auth::AuthError::TokenExpired => (StatusCode::BAD_REQUEST, "token_expired"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(PasswordResetResponse {
        message: "Token is valid".to_string(),
    }))
}

/// Reset password with token
/// POST /api/auth/reset-password
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<PasswordResetResponse>, (StatusCode, Json<ErrorResponse>)> {
    let password_reset_service = &state.password_reset_service;

    // Validate password strength (basic validation, can be enhanced)
    if request.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "weak_password",
                "Password must be at least 8 characters long",
            )),
        ));
    }

    password_reset_service
        .reset_password(&request.token, &request.new_password)
        .await
        .map_err(|e| {
            tracing::error!("Password reset error: {}", e);
            let (status, error_code) = match &e {
                ciam_auth::AuthError::InvalidToken(_) => {
                    (StatusCode::BAD_REQUEST, "invalid_token")
                }
                ciam_auth::AuthError::TokenExpired => (StatusCode::BAD_REQUEST, "token_expired"),
                ciam_auth::AuthError::WeakPassword(_) => {
                    (StatusCode::BAD_REQUEST, "weak_password")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(PasswordResetResponse {
        message: "Password reset successfully. Please log in with your new password.".to_string(),
    }))
}
