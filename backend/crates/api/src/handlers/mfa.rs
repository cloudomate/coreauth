use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MfaEnrollResponse {
    pub method_id: Uuid,
    pub method_type: String,
    pub secret: String,
    pub qr_code_uri: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MfaMethod {
    pub id: Uuid,
    pub method_type: String,
    pub name: Option<String>,
    pub verified: bool,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMfaRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct NameMfaMethodRequest {
    pub name: String,
}

/// Enroll in TOTP MFA
/// POST /api/mfa/enroll/totp
pub async fn enroll_totp(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<MfaEnrollResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate TOTP secret
    let secret = ciam_auth::generate_secret();

    // Generate QR code URI
    let qr_uri = ciam_auth::generate_totp_uri(&secret, &auth_user.email, "CIAM");

    // Create MFA method in database (unverified)
    let method_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO mfa_methods (user_id, method_type, secret, verified)
        VALUES ($1, 'totp', $2, false)
        RETURNING id
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&secret)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Generate backup codes
    let backup_codes = ciam_auth::generate_backup_codes();

    // Hash and store backup codes
    for code in &backup_codes {
        let code_hash = ciam_auth::hash_backup_code(code).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("hash_error", &e.to_string())),
            )
        })?;

        sqlx::query(
            r#"
            INSERT INTO mfa_backup_codes (user_id, code_hash)
            VALUES ($1, $2)
            "#,
        )
        .bind(auth_user.user_id)
        .bind(&code_hash)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;
    }

    Ok(Json(MfaEnrollResponse {
        method_id,
        method_type: "totp".to_string(),
        secret: secret.clone(),
        qr_code_uri: qr_uri,
        backup_codes,
    }))
}

/// Verify and activate TOTP MFA method
/// POST /api/mfa/totp/:method_id/verify
pub async fn verify_totp(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(method_id): Path<Uuid>,
    Json(request): Json<VerifyMfaRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Get the MFA method
    let (secret, verified): (String, bool) = sqlx::query_as(
        r#"
        SELECT secret, verified
        FROM mfa_methods
        WHERE id = $1 AND user_id = $2 AND method_type = 'totp'
        "#,
    )
    .bind(method_id)
    .bind(auth_user.user_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("method_not_found", "MFA method not found")),
        )
    })?;

    if verified {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "already_verified",
                "This MFA method is already verified",
            )),
        ));
    }

    // Verify the TOTP code
    let is_valid = ciam_auth::verify_totp(&secret, &request.code).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("verification_error", &e.to_string())),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_code", "Invalid verification code")),
        ));
    }

    // Mark method as verified and enable MFA for user
    sqlx::query(
        r#"
        UPDATE mfa_methods
        SET verified = true, last_used_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(method_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Enable MFA for the user
    sqlx::query(
        r#"
        UPDATE users
        SET mfa_enabled = true
        WHERE id = $1
        "#,
    )
    .bind(auth_user.user_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// List user's MFA methods
/// GET /api/mfa/methods
pub async fn list_mfa_methods(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<MfaMethod>>, (StatusCode, Json<ErrorResponse>)> {
    let methods = sqlx::query_as::<_, (Uuid, String, Option<String>, bool, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT id, method_type, name, verified, last_used_at
        FROM mfa_methods
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    let mfa_methods = methods
        .into_iter()
        .map(|(id, method_type, name, verified, last_used_at)| MfaMethod {
            id,
            method_type,
            name,
            verified,
            last_used_at,
        })
        .collect();

    Ok(Json(mfa_methods))
}

/// Delete an MFA method
/// DELETE /api/mfa/methods/:method_id
pub async fn delete_mfa_method(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(method_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Delete the method
    let result = sqlx::query(
        r#"
        DELETE FROM mfa_methods
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(method_id)
    .bind(auth_user.user_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("method_not_found", "MFA method not found")),
        ));
    }

    // Check if user has any verified methods left
    let has_verified_methods: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM mfa_methods
            WHERE user_id = $1 AND verified = true
        )
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // If no verified methods left, disable MFA
    if !has_verified_methods {
        sqlx::query(
            r#"
            UPDATE users
            SET mfa_enabled = false
            WHERE id = $1
            "#,
        )
        .bind(auth_user.user_id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Regenerate backup codes
/// POST /api/mfa/backup-codes/regenerate
pub async fn regenerate_backup_codes(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Delete old backup codes
    sqlx::query(
        r#"
        DELETE FROM mfa_backup_codes
        WHERE user_id = $1
        "#,
    )
    .bind(auth_user.user_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Generate new backup codes
    let backup_codes = ciam_auth::generate_backup_codes();

    // Hash and store new backup codes
    for code in &backup_codes {
        let code_hash = ciam_auth::hash_backup_code(code).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("hash_error", &e.to_string())),
            )
        })?;

        sqlx::query(
            r#"
            INSERT INTO mfa_backup_codes (user_id, code_hash)
            VALUES ($1, $2)
            "#,
        )
        .bind(auth_user.user_id)
        .bind(&code_hash)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;
    }

    Ok(Json(backup_codes))
}

/// Enroll in TOTP MFA using enrollment token (unauthenticated)
/// POST /api/mfa/enroll-with-token/totp
#[derive(Debug, Deserialize)]
pub struct EnrollWithTokenRequest {
    pub enrollment_token: String,
}

pub async fn enroll_totp_with_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EnrollWithTokenRequest>,
) -> Result<Json<MfaEnrollResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate enrollment token
    let claims = state
        .auth_service
        .jwt
        .validate_enrollment_token(&request.enrollment_token)
        .map_err(|e| {
            tracing::error!("Invalid enrollment token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_token", "Invalid or expired enrollment token")),
            )
        })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("invalid_user_id", &e.to_string())),
        )
    })?;

    // Generate TOTP secret
    let secret = ciam_auth::generate_secret();

    // Generate QR code URI
    let qr_uri = ciam_auth::generate_totp_uri(&secret, &claims.email, "CoreAuth");

    // Create MFA method in database (unverified)
    let method_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO mfa_methods (user_id, method_type, secret, verified)
        VALUES ($1, 'totp', $2, false)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&secret)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Generate backup codes
    let backup_codes = ciam_auth::generate_backup_codes();

    // Hash and store backup codes
    for code in &backup_codes {
        let code_hash = ciam_auth::hash_backup_code(code).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("hash_error", &e.to_string())),
            )
        })?;

        sqlx::query(
            r#"
            INSERT INTO mfa_backup_codes (user_id, code_hash)
            VALUES ($1, $2)
            "#,
        )
        .bind(user_id)
        .bind(&code_hash)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;
    }

    Ok(Json(MfaEnrollResponse {
        method_id,
        method_type: "totp".to_string(),
        secret,
        qr_code_uri: qr_uri,
        backup_codes,
    }))
}

/// Verify TOTP and complete enrollment using enrollment token
/// POST /api/mfa/verify-with-token/totp/:method_id
#[derive(Debug, Deserialize)]
pub struct VerifyWithTokenRequest {
    pub enrollment_token: String,
    pub code: String,
}

pub async fn verify_totp_with_token(
    State(state): State<Arc<AppState>>,
    Path(method_id): Path<Uuid>,
    Json(request): Json<VerifyWithTokenRequest>,
) -> Result<Json<ciam_auth::AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate enrollment token
    let claims = state
        .auth_service
        .jwt
        .validate_enrollment_token(&request.enrollment_token)
        .map_err(|e| {
            tracing::error!("Invalid enrollment token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid_token", "Invalid or expired enrollment token")),
            )
        })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("invalid_user_id", &e.to_string())),
        )
    })?;

    // Get MFA method
    let method: (String,) = sqlx::query_as(
        r#"
        SELECT secret FROM mfa_methods
        WHERE id = $1 AND user_id = $2 AND verified = false
        "#,
    )
    .bind(method_id)
    .bind(user_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("method_not_found", "MFA method not found")),
        )
    })?;

    let secret = method.0;

    // Verify TOTP code
    let is_valid = ciam_auth::verify_totp(&secret, &request.code).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("verification_error", &e.to_string())),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_code", "Invalid verification code")),
        ));
    }

    // Mark method as verified
    sqlx::query(
        r#"
        UPDATE mfa_methods
        SET verified = true
        WHERE id = $1
        "#,
    )
    .bind(method_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Get user details
    let user = sqlx::query_as::<_, ciam_models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", &e.to_string())),
        )
    })?;

    // Parse organization ID from claims
    let org_id = claims.organization_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

    // Get organization details if applicable
    let (org_slug, role) = if let Some(org_id) = org_id {
        let membership = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT o.slug, om.role
            FROM organizations o
            JOIN organization_members om ON om.organization_id = o.id
            WHERE o.id = $1 AND om.user_id = $2
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;

        membership.map(|(slug, role)| (Some(slug), Some(role))).unwrap_or((None, None))
    } else {
        (None, None)
    };

    // Generate regular auth tokens now that MFA is set up
    let access_token = state.auth_service.jwt.generate_access_token(
        user.id,
        &user.email,
        org_id,
        org_slug.clone(),
        role.clone(),
        user.is_platform_admin,
    ).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("token_error", &e.to_string())),
        )
    })?;

    let refresh_token = state.auth_service.jwt.generate_refresh_token(
        user.id,
        &user.email,
        org_id,
        org_slug,
        role,
        user.is_platform_admin,
    ).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("token_error", &e.to_string())),
        )
    })?;

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", &e.to_string())),
            )
        })?;

    // Create user profile for response using From<User> implementation
    let user_profile = ciam_models::user::UserProfile::from(user.clone());

    Ok(Json(ciam_auth::AuthResponse::Success {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: user_profile,
    }))
}
