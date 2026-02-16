use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use ciam_auth::{ApplicationService, AuthError};
use ciam_models::{
    Application, ApplicationWithSecret, CreateApplication, UpdateApplication,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct ApplicationListResponse {
    pub applications: Vec<Application>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Create a new application
/// POST /api/organizations/:org_id/applications
pub async fn create_application(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(mut request): Json<CreateApplication>,
) -> Result<Json<ApplicationWithSecret>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    // Set the organization_id from the path parameter
    request.organization_id = Some(org_id);

    let service = ApplicationService::new(state.auth_service.db.clone());

    let app_with_secret = service
        .create(request, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(app_with_secret))
}

/// Get application by ID
/// GET /api/organizations/:org_id/applications/:app_id
pub async fn get_application(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Application>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ApplicationService::new(state.auth_service.db.clone());

    let app = service
        .get(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    // Verify app belongs to the organization
    if app.organization_id != Some(org_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Application not found in this organization",
            )),
        ));
    }

    Ok(Json(app))
}

/// List applications for an organization
/// GET /api/organizations/:org_id/applications
pub async fn list_applications(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApplicationListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ApplicationService::new(state.auth_service.db.clone());

    let applications = service
        .list_by_organization(org_id, query.limit, query.offset)
        .await
        .map_err(|e| handle_error(e))?;

    let total = service
        .count_by_organization(org_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(ApplicationListResponse {
        applications,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

/// Update application
/// PUT /api/organizations/:org_id/applications/:app_id
pub async fn update_application(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateApplication>,
) -> Result<Json<Application>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ApplicationService::new(state.auth_service.db.clone());

    // Verify app belongs to the organization
    let existing = service
        .get(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != Some(org_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Application not found in this organization",
            )),
        ));
    }

    let app = service
        .update(app_id, request, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(app))
}

/// Rotate application client secret
/// POST /api/organizations/:org_id/applications/:app_id/rotate-secret
pub async fn rotate_secret(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApplicationWithSecret>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ApplicationService::new(state.auth_service.db.clone());

    // Verify app belongs to the organization
    let existing = service
        .get(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != Some(org_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Application not found in this organization",
            )),
        ));
    }

    let app_with_secret = service
        .rotate_secret(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(app_with_secret))
}

/// Delete application
/// DELETE /api/organizations/:org_id/applications/:app_id
pub async fn delete_application(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ApplicationService::new(state.auth_service.db.clone());

    // Verify app belongs to the organization
    let existing = service
        .get(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != Some(org_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Application not found in this organization",
            )),
        ));
    }

    service
        .delete(app_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(StatusCode::NO_CONTENT)
}

// Helper function to convert AuthError to HTTP response
fn handle_error(error: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        AuthError::NotFound(msg) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &msg)),
        ),
        AuthError::InvalidInput(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_input", &msg)),
        ),
        AuthError::ValidationError(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("validation_error", &msg)),
        ),
        AuthError::Forbidden(msg) => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("forbidden", &msg)),
        ),
        AuthError::AlreadyExists(msg) => (
            StatusCode::CONFLICT,
            Json(ErrorResponse::new("already_exists", &msg)),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "An internal error occurred")),
        ),
    }
}
