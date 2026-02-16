-- Migration 014: Standardize Naming - Organizations → Tenants
-- This fixes the confusing terminology where "organizations" was used for both
-- the top-level CoreAuth customer AND their sub-organizations.
--
-- New clear model:
--   tenants (table) = CoreAuth customers
--   ├── hierarchy_level = 0: Root tenant (the CoreAuth customer)
--   ├── hierarchy_level = 1: Sub-organization (B2B customer of the tenant)
--   └── account_type: 'personal' or 'business'

-- ============================================================
-- 1. Rename organizations table to tenants
-- ============================================================

ALTER TABLE IF EXISTS organizations RENAME TO tenants;

-- ============================================================
-- 2. Add account_type column
-- ============================================================

ALTER TABLE tenants
ADD COLUMN IF NOT EXISTS account_type VARCHAR(20) DEFAULT 'business'
CHECK (account_type IN ('personal', 'business'));

COMMENT ON COLUMN tenants.account_type IS 'Account type: personal (individual) or business (company)';

-- ============================================================
-- 3. Rename parent_organization_id to parent_tenant_id
-- ============================================================

-- Drop old constraint if exists
ALTER TABLE tenants DROP CONSTRAINT IF EXISTS organizations_parent_organization_id_fkey;

-- Rename column
ALTER TABLE tenants RENAME COLUMN parent_organization_id TO parent_tenant_id;

-- Add new foreign key
ALTER TABLE tenants
ADD CONSTRAINT tenants_parent_tenant_id_fkey
FOREIGN KEY (parent_tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

-- ============================================================
-- 4. Rename organization_members to tenant_members
-- ============================================================

ALTER TABLE IF EXISTS organization_members RENAME TO tenant_members;

-- Rename the organization_id column in tenant_members
ALTER TABLE tenant_members RENAME COLUMN organization_id TO tenant_id;

-- Update constraints
ALTER TABLE tenant_members DROP CONSTRAINT IF EXISTS organization_members_organization_id_fkey;
ALTER TABLE tenant_members DROP CONSTRAINT IF EXISTS organization_members_user_id_organization_id_key;
ALTER TABLE tenant_members DROP CONSTRAINT IF EXISTS organization_members_pkey;

ALTER TABLE tenant_members
ADD CONSTRAINT tenant_members_pkey PRIMARY KEY (id);

ALTER TABLE tenant_members
ADD CONSTRAINT tenant_members_tenant_id_fkey
FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

ALTER TABLE tenant_members
ADD CONSTRAINT tenant_members_user_id_tenant_id_key
UNIQUE (user_id, tenant_id);

-- ============================================================
-- 5. Rename indexes
-- ============================================================

-- Drop old indexes
DROP INDEX IF EXISTS idx_organizations_parent;
DROP INDEX IF EXISTS idx_organizations_hierarchy_path;
DROP INDEX IF EXISTS idx_org_members_org;
DROP INDEX IF EXISTS idx_org_members_role;

-- Create new indexes
CREATE INDEX IF NOT EXISTS idx_tenants_parent
ON tenants(parent_tenant_id)
WHERE parent_tenant_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_tenants_hierarchy_path
ON tenants(hierarchy_path text_pattern_ops);

CREATE INDEX IF NOT EXISTS idx_tenant_members_tenant
ON tenant_members(tenant_id);

CREATE INDEX IF NOT EXISTS idx_tenant_members_role
ON tenant_members(tenant_id, role);

CREATE INDEX IF NOT EXISTS idx_tenants_account_type
ON tenants(account_type);

-- ============================================================
-- 6. Update the hierarchy trigger function
-- ============================================================

CREATE OR REPLACE FUNCTION update_tenant_hierarchy()
RETURNS TRIGGER AS $$
BEGIN
    -- Root tenant (no parent)
    IF NEW.parent_tenant_id IS NULL THEN
        NEW.hierarchy_level := 0;
        NEW.hierarchy_path := NEW.id::text;
    ELSE
        -- Sub-organization - inherit from parent
        SELECT
            hierarchy_level + 1,
            hierarchy_path || '/' || NEW.id::text
        INTO
            NEW.hierarchy_level,
            NEW.hierarchy_path
        FROM tenants
        WHERE id = NEW.parent_tenant_id;

        -- Validate hierarchy level doesn't exceed max depth (2 levels)
        IF NEW.hierarchy_level > 1 THEN
            RAISE EXCEPTION 'Maximum hierarchy depth of 2 levels exceeded. Sub-organizations can only be created directly under root tenants.';
        END IF;

        -- Prevent circular references
        IF NEW.id = NEW.parent_tenant_id THEN
            RAISE EXCEPTION 'Tenant cannot be its own parent';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop old trigger and create new one
DROP TRIGGER IF EXISTS organization_hierarchy_trigger ON tenants;
DROP TRIGGER IF EXISTS tenant_hierarchy_trigger ON tenants;

CREATE TRIGGER tenant_hierarchy_trigger
    BEFORE INSERT OR UPDATE OF parent_tenant_id ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_tenant_hierarchy();

-- Drop old function
DROP FUNCTION IF EXISTS update_organization_hierarchy();

-- ============================================================
-- 7. Update foreign keys in other tables that reference organizations
-- ============================================================

-- connections table: organization_id → tenant_id (for org-scoped connections)
ALTER TABLE connections DROP CONSTRAINT IF EXISTS connections_organization_id_fkey;
ALTER TABLE connections RENAME COLUMN organization_id TO tenant_id;
ALTER TABLE connections
ADD CONSTRAINT connections_tenant_id_fkey
FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

-- Update the check constraint
ALTER TABLE connections DROP CONSTRAINT IF EXISTS connections_scope_org_check;
ALTER TABLE connections
ADD CONSTRAINT connections_scope_tenant_check
CHECK (
    ((scope = 'platform') AND (tenant_id IS NULL)) OR
    ((scope = 'organization') AND (tenant_id IS NOT NULL))
);

DROP INDEX IF EXISTS idx_connections_org;
CREATE INDEX IF NOT EXISTS idx_connections_tenant
ON connections(tenant_id) WHERE tenant_id IS NOT NULL;

-- ============================================================
-- 8. Update users table: default_organization_id → default_tenant_id
-- ============================================================

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_default_organization_id_fkey;
ALTER TABLE users RENAME COLUMN default_organization_id TO default_tenant_id;
ALTER TABLE users
ADD CONSTRAINT users_default_tenant_id_fkey
FOREIGN KEY (default_tenant_id) REFERENCES tenants(id) ON DELETE SET NULL;

-- ============================================================
-- 9. Update other tables with organization_id references
-- ============================================================

-- audit_logs: keep tenant_id, rename organization_id to sub_tenant_id
ALTER TABLE audit_logs RENAME COLUMN organization_id TO sub_tenant_id;
DROP INDEX IF EXISTS idx_audit_logs_org;
CREATE INDEX IF NOT EXISTS idx_audit_logs_sub_tenant
ON audit_logs(sub_tenant_id) WHERE sub_tenant_id IS NOT NULL;

-- relation_tuples: rename organization_id to sub_tenant_id
ALTER TABLE relation_tuples RENAME COLUMN organization_id TO sub_tenant_id;
DROP INDEX IF EXISTS idx_relation_tuples_org;
CREATE INDEX IF NOT EXISTS idx_relation_tuples_sub_tenant
ON relation_tuples(sub_tenant_id) WHERE sub_tenant_id IS NOT NULL;

-- ============================================================
-- 10. Update constraint names
-- ============================================================

ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_isolation_mode_check;
ALTER TABLE tenants DROP CONSTRAINT IF EXISTS organizations_isolation_mode_check;
ALTER TABLE tenants
ADD CONSTRAINT tenants_isolation_mode_check
CHECK (isolation_mode IN ('pool', 'silo'));

-- ============================================================
-- 11. Add comments for clarity
-- ============================================================

COMMENT ON TABLE tenants IS 'CoreAuth customer accounts (tenants). hierarchy_level=0 is root tenant, hierarchy_level=1 is sub-organization.';
COMMENT ON COLUMN tenants.parent_tenant_id IS 'Parent tenant for sub-organizations (NULL for root tenants)';
COMMENT ON COLUMN tenants.hierarchy_level IS 'Depth in hierarchy: 0=root tenant, 1=sub-organization';
COMMENT ON COLUMN tenants.hierarchy_path IS 'Materialized path for hierarchy queries (tenant_id/sub_org_id)';

COMMENT ON TABLE tenant_members IS 'User membership in tenants/sub-organizations';
COMMENT ON COLUMN tenant_members.tenant_id IS 'The tenant or sub-organization the user belongs to';

-- ============================================================
-- Done! The naming is now consistent:
--   - tenants table (was organizations)
--   - tenant_id foreign keys (was organization_id)
--   - tenant_members table (was organization_members)
--   - parent_tenant_id (was parent_organization_id)
-- ============================================================
