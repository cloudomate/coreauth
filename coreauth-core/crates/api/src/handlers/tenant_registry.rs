//! Tenant Registry Management Handlers
//!
//! API endpoints for managing tenant database isolation:
//! - Create/list/update tenants
//! - Configure dedicated databases
//! - View tenant status and routing info

use crate::handlers::auth::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use ciam_database::{IsolationMode, TenantRecord, TenantRouterStats};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub isolation_mode: String, // "shared" or "dedicated"
}

#[derive(Debug, Deserialize)]
pub struct ConfigureDedicatedDbRequest {
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub isolation_mode: String,
    pub status: String,
    pub database_host: Option<String>,
    pub database_name: Option<String>,
    pub created_at: String,
}

impl From<TenantRecord> for TenantResponse {
    fn from(t: TenantRecord) -> Self {
        Self {
            id: t.id,
            slug: t.slug,
            name: t.name,
            isolation_mode: t.isolation_mode,
            status: t.status,
            database_host: t.database_host,
            database_name: t.database_name,
            created_at: t.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    #[serde(default)]
    pub include_inactive: bool,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new tenant
/// POST /api/admin/tenants
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate slug format
    if !is_valid_slug(&request.slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_slug",
                "Slug must be lowercase alphanumeric with hyphens, 3-63 characters",
            )),
        ));
    }

    let isolation_mode = match request.isolation_mode.to_lowercase().as_str() {
        "dedicated" | "silo" => IsolationMode::Dedicated,
        _ => IsolationMode::Shared,
    };

    match state
        .tenant_router
        .create_tenant(&request.slug, &request.name, isolation_mode)
        .await
    {
        Ok(tenant) => {
            tracing::info!(
                "Created tenant: {} ({}) - {}",
                tenant.name,
                tenant.slug,
                tenant.isolation_mode
            );
            Ok(Json(tenant.into()))
        }
        Err(e) => {
            tracing::error!("Failed to create tenant: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("create_tenant_failed", &e.to_string())),
            ))
        }
    }
}

/// List all tenants
/// GET /api/admin/tenants
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<ListTenantsQuery>,
) -> Result<Json<Vec<TenantResponse>>, (StatusCode, Json<ErrorResponse>)> {
    match state.tenant_router.list_tenants(query.include_inactive).await {
        Ok(tenants) => {
            let responses: Vec<TenantResponse> = tenants.into_iter().map(Into::into).collect();
            Ok(Json(responses))
        }
        Err(e) => {
            tracing::error!("Failed to list tenants: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_tenants_failed", &e.to_string())),
            ))
        }
    }
}

/// Get a specific tenant
/// GET /api/admin/tenants/:tenant_id
pub async fn get_tenant(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.tenant_router.get_tenant_record(tenant_id).await {
        Ok(tenant) => Ok(Json(tenant.into())),
        Err(e) => {
            tracing::error!("Failed to get tenant: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("tenant_not_found", &e.to_string())),
            ))
        }
    }
}

/// Configure dedicated database for a tenant
/// POST /api/admin/tenants/:tenant_id/database
pub async fn configure_dedicated_database(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<ConfigureDedicatedDbRequest>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // First verify tenant exists and is in dedicated mode
    let tenant = state
        .tenant_router
        .get_tenant_record(tenant_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("tenant_not_found", &e.to_string())),
            )
        })?;

    if tenant.isolation_mode() != IsolationMode::Dedicated {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_isolation_mode",
                "Tenant must be in dedicated isolation mode to configure a dedicated database",
            )),
        ));
    }

    match state
        .tenant_router
        .configure_dedicated_database(
            tenant_id,
            &request.host,
            request.port,
            &request.database_name,
            &request.username,
            &request.password,
        )
        .await
    {
        Ok(tenant) => {
            tracing::info!(
                "Configured dedicated database for tenant {}: {}:{}/{}",
                tenant.slug,
                request.host,
                request.port,
                request.database_name
            );
            Ok(Json(tenant.into()))
        }
        Err(e) => {
            tracing::error!("Failed to configure dedicated database: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "configure_database_failed",
                    &e.to_string(),
                )),
            ))
        }
    }
}

/// Activate a shared tenant
/// POST /api/admin/tenants/:tenant_id/activate
pub async fn activate_tenant(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    let tenant = state
        .tenant_router
        .get_tenant_record(tenant_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("tenant_not_found", &e.to_string())),
            )
        })?;

    // For dedicated tenants, database must be configured first
    if tenant.isolation_mode() == IsolationMode::Dedicated && tenant.database_host.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "database_not_configured",
                "Dedicated tenant requires database configuration before activation",
            )),
        ));
    }

    match state.tenant_router.activate_shared_tenant(tenant_id).await {
        Ok(tenant) => {
            tracing::info!("Activated tenant: {}", tenant.slug);
            Ok(Json(tenant.into()))
        }
        Err(e) => {
            tracing::error!("Failed to activate tenant: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("activate_failed", &e.to_string())),
            ))
        }
    }
}

/// Suspend a tenant
/// POST /api/admin/tenants/:tenant_id/suspend
pub async fn suspend_tenant(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.tenant_router.suspend_tenant(tenant_id).await {
        Ok(()) => {
            tracing::info!("Suspended tenant: {}", tenant_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            tracing::error!("Failed to suspend tenant: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("suspend_failed", &e.to_string())),
            ))
        }
    }
}

/// Get tenant router statistics
/// GET /api/admin/tenants/stats
pub async fn get_router_stats(
    State(state): State<Arc<AppState>>,
) -> Json<TenantRouterStats> {
    Json(state.tenant_router.stats())
}

/// Test database connection for a tenant
/// POST /api/admin/tenants/:tenant_id/test-connection
pub async fn test_tenant_connection(
    State(state): State<Arc<AppState>>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<ConnectionTestResult>, (StatusCode, Json<ErrorResponse>)> {
    let start = std::time::Instant::now();

    match state.tenant_router.get_tenant_pool(tenant_id).await {
        Ok(pool) => {
            // Try to execute a simple query
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => {
                    let latency_ms = start.elapsed().as_millis() as u64;
                    Ok(Json(ConnectionTestResult {
                        success: true,
                        latency_ms,
                        error: None,
                    }))
                }
                Err(e) => Ok(Json(ConnectionTestResult {
                    success: false,
                    latency_ms: start.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                })),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("connection_failed", &e.to_string())),
        )),
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn is_valid_slug(slug: &str) -> bool {
    if slug.len() < 3 || slug.len() > 63 {
        return false;
    }

    // Must start with lowercase letter
    if !slug.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
        return false;
    }

    // Must end with lowercase letter or digit
    if !slug
        .chars()
        .last()
        .map(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .unwrap_or(false)
    {
        return false;
    }

    // Only lowercase letters, digits, and hyphens
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
