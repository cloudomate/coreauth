use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use ciam_models::{AuditLog, AuditLogQuery, AuditEventCategory, AuditStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct QueryAuditLogsRequest {
    pub event_types: Option<Vec<String>>,
    pub event_categories: Option<Vec<AuditEventCategory>>,
    pub actor_id: Option<String>,
    pub target_id: Option<String>,
    pub status: Option<AuditStatus>,
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query audit logs
/// GET /api/audit/logs
pub async fn query_audit_logs(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<QueryAuditLogsRequest>,
) -> Result<Json<AuditLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let query = AuditLogQuery {
        tenant_id: auth_user.tenant_id,
        event_types: params.event_types,
        event_categories: params.event_categories,
        actor_id: params.actor_id,
        target_id: params.target_id,
        status: params.status,
        from_date: params.from_date,
        to_date: params.to_date,
        limit: params.limit.or(Some(100)),
        offset: params.offset.or(Some(0)),
    };

    let logs = state.audit_service
        .query(query.clone())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    let total = state.audit_service
        .count(auth_user.tenant_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    Ok(Json(AuditLogsResponse {
        logs,
        total,
        limit: query.limit.unwrap_or(100),
        offset: query.offset.unwrap_or(0),
    }))
}

/// Get a specific audit log by ID
/// GET /api/audit/logs/:id
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditLog>, (StatusCode, Json<ErrorResponse>)> {
    let log = state.audit_service
        .get_by_id(auth_user.tenant_id, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", "Audit log not found")),
            )
        })?;

    Ok(Json(log))
}

/// Get security events
/// GET /api/audit/security-events
pub async fn get_security_events(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<LimitQuery>,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(500);

    let logs = state.audit_service
        .get_security_events(auth_user.tenant_id, limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    Ok(Json(logs))
}

/// Get failed login attempts for a user
/// GET /api/audit/failed-logins/:user_id
pub async fn get_failed_logins(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Query(params): Query<SinceQuery>,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, Json<ErrorResponse>)> {
    let since = params
        .since
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::hours(24));

    let logs = state.audit_service
        .get_failed_logins(auth_user.tenant_id, &user_id, since)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    Ok(Json(logs))
}

/// Export audit logs for a date range
/// GET /api/audit/export
pub async fn export_audit_logs(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, Json<ErrorResponse>)> {
    let from_date = params
        .from_date
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("bad_request", "from_date is required")),
            )
        })?;

    let to_date = params
        .to_date
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("bad_request", "to_date is required")),
            )
        })?;

    let logs = state.audit_service
        .export(auth_user.tenant_id, from_date, to_date)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    Ok(Json(logs))
}

/// Get audit log statistics
/// GET /api/audit/stats
pub async fn get_audit_stats(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<AuditStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let total = state.audit_service
        .count(auth_user.tenant_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    // Get recent security events count
    let security_events = state.audit_service
        .get_security_events(auth_user.tenant_id, 1000)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", &e.to_string())),
            )
        })?;

    let security_count = security_events.len() as i64;

    Ok(Json(AuditStatsResponse {
        total_events: total,
        security_events: security_count,
    }))
}

// Helper query structures
#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SinceQuery {
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AuditStatsResponse {
    pub total_events: i64,
    pub security_events: i64,
}
