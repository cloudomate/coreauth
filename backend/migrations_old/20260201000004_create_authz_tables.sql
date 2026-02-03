-- Create enum types for authorization
DO $$ BEGIN
    CREATE TYPE application_type AS ENUM ('service', 'webapp', 'spa', 'native');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE subject_type AS ENUM ('user', 'application', 'group', 'userset');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Applications (Service Principals)
-- NOTE: Applications table is created in migration 006 (hierarchical model)
-- This section is commented out to avoid conflicts
-- The hierarchical model creates applications with proper OAuth/OIDC structure

-- Relation Tuples (Zanzibar-style)
CREATE TABLE relation_tuples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    namespace VARCHAR(100) NOT NULL,      -- e.g., "document", "folder", "organization"
    object_id VARCHAR(255) NOT NULL,      -- e.g., "doc123"
    relation VARCHAR(100) NOT NULL,       -- e.g., "viewer", "editor", "owner"
    subject_type subject_type NOT NULL,
    subject_id VARCHAR(255) NOT NULL,     -- e.g., "alice", "app-service-1"
    subject_relation VARCHAR(100),        -- For userset subjects, e.g., "member"
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Unique constraint to prevent duplicate tuples
CREATE UNIQUE INDEX idx_tuples_unique ON relation_tuples(
    tenant_id, namespace, object_id, relation, subject_type, subject_id,
    COALESCE(subject_relation, '')
);

-- Indexes for efficient tuple lookups
CREATE INDEX idx_tuples_lookup ON relation_tuples(tenant_id, namespace, object_id, relation);
CREATE INDEX idx_tuples_subject ON relation_tuples(tenant_id, subject_type, subject_id);
CREATE INDEX idx_tuples_object ON relation_tuples(tenant_id, namespace, object_id);
CREATE INDEX idx_tuples_ns_rel ON relation_tuples(tenant_id, namespace, relation);

-- Application Access Tokens (for service-to-service auth)
-- NOTE: Commented out because it depends on applications table created in migration 006
-- Will be added in a future migration after applications table exists

-- Comments
COMMENT ON TABLE relation_tuples IS 'Zanzibar-style relation tuples for fine-grained authorization (ReBAC)';

COMMENT ON COLUMN relation_tuples.namespace IS 'Resource type (e.g., document, folder)';
COMMENT ON COLUMN relation_tuples.object_id IS 'Specific resource instance ID';
COMMENT ON COLUMN relation_tuples.relation IS 'Permission or role (e.g., viewer, editor, owner)';
COMMENT ON COLUMN relation_tuples.subject_type IS 'Type of entity that has the permission';
COMMENT ON COLUMN relation_tuples.subject_id IS 'ID of the subject entity';
COMMENT ON COLUMN relation_tuples.subject_relation IS 'Used for computed usersets (e.g., group#member)';
