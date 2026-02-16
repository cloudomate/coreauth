-- ============================================================
-- CoreAuth CIAM - Applications & Connections
-- ============================================================
-- OAuth clients, SSO connections, OIDC providers
-- ============================================================

-- ============================================================
-- Applications (OAuth Clients)
-- ============================================================

CREATE TABLE applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    -- Identity
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    logo_url TEXT,
    app_type application_type NOT NULL DEFAULT 'webapp',

    -- OAuth2 credentials
    client_id VARCHAR(255) UNIQUE NOT NULL,
    client_secret_hash VARCHAR(255),

    -- OAuth2 URLs
    callback_urls TEXT[] DEFAULT '{}',
    allowed_logout_urls TEXT[] DEFAULT '{}',
    allowed_web_origins TEXT[] DEFAULT '{}',

    -- OAuth2 settings
    grant_types TEXT[] DEFAULT ARRAY['authorization_code', 'refresh_token'],
    response_types TEXT[] DEFAULT ARRAY['code'],
    allowed_scopes TEXT[] DEFAULT ARRAY['openid', 'profile', 'email'],
    token_endpoint_auth_method VARCHAR(50) DEFAULT 'client_secret_post',

    -- Token TTLs
    access_token_ttl_seconds INTEGER DEFAULT 3600,
    refresh_token_ttl_seconds INTEGER DEFAULT 2592000,
    id_token_ttl_seconds INTEGER DEFAULT 3600,

    -- Flags
    is_active BOOLEAN DEFAULT true,
    is_first_party BOOLEAN DEFAULT false,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_applications_slug_tenant ON applications(tenant_id, slug) WHERE tenant_id IS NOT NULL;
CREATE UNIQUE INDEX idx_applications_slug_global ON applications(slug) WHERE tenant_id IS NULL;
CREATE INDEX idx_applications_tenant ON applications(tenant_id);
CREATE INDEX idx_applications_client_id ON applications(client_id);
CREATE INDEX idx_applications_allowed_scopes ON applications USING GIN(allowed_scopes);

CREATE TRIGGER applications_updated_at
    BEFORE UPDATE ON applications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Connections (SSO/Identity Providers)
-- ============================================================

CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('oidc', 'saml', 'social', 'database', 'passwordless')),
    scope VARCHAR(50) DEFAULT 'organization' CHECK (scope IN ('platform', 'organization')),

    -- Provider configuration
    config JSONB NOT NULL DEFAULT '{}',

    -- Status
    is_enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT connections_scope_tenant_check CHECK (
        (scope = 'platform' AND tenant_id IS NULL) OR
        (scope = 'organization' AND tenant_id IS NOT NULL)
    )
);

CREATE INDEX idx_connections_tenant ON connections(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_connections_type ON connections(type);

CREATE TRIGGER connections_updated_at
    BEFORE UPDATE ON connections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- OIDC Providers (Enterprise SSO)
-- ============================================================

CREATE TABLE oidc_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    connection_id UUID REFERENCES connections(id) ON DELETE SET NULL,

    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100),
    provider_type VARCHAR(50) NOT NULL DEFAULT 'custom',

    -- OIDC Configuration
    issuer TEXT NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    client_secret VARCHAR(255) NOT NULL,
    authorization_endpoint TEXT NOT NULL DEFAULT '',
    token_endpoint TEXT NOT NULL DEFAULT '',
    userinfo_endpoint TEXT,
    jwks_uri TEXT NOT NULL DEFAULT '',
    discovery_url TEXT,

    -- Scopes
    scopes TEXT[] DEFAULT ARRAY['openid', 'profile', 'email'],

    -- Claim mappings
    claim_mappings JSONB DEFAULT '{}',
    groups_claim VARCHAR(100) DEFAULT 'groups',
    group_role_mappings JSONB DEFAULT '{}',
    allowed_group_id VARCHAR(255),

    -- Flags
    is_enabled BOOLEAN DEFAULT true,
    auto_create_users BOOLEAN DEFAULT true,
    sync_user_profile BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, slug)
);

CREATE INDEX idx_oidc_providers_tenant ON oidc_providers(tenant_id);
CREATE INDEX idx_oidc_providers_slug ON oidc_providers(tenant_id, slug);

CREATE TRIGGER oidc_providers_updated_at
    BEFORE UPDATE ON oidc_providers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Social Connection Templates
-- ============================================================

CREATE TABLE social_connection_templates (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    logo_url TEXT,
    documentation_url TEXT,
    default_scopes TEXT[] NOT NULL,
    required_fields TEXT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Pre-populate social provider templates
INSERT INTO social_connection_templates (id, name, logo_url, default_scopes, required_fields) VALUES
    ('google', 'Google', 'https://www.google.com/favicon.ico', ARRAY['openid', 'email', 'profile'], ARRAY['client_id', 'client_secret']),
    ('github', 'GitHub', 'https://github.com/favicon.ico', ARRAY['user:email', 'read:user'], ARRAY['client_id', 'client_secret']),
    ('microsoft', 'Microsoft', 'https://www.microsoft.com/favicon.ico', ARRAY['openid', 'email', 'profile'], ARRAY['client_id', 'client_secret', 'tenant_id']),
    ('facebook', 'Facebook', 'https://www.facebook.com/favicon.ico', ARRAY['email', 'public_profile'], ARRAY['client_id', 'client_secret']),
    ('apple', 'Apple', 'https://www.apple.com/favicon.ico', ARRAY['name', 'email'], ARRAY['client_id', 'team_id', 'key_id', 'private_key']),
    ('linkedin', 'LinkedIn', 'https://www.linkedin.com/favicon.ico', ARRAY['r_emailaddress', 'r_liteprofile'], ARRAY['client_id', 'client_secret'])
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- Application-Connection Mappings
-- ============================================================

CREATE TABLE application_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES connections(id) ON DELETE CASCADE,

    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(application_id, connection_id)
);

CREATE INDEX idx_app_connections_app ON application_connections(application_id);
CREATE INDEX idx_app_connections_conn ON application_connections(connection_id);
