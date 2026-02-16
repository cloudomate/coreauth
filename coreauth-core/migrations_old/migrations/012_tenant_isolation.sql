-- ============================================================
-- Tenant Isolation & FGA Stores Migration
-- ============================================================
--
-- This migration adds support for:
-- 1. Dedicated database per tenant (your customers)
-- 2. FGA (Fine-Grained Authorization) stores with authorization models
-- 3. Tenant database routing configuration
--
-- Architecture:
--   Master DB: tenant_registry, billing, platform config
--   Tenant DB: users, sessions, fga_stores, relation_tuples, etc.
--
-- ============================================================

-- ============================================================
-- MASTER DATABASE TABLES (Platform-level)
-- ============================================================

-- Tenant Registry - tracks all your customers and their database locations
CREATE TABLE IF NOT EXISTS tenant_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT UNIQUE NOT NULL,                    -- e.g., "acme-corp"
    name TEXT NOT NULL,                           -- e.g., "Acme Corporation"

    -- Database isolation configuration
    isolation_mode TEXT NOT NULL DEFAULT 'dedicated', -- 'dedicated' or 'shared'
    database_host TEXT,                           -- e.g., "tenant-acme.db.example.com"
    database_port INTEGER DEFAULT 5432,
    database_name TEXT,                           -- e.g., "ciam_acme"
    database_user TEXT,                           -- encrypted credentials
    database_password_encrypted TEXT,             -- encrypted with platform key

    -- Connection pool settings
    pool_min_connections INTEGER DEFAULT 1,
    pool_max_connections INTEGER DEFAULT 10,

    -- Status
    status TEXT NOT NULL DEFAULT 'provisioning',  -- 'provisioning', 'active', 'suspended', 'deleted'
    provisioned_at TIMESTAMPTZ,

    -- Billing link
    stripe_customer_id TEXT,
    subscription_plan TEXT DEFAULT 'free',

    -- Metadata
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_registry_slug ON tenant_registry(slug);
CREATE INDEX IF NOT EXISTS idx_tenant_registry_status ON tenant_registry(status);

-- Tenant Admin Users (platform-level admins who manage tenants)
CREATE TABLE IF NOT EXISTS tenant_admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant_registry(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,                  -- Argon2id hashed
    name TEXT,
    role TEXT NOT NULL DEFAULT 'admin',           -- 'owner', 'admin', 'viewer'
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

CREATE INDEX IF NOT EXISTS idx_tenant_admins_tenant ON tenant_admins(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_admins_email ON tenant_admins(email);

-- ============================================================
-- FGA STORES (Lives in each Tenant's Database)
-- ============================================================

-- FGA Stores - containers for authorization models and tuples
-- Each tenant can have multiple stores (e.g., production, staging)
CREATE TABLE IF NOT EXISTS fga_stores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,                           -- e.g., "Production", "Development"
    description TEXT,

    -- Current active model version
    current_model_version INTEGER DEFAULT 0,

    -- API access
    api_key_hash TEXT,                            -- hashed API key for programmatic access
    api_key_prefix TEXT,                          -- first 8 chars for identification

    -- Status
    is_active BOOLEAN DEFAULT true,

    -- Stats (denormalized for quick access)
    tuple_count BIGINT DEFAULT 0,

    -- Metadata
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fga_stores_active ON fga_stores(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_fga_stores_api_key ON fga_stores(api_key_prefix);

-- Authorization Models - defines types, relations, and permissions
-- Follows OpenFGA/Zanzibar model format
CREATE TABLE IF NOT EXISTS fga_authorization_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,                     -- incremental version number

    -- Model definition (OpenFGA-compatible DSL stored as JSON)
    -- Example: {"type_definitions": [{"type": "document", "relations": {...}}]}
    schema_json JSONB NOT NULL,

    -- Human-readable DSL (optional, for display)
    schema_dsl TEXT,

    -- Validation status
    is_valid BOOLEAN DEFAULT true,
    validation_errors JSONB,

    -- Metadata
    created_by TEXT,                              -- user who created this version
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(store_id, version)
);

CREATE INDEX IF NOT EXISTS idx_fga_models_store ON fga_authorization_models(store_id);
CREATE INDEX IF NOT EXISTS idx_fga_models_version ON fga_authorization_models(store_id, version DESC);

-- Type Definitions (parsed from authorization model for quick validation)
CREATE TABLE IF NOT EXISTS fga_type_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,
    model_version INTEGER NOT NULL,

    type_name TEXT NOT NULL,                      -- e.g., "document", "folder", "user"
    relations JSONB NOT NULL,                     -- {"viewer": {...}, "editor": {...}}

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(store_id, model_version, type_name)
);

CREATE INDEX IF NOT EXISTS idx_fga_types_store ON fga_type_definitions(store_id, model_version);

-- Update relation_tuples to support FGA stores
-- Add store_id column if not exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'relation_tuples' AND column_name = 'store_id'
    ) THEN
        ALTER TABLE relation_tuples ADD COLUMN store_id UUID REFERENCES fga_stores(id) ON DELETE CASCADE;
        CREATE INDEX idx_relation_tuples_store ON relation_tuples(store_id);
    END IF;
END $$;

-- FGA Store API Keys (for application access)
CREATE TABLE IF NOT EXISTS fga_store_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,

    name TEXT NOT NULL,                           -- e.g., "Production API Key"
    key_hash TEXT NOT NULL,                       -- SHA256 hash of the key
    key_prefix TEXT NOT NULL,                     -- first 8 chars: "fga_prod"

    -- Permissions
    permissions TEXT[] DEFAULT ARRAY['read', 'write', 'check'],  -- 'read', 'write', 'check', 'admin'

    -- Rate limiting
    rate_limit_per_minute INTEGER DEFAULT 1000,

    -- Status
    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fga_api_keys_store ON fga_store_api_keys(store_id);
CREATE INDEX IF NOT EXISTS idx_fga_api_keys_prefix ON fga_store_api_keys(key_prefix);

-- ============================================================
-- AUDIT & METRICS
-- ============================================================

-- FGA Check Audit Log (for debugging and compliance)
CREATE TABLE IF NOT EXISTS fga_check_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,

    -- Request details
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    relation TEXT NOT NULL,
    object_type TEXT NOT NULL,
    object_id TEXT NOT NULL,
    context JSONB,

    -- Result
    allowed BOOLEAN NOT NULL,
    resolution_path JSONB,                        -- how permission was resolved

    -- Performance
    latency_ms INTEGER,
    cache_hit BOOLEAN DEFAULT false,

    -- Source
    api_key_id UUID REFERENCES fga_store_api_keys(id),
    ip_address TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Partition by month for performance
CREATE INDEX IF NOT EXISTS idx_fga_check_logs_store_time ON fga_check_logs(store_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fga_check_logs_subject ON fga_check_logs(store_id, subject_type, subject_id);

-- ============================================================
-- HELPER FUNCTIONS
-- ============================================================

-- Function to get current model for a store
CREATE OR REPLACE FUNCTION get_current_fga_model(p_store_id UUID)
RETURNS TABLE(schema_json JSONB, version INTEGER) AS $$
BEGIN
    RETURN QUERY
    SELECT am.schema_json, am.version
    FROM fga_authorization_models am
    JOIN fga_stores s ON s.id = am.store_id
    WHERE am.store_id = p_store_id
      AND am.version = s.current_model_version
      AND am.is_valid = true;
END;
$$ LANGUAGE plpgsql;

-- Function to validate relation against model
CREATE OR REPLACE FUNCTION validate_fga_tuple(
    p_store_id UUID,
    p_object_type TEXT,
    p_relation TEXT,
    p_subject_type TEXT
) RETURNS BOOLEAN AS $$
DECLARE
    v_type_def JSONB;
    v_relation_def JSONB;
BEGIN
    -- Get type definition
    SELECT relations INTO v_type_def
    FROM fga_type_definitions td
    JOIN fga_stores s ON s.id = td.store_id
    WHERE td.store_id = p_store_id
      AND td.model_version = s.current_model_version
      AND td.type_name = p_object_type;

    IF v_type_def IS NULL THEN
        RETURN false;  -- Unknown object type
    END IF;

    -- Check if relation exists
    v_relation_def := v_type_def -> p_relation;
    IF v_relation_def IS NULL THEN
        RETURN false;  -- Unknown relation
    END IF;

    -- TODO: Validate subject type against relation's allowed types
    RETURN true;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update tuple count
CREATE OR REPLACE FUNCTION update_fga_store_tuple_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fga_stores SET tuple_count = tuple_count + 1, updated_at = NOW()
        WHERE id = NEW.store_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fga_stores SET tuple_count = tuple_count - 1, updated_at = NOW()
        WHERE id = OLD.store_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Only create trigger if relation_tuples has store_id
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'relation_tuples' AND column_name = 'store_id'
    ) THEN
        DROP TRIGGER IF EXISTS trigger_update_tuple_count ON relation_tuples;
        CREATE TRIGGER trigger_update_tuple_count
        AFTER INSERT OR DELETE ON relation_tuples
        FOR EACH ROW EXECUTE FUNCTION update_fga_store_tuple_count();
    END IF;
END $$;
