use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use ciam_auth::{ActionService, AuthError};
use ciam_models::{
    Action, ActionContext, ActionExecution, ActionResult, CreateAction, UpdateAction,
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
pub struct ActionListResponse {
    pub actions: Vec<Action>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct ExecutionListResponse {
    pub executions: Vec<ActionExecution>,
    pub limit: i64,
    pub offset: i64,
}

/// Create a new action
/// POST /api/organizations/:org_id/actions
pub async fn create_action(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(mut request): Json<CreateAction>,
) -> Result<Json<Action>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    // Set organization_id from path
    request.organization_id = org_id;

    let service = ActionService::new(state.auth_service.db.clone());

    let action = service
        .create(request, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(action))
}

/// Get action by ID
/// GET /api/organizations/:org_id/actions/:action_id
pub async fn get_action(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, action_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Action>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ActionService::new(state.auth_service.db.clone());

    let action = service
        .get(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    // Verify action belongs to the organization
    if action.organization_id != org_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Action not found in this organization",
            )),
        ));
    }

    Ok(Json(action))
}

/// List actions for an organization
/// GET /api/organizations/:org_id/actions
pub async fn list_actions(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ActionListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ActionService::new(state.auth_service.db.clone());

    let actions = service
        .list_by_organization(org_id, query.limit, query.offset)
        .await
        .map_err(|e| handle_error(e))?;

    let total = service
        .count_by_organization(org_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(ActionListResponse {
        actions,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

/// Update action
/// PUT /api/organizations/:org_id/actions/:action_id
pub async fn update_action(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, action_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateAction>,
) -> Result<Json<Action>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ActionService::new(state.auth_service.db.clone());

    // Verify action belongs to the organization
    let existing = service
        .get(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != org_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Action not found in this organization",
            )),
        ));
    }

    let action = service
        .update(action_id, request, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(action))
}

/// Delete action
/// DELETE /api/organizations/:org_id/actions/:action_id
pub async fn delete_action(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, action_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ActionService::new(state.auth_service.db.clone());

    // Verify action belongs to the organization
    let existing = service
        .get(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != org_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Action not found in this organization",
            )),
        ));
    }

    service
        .delete(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Test action with sample context
/// POST /api/organizations/:org_id/actions/:action_id/test
pub async fn test_action(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, action_id)): Path<(Uuid, Uuid)>,
    Json(context): Json<ActionContext>,
) -> Result<Json<ActionResult>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has admin role in this organization

    let service = ActionService::new(state.auth_service.db.clone());

    // Verify action belongs to the organization
    let existing = service
        .get(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != org_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Action not found in this organization",
            )),
        ));
    }

    let result = service
        .test_action(action_id, context, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(result))
}

/// Get action execution history
/// GET /api/organizations/:org_id/actions/:action_id/executions
pub async fn get_action_executions(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((org_id, action_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ExecutionListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ActionService::new(state.auth_service.db.clone());

    // Verify action belongs to the organization
    let existing = service
        .get(action_id, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    if existing.organization_id != org_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                "Action not found in this organization",
            )),
        ));
    }

    let executions = service
        .get_executions(action_id, query.limit, query.offset, auth_user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(ExecutionListResponse {
        executions,
        limit: query.limit,
        offset: query.offset,
    }))
}

/// Get recent executions for all actions in an organization
/// GET /api/organizations/:org_id/actions/executions
pub async fn get_organization_executions(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ExecutionListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Check if user has access to this organization

    let service = ActionService::new(state.auth_service.db.clone());

    let executions = service
        .get_organization_executions(org_id, query.limit, query.offset)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(ExecutionListResponse {
        executions,
        limit: query.limit,
        offset: query.offset,
    }))
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
