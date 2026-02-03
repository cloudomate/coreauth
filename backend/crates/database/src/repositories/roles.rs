use crate::error::{DatabaseError, Result};
use ciam_models::role::{AssignRole, NewRole, UpdateRole};
use ciam_models::{Permission, Role};
use sqlx::PgPool;
use uuid::Uuid;

pub struct RoleRepository {
    pool: PgPool,
}

impl RoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new role
    pub async fn create(&self, new_role: &NewRole) -> Result<Role> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (tenant_id, name, description, parent_role_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&new_role.tenant_id)
        .bind(&new_role.name)
        .bind(&new_role.description)
        .bind(&new_role.parent_role_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(role)
    }

    /// Find role by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Role> {
        let role = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Role", &id.to_string()))?;

        Ok(role)
    }

    /// Find role by name and tenant
    pub async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Role> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE tenant_id = $1 AND name = $2",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Role", name))?;

        Ok(role)
    }

    /// List all roles for a tenant
    pub async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    /// Update role
    pub async fn update(&self, id: Uuid, update: &UpdateRole) -> Result<Role> {
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE roles SET updated_at = NOW()");

        let mut has_updates = false;

        if let Some(ref name) = update.name {
            query_builder.push(", name = ");
            query_builder.push_bind(name);
            has_updates = true;
        }

        if let Some(ref description) = update.description {
            query_builder.push(", description = ");
            query_builder.push_bind(description);
            has_updates = true;
        }

        if let Some(parent_role_id) = update.parent_role_id {
            query_builder.push(", parent_role_id = ");
            query_builder.push_bind(parent_role_id);
            has_updates = true;
        }

        if !has_updates {
            return self.find_by_id(id).await;
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let role = query_builder
            .build_query_as::<Role>()
            .fetch_one(&self.pool)
            .await?;

        Ok(role)
    }

    /// Delete role
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // User-Role Management

    /// Assign role to user
    pub async fn assign_to_user(&self, assignment: &AssignRole) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&assignment.user_id)
        .bind(&assignment.role_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(())
    }

    /// Revoke role from user
    pub async fn revoke_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get all roles for a user
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT r.* FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    // Permission Management

    /// Get all permissions for a role
    pub async fn get_permissions(&self, role_id: Uuid) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    /// Get all permissions for a user (including from all their roles)
    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT DISTINCT p.* FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    /// Assign permission to role
    pub async fn assign_permission(&self, role_id: Uuid, permission_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Revoke permission from role
    pub async fn revoke_permission(&self, role_id: Uuid, permission_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2",
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if user has a specific permission
    pub async fn user_has_permission(
        &self,
        user_id: Uuid,
        permission_name: &str,
    ) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                JOIN user_roles ur ON rp.role_id = ur.role_id
                WHERE ur.user_id = $1 AND p.name = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(permission_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.0).unwrap_or(false))
    }
}
