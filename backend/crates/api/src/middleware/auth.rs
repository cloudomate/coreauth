use crate::handlers::auth::ErrorResponse;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use ciam_auth::{AuthService, Claims};
use std::sync::Arc;
use uuid::Uuid;

/// Authenticated user context
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: Uuid::parse_str(&claims.sub).unwrap(),
            tenant_id: claims.tenant_id
                .and_then(|tid| Uuid::parse_str(&tid).ok())
                .unwrap_or_else(|| Uuid::nil()),
            email: claims.email,
        }
    }
}

/// Extract and validate JWT from Authorization header
pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
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
                    "Invalid Authorization header format",
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

    Ok(auth_header[7..].to_string())
}

/// Validate JWT and return claims
pub fn validate_token(
    auth_service: &AuthService,
    token: &str,
) -> Result<Claims, (StatusCode, Json<ErrorResponse>)> {
    auth_service
        .jwt
        .validate_access_token(token)
        .map_err(|e| {
            tracing::error!("Token validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_token", &e.to_string())),
            )
        })
}

/// Check if user has a specific role for their tenant
pub async fn has_role(
    auth_service: &AuthService,
    user_id: Uuid,
    tenant_id: Uuid,
    role_name: &str,
) -> Result<bool, (StatusCode, Json<ErrorResponse>)> {
    // Query user roles
    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.name
        FROM roles r
        INNER JOIN user_roles ur ON r.id = ur.role_id
        WHERE ur.user_id = $1 AND r.tenant_id = $2
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .fetch_all(auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    Ok(roles.iter().any(|r| r == role_name))
}

/// Middleware to require authentication
pub async fn require_auth(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers)?;
    let claims = validate_token(&state.auth_service, &token)?;

    // Add user context to request extensions
    request.extensions_mut().insert(AuthUser::from(claims));

    Ok(next.run(request).await)
}

/// Middleware to require tenant admin role
pub async fn require_tenant_admin(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers)?;
    let claims = validate_token(&state.auth_service, &token)?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("invalid_user_id", "Invalid user ID in token")),
        )
    })?;

    let tenant_id = claims.tenant_id.as_ref()
        .and_then(|tid| Uuid::parse_str(tid).ok())
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("invalid_tenant_id", "Invalid or missing tenant ID in token")),
            )
        })?;

    // Check if user has admin role for this tenant
    let is_admin = has_role(&state.auth_service, user_id, tenant_id, "admin").await?;

    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "insufficient_permissions",
                "This action requires tenant admin role",
            )),
        ));
    }

    // Add user context to request extensions
    request.extensions_mut().insert(AuthUser::from(claims));

    Ok(next.run(request).await)
}
