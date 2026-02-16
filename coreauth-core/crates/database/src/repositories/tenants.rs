use crate::error::{DatabaseError, Result};
use ciam_models::{CreateOrganization, Organization, UpdateOrganization};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OrganizationRepository {
    pool: PgPool,
}

impl OrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new organization (tenant or sub-organization)
    pub async fn create(&self, request: &CreateOrganization) -> Result<Organization> {
        let settings = request.settings.clone().unwrap_or_default();
        let settings_json = sqlx::types::Json(&settings);
        let isolation_mode = request.isolation_mode.clone().unwrap_or_default();

        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO tenants (slug, name, parent_tenant_id, isolation_mode, custom_domain, settings)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&request.slug)
        .bind(&request.name)
        .bind(&request.parent_tenant_id)
        .bind(&isolation_mode)
        .bind(&request.custom_domain)
        .bind(settings_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    /// Find organization by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>("SELECT * FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Organization", &id.to_string()))?;

        Ok(org)
    }

    /// Find organization by slug
    pub async fn find_by_slug(&self, slug: &str) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>("SELECT * FROM tenants WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Organization", slug))?;

        Ok(org)
    }

    /// Find organization by custom domain
    pub async fn find_by_custom_domain(&self, domain: &str) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            "SELECT * FROM tenants WHERE custom_domain = $1",
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Organization", domain))?;

        Ok(org)
    }

    /// List all root organizations (tenants) - paginated
    pub async fn list_tenants(&self, limit: i64, offset: i64) -> Result<Vec<Organization>> {
        let orgs = sqlx::query_as::<_, Organization>(
            r#"
            SELECT * FROM tenants
            WHERE parent_tenant_id IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(orgs)
    }

    /// List organizations under a parent (tenant's organizations)
    pub async fn list_by_parent(&self, parent_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Organization>> {
        let orgs = sqlx::query_as::<_, Organization>(
            r#"
            SELECT * FROM tenants
            WHERE parent_tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(parent_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(orgs)
    }

    /// Get all descendants of an organization (recursive via hierarchy_path)
    pub async fn get_descendants(&self, org_id: Uuid) -> Result<Vec<Organization>> {
        let orgs = sqlx::query_as::<_, Organization>(
            r#"
            SELECT child.* FROM tenants parent
            JOIN tenants child ON child.hierarchy_path LIKE parent.hierarchy_path || '/%'
            WHERE parent.id = $1
            ORDER BY child.hierarchy_level, child.created_at
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(orgs)
    }

    /// Get ancestors of an organization (parent chain to root)
    pub async fn get_ancestors(&self, org_id: Uuid) -> Result<Vec<Organization>> {
        let orgs = sqlx::query_as::<_, Organization>(
            r#"
            WITH RECURSIVE ancestors AS (
                SELECT * FROM tenants WHERE id = $1
                UNION ALL
                SELECT o.* FROM tenants o
                INNER JOIN ancestors a ON o.id = a.parent_tenant_id
            )
            SELECT * FROM ancestors
            WHERE id != $1
            ORDER BY hierarchy_level
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(orgs)
    }

    /// Get root organization (tenant) for any organization
    pub async fn get_root(&self, org_id: Uuid) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            WITH RECURSIVE ancestors AS (
                SELECT * FROM tenants WHERE id = $1
                UNION ALL
                SELECT o.* FROM tenants o
                INNER JOIN ancestors a ON o.id = a.parent_tenant_id
            )
            SELECT * FROM ancestors
            WHERE parent_tenant_id IS NULL
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DatabaseError::not_found("Root organization", &org_id.to_string()))?;

        Ok(org)
    }

    /// Count total organizations (optionally filter by parent)
    pub async fn count(&self, parent_id: Option<Uuid>) -> Result<i64> {
        let count: (i64,) = match parent_id {
            Some(parent) => {
                sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE parent_tenant_id = $1")
                    .bind(parent)
                    .fetch_one(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE parent_tenant_id IS NULL")
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok(count.0)
    }

    /// Update organization (partial update)
    pub async fn update(&self, id: Uuid, request: &UpdateOrganization) -> Result<Organization> {
        // Get current organization
        let current = self.find_by_id(id).await?;

        // Build update with current values as fallbacks
        let name = request.name.as_ref().unwrap_or(&current.name);
        let isolation_mode = request.isolation_mode.as_ref().unwrap_or(&current.isolation_mode);
        let custom_domain = request.custom_domain.as_ref().or(current.custom_domain.as_ref());
        let settings = request.settings.as_ref().unwrap_or(&current.settings);

        let org = sqlx::query_as::<_, Organization>(
            r#"
            UPDATE tenants
            SET name = $1, isolation_mode = $2, custom_domain = $3, settings = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(isolation_mode)
        .bind(custom_domain)
        .bind(sqlx::types::Json(settings))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    /// Update organization settings
    pub async fn update_settings(
        &self,
        id: Uuid,
        settings: &ciam_models::OrganizationSettings,
    ) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            UPDATE tenants
            SET settings = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(sqlx::types::Json(settings))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    /// Update custom domain
    pub async fn update_custom_domain(
        &self,
        id: Uuid,
        custom_domain: Option<String>,
    ) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            UPDATE tenants
            SET custom_domain = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(custom_domain)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    /// Delete organization (cascade deletes children)
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// Backward compatibility alias
pub type TenantRepository = OrganizationRepository;
