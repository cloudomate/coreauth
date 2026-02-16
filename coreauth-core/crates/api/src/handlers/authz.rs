use crate::handlers::auth::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use ciam_authz::{
    Application, ApplicationService, ApplicationType, ApplicationWithSecret,
    CheckRequest, CheckResponse, CreateApplicationRequest, CreateTupleRequest,
    ExpandResponse, PolicyEngine, QueryTuplesRequest, RelationTuple, SubjectType,
    TupleService,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Application Management Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub application_type: ApplicationType,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Create a new application (service principal)
pub async fn create_application(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateAppRequest>,
) -> Result<Json<ApplicationWithSecret>, (StatusCode, Json<ErrorResponse>)> {
    let create_request = CreateApplicationRequest {
        tenant_id: request.tenant_id,
        name: request.name,
        description: request.description,
        application_type: request.application_type,
        redirect_uris: request.redirect_uris,
        allowed_scopes: request.allowed_scopes,
        metadata: request.metadata,
    };

    match state.application_service.create_application(create_request).await {
        Ok(app_with_secret) => Ok(Json(app_with_secret)),
        Err(e) => {
            tracing::error!("Failed to create application: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "create_application_failed",
                    &e.to_string(),
                )),
            ))
        }
    }
}

/// Get application by ID
pub async fn get_application(
    State(state): State<Arc<AppState>>,
    Path((app_id, tenant_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Application>, (StatusCode, Json<ErrorResponse>)> {
    match state.application_service.get_application(app_id, tenant_id).await {
        Ok(app) => Ok(Json(app)),
        Err(e) => {
            tracing::error!("Failed to get application: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("application_not_found", &e.to_string())),
            ))
        }
    }
}

/// List applications for a tenant
pub async fn list_applications(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<Application>>, (StatusCode, Json<ErrorResponse>)> {
    match state.application_service.list_applications(tenant_id).await {
        Ok(apps) => Ok(Json(apps)),
        Err(e) => {
            tracing::error!("Failed to list applications: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_applications_failed", &e.to_string())),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub allowed_scopes: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

/// Update application
pub async fn update_application(
    State(state): State<Arc<AppState>>,
    Path((app_id, tenant_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateAppRequest>,
) -> Result<Json<Application>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .application_service
        .update_application(
            app_id,
            tenant_id,
            request.name,
            request.description,
            request.redirect_uris,
            request.allowed_scopes,
            request.is_active,
        )
        .await
    {
        Ok(app) => Ok(Json(app)),
        Err(e) => {
            tracing::error!("Failed to update application: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("update_application_failed", &e.to_string())),
            ))
        }
    }
}

/// Rotate client secret
pub async fn rotate_secret(
    State(state): State<Arc<AppState>>,
    Path((app_id, tenant_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApplicationWithSecret>, (StatusCode, Json<ErrorResponse>)> {
    match state.application_service.rotate_secret(app_id, tenant_id).await {
        Ok(app_with_secret) => Ok(Json(app_with_secret)),
        Err(e) => {
            tracing::error!("Failed to rotate secret: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("rotate_secret_failed", &e.to_string())),
            ))
        }
    }
}

/// Delete application
pub async fn delete_application(
    State(state): State<Arc<AppState>>,
    Path((app_id, tenant_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.application_service.delete_application(app_id, tenant_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete application: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("delete_application_failed", &e.to_string())),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateAppRequest {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct AuthenticateAppResponse {
    pub application: Application,
    pub access_token: String,
}

/// Authenticate application using client credentials
pub async fn authenticate_application(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AuthenticateAppRequest>,
) -> Result<Json<AuthenticateAppResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .application_service
        .authenticate(&request.client_id, &request.client_secret)
        .await
    {
        Ok(app) => {
            // Generate JWT token for the application
            // For now, we'll use a simple token format. In production, use proper JWT
            let token = format!("app_token_{}", app.id);

            Ok(Json(AuthenticateAppResponse {
                application: app,
                access_token: token,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to authenticate application: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("authentication_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Relation Tuple Management Handlers
// ============================================================================

/// Create a relation tuple
pub async fn create_tuple(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTupleRequest>,
) -> Result<Json<RelationTuple>, (StatusCode, Json<ErrorResponse>)> {
    match state.tuple_service.create_tuple(request).await {
        Ok(tuple) => Ok(Json(tuple)),
        Err(e) => {
            tracing::error!("Failed to create tuple: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("create_tuple_failed", &e.to_string())),
            ))
        }
    }
}

/// Delete a relation tuple
pub async fn delete_tuple(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTupleRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.tuple_service.delete_tuple(request).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete tuple: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("delete_tuple_failed", &e.to_string())),
            ))
        }
    }
}

/// Query relation tuples
pub async fn query_tuples(
    State(state): State<Arc<AppState>>,
    Json(request): Json<QueryTuplesRequest>,
) -> Result<Json<Vec<RelationTuple>>, (StatusCode, Json<ErrorResponse>)> {
    match state.tuple_service.query_tuples(request).await {
        Ok(tuples) => Ok(Json(tuples)),
        Err(e) => {
            tracing::error!("Failed to query tuples: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("query_tuples_failed", &e.to_string())),
            ))
        }
    }
}

/// Get all tuples for a specific object
pub async fn get_object_tuples(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, namespace, object_id)): Path<(Uuid, String, String)>,
) -> Result<Json<Vec<RelationTuple>>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .tuple_service
        .get_object_tuples(tenant_id, &namespace, &object_id)
        .await
    {
        Ok(tuples) => Ok(Json(tuples)),
        Err(e) => {
            tracing::error!("Failed to get object tuples: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("get_object_tuples_failed", &e.to_string())),
            ))
        }
    }
}

/// Get all tuples for a specific subject
pub async fn get_subject_tuples(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, subject_type, subject_id)): Path<(Uuid, String, String)>,
) -> Result<Json<Vec<RelationTuple>>, (StatusCode, Json<ErrorResponse>)> {
    // Parse subject_type string to SubjectType enum
    let subject_type = SubjectType::try_from(subject_type).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_subject_type", &e)),
        )
    })?;

    match state
        .tuple_service
        .get_subject_tuples(tenant_id, subject_type, &subject_id)
        .await
    {
        Ok(tuples) => Ok(Json(tuples)),
        Err(e) => {
            tracing::error!("Failed to get subject tuples: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("get_subject_tuples_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Authorization Check Handlers
// ============================================================================

/// Check if a subject has a relation to an object
pub async fn check_permission(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.policy_engine.check(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to check permission: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("check_permission_failed", &e.to_string())),
            ))
        }
    }
}

/// Expand a relation to show all subjects that have it
pub async fn expand_relation(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, namespace, object_id, relation)): Path<(Uuid, String, String, String)>,
) -> Result<Json<ExpandResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .policy_engine
        .expand(tenant_id, &namespace, &object_id, &relation)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to expand relation: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("expand_relation_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Forward Auth Handler (for downstream apps using Nginx/Traefik)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ForwardAuthRequest {
    pub tenant_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub relation: String,
    pub namespace: String,
    pub object_id: String,
}

/// Forward auth endpoint for Nginx auth_request or Traefik ForwardAuth
/// Returns 200 if allowed, 403 if denied
pub async fn forward_auth(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ForwardAuthRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // You can also extract subject info from Authorization header if needed
    // For now, we'll use the request body

    let check_request = CheckRequest {
        tenant_id: request.tenant_id,
        subject_type: request.subject_type,
        subject_id: request.subject_id,
        relation: request.relation,
        namespace: request.namespace,
        object_id: request.object_id,
        context: Default::default(),
    };

    match state.policy_engine.check(check_request).await {
        Ok(response) => {
            if response.allowed {
                Ok(StatusCode::OK)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse::new(
                        "permission_denied",
                        response.reason.as_deref().unwrap_or("Access denied"),
                    )),
                ))
            }
        }
        Err(e) => {
            tracing::error!("Forward auth check failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("authorization_check_failed", &e.to_string())),
            ))
        }
    }
}

/// Forward auth GET endpoint (for Traefik ForwardAuth)
/// Extracts authorization info from headers
pub async fn forward_auth_get(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Extract authorization information from headers
    // Common headers used:
    // X-Tenant-ID, X-Subject-Type, X-Subject-ID, X-Relation, X-Namespace, X-Object-ID

    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_tenant_id", "X-Tenant-ID header required")),
            )
        })?;

    let subject_type_str = headers
        .get("x-subject-type")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_subject_type", "X-Subject-Type header required")),
            )
        })?;

    let subject_type = SubjectType::try_from(subject_type_str.to_string()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_subject_type", &e)),
        )
    })?;

    let subject_id = headers
        .get("x-subject-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_subject_id", "X-Subject-ID header required")),
            )
        })?;

    let relation = headers
        .get("x-relation")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_relation", "X-Relation header required")),
            )
        })?;

    let namespace = headers
        .get("x-namespace")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_namespace", "X-Namespace header required")),
            )
        })?;

    let object_id = headers
        .get("x-object-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("missing_object_id", "X-Object-ID header required")),
            )
        })?;

    let check_request = CheckRequest {
        tenant_id,
        subject_type,
        subject_id: subject_id.to_string(),
        relation: relation.to_string(),
        namespace: namespace.to_string(),
        object_id: object_id.to_string(),
        context: Default::default(),
    };

    match state.policy_engine.check(check_request).await {
        Ok(response) => {
            if response.allowed {
                Ok(StatusCode::OK)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse::new(
                        "permission_denied",
                        response.reason.as_deref().unwrap_or("Access denied"),
                    )),
                ))
            }
        }
        Err(e) => {
            tracing::error!("Forward auth check failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("authorization_check_failed", &e.to_string())),
            ))
        }
    }
}
