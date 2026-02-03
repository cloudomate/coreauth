-- Migration 002: Remove Legacy Compatibility
-- Remove backward compatibility layers for clean slate

-- Drop the tenants view (alias for organizations)
DROP VIEW IF EXISTS tenants CASCADE;

-- Remove legacy tenant_id column from roles if it exists
-- (roles are already organization-scoped via tenant_id FK to organizations)
-- This is a no-op if the column doesn't exist or is already the FK

-- Clean up any legacy data patterns
-- Update NULL default_organization_id to ensure data integrity
-- (Users should have a default org or be platform admins)

-- Add comment to track migration
COMMENT ON TABLE organizations IS 'Multi-tenant organizations with hierarchical support (formerly aliased as tenants)';
