use ciam_models::{
    AddOrganizationMember, OrganizationMember, OrganizationMemberWithUser, UpdateMemberRole,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct OrganizationMemberRepository {
    pool: PgPool,
}

impl OrganizationMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a user to an organization
    pub async fn add_member(
        &self,
        request: AddOrganizationMember,
    ) -> Result<OrganizationMember, sqlx::Error> {
        let member = sqlx::query_as!(
            OrganizationMember,
            r#"
            INSERT INTO organization_members (user_id, organization_id, role)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, organization_id, role, joined_at
            "#,
            request.user_id,
            request.organization_id,
            request.role
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    /// Remove a user from an organization
    pub async fn remove_member(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            DELETE FROM organization_members
            WHERE user_id = $1 AND organization_id = $2
            "#,
            user_id,
            organization_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update a member's role
    pub async fn update_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        new_role: UpdateMemberRole,
    ) -> Result<OrganizationMember, sqlx::Error> {
        let member = sqlx::query_as!(
            OrganizationMember,
            r#"
            UPDATE organization_members
            SET role = $3
            WHERE user_id = $1 AND organization_id = $2
            RETURNING id, user_id, organization_id, role, joined_at
            "#,
            user_id,
            organization_id,
            new_role.role
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    /// Get a specific membership
    pub async fn get_member(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<OrganizationMember>, sqlx::Error> {
        let member = sqlx::query_as!(
            OrganizationMember,
            r#"
            SELECT id, user_id, organization_id, role, joined_at
            FROM organization_members
            WHERE user_id = $1 AND organization_id = $2
            "#,
            user_id,
            organization_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(member)
    }

    /// Check if user is a member of an organization
    pub async fn is_member(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM organization_members
                WHERE user_id = $1 AND organization_id = $2
            ) as "exists!"
            "#,
            user_id,
            organization_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists)
    }

    /// Check if user has a specific role in an organization
    pub async fn has_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        role: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM organization_members
                WHERE user_id = $1 AND organization_id = $2 AND role = $3
            ) as "exists!"
            "#,
            user_id,
            organization_id,
            role
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists)
    }

    /// List all organizations for a user
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<OrganizationMember>, sqlx::Error> {
        let members = sqlx::query_as!(
            OrganizationMember,
            r#"
            SELECT id, user_id, organization_id, role, joined_at
            FROM organization_members
            WHERE user_id = $1
            ORDER BY joined_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// List all members of an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationMember>, sqlx::Error> {
        let members = sqlx::query_as!(
            OrganizationMember,
            r#"
            SELECT id, user_id, organization_id, role, joined_at
            FROM organization_members
            WHERE organization_id = $1
            ORDER BY joined_at ASC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// List all members of an organization with user details
    pub async fn list_by_organization_with_users(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationMemberWithUser>, sqlx::Error> {
        let members = sqlx::query_as!(
            OrganizationMemberWithUser,
            r#"
            SELECT
                om.id,
                om.user_id,
                om.organization_id,
                om.role,
                om.joined_at,
                u.email,
                u.email_verified as "email_verified!",
                u.is_active as "is_active!",
                u.mfa_enabled as "mfa_enabled!"
            FROM organization_members om
            INNER JOIN users u ON om.user_id = u.id
            WHERE om.organization_id = $1
            ORDER BY om.joined_at ASC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// List members by role
    pub async fn list_by_role(
        &self,
        organization_id: Uuid,
        role: &str,
    ) -> Result<Vec<OrganizationMember>, sqlx::Error> {
        let members = sqlx::query_as!(
            OrganizationMember,
            r#"
            SELECT id, user_id, organization_id, role, joined_at
            FROM organization_members
            WHERE organization_id = $1 AND role = $2
            ORDER BY joined_at ASC
            "#,
            organization_id,
            role
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// Get member count for an organization
    pub async fn count_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM organization_members
            WHERE organization_id = $1
            "#,
            organization_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count)
    }

    /// Get admin count for an organization
    pub async fn count_admins(
        &self,
        organization_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM organization_members
            WHERE organization_id = $1 AND role = 'admin'
            "#,
            organization_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count)
    }
}
