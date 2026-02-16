-- Migration 003: Two-Level Organizations
-- Add support for Tenant → Organization hierarchy (max 2 levels)

-- Add hierarchy columns to organizations table
ALTER TABLE organizations
ADD COLUMN IF NOT EXISTS parent_organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
ADD COLUMN IF NOT EXISTS hierarchy_level INTEGER DEFAULT 0 NOT NULL,
ADD COLUMN IF NOT EXISTS hierarchy_path TEXT;

-- Create index for hierarchical queries
CREATE INDEX IF NOT EXISTS idx_organizations_parent
ON organizations(parent_organization_id)
WHERE parent_organization_id IS NOT NULL;

-- Create index on hierarchy_path for fast descendant queries using btree
-- This allows efficient prefix matching with LIKE queries
CREATE INDEX IF NOT EXISTS idx_organizations_hierarchy_path
ON organizations(hierarchy_path text_pattern_ops);

-- Function to automatically maintain hierarchy on insert/update
CREATE OR REPLACE FUNCTION update_organization_hierarchy()
RETURNS TRIGGER AS $$
BEGIN
    -- Root organization (no parent)
    IF NEW.parent_organization_id IS NULL THEN
        NEW.hierarchy_level := 0;
        NEW.hierarchy_path := NEW.id::text;
    ELSE
        -- Child organization - inherit from parent
        SELECT
            hierarchy_level + 1,
            hierarchy_path || '/' || NEW.id::text
        INTO
            NEW.hierarchy_level,
            NEW.hierarchy_path
        FROM organizations
        WHERE id = NEW.parent_organization_id;

        -- Validate hierarchy level doesn't exceed max depth (2 levels: Tenant → Organization)
        IF NEW.hierarchy_level > 1 THEN
            RAISE EXCEPTION 'Maximum hierarchy depth of 2 levels exceeded. Organizations can only be created under tenants (root organizations).';
        END IF;

        -- Prevent circular references
        IF NEW.id = NEW.parent_organization_id THEN
            RAISE EXCEPTION 'Organization cannot be its own parent';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to maintain hierarchy
DROP TRIGGER IF EXISTS organization_hierarchy_trigger ON organizations;
CREATE TRIGGER organization_hierarchy_trigger
    BEFORE INSERT OR UPDATE OF parent_organization_id ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION update_organization_hierarchy();

-- Initialize hierarchy for existing organizations (all root level)
UPDATE organizations
SET
    hierarchy_level = 0,
    hierarchy_path = id::text
WHERE parent_organization_id IS NULL
  AND (hierarchy_path IS NULL OR hierarchy_level IS NULL);

-- Add constraint to prevent negative hierarchy levels
ALTER TABLE organizations
ADD CONSTRAINT chk_hierarchy_level_non_negative
CHECK (hierarchy_level >= 0);

-- Add constraint to ensure hierarchy_path is not null
ALTER TABLE organizations
ADD CONSTRAINT chk_hierarchy_path_not_null
CHECK (hierarchy_path IS NOT NULL);

-- Add comment
COMMENT ON COLUMN organizations.parent_organization_id IS 'Parent organization (tenant) for B2B structure (NULL = tenant/root)';
COMMENT ON COLUMN organizations.hierarchy_level IS 'Depth in organization tree (0 = tenant, 1 = organization, max 2 levels)';
COMMENT ON COLUMN organizations.hierarchy_path IS 'Materialized path for fast hierarchy queries (tenant_uuid/org_uuid)';
