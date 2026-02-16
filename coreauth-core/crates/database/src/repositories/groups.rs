use crate::error::{DatabaseError, Result};
use ciam_models::{
    Group, GroupMember, GroupMemberWithUser, GroupWithMemberCount,
    CreateGroup, UpdateGroup, AddGroupMember, UpdateGroupMember, GroupRole,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct GroupRepository {
    pool: PgPool,
}

impl GroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Group CRUD
    // ========================================================================

    /// Create a new group
    pub async fn create(&self, request: &CreateGroup) -> Result<Group> {
        let metadata = request.metadata.clone().unwrap_or(serde_json::json!({}));

        let group = sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO groups (tenant_id, name, slug, description, default_role_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&request.tenant_id)
        .bind(&request.name)
        .bind(&request.slug)
        .bind(&request.description)
        .bind(&request.default_role_id)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    /// Get group by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Group> {
        let group = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Group", &id.to_string()))?;

        Ok(group)
    }

    /// Get group by slug within a tenant
    pub async fn get_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Group> {
        let group = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE tenant_id = $1 AND slug = $2"
        )
        .bind(tenant_id)
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Group", slug))?;

        Ok(group)
    }

    /// List groups for a tenant
    pub async fn list_by_tenant(&self, tenant_id: Uuid, include_inactive: bool) -> Result<Vec<Group>> {
        let groups = if include_inactive {
            sqlx::query_as::<_, Group>(
                "SELECT * FROM groups WHERE tenant_id = $1 ORDER BY name"
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Group>(
                "SELECT * FROM groups WHERE tenant_id = $1 AND is_active = true ORDER BY name"
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(groups)
    }

    /// List groups with member count
    pub async fn list_with_member_count(&self, tenant_id: Uuid) -> Result<Vec<GroupWithMemberCount>> {
        // Get groups first
        let groups = self.list_by_tenant(tenant_id, false).await?;

        // Build result with member counts
        let mut result = Vec::with_capacity(groups.len());
        for group in groups {
            let member_count = self.count_members(group.id).await.unwrap_or(0);
            result.push(GroupWithMemberCount { group, member_count });
        }

        Ok(result)
    }

    /// Update a group
    pub async fn update(&self, id: Uuid, request: &UpdateGroup) -> Result<Group> {
        let current = self.get_by_id(id).await?;

        let name = request.name.as_ref().unwrap_or(&current.name);
        let description = request.description.as_ref().or(current.description.as_ref());
        let default_role_id = request.default_role_id.or(current.default_role_id);
        let metadata = request.metadata.as_ref().unwrap_or(&current.metadata);
        let is_active = request.is_active.unwrap_or(current.is_active);

        let group = sqlx::query_as::<_, Group>(
            r#"
            UPDATE groups
            SET name = $1, description = $2, default_role_id = $3, metadata = $4, is_active = $5, updated_at = NOW()
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(default_role_id)
        .bind(metadata)
        .bind(is_active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    /// Delete a group
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // Group Members
    // ========================================================================

    /// Add a member to a group
    pub async fn add_member(&self, group_id: Uuid, request: &AddGroupMember, added_by: Option<Uuid>) -> Result<GroupMember> {
        let role = request.role.clone().unwrap_or_else(|| "member".to_string());

        let member = sqlx::query_as::<_, GroupMember>(
            r#"
            INSERT INTO group_members (group_id, user_id, role, added_by, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (group_id, user_id) DO UPDATE
            SET role = EXCLUDED.role, expires_at = EXCLUDED.expires_at
            RETURNING *
            "#,
        )
        .bind(group_id)
        .bind(&request.user_id)
        .bind(&role)
        .bind(added_by)
        .bind(&request.expires_at)
        .fetch_one(&self.pool)
        .await?;

        // If the group has a default role, assign it to the user
        let group = self.get_by_id(group_id).await?;
        if let Some(role_id) = group.default_role_id {
            let _ = sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                VALUES ($1, $2)
                ON CONFLICT (user_id, role_id) DO NOTHING
                "#,
            )
            .bind(&request.user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await;
        }

        Ok(member)
    }

    /// Remove a member from a group
    pub async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update a member's role in a group
    pub async fn update_member(&self, group_id: Uuid, user_id: Uuid, request: &UpdateGroupMember) -> Result<GroupMember> {
        let member = sqlx::query_as::<_, GroupMember>(
            r#"
            UPDATE group_members
            SET role = COALESCE($1, role), expires_at = COALESCE($2, expires_at)
            WHERE group_id = $3 AND user_id = $4
            RETURNING *
            "#,
        )
        .bind(&request.role)
        .bind(&request.expires_at)
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("GroupMember", &format!("{}:{}", group_id, user_id)))?;

        Ok(member)
    }

    /// List members of a group
    pub async fn list_members(&self, group_id: Uuid) -> Result<Vec<GroupMember>> {
        let members = sqlx::query_as::<_, GroupMember>(
            r#"
            SELECT * FROM group_members
            WHERE group_id = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY added_at
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// List members with user details
    pub async fn list_members_with_users(&self, group_id: Uuid) -> Result<Vec<GroupMemberWithUser>> {
        let rows: Vec<(Uuid, Uuid, Uuid, String, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT gm.id, gm.group_id, gm.user_id, gm.role, gm.added_at, gm.expires_at, u.email, u.metadata->>'full_name' as full_name
            FROM group_members gm
            JOIN users u ON u.id = gm.user_id
            WHERE gm.group_id = $1
            AND (gm.expires_at IS NULL OR gm.expires_at > NOW())
            ORDER BY gm.added_at
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        let members = rows
            .into_iter()
            .map(|(id, group_id, user_id, role, added_at, expires_at, email, full_name)| {
                GroupMemberWithUser {
                    id,
                    group_id,
                    user_id,
                    role,
                    added_at,
                    expires_at,
                    email,
                    full_name,
                }
            })
            .collect();

        Ok(members)
    }

    /// Get groups a user belongs to
    pub async fn get_user_groups(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Vec<Group>> {
        let groups = sqlx::query_as::<_, Group>(
            r#"
            SELECT g.* FROM groups g
            JOIN group_members gm ON gm.group_id = g.id
            WHERE gm.user_id = $1
            AND g.tenant_id = $2
            AND g.is_active = true
            AND (gm.expires_at IS NULL OR gm.expires_at > NOW())
            ORDER BY g.name
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Check if user is member of a group
    pub async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM group_members gm
                JOIN groups g ON g.id = gm.group_id
                WHERE gm.group_id = $1 AND gm.user_id = $2
                AND g.is_active = true
                AND (gm.expires_at IS NULL OR gm.expires_at > NOW())
            )
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.0)
    }

    // ========================================================================
    // Group Roles
    // ========================================================================

    /// Assign a role to a group
    pub async fn assign_role(&self, group_id: Uuid, role_id: Uuid) -> Result<GroupRole> {
        let group_role = sqlx::query_as::<_, GroupRole>(
            r#"
            INSERT INTO group_roles (group_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT (group_id, role_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(group_id)
        .bind(role_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(group_role)
    }

    /// Remove a role from a group
    pub async fn remove_role(&self, group_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM group_roles WHERE group_id = $1 AND role_id = $2")
            .bind(group_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List roles assigned to a group
    pub async fn list_group_roles(&self, group_id: Uuid) -> Result<Vec<GroupRole>> {
        let roles = sqlx::query_as::<_, GroupRole>(
            "SELECT * FROM group_roles WHERE group_id = $1"
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    /// Count members in a group
    pub async fn count_members(&self, group_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM group_members WHERE group_id = $1 AND (expires_at IS NULL OR expires_at > NOW())"
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}
