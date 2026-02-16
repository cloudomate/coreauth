use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ciam_models::{Connection, ConnectionScope, CreateConnection, UpdateConnection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::ErrorResponse;
use crate::AppState;

/// Helper to create error responses
fn err(status: StatusCode, code: &str, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: code.to_string(),
            message: msg.into(),
        }),
    )
}

/// Serializable connection response (redacts sensitive config fields)
#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub name: String,
    pub connection_type: String,
    pub scope: ConnectionScope,
    pub organization_id: Option<Uuid>,
    pub config: serde_json::Value,
    pub is_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Connection> for ConnectionResponse {
    fn from(c: Connection) -> Self {
        // Redact client_secret from config for read responses
        let mut config = c.config.clone();
        if let Some(obj) = config.as_object_mut() {
            if obj.contains_key("client_secret") {
                obj.insert("client_secret".to_string(), serde_json::json!("***"));
            }
        }
        Self {
            id: c.id,
            name: c.name,
            connection_type: c.connection_type,
            scope: c.scope,
            organization_id: c.organization_id,
            config,
            is_enabled: c.is_enabled,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// Request body for creating a connection (org-scoped)
#[derive(Debug, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub connection_type: String,
    pub config: serde_json::Value,
}

/// Request body for creating a platform-scoped connection (admin)
#[derive(Debug, Deserialize)]
pub struct CreatePlatformConnectionRequest {
    pub name: String,
    pub connection_type: String,
    pub config: serde_json::Value,
}

/// Request body for updating a connection
#[derive(Debug, Deserialize)]
pub struct UpdateConnectionRequest {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
}

/// Auth method response
#[derive(Debug, Serialize)]
pub struct AuthMethodResponse {
    pub connection_id: Uuid,
    pub name: String,
    pub method_type: String,
    pub scope: ConnectionScope,
}

// ============================================================================
// Organization-scoped endpoints
// ============================================================================

/// GET /api/organizations/:org_id/connections
pub async fn list_connections(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let connections = state
        .connection_service
        .get_organization_connections(org_id)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    // Also include platform connections so callers see the full picture
    let platform = state
        .connection_service
        .get_platform_connections()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    let mut all: Vec<ConnectionResponse> = platform.into_iter().map(Into::into).collect();
    all.extend(connections.into_iter().map(Into::into));

    Ok(Json(all))
}

/// POST /api/organizations/:org_id/connections
pub async fn create_connection(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateConnectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let create = CreateConnection {
        name: req.name,
        connection_type: req.connection_type,
        scope: ConnectionScope::Organization,
        organization_id: Some(org_id),
        config: req.config,
    };

    let connection = state
        .connection_service
        .create(create)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("InvalidInput") || msg.contains("Unsupported") {
                err(StatusCode::BAD_REQUEST, "invalid_input", msg)
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
        })?;

    Ok((StatusCode::CREATED, Json(ConnectionResponse::from(connection))))
}

/// GET /api/organizations/:org_id/connections/:conn_id
pub async fn get_connection(
    State(state): State<Arc<AppState>>,
    Path((org_id, conn_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let connection = state
        .connection_service
        .get_by_id(conn_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                err(StatusCode::NOT_FOUND, "not_found", msg)
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
        })?;

    // Verify the connection belongs to this org or is platform-scoped
    if connection.scope == ConnectionScope::Organization
        && connection.organization_id != Some(org_id)
    {
        return Err(err(StatusCode::NOT_FOUND, "not_found", "Connection not found"));
    }

    Ok(Json(ConnectionResponse::from(connection)))
}

/// PUT /api/organizations/:org_id/connections/:conn_id
pub async fn update_connection(
    State(state): State<Arc<AppState>>,
    Path((org_id, conn_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateConnectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Verify ownership
    let existing = state
        .connection_service
        .get_by_id(conn_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                err(StatusCode::NOT_FOUND, "not_found", msg)
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
        })?;

    if existing.organization_id != Some(org_id) {
        return Err(err(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Cannot update connections not owned by this organization",
        ));
    }

    let updates = UpdateConnection {
        name: req.name,
        config: req.config,
        is_enabled: req.is_enabled,
    };

    let connection = state
        .connection_service
        .update(conn_id, updates)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    Ok(Json(ConnectionResponse::from(connection)))
}

/// DELETE /api/organizations/:org_id/connections/:conn_id
pub async fn delete_connection(
    State(state): State<Arc<AppState>>,
    Path((org_id, conn_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Verify ownership
    let existing = state
        .connection_service
        .get_by_id(conn_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                err(StatusCode::NOT_FOUND, "not_found", msg)
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
        })?;

    if existing.organization_id != Some(org_id) {
        return Err(err(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Cannot delete connections not owned by this organization",
        ));
    }

    state
        .connection_service
        .delete(conn_id)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/organizations/:org_id/connections/auth-methods
pub async fn get_auth_methods(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let methods = state
        .connection_service
        .get_available_auth_methods(Some(org_id))
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    let responses: Vec<AuthMethodResponse> = methods
        .into_iter()
        .map(|m| AuthMethodResponse {
            connection_id: m.connection_id,
            name: m.name,
            method_type: m.method_type,
            scope: m.scope,
        })
        .collect();

    Ok(Json(responses))
}

// ============================================================================
// Admin endpoints
// ============================================================================

/// GET /api/admin/connections
pub async fn list_all_connections(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let connections = state
        .connection_service
        .list_all()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    let responses: Vec<ConnectionResponse> = connections.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

/// POST /api/admin/connections
pub async fn create_platform_connection(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePlatformConnectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let create = CreateConnection {
        name: req.name,
        connection_type: req.connection_type,
        scope: ConnectionScope::Platform,
        organization_id: None,
        config: req.config,
    };

    let connection = state
        .connection_service
        .create(create)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("InvalidInput") || msg.contains("Unsupported") {
                err(StatusCode::BAD_REQUEST, "invalid_input", msg)
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg)
            }
        })?;

    Ok((StatusCode::CREATED, Json(ConnectionResponse::from(connection))))
}
