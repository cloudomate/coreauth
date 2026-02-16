-- ============================================================
-- CoreAuth CIAM - Authorization (RBAC + ReBAC)
-- ============================================================
-- Roles, permissions, relation tuples (Zanzibar-style)
-- ============================================================

-- ============================================================
-- Roles (Per-Tenant)
-- ============================================================

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT false,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_roles_tenant ON roles(tenant_id);

CREATE TRIGGER roles_updated_at
    BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Permissions (Per-Tenant)
-- ============================================================

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,
    description TEXT,
    resource VARCHAR(100),
    action VARCHAR(50),

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_permissions_tenant ON permissions(tenant_id);
CREATE INDEX idx_permissions_resource ON permissions(tenant_id, resource, action);

-- ============================================================
-- Role-Permission Mappings
-- ============================================================

CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission ON role_permissions(permission_id);

-- ============================================================
-- User-Role Assignments
-- ============================================================

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,

    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);

-- ============================================================
-- Relation Tuples (ReBAC / Zanzibar-style)
-- ============================================================

CREATE TABLE relation_tuples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sub_tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    store_id UUID, -- FK added in 007_fga.sql

    -- Object (resource)
    object_type VARCHAR(100) NOT NULL,
    object_id VARCHAR(255) NOT NULL,

    -- Relation
    relation VARCHAR(100) NOT NULL,

    -- Subject (who has the relation)
    subject_type subject_type NOT NULL,
    subject_id VARCHAR(255) NOT NULL,
    subject_relation VARCHAR(100),

    -- Conditions
    condition JSONB,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, object_type, object_id, relation, subject_type, subject_id, subject_relation)
);

CREATE INDEX idx_relation_tuples_tenant ON relation_tuples(tenant_id);
CREATE INDEX idx_relation_tuples_sub_tenant ON relation_tuples(sub_tenant_id) WHERE sub_tenant_id IS NOT NULL;
CREATE INDEX idx_relation_tuples_object ON relation_tuples(tenant_id, object_type, object_id);
CREATE INDEX idx_relation_tuples_subject ON relation_tuples(tenant_id, subject_type, subject_id);
CREATE INDEX idx_relation_tuples_relation ON relation_tuples(tenant_id, object_type, relation);

CREATE TRIGGER relation_tuples_updated_at
    BEFORE UPDATE ON relation_tuples
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Resource Definitions (for ReBAC)
-- ============================================================

CREATE TABLE resource_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    type_name VARCHAR(100) NOT NULL,
    display_name VARCHAR(255),
    description TEXT,

    relations JSONB DEFAULT '{}',
    permissions JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, type_name)
);

CREATE INDEX idx_resource_definitions_tenant ON resource_definitions(tenant_id);

CREATE TRIGGER resource_definitions_updated_at
    BEFORE UPDATE ON resource_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
