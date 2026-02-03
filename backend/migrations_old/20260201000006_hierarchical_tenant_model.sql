-- Migration: Hierarchical Tenant Model (Auth0-style Organizations)
-- This migration transforms the flat tenant model into a hierarchical model
-- where users can be platform admins or organization members

-- ============================================================
-- STEP 1: Create platform_config table
-- ============================================================

CREATE TABLE IF NOT EXISTS platform_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Insert default platform config for on-prem deployment
INSERT INTO platform_config (name, slug)
VALUES ('OnPrem CIAM Platform', 'onprem')
ON CONFLICT (slug) DO NOTHING;

COMMENT ON TABLE platform_config IS 'Root platform configuration (Auth0 Tenant equivalent)';

-- ============================================================
-- STEP 2: Rename tenants to organizations
-- ============================================================

-- Check if organizations table doesn't exist yet
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'organizations') THEN
        -- Rename tenants table to organizations
        ALTER TABLE tenants RENAME TO organizations;

        -- Rename sequence
        ALTER SEQUENCE IF EXISTS tenants_id_seq RENAME TO organizations_id_seq;

        -- Rename indexes
        ALTER INDEX IF EXISTS idx_tenants_slug RENAME TO idx_organizations_slug;

        RAISE NOTICE 'Renamed tenants table to organizations';
    ELSE
        RAISE NOTICE 'Organizations table already exists, skipping rename';
    END IF;
END $$;

COMMENT ON TABLE organizations IS 'Organizations (customer workspaces) - formerly tenants';

-- ============================================================
-- STEP 3: Modify users table for global pool
-- ============================================================

-- Make tenant_id nullable (users can exist without org membership)
DO $$
BEGIN
    -- Check if column is NOT NULL
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users'
        AND column_name = 'tenant_id'
        AND is_nullable = 'NO'
    ) THEN
        ALTER TABLE users ALTER COLUMN tenant_id DROP NOT NULL;
        RAISE NOTICE 'Made users.tenant_id nullable';
    END IF;
END $$;

-- Rename tenant_id to default_organization_id for clarity
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users'
        AND column_name = 'tenant_id'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users'
        AND column_name = 'default_organization_id'
    ) THEN
        ALTER TABLE users RENAME COLUMN tenant_id TO default_organization_id;
        RAISE NOTICE 'Renamed users.tenant_id to default_organization_id';
    END IF;
END $$;

-- Add platform admin flag
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_platform_admin BOOLEAN DEFAULT false NOT NULL;

-- Update foreign key constraint
DO $$
BEGIN
    -- Drop old constraint if exists
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'users_tenant_id_fkey'
        AND table_name = 'users'
    ) THEN
        ALTER TABLE users DROP CONSTRAINT users_tenant_id_fkey;
    END IF;

    -- Add new constraint with correct column name
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'users_default_organization_id_fkey'
        AND table_name = 'users'
    ) THEN
        ALTER TABLE users
        ADD CONSTRAINT users_default_organization_id_fkey
        FOREIGN KEY (default_organization_id)
        REFERENCES organizations(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Create index on platform admins
CREATE INDEX IF NOT EXISTS idx_users_platform_admin
ON users(is_platform_admin) WHERE is_platform_admin = true;

COMMENT ON COLUMN users.default_organization_id IS 'Default organization for user (nullable - platform admins may have none)';
COMMENT ON COLUMN users.is_platform_admin IS 'Platform administrator flag (can manage entire system)';

-- ============================================================
-- STEP 4: Create organization_members table
-- ============================================================

CREATE TABLE IF NOT EXISTS organization_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Organization-scoped role
    role TEXT NOT NULL DEFAULT 'member',

    -- Metadata
    joined_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Ensure user can only be member once per org
    UNIQUE(user_id, organization_id)
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_org_members_user
ON organization_members(user_id);

CREATE INDEX IF NOT EXISTS idx_org_members_org
ON organization_members(organization_id);

CREATE INDEX IF NOT EXISTS idx_org_members_role
ON organization_members(organization_id, role);

COMMENT ON TABLE organization_members IS 'Many-to-many relationship between users and organizations';
COMMENT ON COLUMN organization_members.role IS 'Organization-scoped role: admin, member, viewer, etc.';

-- Migrate existing users to organization_members
INSERT INTO organization_members (user_id, organization_id, role)
SELECT
    u.id,
    u.default_organization_id,
    CASE
        WHEN EXISTS (
            SELECT 1 FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND r.name = 'admin'
        ) THEN 'admin'
        ELSE 'member'
    END as role
FROM users u
WHERE u.default_organization_id IS NOT NULL
ON CONFLICT (user_id, organization_id) DO NOTHING;

-- ============================================================
-- STEP 5: Create connections table
-- ============================================================

CREATE TABLE IF NOT EXISTS connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Connection name
    name TEXT NOT NULL,

    -- Connection type
    type TEXT NOT NULL CHECK (type IN ('database', 'oidc', 'saml', 'oauth2')),

    -- Scope: platform-level or organization-level
    scope TEXT NOT NULL CHECK (scope IN ('platform', 'organization')),

    -- For org-level connections, reference the organization
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,

    -- Configuration (OIDC settings, SAML metadata, etc.)
    config JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Status
    is_enabled BOOLEAN DEFAULT true NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Constraint: platform connections have no org, org connections must have org
    CONSTRAINT connections_scope_org_check CHECK (
        (scope = 'platform' AND organization_id IS NULL) OR
        (scope = 'organization' AND organization_id IS NOT NULL)
    )
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_connections_scope
ON connections(scope);

CREATE INDEX IF NOT EXISTS idx_connections_org
ON connections(organization_id) WHERE organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_connections_enabled
ON connections(is_enabled) WHERE is_enabled = true;

CREATE INDEX IF NOT EXISTS idx_connections_type
ON connections(type);

COMMENT ON TABLE connections IS 'Authentication connections (database, OIDC, SAML) - can be platform or org-level';
COMMENT ON COLUMN connections.scope IS 'platform = available to all, organization = specific org only';
COMMENT ON COLUMN connections.config IS 'Connection-specific configuration (issuer, client_id, endpoints, etc.)';

-- Migrate existing oidc_providers to connections
INSERT INTO connections (name, type, scope, organization_id, config, is_enabled)
SELECT
    op.name,
    'oidc',
    'organization',
    op.tenant_id,
    jsonb_build_object(
        'issuer', op.issuer,
        'client_id', op.client_id,
        'client_secret', op.client_secret,
        'authorization_endpoint', op.authorization_endpoint,
        'token_endpoint', op.token_endpoint,
        'userinfo_endpoint', op.userinfo_endpoint,
        'jwks_uri', op.jwks_uri,
        'scopes', op.scopes,
        'claim_mappings', op.claim_mappings
    ),
    op.is_active
FROM oidc_providers op
WHERE NOT EXISTS (
    SELECT 1 FROM connections c
    WHERE c.organization_id = op.tenant_id
    AND c.name = op.name
);

-- Create default database connection (platform-level)
INSERT INTO connections (name, type, scope, organization_id, config, is_enabled)
VALUES (
    'Username-Password',
    'database',
    'platform',
    NULL,
    '{
        "password_policy": {
            "min_length": 8,
            "require_uppercase": false,
            "require_lowercase": false,
            "require_number": false,
            "require_special": false
        }
    }'::jsonb,
    true
)
ON CONFLICT DO NOTHING;

-- ============================================================
-- STEP 6: Create applications table
-- ============================================================

CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Application details
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,

    -- Application type
    type TEXT NOT NULL CHECK (type IN ('web', 'spa', 'native', 'api')),

    -- OAuth/OIDC credentials
    client_id TEXT UNIQUE NOT NULL,
    client_secret TEXT,  -- NULL for public clients (SPAs, native apps)

    -- URLs
    callback_urls JSONB DEFAULT '[]'::jsonb,
    logout_urls JSONB DEFAULT '[]'::jsonb,
    web_origins JSONB DEFAULT '[]'::jsonb,

    -- Allowed connections (array of connection IDs)
    allowed_connections JSONB DEFAULT '[]'::jsonb,

    -- Access control
    require_organization BOOLEAN DEFAULT false NOT NULL,
    platform_admin_only BOOLEAN DEFAULT false NOT NULL,

    -- Token settings
    access_token_lifetime_seconds INTEGER DEFAULT 3600 NOT NULL,
    refresh_token_lifetime_seconds INTEGER DEFAULT 2592000 NOT NULL,  -- 30 days

    -- Status
    is_enabled BOOLEAN DEFAULT true NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_applications_slug
ON applications(slug);

CREATE INDEX IF NOT EXISTS idx_applications_client_id
ON applications(client_id);

CREATE INDEX IF NOT EXISTS idx_applications_enabled
ON applications(is_enabled) WHERE is_enabled = true;

COMMENT ON TABLE applications IS 'OAuth/OIDC applications (different apps with different auth flows)';
COMMENT ON COLUMN applications.type IS 'web = server-side, spa = browser, native = mobile, api = machine-to-machine';
COMMENT ON COLUMN applications.require_organization IS 'If true, user must select organization to login';
COMMENT ON COLUMN applications.platform_admin_only IS 'If true, only platform admins can access';
COMMENT ON COLUMN applications.allowed_connections IS 'Array of connection IDs that this app can use';

-- Create default applications
DO $$
DECLARE
    db_connection_id UUID;
BEGIN
    -- Get the database connection ID
    SELECT id INTO db_connection_id
    FROM connections
    WHERE scope = 'platform' AND type = 'database'
    LIMIT 1;

    -- Admin Portal (platform admins only)
    INSERT INTO applications (
        name, slug, type, client_id, client_secret,
        platform_admin_only, require_organization,
        callback_urls, logout_urls,
        allowed_connections
    )
    VALUES (
        'Admin Portal',
        'admin-portal',
        'web',
        gen_random_uuid()::text,
        encode(gen_random_bytes(32), 'hex'),
        true,   -- platform_admin_only
        false,  -- require_organization
        '["http://localhost:3000/callback", "http://localhost:3001/callback"]'::jsonb,
        '["http://localhost:3000", "http://localhost:3001"]'::jsonb,
        jsonb_build_array(db_connection_id)
    )
    ON CONFLICT (slug) DO NOTHING;

    -- Customer Application (org members)
    INSERT INTO applications (
        name, slug, type, client_id,
        platform_admin_only, require_organization,
        callback_urls, logout_urls,
        allowed_connections
    )
    VALUES (
        'Customer Application',
        'customer-app',
        'spa',
        gen_random_uuid()::text,
        false,  -- platform_admin_only
        true,   -- require_organization
        '["http://localhost:3000/callback", "http://localhost:3001/callback"]'::jsonb,
        '["http://localhost:3000", "http://localhost:3001"]'::jsonb,
        jsonb_build_array(db_connection_id)
    )
    ON CONFLICT (slug) DO NOTHING;

    -- API Application (machine-to-machine)
    INSERT INTO applications (
        name, slug, type, client_id, client_secret,
        platform_admin_only, require_organization,
        allowed_connections
    )
    VALUES (
        'Platform API',
        'platform-api',
        'api',
        gen_random_uuid()::text,
        encode(gen_random_bytes(32), 'hex'),
        false,  -- platform_admin_only
        false,  -- require_organization
        jsonb_build_array(db_connection_id)
    )
    ON CONFLICT (slug) DO NOTHING;
END $$;

-- ============================================================
-- STEP 7: Update audit logs for new model
-- ============================================================

-- Add organization_id to audit_logs for org-scoped events
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Create index
CREATE INDEX IF NOT EXISTS idx_audit_logs_org
ON audit_logs(organization_id) WHERE organization_id IS NOT NULL;

COMMENT ON COLUMN audit_logs.organization_id IS 'Organization context for org-scoped events (NULL for platform events)';

-- ============================================================
-- STEP 8: Update authorization tuples for org context
-- ============================================================

-- Add organization_id to relation_tuples table for org-scoped permissions
ALTER TABLE relation_tuples ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Create index
CREATE INDEX IF NOT EXISTS idx_relation_tuples_org
ON relation_tuples(organization_id) WHERE organization_id IS NOT NULL;

-- Update existing tuples to have organization context
UPDATE relation_tuples t
SET organization_id = t.tenant_id
WHERE organization_id IS NULL AND tenant_id IS NOT NULL;

COMMENT ON COLUMN relation_tuples.organization_id IS 'Organization context for org-scoped permissions (same as tenant_id)';

-- ============================================================
-- STEP 9: Add updated_at triggers
-- ============================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add trigger to connections
DROP TRIGGER IF EXISTS update_connections_updated_at ON connections;
CREATE TRIGGER update_connections_updated_at
    BEFORE UPDATE ON connections
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add trigger to applications
DROP TRIGGER IF EXISTS update_applications_updated_at ON applications;
CREATE TRIGGER update_applications_updated_at
    BEFORE UPDATE ON applications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- STEP 10: Create views for backward compatibility
-- ============================================================

-- View: tenant-like view of organizations
CREATE OR REPLACE VIEW tenants AS
SELECT
    id,
    slug,
    name,
    isolation_mode,
    custom_domain,
    settings,
    created_at,
    updated_at
FROM organizations;

COMMENT ON VIEW tenants IS 'Backward compatibility view - maps to organizations table';

-- ============================================================
-- Summary
-- ============================================================

DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Hierarchical Tenant Model Migration Complete!';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Created:';
    RAISE NOTICE '  ✓ platform_config table';
    RAISE NOTICE '  ✓ organizations table (renamed from tenants)';
    RAISE NOTICE '  ✓ organization_members table';
    RAISE NOTICE '  ✓ connections table';
    RAISE NOTICE '  ✓ applications table';
    RAISE NOTICE '';
    RAISE NOTICE 'Updated:';
    RAISE NOTICE '  ✓ users table (global pool with optional org)';
    RAISE NOTICE '  ✓ audit_logs table (org context)';
    RAISE NOTICE '  ✓ tuples table (org context)';
    RAISE NOTICE '';
    RAISE NOTICE 'Migrated:';
    RAISE NOTICE '  ✓ Existing users → organization_members';
    RAISE NOTICE '  ✓ Existing oidc_providers → connections';
    RAISE NOTICE '  ✓ Created default applications';
    RAISE NOTICE '';
    RAISE NOTICE 'Next Steps:';
    RAISE NOTICE '  1. Update application models';
    RAISE NOTICE '  2. Update authentication flow';
    RAISE NOTICE '  3. Test platform admin login';
    RAISE NOTICE '  4. Test org member login with SSO';
    RAISE NOTICE '========================================';
END $$;
