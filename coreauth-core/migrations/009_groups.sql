-- ============================================================
-- Groups: User groups for organizing users within tenants
-- Each tenant at any hierarchy level can have groups
-- ============================================================

-- Groups table
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,

    -- Group type: 'standard' for regular groups, 'system' for built-in groups
    group_type VARCHAR(50) DEFAULT 'standard',

    -- Optional: link to a role for automatic role assignment
    default_role_id UUID REFERENCES roles(id) ON DELETE SET NULL,

    -- Metadata
    metadata JSONB DEFAULT '{}',

    -- Flags
    is_active BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Each tenant can only have one group with the same slug
    UNIQUE(tenant_id, slug)
);

CREATE INDEX idx_groups_tenant ON groups(tenant_id);
CREATE INDEX idx_groups_slug ON groups(tenant_id, slug);
CREATE INDEX idx_groups_active ON groups(tenant_id) WHERE is_active = true;

CREATE TRIGGER groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Group members table
CREATE TABLE group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Role within the group (e.g., 'member', 'admin', 'owner')
    role VARCHAR(50) DEFAULT 'member',

    -- When the membership was added
    added_at TIMESTAMPTZ DEFAULT NOW(),

    -- Who added this member (NULL = system)
    added_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Membership can expire
    expires_at TIMESTAMPTZ,

    UNIQUE(group_id, user_id)
);

CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_group_members_expires ON group_members(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================
-- Group Roles: Assign roles to entire groups
-- When a user is in a group, they inherit the group's roles
-- ============================================================

CREATE TABLE group_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(group_id, role_id)
);

CREATE INDEX idx_group_roles_group ON group_roles(group_id);
CREATE INDEX idx_group_roles_role ON group_roles(role_id);

-- ============================================================
-- Helper function: Get all roles for a user including group roles
-- ============================================================

CREATE OR REPLACE FUNCTION get_user_roles_with_groups(p_user_id UUID, p_tenant_id UUID)
RETURNS TABLE(role_id UUID, role_name VARCHAR, source VARCHAR) AS $$
BEGIN
    RETURN QUERY
    -- Direct user roles
    SELECT r.id, r.name, 'direct'::VARCHAR
    FROM user_roles ur
    JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = p_user_id AND r.tenant_id = p_tenant_id

    UNION

    -- Roles inherited from groups
    SELECT r.id, r.name, 'group'::VARCHAR
    FROM group_members gm
    JOIN groups g ON g.id = gm.group_id
    JOIN group_roles gr ON gr.group_id = g.id
    JOIN roles r ON r.id = gr.role_id
    WHERE gm.user_id = p_user_id
      AND g.tenant_id = p_tenant_id
      AND g.is_active = true
      AND (gm.expires_at IS NULL OR gm.expires_at > NOW());
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Helper function: Check if user is member of a group
-- ============================================================

CREATE OR REPLACE FUNCTION is_group_member(p_user_id UUID, p_group_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM group_members gm
        JOIN groups g ON g.id = gm.group_id
        WHERE gm.user_id = p_user_id
          AND gm.group_id = p_group_id
          AND g.is_active = true
          AND (gm.expires_at IS NULL OR gm.expires_at > NOW())
    );
END;
$$ LANGUAGE plpgsql;
