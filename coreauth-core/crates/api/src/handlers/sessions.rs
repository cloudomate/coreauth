use crate::handlers::auth::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// ACTIVE SESSIONS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub location: Option<String>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub user_id: Option<Uuid>,
}

/// List active sessions for a user
/// GET /api/sessions?user_id=xxx
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListSessionsQuery>,
) -> Result<Json<Vec<SessionInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = params.user_id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("missing_user_id", "user_id is required")),
        )
    })?;

    let sessions = sqlx::query(
        r#"
        SELECT id, device_fingerprint, ip_address, user_agent, created_at, expires_at
        FROM sessions
        WHERE user_id = $1 AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch sessions: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    let session_list: Vec<SessionInfo> = sessions
        .into_iter()
        .map(|row| {
            let user_agent: Option<String> = row.get("user_agent");
            let (device_type, browser, os) = parse_user_agent(user_agent.as_deref());

            SessionInfo {
                id: row.get("id"),
                device_fingerprint: row.get("device_fingerprint"),
                ip_address: row.get("ip_address"),
                user_agent,
                device_type,
                browser,
                os,
                location: None, // Would need geo-IP lookup
                is_current: false, // Would need to compare with current session
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            }
        })
        .collect();

    Ok(Json(session_list))
}

/// Revoke a specific session
/// DELETE /api/sessions/:session_id
pub async fn revoke_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("session_not_found", "Session not found")),
        ))
    }
}

/// Revoke all sessions except current
/// POST /api/sessions/revoke-all?user_id=xxx&except_session_id=xxx
#[derive(Debug, Deserialize)]
pub struct RevokeAllQuery {
    pub user_id: Uuid,
    pub except_session_id: Option<Uuid>,
}

pub async fn revoke_all_sessions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RevokeAllQuery>,
) -> Result<Json<RevokeAllResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result = if let Some(except_id) = params.except_session_id {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id != $2")
            .bind(params.user_id)
            .bind(except_id)
            .execute(state.auth_service.db.pool())
            .await
    } else {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(params.user_id)
            .execute(state.auth_service.db.pool())
            .await
    };

    match result {
        Ok(r) => Ok(Json(RevokeAllResponse {
            revoked_count: r.rows_affected() as i32,
        })),
        Err(e) => {
            tracing::error!("Failed to revoke sessions: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            ))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RevokeAllResponse {
    pub revoked_count: i32,
}

// ============================================================================
// LOGIN HISTORY
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LoginAttemptInfo {
    pub id: Uuid,
    pub email: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub successful: bool,
    pub failure_reason: Option<String>,
    pub location: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginHistoryQuery {
    pub user_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub email: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get login history
/// GET /api/login-history?user_id=xxx&limit=50&offset=0
pub async fn get_login_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LoginHistoryQuery>,
) -> Result<Json<LoginHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Build query based on filters
    let (query, bind_user_id, bind_tenant_id, bind_email) = if let Some(user_id) = params.user_id {
        (
            r#"
            SELECT id, email, ip_address, user_agent, successful, failure_reason, attempted_at
            FROM login_attempts
            WHERE user_id = $1
            ORDER BY attempted_at DESC
            LIMIT $2 OFFSET $3
            "#,
            Some(user_id),
            None,
            None,
        )
    } else if let Some(tenant_id) = params.tenant_id {
        (
            r#"
            SELECT id, email, ip_address, user_agent, successful, failure_reason, attempted_at
            FROM login_attempts
            WHERE tenant_id = $1
            ORDER BY attempted_at DESC
            LIMIT $2 OFFSET $3
            "#,
            None,
            Some(tenant_id),
            None,
        )
    } else if let Some(email) = params.email.clone() {
        (
            r#"
            SELECT id, email, ip_address, user_agent, successful, failure_reason, attempted_at
            FROM login_attempts
            WHERE email = $1
            ORDER BY attempted_at DESC
            LIMIT $2 OFFSET $3
            "#,
            None,
            None,
            Some(email),
        )
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "missing_filter",
                "Provide user_id, tenant_id, or email",
            )),
        ));
    };

    let rows = if let Some(user_id) = bind_user_id {
        sqlx::query(query)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(state.auth_service.db.pool())
            .await
    } else if let Some(tenant_id) = bind_tenant_id {
        sqlx::query(query)
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(state.auth_service.db.pool())
            .await
    } else if let Some(email) = bind_email {
        sqlx::query(query)
            .bind(email)
            .bind(limit)
            .bind(offset)
            .fetch_all(state.auth_service.db.pool())
            .await
    } else {
        unreachable!()
    };

    let rows = rows.map_err(|e| {
        tracing::error!("Failed to fetch login history: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    let attempts: Vec<LoginAttemptInfo> = rows
        .into_iter()
        .map(|row| {
            let user_agent: Option<String> = row.get("user_agent");
            let (device_type, browser, os) = parse_user_agent(user_agent.as_deref());

            LoginAttemptInfo {
                id: row.get("id"),
                email: row.get("email"),
                ip_address: row.get("ip_address"),
                user_agent,
                device_type,
                browser,
                os,
                successful: row.get("successful"),
                failure_reason: row.get("failure_reason"),
                location: None, // Would need geo-IP lookup
                attempted_at: row.get("attempted_at"),
            }
        })
        .collect();

    Ok(Json(LoginHistoryResponse {
        attempts,
        limit,
        offset,
    }))
}

#[derive(Debug, Serialize)]
pub struct LoginHistoryResponse {
    pub attempts: Vec<LoginAttemptInfo>,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
// AUDIT LOGS (for Security tab)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AuditLogInfo {
    pub id: Uuid,
    pub event_type: String,
    pub event_category: String,
    pub event_action: String,
    pub actor_name: Option<String>,
    pub actor_ip_address: Option<String>,
    pub target_name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub tenant_id: Uuid,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get audit logs for a tenant
/// GET /api/audit-logs?tenant_id=xxx
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<AuditLogResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let query = if let Some(event_type) = &params.event_type {
        sqlx::query(
            r#"
            SELECT id, event_type, event_category::text, event_action, actor_name,
                   actor_ip_address::text, target_name, description, status, created_at
            FROM audit_logs
            WHERE tenant_id = $1 AND event_type = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(params.tenant_id)
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.auth_service.db.pool())
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, event_type, event_category::text, event_action, actor_name,
                   actor_ip_address::text, target_name, description, status, created_at
            FROM audit_logs
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(params.tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.auth_service.db.pool())
        .await
    };

    let rows = query.map_err(|e| {
        tracing::error!("Failed to fetch audit logs: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    let logs: Vec<AuditLogInfo> = rows
        .into_iter()
        .map(|row| AuditLogInfo {
            id: row.get("id"),
            event_type: row.get("event_type"),
            event_category: row.get("event_category"),
            event_action: row.get("event_action"),
            actor_name: row.get("actor_name"),
            actor_ip_address: row.get("actor_ip_address"),
            target_name: row.get("target_name"),
            description: row.get("description"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(AuditLogResponse { logs, limit, offset }))
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub logs: Vec<AuditLogInfo>,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Parse user agent string to extract device type, browser, and OS
fn parse_user_agent(ua: Option<&str>) -> (String, String, String) {
    let ua = match ua {
        Some(s) => s,
        None => return ("Unknown".to_string(), "Unknown".to_string(), "Unknown".to_string()),
    };

    let ua_lower = ua.to_lowercase();

    // Device type detection
    let device_type = if ua_lower.contains("mobile") || ua_lower.contains("android") && !ua_lower.contains("tablet") {
        "Mobile"
    } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
        "Tablet"
    } else {
        "Desktop"
    };

    // Browser detection
    let browser = if ua_lower.contains("edg/") || ua_lower.contains("edge") {
        "Microsoft Edge"
    } else if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
        "Chrome"
    } else if ua_lower.contains("firefox") {
        "Firefox"
    } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
        "Safari"
    } else if ua_lower.contains("opera") || ua_lower.contains("opr/") {
        "Opera"
    } else if ua_lower.contains("msie") || ua_lower.contains("trident") {
        "Internet Explorer"
    } else {
        "Unknown Browser"
    };

    // OS detection
    let os = if ua_lower.contains("windows nt 10") {
        "Windows 10/11"
    } else if ua_lower.contains("windows") {
        "Windows"
    } else if ua_lower.contains("mac os x") || ua_lower.contains("macos") {
        "macOS"
    } else if ua_lower.contains("iphone") {
        "iOS (iPhone)"
    } else if ua_lower.contains("ipad") {
        "iOS (iPad)"
    } else if ua_lower.contains("android") {
        "Android"
    } else if ua_lower.contains("linux") {
        "Linux"
    } else if ua_lower.contains("cros") {
        "Chrome OS"
    } else {
        "Unknown OS"
    };

    (device_type.to_string(), browser.to_string(), os.to_string())
}
