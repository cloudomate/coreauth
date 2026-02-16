-- ============================================================
-- CoreAuth CIAM - Tenant Database Isolation
-- ============================================================
-- Tenant registry for database routing (shared vs dedicated)
-- ============================================================

-- ============================================================
-- Tenant Registry (Database Routing)
-- ============================================================

CREATE TABLE tenant_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,

    -- Database isolation configuration
    isolation_mode VARCHAR(20) NOT NULL DEFAULT 'shared'
        CHECK (isolation_mode IN ('shared', 'dedicated')),

    -- Dedicated database connection info
    database_host VARCHAR(255),
    database_port INTEGER DEFAULT 5432,
    database_name VARCHAR(255),
    database_user VARCHAR(255),
    database_password_encrypted TEXT,

    -- Connection pool settings
    pool_min_connections INTEGER DEFAULT 1,
    pool_max_connections INTEGER DEFAULT 10,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'provisioning'
        CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted')),
    provisioned_at TIMESTAMPTZ,

    -- Billing link
    stripe_customer_id VARCHAR(255),
    subscription_plan VARCHAR(50) DEFAULT 'free',

    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_tenant_registry_slug ON tenant_registry(slug);
CREATE INDEX idx_tenant_registry_status ON tenant_registry(status);

CREATE TRIGGER tenant_registry_updated_at
    BEFORE UPDATE ON tenant_registry
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

COMMENT ON TABLE tenant_registry IS 'Platform-level tenant registry for database routing. Each entry represents a CoreAuth customer.';
COMMENT ON COLUMN tenant_registry.isolation_mode IS 'shared = uses master database, dedicated = uses separate database';
COMMENT ON COLUMN tenant_registry.database_password_encrypted IS 'AES-256-GCM encrypted database password';

-- ============================================================
-- Tenant Admins (Platform-level)
-- ============================================================

CREATE TABLE tenant_admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant_registry(id) ON DELETE CASCADE,

    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255),

    role VARCHAR(20) NOT NULL DEFAULT 'admin'
        CHECK (role IN ('owner', 'admin', 'viewer')),

    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_tenant_admins_tenant ON tenant_admins(tenant_id);
CREATE INDEX idx_tenant_admins_email ON tenant_admins(email);

CREATE TRIGGER tenant_admins_updated_at
    BEFORE UPDATE ON tenant_admins
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Tenant Activity Log (Platform-level)
-- ============================================================

CREATE TABLE tenant_activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant_registry(id) ON DELETE CASCADE,
    admin_id UUID REFERENCES tenant_admins(id) ON DELETE SET NULL,

    event_type VARCHAR(100) NOT NULL,
    description TEXT,
    metadata JSONB DEFAULT '{}',

    ip_address TEXT,
    user_agent TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_tenant_activity_tenant ON tenant_activity_logs(tenant_id);
CREATE INDEX idx_tenant_activity_created ON tenant_activity_logs(created_at DESC);
