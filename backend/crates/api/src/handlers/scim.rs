//! SCIM 2.0 API Handlers
//!
//! Implements SCIM 2.0 protocol for enterprise user provisioning.
//! Allows identity providers like Okta, Azure AD, and OneLogin to
//! automatically sync users and groups.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use ciam_models::{
    CreateScimGroup, CreateScimToken, CreateScimUser, ResourceType, ScimConfiguration,
    ScimEmail, ScimError, ScimFilter, ScimFilterOp, ScimGroup, ScimGroupRecord, ScimGroupRef,
    ScimListQuery, ScimListResponse, ScimMember, ScimMeta, ScimName, ScimPatchOp,
    ScimPatchRequest, ScimPhoneNumber, ScimToken, ScimTokenResponse, ScimUser, ServiceProviderConfig,
};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::AppState;

// ============================================================================
// SCIM Service Provider Configuration
// ============================================================================

/// GET /scim/v2/ServiceProviderConfig
/// Returns SCIM capabilities of this service
pub async fn get_service_provider_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let base_url = get_base_url(&state);
    let mut config = ServiceProviderConfig::default();
    config.meta.location = Some(format!("{}/scim/v2/ServiceProviderConfig", base_url));

    Json(config)
}

/// GET /scim/v2/ResourceTypes
/// Returns supported resource types (User, Group)
pub async fn get_resource_types(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let base_url = get_base_url(&state);
    let now = Utc::now();

    let user_type = ResourceType {
        schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:ResourceType".to_string()],
        id: "User".to_string(),
        name: "User".to_string(),
        endpoint: "/Users".to_string(),
        description: "User Account".to_string(),
        schema: "urn:ietf:params:scim:schemas:core:2.0:User".to_string(),
        schema_extensions: vec![],
        meta: ScimMeta::new("ResourceType", now, now, &base_url, "User"),
    };

    let group_type = ResourceType {
        schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:ResourceType".to_string()],
        id: "Group".to_string(),
        name: "Group".to_string(),
        endpoint: "/Groups".to_string(),
        description: "Group".to_string(),
        schema: "urn:ietf:params:scim:schemas:core:2.0:Group".to_string(),
        schema_extensions: vec![],
        meta: ScimMeta::new("ResourceType", now, now, &base_url, "Group"),
    };

    Json(vec![user_type, group_type])
}

/// GET /scim/v2/Schemas
/// Returns SCIM schemas (simplified)
pub async fn get_schemas() -> impl IntoResponse {
    // Return basic schema info - full schema is complex
    Json(serde_json::json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
        "totalResults": 2,
        "Resources": [
            {
                "id": "urn:ietf:params:scim:schemas:core:2.0:User",
                "name": "User",
                "description": "User Account"
            },
            {
                "id": "urn:ietf:params:scim:schemas:core:2.0:Group",
                "name": "Group",
                "description": "Group"
            }
        ]
    }))
}

// ============================================================================
// SCIM Users
// ============================================================================

/// GET /scim/v2/Users
/// List users with optional filtering
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScimListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Parse filter if provided
    let filter = query.filter.as_ref().and_then(|f| ScimFilter::parse(f));

    // Build query based on filter
    let (users, total): (Vec<UserRow>, i64) = if let Some(ref f) = filter {
        match (f.attribute.as_str(), &f.operator) {
            ("userName", ScimFilterOp::Eq) | ("emails.value", ScimFilterOp::Eq) => {
                let users: Vec<UserRow> = sqlx::query_as(
                    r#"
                    SELECT u.id, u.email, u.first_name, u.last_name, u.phone,
                           u.is_active, u.scim_external_id, u.created_at, u.updated_at
                    FROM users u
                    JOIN organization_members om ON om.user_id = u.id
                    WHERE om.organization_id = $1 AND u.email = $2
                    "#,
                )
                .bind(org_id)
                .bind(&f.value)
                .fetch_all(state.auth_service.db.pool())
                .await
                .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

                let total = users.len() as i64;
                (users, total)
            }
            ("externalId", ScimFilterOp::Eq) => {
                let users: Vec<UserRow> = sqlx::query_as(
                    r#"
                    SELECT u.id, u.email, u.first_name, u.last_name, u.phone,
                           u.is_active, u.scim_external_id, u.created_at, u.updated_at
                    FROM users u
                    JOIN organization_members om ON om.user_id = u.id
                    WHERE om.organization_id = $1 AND u.scim_external_id = $2
                    "#,
                )
                .bind(org_id)
                .bind(&f.value)
                .fetch_all(state.auth_service.db.pool())
                .await
                .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

                let total = users.len() as i64;
                (users, total)
            }
            _ => {
                // Unsupported filter, return all
                fetch_all_users(&state, org_id, &query).await?
            }
        }
    } else {
        fetch_all_users(&state, org_id, &query).await?
    };

    // Convert to SCIM format
    let scim_users: Vec<ScimUser> = users
        .into_iter()
        .map(|u| user_to_scim(u, &base_url))
        .collect();

    let response = ScimListResponse::new(
        scim_users,
        total,
        query.start_index,
        query.count,
    );

    Ok(Json(response))
}

/// GET /scim/v2/Users/:id
/// Get a single user
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    let user: UserRow = sqlx::query_as(
        r#"
        SELECT u.id, u.email, u.first_name, u.last_name, u.phone,
               u.is_active, u.scim_external_id, u.created_at, u.updated_at
        FROM users u
        JOIN organization_members om ON om.user_id = u.id
        WHERE u.id = $1 AND om.organization_id = $2
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?
    .ok_or_else(|| scim_error(404, "User not found"))?;

    let scim_user = user_to_scim(user, &base_url);
    Ok(Json(scim_user))
}

/// POST /scim/v2/Users
/// Create a new user
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateScimUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Get email from emails array or userName
    let email = req.emails.iter()
        .find(|e| e.primary)
        .or(req.emails.first())
        .map(|e| e.value.clone())
        .unwrap_or_else(|| req.user_name.clone());

    // Check if user already exists
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if let Some(existing_id) = existing {
        // Check if already in this org
        let in_org: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM organization_members WHERE user_id = $1 AND organization_id = $2)"
        )
        .bind(existing_id)
        .bind(org_id)
        .fetch_one(state.auth_service.db.pool())
        .await
        .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

        if in_org {
            return Err(scim_error(409, "User already exists in this organization"));
        }
    }

    // Generate password hash if password provided, otherwise random
    let password_hash = if let Some(ref password) = req.password {
        hash_password(password)
    } else {
        // Generate random password for SCIM-provisioned users
        let random_password: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        hash_password(&random_password)
    };

    // Create user
    let first_name = req.name.as_ref().and_then(|n| n.given_name.clone());
    let last_name = req.name.as_ref().and_then(|n| n.family_name.clone());
    let phone = req.phone_numbers.first().map(|p| p.value.clone());

    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, phone, is_active,
                          scim_external_id, scim_provisioned, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, true)
        RETURNING id
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&phone)
    .bind(req.active)
    .bind(&req.external_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Failed to create user: {}", e)))?;

    // Add to organization
    sqlx::query(
        "INSERT INTO organization_members (user_id, organization_id, role) VALUES ($1, $2, 'member')"
    )
    .bind(user_id)
    .bind(org_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Failed to add user to organization: {}", e)))?;

    // Fetch created user
    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, email, first_name, last_name, phone, is_active,
               scim_external_id, created_at, updated_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let scim_user = user_to_scim(user, &base_url);
    Ok((StatusCode::CREATED, Json(scim_user)))
}

/// PUT /scim/v2/Users/:id
/// Replace a user
pub async fn replace_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(req): Json<CreateScimUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Verify user exists and belongs to org
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM users u
            JOIN organization_members om ON om.user_id = u.id
            WHERE u.id = $1 AND om.organization_id = $2
        )
        "#
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if !exists {
        return Err(scim_error(404, "User not found"));
    }

    // Get email from emails array or userName
    let email = req.emails.iter()
        .find(|e| e.primary)
        .or(req.emails.first())
        .map(|e| e.value.clone())
        .unwrap_or_else(|| req.user_name.clone());

    let first_name = req.name.as_ref().and_then(|n| n.given_name.clone());
    let last_name = req.name.as_ref().and_then(|n| n.family_name.clone());
    let phone = req.phone_numbers.first().map(|p| p.value.clone());

    // Update user
    sqlx::query(
        r#"
        UPDATE users SET
            email = $2,
            first_name = $3,
            last_name = $4,
            phone = $5,
            is_active = $6,
            scim_external_id = $7,
            scim_last_synced_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(&email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&phone)
    .bind(req.active)
    .bind(&req.external_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Failed to update user: {}", e)))?;

    // Fetch updated user
    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, email, first_name, last_name, phone, is_active,
               scim_external_id, created_at, updated_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let scim_user = user_to_scim(user, &base_url);
    Ok(Json(scim_user))
}

/// PATCH /scim/v2/Users/:id
/// Partially update a user
pub async fn patch_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ScimPatchRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Verify user exists and belongs to org
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM users u
            JOIN organization_members om ON om.user_id = u.id
            WHERE u.id = $1 AND om.organization_id = $2
        )
        "#
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if !exists {
        return Err(scim_error(404, "User not found"));
    }

    // Process patch operations
    for op in req.operations {
        match op.op.to_lowercase().as_str() {
            "replace" | "add" => {
                if let Some(path) = &op.path {
                    match path.as_str() {
                        "active" => {
                            if let Some(value) = &op.value {
                                let active = value.as_bool().unwrap_or(true);
                                sqlx::query("UPDATE users SET is_active = $2, updated_at = NOW() WHERE id = $1")
                                    .bind(user_id)
                                    .bind(active)
                                    .execute(state.auth_service.db.pool())
                                    .await
                                    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;
                            }
                        }
                        "name.givenName" => {
                            if let Some(value) = &op.value {
                                let name = value.as_str().unwrap_or("");
                                sqlx::query("UPDATE users SET first_name = $2, updated_at = NOW() WHERE id = $1")
                                    .bind(user_id)
                                    .bind(name)
                                    .execute(state.auth_service.db.pool())
                                    .await
                                    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;
                            }
                        }
                        "name.familyName" => {
                            if let Some(value) = &op.value {
                                let name = value.as_str().unwrap_or("");
                                sqlx::query("UPDATE users SET last_name = $2, updated_at = NOW() WHERE id = $1")
                                    .bind(user_id)
                                    .bind(name)
                                    .execute(state.auth_service.db.pool())
                                    .await
                                    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;
                            }
                        }
                        "externalId" => {
                            if let Some(value) = &op.value {
                                let external_id = value.as_str().unwrap_or("");
                                sqlx::query("UPDATE users SET scim_external_id = $2, updated_at = NOW() WHERE id = $1")
                                    .bind(user_id)
                                    .bind(external_id)
                                    .execute(state.auth_service.db.pool())
                                    .await
                                    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;
                            }
                        }
                        _ => {
                            // Unsupported path, ignore
                        }
                    }
                } else if let Some(value) = &op.value {
                    // No path specified, value should contain multiple attributes
                    if let Some(active) = value.get("active").and_then(|v| v.as_bool()) {
                        sqlx::query("UPDATE users SET is_active = $2, updated_at = NOW() WHERE id = $1")
                            .bind(user_id)
                            .bind(active)
                            .execute(state.auth_service.db.pool())
                            .await
                            .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;
                    }
                }
            }
            "remove" => {
                // Handle remove operations if needed
            }
            _ => {
                return Err(scim_error(400, format!("Unsupported operation: {}", op.op)));
            }
        }
    }

    // Update sync timestamp
    sqlx::query("UPDATE users SET scim_last_synced_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(state.auth_service.db.pool())
        .await
        .ok();

    // Fetch updated user
    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, email, first_name, last_name, phone, is_active,
               scim_external_id, created_at, updated_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let scim_user = user_to_scim(user, &base_url);
    Ok(Json(scim_user))
}

/// DELETE /scim/v2/Users/:id
/// Delete (deactivate) a user
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;

    // Verify user exists and belongs to org
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM users u
            JOIN organization_members om ON om.user_id = u.id
            WHERE u.id = $1 AND om.organization_id = $2
        )
        "#
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if !exists {
        return Err(scim_error(404, "User not found"));
    }

    // Deactivate user instead of deleting
    sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| scim_error(500, format!("Failed to deactivate user: {}", e)))?;

    // Remove from organization
    sqlx::query("DELETE FROM organization_members WHERE user_id = $1 AND organization_id = $2")
        .bind(user_id)
        .bind(org_id)
        .execute(state.auth_service.db.pool())
        .await
        .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// SCIM Groups
// ============================================================================

/// GET /scim/v2/Groups
/// List groups
pub async fn list_groups(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScimListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    let offset = (query.start_index - 1).max(0);

    let groups: Vec<ScimGroupRecord> = sqlx::query_as(
        r#"
        SELECT id, organization_id, display_name, external_id, role_id, description, created_at, updated_at
        FROM scim_groups
        WHERE organization_id = $1
        ORDER BY display_name
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(org_id)
    .bind(query.count)
    .bind(offset)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let total: i64 = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COUNT(*) FROM scim_groups WHERE organization_id = $1"
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?
    .unwrap_or(0);

    // Convert to SCIM format with members
    let mut scim_groups = Vec::new();
    for group in groups {
        let members = get_group_members(&state, group.id).await?;
        scim_groups.push(group_to_scim(group, members, &base_url));
    }

    let response = ScimListResponse::new(scim_groups, total, query.start_index, query.count);
    Ok(Json(response))
}

/// GET /scim/v2/Groups/:id
/// Get a single group
pub async fn get_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    let group: ScimGroupRecord = sqlx::query_as(
        r#"
        SELECT id, organization_id, display_name, external_id, role_id, description, created_at, updated_at
        FROM scim_groups
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(group_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?
    .ok_or_else(|| scim_error(404, "Group not found"))?;

    let members = get_group_members(&state, group_id).await?;
    let scim_group = group_to_scim(group, members, &base_url);

    Ok(Json(scim_group))
}

/// POST /scim/v2/Groups
/// Create a new group
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateScimGroup>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Check if group already exists
    let existing: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM scim_groups WHERE organization_id = $1 AND display_name = $2)"
    )
    .bind(org_id)
    .bind(&req.display_name)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if existing {
        return Err(scim_error(409, "Group with this name already exists"));
    }

    // Create group
    let group_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO scim_groups (organization_id, display_name, external_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(org_id)
    .bind(&req.display_name)
    .bind(&req.external_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Failed to create group: {}", e)))?;

    // Add members
    for member in &req.members {
        if let Ok(user_id) = Uuid::parse_str(&member.value) {
            let _ = sqlx::query(
                "INSERT INTO scim_group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(group_id)
            .bind(user_id)
            .execute(state.auth_service.db.pool())
            .await;
        }
    }

    // Fetch created group
    let group: ScimGroupRecord = sqlx::query_as(
        r#"
        SELECT id, organization_id, display_name, external_id, role_id, description, created_at, updated_at
        FROM scim_groups WHERE id = $1
        "#,
    )
    .bind(group_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let members = get_group_members(&state, group_id).await?;
    let scim_group = group_to_scim(group, members, &base_url);

    Ok((StatusCode::CREATED, Json(scim_group)))
}

/// PATCH /scim/v2/Groups/:id
/// Update group membership
pub async fn patch_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    Json(req): Json<ScimPatchRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;
    let base_url = get_base_url(&state);

    // Verify group exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM scim_groups WHERE id = $1 AND organization_id = $2)"
    )
    .bind(group_id)
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if !exists {
        return Err(scim_error(404, "Group not found"));
    }

    // Process patch operations
    for op in req.operations {
        match op.op.to_lowercase().as_str() {
            "add" => {
                if op.path.as_deref() == Some("members") {
                    if let Some(value) = &op.value {
                        if let Some(members) = value.as_array() {
                            for member in members {
                                if let Some(user_id_str) = member.get("value").and_then(|v| v.as_str()) {
                                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                                        let _ = sqlx::query(
                                            "INSERT INTO scim_group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
                                        )
                                        .bind(group_id)
                                        .bind(user_id)
                                        .execute(state.auth_service.db.pool())
                                        .await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "remove" => {
                if let Some(path) = &op.path {
                    // Parse path like "members[value eq \"user-id\"]"
                    if path.starts_with("members[") {
                        if let Some(user_id_str) = extract_member_id_from_path(path) {
                            if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                                let _ = sqlx::query(
                                    "DELETE FROM scim_group_members WHERE group_id = $1 AND user_id = $2"
                                )
                                .bind(group_id)
                                .bind(user_id)
                                .execute(state.auth_service.db.pool())
                                .await;
                            }
                        }
                    }
                }
            }
            "replace" => {
                if op.path.as_deref() == Some("displayName") {
                    if let Some(value) = &op.value {
                        if let Some(name) = value.as_str() {
                            let _ = sqlx::query(
                                "UPDATE scim_groups SET display_name = $2, updated_at = NOW() WHERE id = $1"
                            )
                            .bind(group_id)
                            .bind(name)
                            .execute(state.auth_service.db.pool())
                            .await;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Fetch updated group
    let group: ScimGroupRecord = sqlx::query_as(
        r#"
        SELECT id, organization_id, display_name, external_id, role_id, description, created_at, updated_at
        FROM scim_groups WHERE id = $1
        "#,
    )
    .bind(group_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let members = get_group_members(&state, group_id).await?;
    let scim_group = group_to_scim(group, members, &base_url);

    Ok(Json(scim_group))
}

/// DELETE /scim/v2/Groups/:id
/// Delete a group
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let (org_id, _token_id) = authenticate_scim(&state, &headers).await?;

    let result = sqlx::query(
        "DELETE FROM scim_groups WHERE id = $1 AND organization_id = $2"
    )
    .bind(group_id)
    .bind(org_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(scim_error(404, "Group not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// SCIM Token Management (Admin API)
// ============================================================================

/// POST /api/organizations/:org_id/scim/tokens
/// Create a new SCIM token
pub async fn create_scim_token(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateScimToken>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    // Generate token
    let token = generate_scim_token();
    let token_hash = hash_token(&token);
    let token_prefix = &token[..12.min(token.len())];

    let token_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO scim_tokens (organization_id, name, token_hash, token_prefix, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&token_hash)
    .bind(token_prefix)
    .bind(&req.expires_at)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Failed to create token: {}", e)))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": token_id,
        "name": req.name,
        "token": token,
        "token_prefix": token_prefix,
        "expires_at": req.expires_at,
        "message": "Save this token now - it won't be shown again"
    }))))
}

/// GET /api/organizations/:org_id/scim/tokens
/// List SCIM tokens
pub async fn list_scim_tokens(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let tokens: Vec<ScimTokenRow> = sqlx::query_as(
        r#"
        SELECT id, name, token_prefix, expires_at, created_at
        FROM scim_tokens
        WHERE organization_id = $1 AND is_active = true
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let responses: Vec<ScimTokenResponse> = tokens.into_iter().map(|t| ScimTokenResponse {
        id: t.id,
        name: t.name,
        token_prefix: t.token_prefix,
        expires_at: t.expires_at,
        created_at: t.created_at,
    }).collect();

    Ok(Json(responses))
}

/// DELETE /api/organizations/:org_id/scim/tokens/:token_id
/// Revoke a SCIM token
pub async fn revoke_scim_token(
    State(state): State<Arc<AppState>>,
    Path((org_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ScimError>)> {
    let result = sqlx::query(
        "UPDATE scim_tokens SET is_active = false, revoked_at = NOW() WHERE id = $1 AND organization_id = $2"
    )
    .bind(token_id)
    .bind(org_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(scim_error(404, "Token not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Authenticate SCIM request using bearer token
async fn authenticate_scim(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<(Uuid, Uuid), (StatusCode, Json<ScimError>)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| scim_error(401, "Missing Authorization header"))?;

    if !auth_header.to_lowercase().starts_with("bearer ") {
        return Err(scim_error(401, "Invalid Authorization header"));
    }

    let token = &auth_header[7..];
    let token_hash = hash_token(token);

    let result: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT id, organization_id FROM scim_tokens
        WHERE token_hash = $1 AND is_active = true
        AND (expires_at IS NULL OR expires_at > NOW())
        "#
    )
    .bind(&token_hash)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|_| scim_error(500, "Database error"))?;

    let (token_id, org_id) = result.ok_or_else(|| scim_error(401, "Invalid or expired token"))?;

    // Update last used
    let _ = sqlx::query(
        "UPDATE scim_tokens SET last_used_at = NOW(), request_count = request_count + 1 WHERE id = $1"
    )
    .bind(token_id)
    .execute(state.auth_service.db.pool())
    .await;

    Ok((org_id, token_id))
}

/// Create SCIM error response
fn scim_error(status: u16, detail: impl Into<String>) -> (StatusCode, Json<ScimError>) {
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(ScimError::new(status, detail)),
    )
}

/// Get base URL for SCIM endpoints
fn get_base_url(state: &Arc<AppState>) -> String {
    std::env::var("ISSUER_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
}

/// Generate SCIM bearer token
fn generate_scim_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    format!("scim_{}", hex::encode(bytes))
}

/// Hash a token using SHA-256
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Hash password (simplified - in production use proper password hashing)
fn hash_password(password: &str) -> String {
    // This should use Argon2 in production
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// SCIM Token row from database
#[derive(Debug, Clone, sqlx::FromRow)]
struct ScimTokenRow {
    id: Uuid,
    name: String,
    token_prefix: String,
    expires_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

/// User row from database
#[derive(Debug, Clone, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
    is_active: bool,
    scim_external_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

/// Convert user row to SCIM user
fn user_to_scim(user: UserRow, base_url: &str) -> ScimUser {
    let name = if user.first_name.is_some() || user.last_name.is_some() {
        Some(ScimName {
            given_name: user.first_name.clone(),
            family_name: user.last_name.clone(),
            formatted: match (&user.first_name, &user.last_name) {
                (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                (Some(f), None) => Some(f.clone()),
                (None, Some(l)) => Some(l.clone()),
                _ => None,
            },
            ..Default::default()
        })
    } else {
        None
    };

    let display_name = name.as_ref()
        .and_then(|n| n.formatted.clone())
        .or_else(|| Some(user.email.clone()));

    ScimUser {
        schemas: ScimUser::schemas(),
        id: user.id.to_string(),
        external_id: user.scim_external_id,
        user_name: user.email.clone(),
        name,
        display_name,
        emails: vec![ScimEmail {
            value: user.email,
            email_type: Some("work".to_string()),
            primary: true,
        }],
        phone_numbers: user.phone.map(|p| vec![ScimPhoneNumber {
            value: p,
            phone_type: Some("work".to_string()),
            primary: true,
        }]).unwrap_or_default(),
        active: user.is_active,
        groups: vec![],
        meta: ScimMeta::new("User", user.created_at, user.updated_at, base_url, &user.id.to_string()),
    }
}

/// Convert group record to SCIM group
fn group_to_scim(group: ScimGroupRecord, members: Vec<ScimMember>, base_url: &str) -> ScimGroup {
    ScimGroup {
        schemas: ScimGroup::schemas(),
        id: group.id.to_string(),
        external_id: group.external_id,
        display_name: group.display_name,
        members,
        meta: ScimMeta::new("Group", group.created_at, group.updated_at, base_url, &group.id.to_string()),
    }
}

/// Get group members
async fn get_group_members(
    state: &Arc<AppState>,
    group_id: Uuid,
) -> Result<Vec<ScimMember>, (StatusCode, Json<ScimError>)> {
    let members: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT u.id, u.email
        FROM users u
        JOIN scim_group_members m ON m.user_id = u.id
        WHERE m.group_id = $1
        "#
    )
    .bind(group_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let base_url = get_base_url(state);
    Ok(members.into_iter().map(|(id, email)| ScimMember {
        value: id.to_string(),
        ref_url: Some(format!("{}/scim/v2/Users/{}", base_url, id)),
        display: Some(email),
    }).collect())
}

/// Fetch all users for listing
async fn fetch_all_users(
    state: &Arc<AppState>,
    org_id: Uuid,
    query: &ScimListQuery,
) -> Result<(Vec<UserRow>, i64), (StatusCode, Json<ScimError>)> {
    let offset = (query.start_index - 1).max(0);

    let users: Vec<UserRow> = sqlx::query_as(
        r#"
        SELECT u.id, u.email, u.first_name, u.last_name, u.phone,
               u.is_active, u.scim_external_id, u.created_at, u.updated_at
        FROM users u
        JOIN organization_members om ON om.user_id = u.id
        WHERE om.organization_id = $1
        ORDER BY u.email
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(org_id)
    .bind(query.count)
    .bind(offset)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?;

    let total: i64 = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(*) FROM users u
        JOIN organization_members om ON om.user_id = u.id
        WHERE om.organization_id = $1
        "#
    )
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| scim_error(500, format!("Database error: {}", e)))?
    .unwrap_or(0);

    Ok((users, total))
}

/// Extract member ID from SCIM path like "members[value eq \"uuid\"]"
fn extract_member_id_from_path(path: &str) -> Option<String> {
    // Parse: members[value eq "uuid"]
    if let Some(start) = path.find("\"") {
        if let Some(end) = path.rfind("\"") {
            if start < end {
                return Some(path[start + 1..end].to_string());
            }
        }
    }
    None
}
