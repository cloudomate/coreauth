//! FGA Store Management Handlers
//!
//! API endpoints for managing Fine-Grained Authorization stores,
//! authorization models, and API keys.

use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use ciam_authz::{
    AuthorizationModel, AuthorizationSchema, CreateApiKeyRequest, CreateStoreRequest,
    FgaStore, FgaStoreApiKey, FgaStoreApiKeyWithSecret, FgaStoreService, UpdateStoreRequest,
    WriteModelRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Store Management Endpoints
// ============================================================================

/// Create a new FGA store
/// POST /api/fga/stores
pub async fn create_store(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateStoreRequest>,
) -> Result<Json<FgaStore>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.create_store(auth_user.tenant_id, request).await {
        Ok(store) => Ok(Json(store)),
        Err(e) => {
            tracing::error!("Failed to create FGA store: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("create_store_failed", &e.to_string())),
            ))
        }
    }
}

/// Get a store by ID
/// GET /api/fga/stores/:store_id
pub async fn get_store(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> Result<Json<FgaStore>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.get_store(store_id).await {
        Ok(store) => Ok(Json(store)),
        Err(e) => {
            tracing::error!("Failed to get FGA store: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("store_not_found", &e.to_string())),
            ))
        }
    }
}

/// List all stores
/// GET /api/fga/stores
#[derive(Debug, Deserialize)]
pub struct ListStoresQuery {
    pub include_inactive: Option<bool>,
}

pub async fn list_stores(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    axum::extract::Query(query): axum::extract::Query<ListStoresQuery>,
) -> Result<Json<Vec<FgaStore>>, (StatusCode, Json<ErrorResponse>)> {
    let include_inactive = query.include_inactive.unwrap_or(false);

    match state.fga_store_service.list_stores(auth_user.tenant_id, include_inactive).await {
        Ok(stores) => Ok(Json(stores)),
        Err(e) => {
            tracing::error!("Failed to list FGA stores: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_stores_failed", &e.to_string())),
            ))
        }
    }
}

/// Update a store
/// PATCH /api/fga/stores/:store_id
pub async fn update_store(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Json(request): Json<UpdateStoreRequest>,
) -> Result<Json<FgaStore>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.update_store(store_id, request).await {
        Ok(store) => Ok(Json(store)),
        Err(e) => {
            tracing::error!("Failed to update FGA store: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("update_store_failed", &e.to_string())),
            ))
        }
    }
}

/// Delete a store
/// DELETE /api/fga/stores/:store_id
#[derive(Debug, Deserialize)]
pub struct DeleteStoreQuery {
    pub hard_delete: Option<bool>,
}

pub async fn delete_store(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<DeleteStoreQuery>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let hard_delete = query.hard_delete.unwrap_or(false);

    match state.fga_store_service.delete_store(store_id, hard_delete).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete FGA store: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("delete_store_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Authorization Model Endpoints
// ============================================================================

/// Write a new authorization model
/// POST /api/fga/stores/:store_id/models
pub async fn write_model(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Json(request): Json<WriteModelRequest>,
) -> Result<Json<AuthorizationModel>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.write_model(store_id, request).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => {
            tracing::error!("Failed to write authorization model: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("write_model_failed", &e.to_string())),
            ))
        }
    }
}

/// Get the current authorization model
/// GET /api/fga/stores/:store_id/models/current
pub async fn get_current_model(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> Result<Json<AuthorizationModel>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.get_current_model(store_id).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => {
            tracing::error!("Failed to get current model: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("model_not_found", &e.to_string())),
            ))
        }
    }
}

/// Get a specific model version
/// GET /api/fga/stores/:store_id/models/:version
pub async fn get_model_version(
    State(state): State<Arc<AppState>>,
    Path((store_id, version)): Path<(Uuid, i32)>,
) -> Result<Json<AuthorizationModel>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.get_model_version(store_id, version).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => {
            tracing::error!("Failed to get model version: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("model_not_found", &e.to_string())),
            ))
        }
    }
}

/// List all model versions
/// GET /api/fga/stores/:store_id/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> Result<Json<Vec<AuthorizationModel>>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.list_models(store_id).await {
        Ok(models) => Ok(Json(models)),
        Err(e) => {
            tracing::error!("Failed to list models: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_models_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// API Key Management Endpoints
// ============================================================================

/// Create an API key for a store
/// POST /api/fga/stores/:store_id/api-keys
pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<FgaStoreApiKeyWithSecret>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.create_api_key(store_id, request).await {
        Ok(key) => Ok(Json(key)),
        Err(e) => {
            tracing::error!("Failed to create API key: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("create_api_key_failed", &e.to_string())),
            ))
        }
    }
}

/// List API keys for a store
/// GET /api/fga/stores/:store_id/api-keys
pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
) -> Result<Json<Vec<FgaStoreApiKey>>, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.list_api_keys(store_id).await {
        Ok(keys) => Ok(Json(keys)),
        Err(e) => {
            tracing::error!("Failed to list API keys: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_api_keys_failed", &e.to_string())),
            ))
        }
    }
}

/// Revoke an API key
/// DELETE /api/fga/stores/:store_id/api-keys/:key_id
pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Path((_store_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.fga_store_service.revoke_api_key(key_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to revoke API key: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("revoke_api_key_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Store-Scoped Authorization Endpoints (using API key)
// ============================================================================

/// Response for FGA check with store context
#[derive(Debug, Serialize)]
pub struct StoreCheckResponse {
    pub allowed: bool,
    pub store_id: Uuid,
    pub resolution_metadata: Option<serde_json::Value>,
}

/// Check permission within a store context
/// POST /api/fga/stores/:store_id/check
/// This endpoint validates against the store's authorization model
#[derive(Debug, Deserialize)]
pub struct StoreCheckRequest {
    pub subject_type: String,
    pub subject_id: String,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
    pub context: Option<serde_json::Value>,
}

pub async fn store_check(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Json(request): Json<StoreCheckRequest>,
) -> Result<Json<StoreCheckResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify the store exists and get the current model
    let store = match state.fga_store_service.get_store(store_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("store_not_found", &e.to_string())),
            ));
        }
    };

    if !store.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("store_inactive", "Store is not active")),
        ));
    }

    // Load the current authorization model for model-aware resolution
    let model = state.fga_store_service.get_current_model(store_id).await.ok();
    let schema = model.as_ref().map(|m| &m.schema_json.0);

    // Convert to the internal check request format
    let subject_type = ciam_authz::SubjectType::try_from(request.subject_type.clone())
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_subject_type", &e)),
            )
        })?;

    let check_request = ciam_authz::CheckRequest {
        tenant_id: store_id,  // Use store_id as the namespace for tuples
        subject_type,
        subject_id: request.subject_id,
        relation: request.relation,
        namespace: request.object_type,
        object_id: request.object_id,
        context: request.context
            .map(|c| c.as_object().cloned().unwrap_or_default())
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect(),
    };

    match state.policy_engine.check_with_model(check_request, schema).await {
        Ok(response) => Ok(Json(StoreCheckResponse {
            allowed: response.allowed,
            store_id,
            resolution_metadata: response.reason.map(|r| serde_json::json!({"reason": r})),
        })),
        Err(e) => {
            tracing::error!("FGA check failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("check_failed", &e.to_string())),
            ))
        }
    }
}

/// Write tuples to a store
/// POST /api/fga/stores/:store_id/tuples
#[derive(Debug, Deserialize)]
pub struct WriteTuplesRequest {
    pub writes: Vec<TupleWrite>,
    pub deletes: Option<Vec<TupleWrite>>,
}

#[derive(Debug, Deserialize)]
pub struct TupleWrite {
    pub subject_type: String,
    pub subject_id: String,
    pub subject_relation: Option<String>,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
}

#[derive(Debug, Serialize)]
pub struct WriteTuplesResponse {
    pub written: usize,
    pub deleted: usize,
}

pub async fn write_tuples(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    Json(request): Json<WriteTuplesRequest>,
) -> Result<Json<WriteTuplesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify store exists and is active
    let store = match state.fga_store_service.get_store(store_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("store_not_found", &e.to_string())),
            ));
        }
    };

    if !store.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("store_inactive", "Store is not active")),
        ));
    }

    let mut written = 0;
    let mut deleted = 0;

    // Process writes
    for tuple in request.writes {
        let subject_type = ciam_authz::SubjectType::try_from(tuple.subject_type.clone())
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("invalid_subject_type", &e)),
                )
            })?;

        let create_request = ciam_authz::CreateTupleRequest {
            tenant_id: store_id,
            namespace: tuple.object_type,
            object_id: tuple.object_id,
            relation: tuple.relation,
            subject_type,
            subject_id: tuple.subject_id,
            subject_relation: tuple.subject_relation,
        };

        match state.tuple_service.create_tuple(create_request).await {
            Ok(_) => written += 1,
            Err(e) => {
                tracing::warn!("Failed to write tuple: {}", e);
                // Continue with other tuples
            }
        }
    }

    // Process deletes
    if let Some(deletes) = request.deletes {
        for tuple in deletes {
            let subject_type = ciam_authz::SubjectType::try_from(tuple.subject_type.clone())
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new("invalid_subject_type", &e)),
                    )
                })?;

            let delete_request = ciam_authz::CreateTupleRequest {
                tenant_id: store_id,
                namespace: tuple.object_type,
                object_id: tuple.object_id,
                relation: tuple.relation,
                subject_type,
                subject_id: tuple.subject_id,
                subject_relation: tuple.subject_relation,
            };

            match state.tuple_service.delete_tuple(delete_request).await {
                Ok(_) => deleted += 1,
                Err(e) => {
                    tracing::warn!("Failed to delete tuple: {}", e);
                    // Continue with other tuples
                }
            }
        }
    }

    Ok(Json(WriteTuplesResponse { written, deleted }))
}

/// Read tuples from a store
/// GET /api/fga/stores/:store_id/tuples
#[derive(Debug, Deserialize)]
pub struct ReadTuplesQuery {
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub relation: Option<String>,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
}

pub async fn read_tuples(
    State(state): State<Arc<AppState>>,
    Path(store_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<ReadTuplesQuery>,
) -> Result<Json<Vec<ciam_authz::RelationTuple>>, (StatusCode, Json<ErrorResponse>)> {
    // Verify store exists
    let _ = state.fga_store_service.get_store(store_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("store_not_found", &e.to_string())),
        )
    })?;

    let subject_type = if let Some(st) = query.subject_type {
        Some(ciam_authz::SubjectType::try_from(st).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_subject_type", &e)),
            )
        })?)
    } else {
        None
    };

    let query_request = ciam_authz::QueryTuplesRequest {
        tenant_id: store_id,
        namespace: query.object_type,
        object_id: query.object_id,
        relation: query.relation,
        subject_type,
        subject_id: query.subject_id,
    };

    match state.tuple_service.query_tuples(query_request).await {
        Ok(tuples) => Ok(Json(tuples)),
        Err(e) => {
            tracing::error!("Failed to read tuples: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("read_tuples_failed", &e.to_string())),
            ))
        }
    }
}
