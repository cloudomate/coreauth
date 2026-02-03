use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    #[serde(default = "default_expires_in_days")]
    pub expires_in_days: i64,
}

fn default_expires_in_days() -> i64 {
    7
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationRequest {
    pub token: String,
    pub password: String,
    pub full_name: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyInvitationQuery {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub invited_by: Uuid,
    pub role_id: Option<Uuid>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateInvitationResponse {
    pub invitation_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AcceptInvitationResponse {
    pub user_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Create invitation (Tenant Admin only)
/// POST /api/tenants/:tenant_id/invitations
pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<CreateInvitationRequest>,
) -> Result<Json<CreateInvitationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let invitation_service = &state.invitation_service;

    let invitation_id = invitation_service
        .create_invitation(
            tenant_id,
            &request.email,
            auth_user.user_id,
            request.role_id,
            request.metadata,
            request.expires_in_days,
        )
        .await
        .map_err(|e| {
            tracing::error!("Create invitation error: {}", e);
            let (status, error_code) = match &e {
                ciam_auth::AuthError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(CreateInvitationResponse {
        invitation_id,
        message: format!("Invitation sent to {}", request.email),
    }))
}

/// Verify invitation token (public)
/// GET /api/invitations/verify?token=xxx
pub async fn verify_invitation(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyInvitationQuery>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let invitation_service = &state.invitation_service;

    let invitation = invitation_service
        .verify_invitation(&query.token)
        .await
        .map_err(|e| {
            let (status, error_code) = match &e {
                ciam_auth::AuthError::InvalidToken(_) => {
                    (StatusCode::BAD_REQUEST, "invalid_token")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(InvitationResponse {
        id: invitation.id,
        tenant_id: invitation.tenant_id,
        email: invitation.email,
        invited_by: invitation.invited_by,
        role_id: invitation.role_id,
        expires_at: invitation.expires_at,
        created_at: invitation.created_at,
        accepted_at: invitation.accepted_at,
    }))
}

/// Accept invitation and create account (public)
/// POST /api/invitations/accept
pub async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AcceptInvitationRequest>,
) -> Result<Json<AcceptInvitationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let invitation_service = &state.invitation_service;

    let user_id = invitation_service
        .accept_invitation(
            &request.token,
            &request.password,
            &request.full_name,
            request.metadata,
        )
        .await
        .map_err(|e| {
            tracing::error!("Accept invitation error: {}", e);
            let (status, error_code) = match &e {
                ciam_auth::AuthError::InvalidToken(_) => {
                    (StatusCode::BAD_REQUEST, "invalid_token")
                }
                ciam_auth::AuthError::WeakPassword(_) => {
                    (StatusCode::BAD_REQUEST, "weak_password")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(AcceptInvitationResponse {
        user_id,
        message: "Account created successfully. You can now log in.".to_string(),
    }))
}

/// List invitations for a tenant (Tenant Admin only)
/// GET /api/tenants/:tenant_id/invitations
pub async fn list_invitations(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<InvitationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let invitation_service = &state.invitation_service;

    let invitations = invitation_service
        .list_invitations(tenant_id, false)
        .await
        .map_err(|e| {
            tracing::error!("List invitations error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    let response: Vec<InvitationResponse> = invitations
        .into_iter()
        .map(|inv| InvitationResponse {
            id: inv.id,
            tenant_id: inv.tenant_id,
            email: inv.email,
            invited_by: inv.invited_by,
            role_id: inv.role_id,
            expires_at: inv.expires_at,
            created_at: inv.created_at,
            accepted_at: inv.accepted_at,
        })
        .collect();

    Ok(Json(response))
}

/// Revoke invitation (Tenant Admin only)
/// DELETE /api/tenants/:tenant_id/invitations/:invitation_id
pub async fn revoke_invitation(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let invitation_service = &state.invitation_service;

    invitation_service
        .revoke_invitation(invitation_id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("Revoke invitation error: {}", e);
            let (status, error_code) = match &e {
                ciam_auth::AuthError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(MessageResponse {
        message: "Invitation revoked successfully".to_string(),
    }))
}

/// Resend invitation (Tenant Admin only)
/// POST /api/tenants/:tenant_id/invitations/:invitation_id/resend
pub async fn resend_invitation(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let invitation_service = &state.invitation_service;

    invitation_service
        .resend_invitation(invitation_id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!("Resend invitation error: {}", e);
            let (status, error_code) = match &e {
                ciam_auth::AuthError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            };

            (status, Json(ErrorResponse::new(error_code, &e.to_string())))
        })?;

    Ok(Json(MessageResponse {
        message: "Invitation resent successfully".to_string(),
    }))
}
