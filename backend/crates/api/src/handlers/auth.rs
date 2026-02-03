use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use ciam_auth::{AuthResponse, RefreshTokenRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
        }
    }
}

// API-level register request that accepts tenant slug
#[derive(Debug, serde::Deserialize)]
pub struct ApiRegisterRequest {
    pub tenant_id: String,  // Accept slug or UUID
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
}

/// Register a new user
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApiRegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Look up tenant by slug or UUID
    let tenant_id: uuid::Uuid = if let Ok(uuid) = uuid::Uuid::parse_str(&request.tenant_id) {
        // It's already a UUID
        uuid
    } else {
        // It's a slug, look it up
        let tenant_uuid: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM organizations WHERE slug = $1"
        )
        .bind(&request.tenant_id)
        .fetch_optional(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Database error looking up tenant: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", "Database error")),
            )
        })?;

        tenant_uuid
            .map(|(id,)| id)
            .ok_or_else(|| {
                tracing::warn!("Tenant not found: {}", request.tenant_id);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("invalid_tenant", "Tenant not found")),
                )
            })?
    };

    // Create service-level register request
    let register_request = ciam_auth::RegisterRequest {
        tenant_id,
        email: request.email.clone(),
        password: request.password,
        phone: request.phone,
        metadata: None,
    };

    match state.auth_service.register(register_request).await {
        Ok(response) => {
            // Send verification email after successful registration
            if let AuthResponse::Success { ref user, .. } = response {
                let user_id = user.id;
                let user_email = request.email.clone();

                // Extract name from metadata or use email
                let user_name = if let Some(ref first_name) = user.metadata.first_name {
                    if let Some(ref last_name) = user.metadata.last_name {
                        format!("{} {}", first_name, last_name)
                    } else {
                        first_name.clone()
                    }
                } else {
                    user_email.split('@').next().unwrap_or("User").to_string()
                };

                // Log successful registration
                let audit_service = Arc::clone(&state.audit_service);
                let email_for_audit = user_email.clone();
                tokio::spawn(async move {
                    if let Err(e) = audit_service
                        .log_registration(tenant_id, user_id, &email_for_audit, None)
                        .await
                    {
                        tracing::error!("Failed to log registration audit: {}", e);
                    }
                });

                // Send verification email in background (don't block registration)
                let verification_service = Arc::clone(&state.verification_service);
                tokio::spawn(async move {
                    if let Err(e) = verification_service
                        .send_verification_email(user_id, tenant_id, &user_email, &user_name)
                        .await
                    {
                        tracing::error!("Failed to send verification email: {}", e);
                    }
                });
            }

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Registration error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("registration_failed", &e.to_string())),
            ))
        }
    }
}

// API-level login request that accepts tenant slug
#[derive(Debug, serde::Deserialize)]
pub struct ApiLoginRequest {
    pub tenant_id: String,  // Accept slug or UUID
    pub email: String,
    pub password: String,
}

/// Login with email and password
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApiLoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Look up tenant by slug or UUID
    let tenant_id: uuid::Uuid = if let Ok(uuid) = uuid::Uuid::parse_str(&request.tenant_id) {
        // It's already a UUID
        uuid
    } else {
        // It's a slug, look it up
        let tenant_uuid: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM organizations WHERE slug = $1"
        )
        .bind(&request.tenant_id)
        .fetch_optional(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Database error looking up tenant: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", "Database error")),
            )
        })?;

        tenant_uuid
            .map(|(id,)| id)
            .ok_or_else(|| {
                tracing::warn!("Tenant not found: {}", request.tenant_id);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("invalid_credentials", "Invalid credentials")),
                )
            })?
    };

    // Clone email for potential audit logging in error case
    let email_for_audit = request.email.clone();

    // Create service-level login request
    let login_request = ciam_auth::LoginRequest {
        tenant_id,
        email: request.email,
        password: request.password,
        device_fingerprint: None,
        ip_address: None,
        user_agent: None,
    };

    match state.auth_service.login(login_request).await {
        Ok(response) => {
            // Log successful login
            if let AuthResponse::Success { ref user, .. } = response {
                let audit_service = Arc::clone(&state.audit_service);
                let user_id = user.id;
                let email = user.email.clone();
                tokio::spawn(async move {
                    if let Err(e) = audit_service
                        .log_login_success(tenant_id, user_id, &email, None)
                        .await
                    {
                        tracing::error!("Failed to log login audit: {}", e);
                    }
                });
            }
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Login error: {}", e);

            // Log failed login attempt
            let audit_service = Arc::clone(&state.audit_service);
            let email = email_for_audit;
            let error_msg = e.to_string();
            tokio::spawn(async move {
                if let Err(audit_err) = audit_service
                    .log_login_failure(tenant_id, &email, &error_msg, None)
                    .await
                {
                    tracing::error!("Failed to log failed login audit: {}", audit_err);
                }
            });

            let status_code = match e {
                ciam_auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                ciam_auth::AuthError::UserInactive => StatusCode::FORBIDDEN,
                _ => StatusCode::BAD_REQUEST,
            };
            Err((
                status_code,
                Json(ErrorResponse::new("login_failed", &e.to_string())),
            ))
        }
    }
}

/// API-level hierarchical login request
#[derive(Debug, serde::Deserialize)]
pub struct ApiHierarchicalLoginRequest {
    pub email: String,
    pub password: String,
    pub organization_slug: Option<String>,  // Optional organization context
}

/// Hierarchical login with optional organization context
/// Supports both platform admin login (no org) and org member login
pub async fn login_hierarchical(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ApiHierarchicalLoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Clone email for audit logging
    let email_for_audit = request.email.clone();
    let org_slug_for_audit = request.organization_slug.clone();

    // Create service-level login request
    let login_request = ciam_auth::HierarchicalLoginRequest {
        email: request.email,
        password: request.password,
        organization_slug: request.organization_slug,
        organization_id: None,  // Let the service resolve from slug
        device_fingerprint: None,
        ip_address: None,
        user_agent: None,
    };

    match state.auth_service.login_hierarchical(login_request).await {
        Ok(response) => {
            // Log successful login
            if let AuthResponse::Success { ref user, .. } = response {
                let audit_service = Arc::clone(&state.audit_service);
                let user_id = user.id;
                let user_email = user.email.clone();
                let org_slug = org_slug_for_audit.clone();

                tokio::spawn(async move {
                    let description = if let Some(org) = &org_slug {
                        format!("User logged in to organization '{}'", org)
                    } else {
                        "Platform admin logged in".to_string()
                    };

                    // For hierarchical login, we don't have a simple tenant_id
                    // Use a nil UUID if no organization context
                    let tenant_id = Uuid::nil();

                    let mut metadata = serde_json::Map::new();
                    if let Some(slug) = &org_slug {
                        metadata.insert("organization_slug".to_string(), serde_json::json!(slug));
                    }

                    if let Err(e) = audit_service
                        .log(ciam_models::AuditLogBuilder::new(
                            tenant_id,
                            ciam_models::audit::events::USER_LOGIN,
                            ciam_models::AuditEventCategory::Authentication,
                        )
                        .actor("user", user_id.to_string())
                        .actor_name(&user_email)
                        .description(&description)
                        .metadata(serde_json::Value::Object(metadata))
                        .build())
                        .await
                    {
                        tracing::error!("Failed to log successful login audit: {}", e);
                    }
                });
            }

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Hierarchical login error: {}", e);

            // Convert error to string before spawning
            let error_message = e.to_string();
            let error_message_for_spawn = error_message.clone();

            // Log failed login attempt
            let audit_service = Arc::clone(&state.audit_service);
            let email = email_for_audit;
            let org_slug = org_slug_for_audit;
            tokio::spawn(async move {
                let description = if let Some(org) = &org_slug {
                    format!("Failed login attempt for organization '{}'", org)
                } else {
                    "Failed platform admin login attempt".to_string()
                };

                let tenant_id = Uuid::nil();

                let mut metadata = serde_json::Map::new();
                if let Some(slug) = &org_slug {
                    metadata.insert("organization_slug".to_string(), serde_json::json!(slug));
                }
                metadata.insert("error".to_string(), serde_json::json!(error_message_for_spawn));

                if let Err(audit_err) = audit_service
                    .log(ciam_models::AuditLogBuilder::new(
                        tenant_id,
                        ciam_models::audit::events::USER_LOGIN_FAILED,
                        ciam_models::AuditEventCategory::Authentication,
                    )
                    .actor_name(&email)
                    .description(&description)
                    .metadata(serde_json::Value::Object(metadata))
                    .build())
                    .await
                {
                    tracing::error!("Failed to log failed login audit: {}", audit_err);
                }
            });

            let status_code = match e {
                ciam_auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                ciam_auth::AuthError::UserInactive => StatusCode::FORBIDDEN,
                ciam_auth::AuthError::Forbidden(_) => StatusCode::FORBIDDEN,
                ciam_auth::AuthError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            };
            Err((
                status_code,
                Json(ErrorResponse::new("login_failed", &error_message)),
            ))
        }
    }
}

/// Refresh access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.auth_service.refresh_token(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Token refresh error: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("refresh_failed", &e.to_string())),
            ))
        }
    }
}

/// Logout (invalidate session)
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Extract Bearer token from Authorization header
    let token = extract_bearer_token(&headers)?;

    // Validate token to get user info for audit logging
    let claims = state.auth_service.jwt.validate_access_token(&token)
        .map_err(|e| {
            tracing::error!("Token validation failed during logout: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_token", &e.to_string())),
            )
        })?;

    let user_id: Uuid = claims.sub.parse().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_token", "Invalid user ID in token")),
        )
    })?;

    // For hierarchical model, tenant_id is optional
    let tenant_id: Uuid = claims.tenant_id
        .and_then(|tid| tid.parse().ok())
        .unwrap_or_else(|| Uuid::nil());

    match state.auth_service.logout(token).await {
        Ok(_) => {
            // Log successful logout
            let audit_service = Arc::clone(&state.audit_service);
            let email = claims.email.clone();
            tokio::spawn(async move {
                if let Err(e) = audit_service
                    .log(ciam_models::AuditLogBuilder::new(
                        tenant_id,
                        ciam_models::audit::events::USER_LOGOUT,
                        ciam_models::AuditEventCategory::Authentication,
                    )
                    .actor("user", user_id.to_string())
                    .actor_name(&email)
                    .description("User logged out")
                    .build())
                    .await
                {
                    tracing::error!("Failed to log logout audit: {}", e);
                }
            });

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            tracing::error!("Logout error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("logout_failed", &e.to_string())),
            ))
        }
    }
}

/// Get current user profile
pub async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ciam_models::user::UserProfile>, (StatusCode, Json<ErrorResponse>)> {
    // Extract Bearer token from Authorization header
    let token = extract_bearer_token(&headers)?;

    // Validate token
    match state.auth_service.validate(token).await {
        Ok(claims) => {
            let user_id = claims.sub.parse().map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("invalid_token", "Invalid user ID in token")),
                )
            })?;

            // Get user
            match state.auth_service.get_user(user_id).await {
                Ok(user) => Ok(Json(user.into())),
                Err(e) => {
                    tracing::error!("Get user error: {}", e);
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse::new("user_not_found", &e.to_string())),
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Token validation error: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("unauthorized", &e.to_string())),
            ))
        }
    }
}

/// Helper function to extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new(
                    "missing_auth_header",
                    "Authorization header is required",
                )),
            )
        })?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new(
                    "invalid_auth_header",
                    "Authorization header is not valid UTF-8",
                )),
            )
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "invalid_auth_scheme",
                "Authorization header must use Bearer scheme",
            )),
        ));
    }

    Ok(&auth_header[7..]) // Strip "Bearer " prefix
}
