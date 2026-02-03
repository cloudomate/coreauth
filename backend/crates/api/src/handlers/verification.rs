use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub message: String,
    pub email_verified: bool,
}

#[derive(Debug, Serialize)]
pub struct ResendVerificationResponse {
    pub message: String,
}

/// Verify email with token
/// GET /api/verify-email?token=xxx
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<VerifyEmailResponse>, (StatusCode, Json<ErrorResponse>)> {
    let verification_service = &state.verification_service;

    verification_service
        .verify_email(&query.token)
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

    Ok(Json(VerifyEmailResponse {
        message: "Email verified successfully".to_string(),
        email_verified: true,
    }))
}

/// Resend verification email
/// POST /api/auth/resend-verification
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ResendVerificationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let verification_service = &state.verification_service;

    verification_service
        .resend_verification_email(auth_user.user_id, auth_user.tenant_id)
        .await
        .map_err(|e| {
            let (status, error_code) = match &e {
                ciam_auth::AuthError::NotFound(_) => (StatusCode::NOT_FOUND, "user_not_found"),
                ciam_auth::AuthError::BadRequest(_) => {
                    (StatusCode::BAD_REQUEST, "already_verified")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(ResendVerificationResponse {
        message: "Verification email sent successfully".to_string(),
    }))
}
