-- SCIM 2.0 Provisioning Support Migration
-- Enables enterprise IdPs (Okta, Azure AD, OneLogin) to sync users automatically

-- ============================================================================
-- SCIM TOKENS TABLE
-- ============================================================================
-- Bearer tokens for SCIM API authentication (organization-scoped)

CREATE TABLE IF NOT EXISTS scim_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Token identification
    name TEXT NOT NULL,                   -- "Okta SCIM Token", "Azure AD Sync"
    token_hash TEXT NOT NULL UNIQUE,      -- SHA-256 hash of the actual token
    token_prefix TEXT NOT NULL,           -- First 8 chars for identification (scim_xxx...)

    -- Usage tracking
    last_used_at TIMESTAMPTZ,
    last_used_ip TEXT,
    request_count INTEGER DEFAULT 0,

    -- Expiration and status
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    revoked_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by UUID REFERENCES users(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_scim_tokens_organization_id ON scim_tokens(organization_id);
CREATE INDEX IF NOT EXISTS idx_scim_tokens_token_hash ON scim_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_scim_tokens_active ON scim_tokens(is_active) WHERE is_active = true;

-- ============================================================================
-- ADD SCIM FIELDS TO USERS TABLE
-- ============================================================================
-- Track external identity provider IDs for user mapping

ALTER TABLE users ADD COLUMN IF NOT EXISTS scim_external_id TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS scim_provisioned BOOLEAN DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS scim_last_synced_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_users_scim_external_id ON users(scim_external_id) WHERE scim_external_id IS NOT NULL;

-- ============================================================================
-- SCIM GROUPS TABLE
-- ============================================================================
-- SCIM groups map to roles within an organization

CREATE TABLE IF NOT EXISTS scim_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- SCIM identification
    display_name TEXT NOT NULL,
    external_id TEXT,                     -- ID from the identity provider

    -- Optional mapping to internal role
    role_id UUID REFERENCES roles(id) ON DELETE SET NULL,

    -- Metadata
    description TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Unique display name per organization
    UNIQUE(organization_id, display_name)
);

CREATE INDEX IF NOT EXISTS idx_scim_groups_organization_id ON scim_groups(organization_id);
CREATE INDEX IF NOT EXISTS idx_scim_groups_external_id ON scim_groups(external_id) WHERE external_id IS NOT NULL;

-- ============================================================================
-- SCIM GROUP MEMBERSHIPS TABLE
-- ============================================================================
-- Many-to-many relationship between users and SCIM groups

CREATE TABLE IF NOT EXISTS scim_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES scim_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- One membership per user per group
    UNIQUE(group_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_scim_group_members_group_id ON scim_group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_scim_group_members_user_id ON scim_group_members(user_id);

-- ============================================================================
-- SCIM PROVISIONING LOG TABLE
-- ============================================================================
-- Audit trail for SCIM operations

CREATE TABLE IF NOT EXISTS scim_provisioning_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token_id UUID REFERENCES scim_tokens(id) ON DELETE SET NULL,

    -- Operation details
    operation TEXT NOT NULL,              -- 'CREATE', 'UPDATE', 'DELETE', 'PATCH'
    resource_type TEXT NOT NULL,          -- 'User', 'Group'
    resource_id UUID,                     -- User or Group ID
    external_id TEXT,                     -- External ID if provided

    -- Request/Response
    request_path TEXT NOT NULL,
    request_method TEXT NOT NULL,
    request_body JSONB,
    response_status INTEGER NOT NULL,
    response_body JSONB,

    -- Error tracking
    error_message TEXT,

    -- Client info
    client_ip TEXT,
    user_agent TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    duration_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_scim_logs_organization_id ON scim_provisioning_logs(organization_id);
CREATE INDEX IF NOT EXISTS idx_scim_logs_resource ON scim_provisioning_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_scim_logs_created_at ON scim_provisioning_logs(created_at);

-- ============================================================================
-- SCIM CONFIGURATION TABLE
-- ============================================================================
-- Per-organization SCIM settings

CREATE TABLE IF NOT EXISTS scim_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,

    -- Feature flags
    is_enabled BOOLEAN DEFAULT false,
    auto_create_users BOOLEAN DEFAULT true,
    auto_update_users BOOLEAN DEFAULT true,
    auto_deactivate_users BOOLEAN DEFAULT true,   -- Deactivate instead of delete
    sync_groups BOOLEAN DEFAULT true,

    -- Attribute mapping
    attribute_mapping JSONB DEFAULT '{
        "userName": "email",
        "name.givenName": "first_name",
        "name.familyName": "last_name",
        "displayName": "display_name",
        "emails[primary].value": "email",
        "active": "is_active"
    }',

    -- Default role for provisioned users
    default_role TEXT DEFAULT 'member',

    -- Statistics
    total_users_provisioned INTEGER DEFAULT 0,
    total_groups_provisioned INTEGER DEFAULT 0,
    last_sync_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- ============================================================================
-- HELPER FUNCTION FOR SCIM SCHEMA
-- ============================================================================

-- Function to generate SCIM resource meta
CREATE OR REPLACE FUNCTION scim_resource_meta(
    resource_type TEXT,
    resource_id UUID,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    base_url TEXT DEFAULT ''
) RETURNS JSONB AS $$
BEGIN
    RETURN jsonb_build_object(
        'resourceType', resource_type,
        'created', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
        'lastModified', to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
        'location', base_url || '/scim/v2/' || resource_type || 's/' || resource_id,
        'version', 'W/"' || md5(updated_at::text) || '"'
    );
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE scim_tokens IS 'Bearer tokens for SCIM API authentication';
COMMENT ON TABLE scim_groups IS 'SCIM groups for enterprise user provisioning';
COMMENT ON TABLE scim_group_members IS 'User membership in SCIM groups';
COMMENT ON TABLE scim_provisioning_logs IS 'Audit trail for SCIM operations';
COMMENT ON TABLE scim_configurations IS 'Per-organization SCIM settings';

COMMENT ON COLUMN users.scim_external_id IS 'External ID from identity provider for SCIM mapping';
COMMENT ON COLUMN users.scim_provisioned IS 'Whether user was created via SCIM provisioning';
COMMENT ON COLUMN scim_tokens.token_hash IS 'SHA-256 hash of the bearer token';
COMMENT ON COLUMN scim_tokens.token_prefix IS 'First 8 characters for token identification';
