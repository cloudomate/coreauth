-- ============================================================
-- CoreAuth CIAM - Fine-Grained Authorization (FGA)
-- ============================================================
-- OpenFGA-compatible authorization stores, models, and checks
-- ============================================================

-- ============================================================
-- FGA Stores (Authorization Model Containers)
-- ============================================================

CREATE TABLE fga_stores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Current active model version
    current_model_version INTEGER DEFAULT 0,

    -- API access
    api_key_hash VARCHAR(255),
    api_key_prefix VARCHAR(20),

    is_active BOOLEAN DEFAULT true,

    -- Statistics
    tuple_count BIGINT DEFAULT 0,

    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_fga_stores_tenant ON fga_stores(tenant_id);
CREATE INDEX idx_fga_stores_active ON fga_stores(is_active) WHERE is_active = true;
CREATE INDEX idx_fga_stores_api_key ON fga_stores(api_key_prefix);

CREATE TRIGGER fga_stores_updated_at
    BEFORE UPDATE ON fga_stores
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- FGA Authorization Models
-- ============================================================

CREATE TABLE fga_authorization_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,

    -- Model definition (OpenFGA-compatible JSON)
    schema_json JSONB NOT NULL,
    schema_dsl TEXT,

    is_valid BOOLEAN DEFAULT true,
    validation_errors JSONB,

    created_by VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(store_id, version)
);

CREATE INDEX idx_fga_models_store ON fga_authorization_models(store_id);
CREATE INDEX idx_fga_models_version ON fga_authorization_models(store_id, version DESC);

-- ============================================================
-- FGA Type Definitions (Parsed from Model)
-- ============================================================

CREATE TABLE fga_type_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,
    model_version INTEGER NOT NULL,

    type_name VARCHAR(100) NOT NULL,
    relations JSONB NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(store_id, model_version, type_name)
);

CREATE INDEX idx_fga_types_store ON fga_type_definitions(store_id, model_version);

-- ============================================================
-- FGA Store API Keys
-- ============================================================

CREATE TABLE fga_store_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(20) NOT NULL,

    permissions TEXT[] DEFAULT ARRAY['read', 'write', 'check'],
    rate_limit_per_minute INTEGER DEFAULT 1000,

    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_fga_api_keys_store ON fga_store_api_keys(store_id);
CREATE INDEX idx_fga_api_keys_prefix ON fga_store_api_keys(key_prefix);

CREATE TRIGGER fga_api_keys_updated_at
    BEFORE UPDATE ON fga_store_api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- FGA Check Logs (Audit Trail)
-- ============================================================

CREATE TABLE fga_check_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES fga_stores(id) ON DELETE CASCADE,

    -- Request
    subject_type VARCHAR(100) NOT NULL,
    subject_id VARCHAR(255) NOT NULL,
    relation VARCHAR(100) NOT NULL,
    object_type VARCHAR(100) NOT NULL,
    object_id VARCHAR(255) NOT NULL,
    context JSONB,

    -- Result
    allowed BOOLEAN NOT NULL,
    resolution_path JSONB,

    -- Performance
    latency_ms INTEGER,
    cache_hit BOOLEAN DEFAULT false,

    -- Source
    api_key_id UUID REFERENCES fga_store_api_keys(id) ON DELETE SET NULL,
    ip_address TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_fga_check_logs_store ON fga_check_logs(store_id, created_at DESC);
CREATE INDEX idx_fga_check_logs_subject ON fga_check_logs(store_id, subject_type, subject_id);

-- ============================================================
-- Add store_id FK to relation_tuples (column defined in 003)
-- ============================================================

ALTER TABLE relation_tuples
ADD CONSTRAINT fk_relation_tuples_store
FOREIGN KEY (store_id) REFERENCES fga_stores(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_relation_tuples_store ON relation_tuples(store_id) WHERE store_id IS NOT NULL;

-- ============================================================
-- FGA Helper Functions
-- ============================================================

-- Get current model for a store
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

-- Validate tuple against model
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
    SELECT relations INTO v_type_def
    FROM fga_type_definitions td
    JOIN fga_stores s ON s.id = td.store_id
    WHERE td.store_id = p_store_id
      AND td.model_version = s.current_model_version
      AND td.type_name = p_object_type;

    IF v_type_def IS NULL THEN
        RETURN false;
    END IF;

    v_relation_def := v_type_def -> p_relation;
    IF v_relation_def IS NULL THEN
        RETURN false;
    END IF;

    RETURN true;
END;
$$ LANGUAGE plpgsql;

-- Update tuple count on store
CREATE OR REPLACE FUNCTION update_fga_store_tuple_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.store_id IS NOT NULL THEN
        UPDATE fga_stores SET tuple_count = tuple_count + 1, updated_at = NOW()
        WHERE id = NEW.store_id;
    ELSIF TG_OP = 'DELETE' AND OLD.store_id IS NOT NULL THEN
        UPDATE fga_stores SET tuple_count = tuple_count - 1, updated_at = NOW()
        WHERE id = OLD.store_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_tuple_count
    AFTER INSERT OR DELETE ON relation_tuples
    FOR EACH ROW EXECUTE FUNCTION update_fga_store_tuple_count();
