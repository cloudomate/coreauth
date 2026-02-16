-- Migration 004: Application Organization Scoping
-- Make applications organization-scoped instead of global

-- Clean slate - remove existing applications data
DELETE FROM applications;

-- Add organization_id column to applications
ALTER TABLE applications
ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Create index on organization_id
CREATE INDEX IF NOT EXISTS idx_applications_organization
ON applications(organization_id)
WHERE organization_id IS NOT NULL;

-- Drop existing unique constraint on slug (was globally unique)
ALTER TABLE applications
DROP CONSTRAINT IF EXISTS applications_slug_key;

-- Create composite unique constraint: slug must be unique within an organization
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_slug_org
ON applications(organization_id, slug)
WHERE organization_id IS NOT NULL;

-- For platform-level/global applications (organization_id IS NULL), slug must still be globally unique
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_slug_global
ON applications(slug)
WHERE organization_id IS NULL;

-- Add comment
COMMENT ON COLUMN applications.organization_id IS 'Organization that owns this application (NULL = platform-level/global application)';

-- Ensure client_id remains globally unique
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_client_id
ON applications(client_id);

-- Update application type enum if needed (ensure it matches the schema)
-- The application type is already defined as TEXT with CHECK constraint in 001_init.sql
-- No changes needed here
