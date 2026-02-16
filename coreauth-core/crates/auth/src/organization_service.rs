use crate::error::{AuthError, Result};
use ciam_database::{Database, OrganizationMemberRepository, TenantRepository};
use ciam_models::{
    organization_member::{
        roles, AddOrganizationMember, OrganizationMember, OrganizationMemberWithUser,
        UpdateMemberRole,
    },
    Tenant,
};
use uuid::Uuid;

pub struct OrganizationService {
    db: Database,
    member_repo: OrganizationMemberRepository,
    org_repo: TenantRepository, // Note: TenantRepository manages organizations
}

impl OrganizationService {
    pub fn new(db: Database) -> Self {
        let pool = db.pool().clone();

        Self {
            db,
            member_repo: OrganizationMemberRepository::new(pool.clone()),
            org_repo: TenantRepository::new(pool),
        }
    }

    /// Add a user to an organization
    pub async fn add_member(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        role: String,
    ) -> Result<OrganizationMember> {
        // Validate role
        if !self.is_valid_role(&role) {
            return Err(AuthError::InvalidInput(format!(
                "Invalid role: {}. Must be one of: admin, member, viewer, billing",
                role
            )));
        }

        // Check if organization exists
        self.org_repo
            .find_by_id(organization_id)
            .await
            .map_err(|_| AuthError::NotFound("Organization not found".to_string()))?;

        // Check if user is already a member
        if self
            .member_repo
            .is_member(user_id, organization_id)
            .await?
        {
            return Err(AuthError::AlreadyExists(
                "User is already a member of this organization".to_string(),
            ));
        }

        // Add member
        let request = AddOrganizationMember {
            user_id,
            organization_id,
            role,
        };

        let member = self
            .member_repo
            .add_member(request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(member)
    }

    /// Remove a user from an organization
    pub async fn remove_member(&self, user_id: Uuid, organization_id: Uuid) -> Result<()> {
        // Check if user is the last admin
        let admin_count = self.member_repo.count_admins(organization_id).await?;
        let is_admin = self
            .member_repo
            .has_role(user_id, organization_id, roles::ADMIN)
            .await?;

        if is_admin && admin_count <= 1 {
            return Err(AuthError::Forbidden(
                "Cannot remove the last admin from an organization".to_string(),
            ));
        }

        // Remove member
        let removed = self
            .member_repo
            .remove_member(user_id, organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        if !removed {
            return Err(AuthError::NotFound("Membership not found".to_string()));
        }

        Ok(())
    }

    /// Update a member's role
    pub async fn update_member_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        new_role: String,
    ) -> Result<OrganizationMember> {
        // Validate role
        if !self.is_valid_role(&new_role) {
            return Err(AuthError::InvalidInput(format!(
                "Invalid role: {}. Must be one of: admin, member, viewer, billing",
                new_role
            )));
        }

        // Check if this would remove the last admin
        let current_member = self
            .member_repo
            .get_member(user_id, organization_id)
            .await?
            .ok_or_else(|| AuthError::NotFound("Membership not found".to_string()))?;

        if current_member.role == roles::ADMIN && new_role != roles::ADMIN {
            let admin_count = self.member_repo.count_admins(organization_id).await?;
            if admin_count <= 1 {
                return Err(AuthError::Forbidden(
                    "Cannot remove admin role from the last admin".to_string(),
                ));
            }
        }

        // Update role
        let request = UpdateMemberRole { role: new_role };
        let member = self
            .member_repo
            .update_role(user_id, organization_id, request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(member)
    }

    /// Get a specific membership
    pub async fn get_member(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<OrganizationMember> {
        self.member_repo
            .get_member(user_id, organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or_else(|| AuthError::NotFound("Membership not found".to_string()))
    }

    /// Check if user is a member of an organization
    pub async fn is_member(&self, user_id: Uuid, organization_id: Uuid) -> Result<bool> {
        self.member_repo
            .is_member(user_id, organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Check if user has a specific role in an organization
    pub async fn has_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        role: &str,
    ) -> Result<bool> {
        self.member_repo
            .has_role(user_id, organization_id, role)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Check if user is an admin in an organization
    pub async fn is_admin(&self, user_id: Uuid, organization_id: Uuid) -> Result<bool> {
        self.has_role(user_id, organization_id, roles::ADMIN)
            .await
    }

    /// List all organizations for a user
    pub async fn list_user_organizations(&self, user_id: Uuid) -> Result<Vec<OrganizationMember>> {
        self.member_repo
            .list_by_user(user_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// List all members of an organization
    pub async fn list_organization_members(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationMember>> {
        self.member_repo
            .list_by_organization(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// List all members of an organization with user details
    pub async fn list_organization_members_with_users(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationMemberWithUser>> {
        self.member_repo
            .list_by_organization_with_users(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// List members by role
    pub async fn list_members_by_role(
        &self,
        organization_id: Uuid,
        role: &str,
    ) -> Result<Vec<OrganizationMember>> {
        if !self.is_valid_role(role) {
            return Err(AuthError::InvalidInput(format!("Invalid role: {}", role)));
        }

        self.member_repo
            .list_by_role(organization_id, role)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get member count for an organization
    pub async fn get_member_count(&self, organization_id: Uuid) -> Result<i64> {
        self.member_repo
            .count_by_organization(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get admin count for an organization
    pub async fn get_admin_count(&self, organization_id: Uuid) -> Result<i64> {
        self.member_repo
            .count_admins(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get organization details with member info
    pub async fn get_organization_with_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<OrganizationWithStats> {
        let organization = self
            .org_repo
            .find_by_id(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        let member_count = self.get_member_count(organization_id).await?;
        let admin_count = self.get_admin_count(organization_id).await?;

        Ok(OrganizationWithStats {
            organization,
            member_count,
            admin_count,
        })
    }

    /// Validate that a role is valid
    fn is_valid_role(&self, role: &str) -> bool {
        matches!(
            role,
            roles::ADMIN | roles::MEMBER | roles::VIEWER | roles::BILLING
        )
    }
}

#[derive(Debug, serde::Serialize)]
pub struct OrganizationWithStats {
    #[serde(flatten)]
    pub organization: Tenant,
    pub member_count: i64,
    pub admin_count: i64,
}
