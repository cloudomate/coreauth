//! Groups Management Handlers
//!
//! API endpoints for managing groups within tenants.
//! Groups can be used to organize users and assign collective permissions.

use crate::handlers::auth::ErrorResponse;
use crate::middleware::auth::AuthUser;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use ciam_database::repositories::groups::GroupRepository;
use ciam_models::{
    Group, GroupMember, GroupMemberWithUser, GroupWithMemberCount,
    CreateGroup, UpdateGroup, AddGroupMember, UpdateGroupMember, GroupRole,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListGroupsQuery {
    #[serde(default)]
    pub include_inactive: bool,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    #[serde(flatten)]
    pub group: Group,
    pub member_count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GroupListResponse {
    pub groups: Vec<GroupWithMemberCount>,
    pub total: usize,
}

// ============================================================================
// Group CRUD Endpoints
// ============================================================================

/// Create a new group
/// POST /api/tenants/:tenant_id/groups
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
    Json(mut request): Json<CreateGroup>,
) -> Result<Json<Group>, (StatusCode, Json<ErrorResponse>)> {
    // Set tenant_id from path
    request.tenant_id = tenant_id;

    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    match repo.create(&request).await {
        Ok(group) => Ok(Json(group)),
        Err(e) => {
            tracing::error!("Failed to create group: {}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse::new("group_exists", "A group with this slug already exists")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("create_failed", &e.to_string())),
                ))
            }
        }
    }
}

/// Get a group by ID
/// GET /api/tenants/:tenant_id/groups/:group_id
pub async fn get_group(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GroupResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    let group = repo.get_by_id(group_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    // Verify group belongs to the tenant
    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    let member_count = repo.count_members(group_id).await.ok();

    Ok(Json(GroupResponse { group, member_count }))
}

/// List groups for a tenant
/// GET /api/tenants/:tenant_id/groups
pub async fn list_groups(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<ListGroupsQuery>,
) -> Result<Json<GroupListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    match repo.list_with_member_count(tenant_id).await {
        Ok(groups) => {
            let total = groups.len();
            Ok(Json(GroupListResponse { groups, total }))
        }
        Err(e) => {
            tracing::error!("Failed to list groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_failed", &e.to_string())),
            ))
        }
    }
}

/// Update a group
/// PUT /api/tenants/:tenant_id/groups/:group_id
pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateGroup>,
) -> Result<Json<Group>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let existing = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if existing.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.update(group_id, &request).await {
        Ok(group) => Ok(Json(group)),
        Err(e) => {
            tracing::error!("Failed to update group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("update_failed", &e.to_string())),
            ))
        }
    }
}

/// Delete a group
/// DELETE /api/tenants/:tenant_id/groups/:group_id
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let existing = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if existing.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.delete(group_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("delete_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Group Member Endpoints
// ============================================================================

/// Add a member to a group
/// POST /api/tenants/:tenant_id/groups/:group_id/members
pub async fn add_member(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AddGroupMember>,
) -> Result<Json<GroupMember>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.add_member(group_id, &request, Some(auth_user.user_id)).await {
        Ok(member) => Ok(Json(member)),
        Err(e) => {
            tracing::error!("Failed to add member: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("add_member_failed", &e.to_string())),
            ))
        }
    }
}

/// Remove a member from a group
/// DELETE /api/tenants/:tenant_id/groups/:group_id/members/:user_id
pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id, user_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.remove_member(group_id, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to remove member: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("remove_member_failed", &e.to_string())),
            ))
        }
    }
}

/// List members of a group
/// GET /api/tenants/:tenant_id/groups/:group_id/members
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<GroupMemberWithUser>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.list_members_with_users(group_id).await {
        Ok(members) => Ok(Json(members)),
        Err(e) => {
            tracing::error!("Failed to list members: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_members_failed", &e.to_string())),
            ))
        }
    }
}

/// Update a member's role in a group
/// PATCH /api/tenants/:tenant_id/groups/:group_id/members/:user_id
pub async fn update_member(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id, user_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<UpdateGroupMember>,
) -> Result<Json<GroupMember>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.update_member(group_id, user_id, &request).await {
        Ok(member) => Ok(Json(member)),
        Err(e) => {
            tracing::error!("Failed to update member: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("update_member_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// Group Role Endpoints
// ============================================================================

/// Assign a role to a group
/// POST /api/tenants/:tenant_id/groups/:group_id/roles
#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

pub async fn assign_role(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AssignRoleRequest>,
) -> Result<Json<GroupRole>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.assign_role(group_id, request.role_id).await {
        Ok(group_role) => Ok(Json(group_role)),
        Err(e) => {
            tracing::error!("Failed to assign role: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("assign_role_failed", &e.to_string())),
            ))
        }
    }
}

/// Remove a role from a group
/// DELETE /api/tenants/:tenant_id/groups/:group_id/roles/:role_id
pub async fn remove_role(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.remove_role(group_id, role_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to remove role: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("remove_role_failed", &e.to_string())),
            ))
        }
    }
}

/// List roles assigned to a group
/// GET /api/tenants/:tenant_id/groups/:group_id/roles
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<GroupRole>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    // Verify group belongs to tenant
    let group = repo.get_by_id(group_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        )
    })?;

    if group.tenant_id != tenant_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Group not found")),
        ));
    }

    match repo.list_group_roles(group_id).await {
        Ok(roles) => Ok(Json(roles)),
        Err(e) => {
            tracing::error!("Failed to list roles: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_roles_failed", &e.to_string())),
            ))
        }
    }
}

// ============================================================================
// User Groups Endpoint
// ============================================================================

/// Get groups a user belongs to
/// GET /api/tenants/:tenant_id/users/:user_id/groups
pub async fn get_user_groups(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path((tenant_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Group>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = GroupRepository::new(state.auth_service.db.pool().clone());

    match repo.get_user_groups(user_id, tenant_id).await {
        Ok(groups) => Ok(Json(groups)),
        Err(e) => {
            tracing::error!("Failed to get user groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("get_user_groups_failed", &e.to_string())),
            ))
        }
    }
}
