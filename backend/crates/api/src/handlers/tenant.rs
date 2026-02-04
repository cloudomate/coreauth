use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use ciam_models::{OrganizationSettings, SecuritySettings as ModelSecuritySettings};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub admin_email: String,
    pub admin_password: String,
    pub admin_full_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub admin_user_id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecurityPolicyRequest {
    pub mfa_required: Option<bool>,
    pub password_min_length: Option<i32>,
    pub max_login_attempts: Option<i32>,
    pub lockout_duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SecurityPolicyResponse {
    pub tenant_id: Uuid,
    pub security: SecuritySettings,
}

#[derive(Debug, Serialize)]
pub struct SecuritySettings {
    pub mfa_required: bool,
    pub password_min_length: i32,
    pub max_login_attempts: i32,
    pub lockout_duration_minutes: i32,
}

#[derive(Debug, Serialize)]
pub struct MfaStatusResponse {
    pub total_users: i64,
    pub users_with_mfa: i64,
    pub mfa_coverage_percent: f64,
}

/// Create a new tenant with admin user (Public - Tenant Onboarding)
/// POST /api/tenants
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<Json<CreateTenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate slug format (alphanumeric and hyphens only)
    if !request.slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_slug",
                "Tenant slug can only contain letters, numbers, and hyphens",
            )),
        ));
    }

    // Check if slug already exists
    let existing_tenant: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM organizations WHERE slug = $1
        "#,
    )
    .bind(&request.slug)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error checking tenant slug: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    if existing_tenant.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse::new(
                "tenant_exists",
                "A tenant with this slug already exists",
            )),
        ));
    }

    // Check if admin email already exists
    let existing_user: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM users WHERE email = $1
        "#,
    )
    .bind(&request.admin_email)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error checking user email: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse::new(
                "email_exists",
                "A user with this email already exists",
            )),
        ));
    }

    // Hash password
    let password_hash = ciam_auth::PasswordHasher::hash(&request.admin_password).map_err(|e| {
        tracing::error!("Password hashing error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Password hashing failed")),
        )
    })?;

    // Start transaction
    let mut tx = state
        .auth_service
        .db
        .pool()
        .begin()
        .await
        .map_err(|e| {
            tracing::error!("Transaction error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("internal_error", "Database error")),
            )
        })?;

    // Create organization (hierarchical model)
    let tenant_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organizations (name, slug)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&request.name)
    .bind(&request.slug)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating organization: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to create organization")),
        )
    })?;

    // Create admin role for this tenant
    let admin_role_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO roles (tenant_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(tenant_id)
    .bind("admin")
    .bind("Tenant administrator with full access to all features")
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating admin role: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to create admin role")),
        )
    })?;

    // Create admin user with tenant_admin role in metadata
    let admin_metadata = serde_json::json!({
        "roles": ["tenant_admin"],
        "first_name": request.admin_full_name.split_whitespace().next().unwrap_or(""),
        "last_name": request.admin_full_name.split_whitespace().nth(1).unwrap_or(""),
        "full_name": request.admin_full_name,
        "tenant_id": tenant_id.to_string()
    });

    let admin_user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (default_organization_id, email, password_hash, email_verified, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(tenant_id)
    .bind(&request.admin_email)
    .bind(&password_hash)
    .bind(true) // Admin email is pre-verified during onboarding
    .bind(true)
    .bind(&admin_metadata)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating admin user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to create admin user")),
        )
    })?;

    // Add user to organization_members table (hierarchical model)
    sqlx::query(
        r#"
        INSERT INTO organization_members (user_id, organization_id, role)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(admin_user_id)
    .bind(tenant_id)
    .bind("admin")
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error adding user to organization: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to add user to organization")),
        )
    })?;

    // Link admin user to admin role
    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id, granted_by)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(admin_user_id)
    .bind(admin_role_id)
    .bind(admin_user_id) // Self-granted during tenant creation
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error assigning admin role: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to assign admin role")),
        )
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Transaction commit error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    tracing::info!(
        "Tenant created: id={}, name={}, admin_user_id={}",
        tenant_id,
        request.name,
        admin_user_id
    );

    Ok(Json(CreateTenantResponse {
        tenant_id,
        tenant_name: request.name,
        admin_user_id,
        message: "Tenant created successfully. You can now log in as the tenant administrator.".to_string(),
    }))
}

/// Public response for organization lookup
#[derive(Debug, Serialize)]
pub struct PublicOrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// Get organization by slug (Public - for login page)
/// GET /api/organizations/by-slug/:slug
pub async fn get_organization_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<PublicOrganizationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result: Option<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT id, name, slug FROM organizations
        WHERE slug = $1 AND parent_organization_id IS NULL
        "#,
    )
    .bind(&slug)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching organization by slug: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", "Failed to fetch organization")),
        )
    })?;

    match result {
        Some((id, name, slug)) => Ok(Json(PublicOrganizationResponse { id, name, slug })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Organization not found")),
        )),
    }
}

#[derive(Debug, Serialize)]
pub struct TenantUserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub role: String,
    pub status: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

/// List all users in a tenant (Tenant Admin only)
/// GET /api/tenants/:tenant_id/users
pub async fn list_tenant_users(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<TenantUserResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user has access to this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    // Query all users in the tenant via organization_members table
    let users: Vec<TenantUserResponse> = sqlx::query_as::<_, (Uuid, String, serde_json::Value, String, bool, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT
            u.id,
            u.email,
            u.metadata,
            om.role,
            u.is_active,
            om.joined_at,
            u.last_login_at,
            u.created_at
        FROM users u
        INNER JOIN organization_members om ON u.id = om.user_id
        WHERE om.organization_id = $1
        ORDER BY om.joined_at DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching tenant users: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?
    .into_iter()
    .map(|(id, email, metadata, role, is_active, joined_at, last_login, _created_at)| {
        let full_name = metadata
            .get("full_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let status = if is_active {
            "active".to_string()
        } else {
            "inactive".to_string()
        };

        TenantUserResponse {
            id,
            email,
            full_name,
            role,
            status,
            joined_at,
            last_login,
        }
    })
    .collect();

    Ok(Json(users))
}

#[derive(Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

/// Update a user's role in a tenant (Tenant Admin only)
/// PUT /api/tenants/:tenant_id/users/:user_id/role
pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Verify requester has access to this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    // Validate role
    if request.role != "admin" && request.role != "member" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_role",
                "Role must be 'admin' or 'member'",
            )),
        ));
    }

    // Prevent users from changing their own role
    if auth_user.user_id == user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "cannot_modify_self",
                "You cannot change your own role",
            )),
        ));
    }

    // Update the user's role in the organization_members table
    let result = sqlx::query(
        r#"
        UPDATE organization_members
        SET role = $1
        WHERE organization_id = $2 AND user_id = $3
        "#,
    )
    .bind(&request.role)
    .bind(tenant_id)
    .bind(user_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error updating user role: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to update user role")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "user_not_found",
                "User not found in this tenant",
            )),
        ));
    }

    tracing::info!(
        "User role updated: tenant={}, user={}, new_role={}",
        tenant_id,
        user_id,
        request.role
    );

    Ok(Json(serde_json::json!({
        "message": "User role updated successfully",
        "user_id": user_id,
        "new_role": request.role,
    })))
}

#[derive(Deserialize)]
pub struct UpdateSecuritySettingsRequest {
    pub mfa_required: Option<bool>,
    pub mfa_grace_period_days: Option<i32>,
    pub allowed_mfa_methods: Option<Vec<String>>,
    pub password_min_length: Option<usize>,
    pub session_timeout_hours: Option<i64>,
    pub password_require_uppercase: Option<bool>,
    pub password_require_lowercase: Option<bool>,
    pub password_require_number: Option<bool>,
    pub password_require_special: Option<bool>,
    pub max_login_attempts: Option<i32>,
    pub lockout_duration_minutes: Option<i32>,
}

/// Get organization security settings (Tenant/Org Admin)
/// GET /api/organizations/:org_id/security
pub async fn get_security_settings(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ModelSecuritySettings>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user has access to this organization
    if auth_user.tenant_id != org_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this organization",
            )),
        ));
    }

    // Fetch organization settings
    let org: (serde_json::Value,) = sqlx::query_as(
        r#"SELECT settings FROM organizations WHERE id = $1"#
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching organization: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    let settings: OrganizationSettings = serde_json::from_value(org.0)
        .unwrap_or_default();

    Ok(Json(settings.security))
}

/// Update organization security settings (Tenant/Org Admin)
/// PUT /api/organizations/:org_id/security
pub async fn update_security_settings(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<UpdateSecuritySettingsRequest>,
) -> Result<Json<ModelSecuritySettings>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user has access to this organization
    if auth_user.tenant_id != org_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this organization",
            )),
        ));
    }

    // Fetch current settings
    let org: (serde_json::Value,) = sqlx::query_as(
        r#"SELECT settings FROM organizations WHERE id = $1"#
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching organization: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    let mut settings: OrganizationSettings = serde_json::from_value(org.0)
        .unwrap_or_default();

    // Update security settings
    if let Some(mfa_required) = request.mfa_required {
        settings.security.mfa_required = mfa_required;
        if mfa_required && settings.security.mfa_enforcement_date.is_none() {
            settings.security.mfa_enforcement_date = Some(chrono::Utc::now());
        }
    }
    if let Some(grace_period) = request.mfa_grace_period_days {
        settings.security.mfa_grace_period_days = grace_period;
    }
    if let Some(methods) = request.allowed_mfa_methods {
        settings.security.allowed_mfa_methods = methods;
    }
    if let Some(min_length) = request.password_min_length {
        settings.security.password_min_length = min_length;
    }
    if let Some(timeout) = request.session_timeout_hours {
        settings.security.session_timeout_hours = timeout;
    }
    if let Some(require_upper) = request.password_require_uppercase {
        settings.security.password_require_uppercase = require_upper;
    }
    if let Some(require_lower) = request.password_require_lowercase {
        settings.security.password_require_lowercase = require_lower;
    }
    if let Some(require_number) = request.password_require_number {
        settings.security.password_require_number = require_number;
    }
    if let Some(require_special) = request.password_require_special {
        settings.security.password_require_special = require_special;
    }
    if let Some(max_attempts) = request.max_login_attempts {
        settings.security.max_login_attempts = max_attempts;
    }
    if let Some(lockout) = request.lockout_duration_minutes {
        settings.security.lockout_duration_minutes = lockout;
    }

    // Update in database
    let settings_json = serde_json::to_value(&settings).map_err(|e| {
        tracing::error!("Failed to serialize settings: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to serialize settings")),
        )
    })?;

    sqlx::query(
        r#"UPDATE organizations SET settings = $1 WHERE id = $2"#
    )
    .bind(settings_json)
    .bind(org_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error updating organization settings: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to update settings")),
        )
    })?;

    tracing::info!(
        "Security settings updated: org={}, mfa_required={:?}",
        org_id,
        request.mfa_required
    );

    Ok(Json(settings.security))
}

/*
 * SECURITY POLICY FUNCTIONS DISABLED
 *
 * The following functions are temporarily disabled because the organizations table
 * doesn't have security policy columns (mfa_required, password_min_length, etc.).
 * These need to be added to the schema before these endpoints can be enabled.
 */

/*
/// Get tenant security policy (Tenant Admin only)
/// GET /api/tenants/:tenant_id/security
pub async fn get_security_policy(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<SecurityPolicyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let tenant: Option<(bool, i32, i32, i32)> = sqlx::query_as(
        r#"
        SELECT mfa_required, password_min_length, max_login_attempts, lockout_duration_minutes
        FROM tenants
        WHERE id = $1
        "#,
    )
    .bind(tenant_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    let (mfa_required, password_min_length, max_login_attempts, lockout_duration_minutes) =
        tenant.ok_or((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Tenant not found")),
        ))?;

    Ok(Json(SecurityPolicyResponse {
        tenant_id,
        security: SecuritySettings {
            mfa_required,
            password_min_length,
            max_login_attempts,
            lockout_duration_minutes,
        },
    }))
}

/// Update tenant security policy (Tenant Admin only)
/// POST /api/tenants/:tenant_id/security
pub async fn update_security_policy(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<UpdateSecurityPolicyRequest>,
) -> Result<Json<SecurityPolicyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    // Validate inputs
    if let Some(password_min_length) = request.password_min_length {
        if password_min_length < 8 || password_min_length > 32 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_value",
                    "Password minimum length must be between 8 and 32",
                )),
            ));
        }
    }

    if let Some(max_login_attempts) = request.max_login_attempts {
        if max_login_attempts < 3 || max_login_attempts > 10 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_value",
                    "Max login attempts must be between 3 and 10",
                )),
            ));
        }
    }

    if let Some(lockout_duration) = request.lockout_duration_minutes {
        if lockout_duration < 5 || lockout_duration > 60 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_value",
                    "Lockout duration must be between 5 and 60 minutes",
                )),
            ));
        }
    }

    // Check if at least one field is being updated
    if request.mfa_required.is_none()
        && request.password_min_length.is_none()
        && request.max_login_attempts.is_none()
        && request.lockout_duration_minutes.is_none()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_request", "No updates provided")),
        ));
    }

    // Execute individual updates for each field
    if let Some(mfa_required) = request.mfa_required {
        sqlx::query("UPDATE tenants SET mfa_required = $1 WHERE id = $2")
            .bind(mfa_required)
            .bind(tenant_id)
            .execute(state.auth_service.db.pool())
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("internal_error", "Database error")),
                )
            })?;
    }

    if let Some(password_min_length) = request.password_min_length {
        sqlx::query("UPDATE tenants SET password_min_length = $1 WHERE id = $2")
            .bind(password_min_length)
            .bind(tenant_id)
            .execute(state.auth_service.db.pool())
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("internal_error", "Database error")),
                )
            })?;
    }

    if let Some(max_login_attempts) = request.max_login_attempts {
        sqlx::query("UPDATE tenants SET max_login_attempts = $1 WHERE id = $2")
            .bind(max_login_attempts)
            .bind(tenant_id)
            .execute(state.auth_service.db.pool())
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("internal_error", "Database error")),
                )
            })?;
    }

    if let Some(lockout_duration) = request.lockout_duration_minutes {
        sqlx::query("UPDATE tenants SET lockout_duration_minutes = $1 WHERE id = $2")
            .bind(lockout_duration)
            .bind(tenant_id)
            .execute(state.auth_service.db.pool())
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("internal_error", "Database error")),
                )
            })?;
    }

    // Fetch updated settings
    get_security_policy(State(state), Extension(auth_user), Path(tenant_id)).await
}

/// Get MFA status for tenant (Tenant Admin only)
/// GET /api/tenants/:tenant_id/mfa-status
pub async fn get_mfa_status(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<MfaStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    let total_users: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND is_active = true
        "#,
    )
    .bind(tenant_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    let users_with_mfa: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT user_id)
        FROM mfa_methods
        WHERE user_id IN (SELECT id FROM users WHERE tenant_id = $1 AND is_active = true)
          AND verified = true
        "#,
    )
    .bind(tenant_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    let mfa_coverage_percent = if total_users > 0 {
        (users_with_mfa as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(MfaStatusResponse {
        total_users,
        users_with_mfa,
        mfa_coverage_percent,
    }))
}

/// Enforce MFA for all users in tenant (Tenant Admin only)
/// POST /api/tenants/:tenant_id/enforce-mfa
pub async fn enforce_mfa(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<SecurityPolicyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify user is admin of this tenant
    if auth_user.tenant_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "forbidden",
                "You don't have access to this tenant",
            )),
        ));
    }

    sqlx::query(
        r#"
        UPDATE tenants SET mfa_required = true WHERE id = $1
        "#,
    )
    .bind(tenant_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Database error")),
        )
    })?;

    tracing::info!("MFA enforced for tenant: {}", tenant_id);

    get_security_policy(State(state), Extension(auth_user), Path(tenant_id)).await
}
*/
