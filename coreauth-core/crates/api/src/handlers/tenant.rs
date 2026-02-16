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
    #[serde(default = "default_account_type")]
    pub account_type: String, // "personal" or "business"
    #[serde(default = "default_isolation_mode")]
    pub isolation_mode: String, // "shared" or "dedicated"
}

fn default_account_type() -> String {
    "business".to_string()
}

fn default_isolation_mode() -> String {
    "shared".to_string()
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub admin_user_id: Uuid,
    pub message: String,
    pub email_verification_required: bool,
    pub isolation_mode: String,
    pub database_setup_required: bool, // True for dedicated mode
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
    // Check if email verification is required (configurable via environment)
    let require_email_verification = std::env::var("REQUIRE_EMAIL_VERIFICATION")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false); // Default: email verification disabled

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

    // Validate account_type
    if request.account_type != "personal" && request.account_type != "business" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_account_type",
                "Account type must be 'personal' or 'business'",
            )),
        ));
    }

    // Validate isolation_mode
    let isolation_mode = request.isolation_mode.to_lowercase();
    if isolation_mode != "shared" && isolation_mode != "dedicated" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_isolation_mode",
                "Isolation mode must be 'shared' or 'dedicated'",
            )),
        ));
    }

    // Personal accounts always use shared mode
    let effective_isolation_mode = if request.account_type == "personal" {
        "shared".to_string()
    } else {
        isolation_mode.clone()
    };

    // Check if slug already exists
    let existing_tenant: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM tenants WHERE slug = $1
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

    // Create tenant
    let tenant_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO tenants (name, slug, account_type, isolation_mode)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&request.name)
    .bind(&request.slug)
    .bind(&request.account_type)
    .bind(&effective_isolation_mode)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error creating tenant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to create tenant")),
        )
    })?;

    // Register tenant in tenant_registry for database routing
    // Shared tenants are auto-activated, dedicated tenants need database configuration
    let registry_status = if effective_isolation_mode == "shared" {
        "active"
    } else {
        "provisioning"
    };

    sqlx::query(
        r#"
        INSERT INTO tenant_registry (id, slug, name, isolation_mode, status)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(tenant_id)
    .bind(&request.slug)
    .bind(&request.name)
    .bind(&effective_isolation_mode)
    .bind(registry_status)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error registering tenant in registry: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to register tenant in database router")),
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
        INSERT INTO users (default_tenant_id, email, password_hash, email_verified, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(tenant_id)
    .bind(&request.admin_email)
    .bind(&password_hash)
    .bind(!require_email_verification) // If verification required, set to false; otherwise true
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

    // Add user to tenant_members table
    sqlx::query(
        r#"
        INSERT INTO tenant_members (user_id, tenant_id, role)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(admin_user_id)
    .bind(tenant_id)
    .bind("admin")
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Error adding user to tenant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("internal_error", "Failed to add user to tenant")),
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
        "Tenant created: id={}, name={}, admin_user_id={}, isolation_mode={}",
        tenant_id,
        request.name,
        admin_user_id,
        effective_isolation_mode
    );

    // Send verification email if required (configurable)
    if require_email_verification {
        let verification_service = Arc::clone(&state.verification_service);
        let admin_email = request.admin_email.clone();
        let admin_name = request.admin_full_name.clone();
        tokio::spawn(async move {
            if let Err(e) = verification_service
                .send_verification_email(admin_user_id, tenant_id, &admin_email, &admin_name, None)
                .await
            {
                tracing::error!("Failed to send verification email for tenant admin: {}", e);
            } else {
                tracing::info!("Verification email sent to tenant admin: {}", admin_email);
            }
        });
    }

    let message = if require_email_verification {
        "Account created successfully. Please check your email to verify your account.".to_string()
    } else {
        "Account created successfully.".to_string()
    };

    let database_setup_required = effective_isolation_mode == "dedicated";

    Ok(Json(CreateTenantResponse {
        tenant_id,
        tenant_name: request.name,
        admin_user_id,
        message,
        email_verification_required: require_email_verification,
        isolation_mode: effective_isolation_mode,
        database_setup_required,
    }))
}

/// Public response for tenant lookup
#[derive(Debug, Serialize)]
pub struct PublicTenantResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

// Keep old name for backwards compatibility
pub type PublicOrganizationResponse = PublicTenantResponse;

/// Get tenant by slug (Public - for login page)
/// GET /api/tenants/by-slug/:slug
pub async fn get_tenant_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<PublicTenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result: Option<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT id, name, slug FROM tenants
        WHERE slug = $1 AND parent_tenant_id IS NULL
        "#,
    )
    .bind(&slug)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching tenant by slug: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", "Failed to fetch tenant")),
        )
    })?;

    match result {
        Some((id, name, slug)) => Ok(Json(PublicTenantResponse { id, name, slug })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Tenant not found")),
        )),
    }
}

// Keep old function name for backwards compatibility
pub async fn get_organization_by_slug(
    state: State<Arc<AppState>>,
    path: Path<String>,
) -> Result<Json<PublicTenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_tenant_by_slug(state, path).await
}

#[derive(Debug, Serialize)]
pub struct TenantUserResponse {
    pub id: Uuid,
    pub email: String,
    pub metadata: serde_json::Value,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
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

    // Query all users in the tenant via tenant_members table
    let users: Vec<TenantUserResponse> = sqlx::query_as::<_, (Uuid, String, serde_json::Value, String, bool, bool, bool, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT
            u.id,
            u.email,
            u.metadata,
            tm.role,
            u.is_active,
            u.email_verified,
            u.mfa_enabled,
            u.created_at,
            u.last_login_at
        FROM users u
        INNER JOIN tenant_members tm ON u.id = tm.user_id
        WHERE tm.tenant_id = $1
        ORDER BY u.created_at DESC
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
    .map(|(id, email, metadata, role, is_active, email_verified, mfa_enabled, created_at, last_login_at)| {
        TenantUserResponse {
            id,
            email,
            metadata,
            role,
            is_active,
            email_verified,
            mfa_enabled,
            created_at,
            last_login_at,
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

    // Update the user's role in the tenant_members table
    let result = sqlx::query(
        r#"
        UPDATE tenant_members
        SET role = $1
        WHERE tenant_id = $2 AND user_id = $3
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
    pub require_email_verification: Option<bool>,
    pub enforce_sso: Option<bool>,
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
        r#"SELECT settings FROM tenants WHERE id = $1"#
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
        r#"SELECT settings FROM tenants WHERE id = $1"#
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
    if let Some(require_verification) = request.require_email_verification {
        settings.security.require_email_verification = require_verification;
    }
    if let Some(enforce_sso) = request.enforce_sso {
        settings.security.enforce_sso = enforce_sso;
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
        r#"UPDATE tenants SET settings = $1 WHERE id = $2"#
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

// ============================================================================
// BRANDING SETTINGS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateBrandingRequest {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub favicon_url: Option<String>,
    pub app_name: Option<String>,
    pub background_color: Option<String>,
    pub background_image_url: Option<String>,
    pub button_text_color: Option<String>,
    pub terms_url: Option<String>,
    pub privacy_url: Option<String>,
    pub support_url: Option<String>,
    pub custom_css: Option<String>,
}

/// Get organization branding settings
/// GET /api/organizations/:org_id/branding
pub async fn get_branding(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ciam_models::BrandingSettings>, (StatusCode, Json<ErrorResponse>)> {
    if auth_user.tenant_id != org_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("forbidden", "You don't have access to this organization")),
        ));
    }

    let org: (serde_json::Value,) = sqlx::query_as(
        r#"SELECT settings FROM tenants WHERE id = $1"#
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching organization: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("internal_error", "Database error")))
    })?;

    let settings: OrganizationSettings = serde_json::from_value(org.0).unwrap_or_default();
    Ok(Json(settings.branding))
}

/// Update organization branding settings
/// PUT /api/organizations/:org_id/branding
pub async fn update_branding(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<UpdateBrandingRequest>,
) -> Result<Json<ciam_models::BrandingSettings>, (StatusCode, Json<ErrorResponse>)> {
    if auth_user.tenant_id != org_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("forbidden", "You don't have access to this organization")),
        ));
    }

    // Fetch current settings
    let org: (serde_json::Value,) = sqlx::query_as(
        r#"SELECT settings FROM tenants WHERE id = $1"#
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching organization: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("internal_error", "Database error")))
    })?;

    let mut settings: OrganizationSettings = serde_json::from_value(org.0).unwrap_or_default();

    // Merge branding updates
    if let Some(logo_url) = request.logo_url {
        settings.branding.logo_url = Some(logo_url);
    }
    if let Some(primary_color) = request.primary_color {
        settings.branding.primary_color = Some(primary_color);
    }
    if let Some(favicon_url) = request.favicon_url {
        settings.branding.favicon_url = Some(favicon_url);
    }
    if let Some(app_name) = request.app_name {
        settings.branding.app_name = Some(app_name);
    }
    if let Some(background_color) = request.background_color {
        settings.branding.background_color = Some(background_color);
    }
    if let Some(background_image_url) = request.background_image_url {
        settings.branding.background_image_url = Some(background_image_url);
    }
    if let Some(button_text_color) = request.button_text_color {
        settings.branding.button_text_color = Some(button_text_color);
    }
    if let Some(terms_url) = request.terms_url {
        settings.branding.terms_url = Some(terms_url);
    }
    if let Some(privacy_url) = request.privacy_url {
        settings.branding.privacy_url = Some(privacy_url);
    }
    if let Some(support_url) = request.support_url {
        settings.branding.support_url = Some(support_url);
    }
    if let Some(custom_css) = request.custom_css {
        settings.branding.custom_css = Some(custom_css);
    }

    // Save back
    let settings_json = serde_json::to_value(&settings).map_err(|e| {
        tracing::error!("Failed to serialize settings: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("internal_error", "Failed to serialize settings")))
    })?;

    sqlx::query(r#"UPDATE tenants SET settings = $1 WHERE id = $2"#)
        .bind(settings_json)
        .bind(org_id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Database error updating branding: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("internal_error", "Failed to update branding")))
        })?;

    tracing::info!("Branding updated for org={}", org_id);
    Ok(Json(settings.branding))
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
        SELECT COUNT(*) FROM users WHERE default_tenant_id = $1 AND is_active = true
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
        WHERE user_id IN (SELECT id FROM users WHERE default_tenant_id = $1 AND is_active = true)
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
