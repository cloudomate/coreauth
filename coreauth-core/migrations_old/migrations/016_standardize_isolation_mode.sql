-- Migration 016: Standardize Isolation Mode Terminology
-- Changes 'pool'/'silo' to 'shared'/'dedicated' for consistency

-- 1. Drop the old constraint
ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_isolation_mode_check;

-- 2. Update existing values
UPDATE tenants SET isolation_mode = 'shared' WHERE isolation_mode = 'pool';
UPDATE tenants SET isolation_mode = 'dedicated' WHERE isolation_mode = 'silo';

-- 3. Change the default
ALTER TABLE tenants ALTER COLUMN isolation_mode SET DEFAULT 'shared';

-- 4. Add new constraint with standardized terminology
ALTER TABLE tenants
ADD CONSTRAINT tenants_isolation_mode_check
CHECK (isolation_mode IN ('shared', 'dedicated'));

-- 5. Add comment
COMMENT ON COLUMN tenants.isolation_mode IS 'Database isolation mode: shared (multi-tenant database) or dedicated (tenant-specific database)';
