-- OIDC Providers table for external identity providers (Azure Entra, Google, etc.)
CREATE TABLE oidc_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- 'azure', 'google', 'okta', 'custom'
    issuer TEXT NOT NULL,
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL, -- Should be encrypted in production
    authorization_endpoint TEXT NOT NULL,
    token_endpoint TEXT NOT NULL,
    userinfo_endpoint TEXT,
    jwks_uri TEXT NOT NULL,
    scopes VARCHAR(100)[] DEFAULT ARRAY['openid', 'profile', 'email'],
    claim_mappings JSONB NOT NULL DEFAULT '{
        "email": "email",
        "first_name": "given_name",
        "last_name": "family_name",
        "phone": "phone_number"
    }'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_oidc_providers_tenant ON oidc_providers(tenant_id);
CREATE INDEX idx_oidc_providers_type ON oidc_providers(provider_type);
CREATE INDEX idx_oidc_providers_active ON oidc_providers(is_active) WHERE is_active = true;

-- Trigger for updated_at
CREATE TRIGGER update_oidc_providers_updated_at
    BEFORE UPDATE ON oidc_providers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- Add provider_user_id to users table for linking external identities
ALTER TABLE users ADD COLUMN IF NOT EXISTS provider_user_id TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS provider_id UUID REFERENCES oidc_providers(id) ON DELETE SET NULL;

-- Index for provider lookups
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider_id, provider_user_id);
