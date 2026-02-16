-- ============================================================
-- CoreAuth CIAM - Core Schema
-- ============================================================
-- Foundation: Extensions, types, tenants, users, sessions
-- ============================================================

-- Extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================
-- Custom Types
-- ============================================================

CREATE TYPE application_type AS ENUM ('service', 'webapp', 'spa', 'native');

CREATE TYPE audit_event_category AS ENUM (
    'authentication', 'authorization', 'user_management',
    'tenant_management', 'security', 'admin', 'system'
);

CREATE TYPE subject_type AS ENUM ('user', 'application', 'group', 'userset');

-- ============================================================
-- Helper Functions
-- ============================================================

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Tenants (CoreAuth Customers)
-- ============================================================
-- hierarchy_level=0: Root tenant (CoreAuth customer)
-- hierarchy_level=1: Sub-organization (B2B customer of tenant)

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    account_type VARCHAR(20) DEFAULT 'business' CHECK (account_type IN ('personal', 'business')),
    isolation_mode VARCHAR(20) DEFAULT 'shared' CHECK (isolation_mode IN ('shared', 'dedicated')),

    -- Hierarchy (max 2 levels)
    parent_tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    hierarchy_level INTEGER DEFAULT 0 NOT NULL CHECK (hierarchy_level >= 0),
    hierarchy_path TEXT NOT NULL,

    -- Customization
    custom_domain VARCHAR(255),
    settings JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_parent ON tenants(parent_tenant_id) WHERE parent_tenant_id IS NOT NULL;
CREATE INDEX idx_tenants_hierarchy_path ON tenants(hierarchy_path text_pattern_ops);
CREATE INDEX idx_tenants_account_type ON tenants(account_type);

-- Hierarchy maintenance trigger
CREATE OR REPLACE FUNCTION update_tenant_hierarchy()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.parent_tenant_id IS NULL THEN
        NEW.hierarchy_level := 0;
        NEW.hierarchy_path := NEW.id::text;
    ELSE
        SELECT hierarchy_level + 1, hierarchy_path || '/' || NEW.id::text
        INTO NEW.hierarchy_level, NEW.hierarchy_path
        FROM tenants WHERE id = NEW.parent_tenant_id;

        IF NEW.hierarchy_level > 1 THEN
            RAISE EXCEPTION 'Maximum hierarchy depth of 2 levels exceeded';
        END IF;
        IF NEW.id = NEW.parent_tenant_id THEN
            RAISE EXCEPTION 'Tenant cannot be its own parent';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tenant_hierarchy_trigger
    BEFORE INSERT OR UPDATE OF parent_tenant_id ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_tenant_hierarchy();

CREATE TRIGGER tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

COMMENT ON TABLE tenants IS 'CoreAuth customer accounts. hierarchy_level=0 is root tenant, hierarchy_level=1 is sub-organization.';

-- ============================================================
-- Users (Global User Pool)
-- ============================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    default_tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,

    -- Credentials
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT false,
    phone VARCHAR(50),
    phone_verified BOOLEAN DEFAULT false,
    password_hash VARCHAR(255),

    -- Profile
    metadata JSONB DEFAULT '{}',

    -- Status
    is_active BOOLEAN DEFAULT true,
    is_platform_admin BOOLEAN DEFAULT false,

    -- MFA
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_enforced_at TIMESTAMPTZ,
    mfa_secret VARCHAR(255),
    mfa_backup_codes TEXT[],

    -- SCIM provisioning
    scim_external_id TEXT,
    scim_provisioned BOOLEAN DEFAULT false,
    scim_last_synced_at TIMESTAMPTZ,

    -- Timestamps
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_default_tenant ON users(default_tenant_id);
CREATE INDEX idx_users_scim_external_id ON users(scim_external_id) WHERE scim_external_id IS NOT NULL;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Tenant Members (User-Tenant Relationships)
-- ============================================================

CREATE TABLE tenant_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',
    joined_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, tenant_id)
);

CREATE INDEX idx_tenant_members_tenant ON tenant_members(tenant_id);
CREATE INDEX idx_tenant_members_role ON tenant_members(tenant_id, role);

-- ============================================================
-- Sessions (User Login Sessions)
-- ============================================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    token_hash VARCHAR(255) UNIQUE NOT NULL,
    refresh_token_hash VARCHAR(255) UNIQUE,

    ip_address TEXT,
    user_agent TEXT,
    device_info JSONB DEFAULT '{}',
    device_fingerprint VARCHAR(255),

    expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_tenant ON sessions(tenant_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

-- ============================================================
-- Audit Logs
-- ============================================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    sub_tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    event_type VARCHAR(100) NOT NULL,
    category audit_event_category NOT NULL,
    description TEXT,
    metadata JSONB DEFAULT '{}',

    ip_address TEXT,
    user_agent TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_tenant ON audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_sub_tenant ON audit_logs(sub_tenant_id) WHERE sub_tenant_id IS NOT NULL;
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
