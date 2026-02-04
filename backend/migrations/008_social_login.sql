-- Social Login Support Migration
-- Adds support for social identity providers (Google, GitHub, Microsoft, etc.)

-- ============================================================================
-- USER IDENTITIES TABLE
-- ============================================================================
-- Stores linked social/external identities for users
-- Allows users to link multiple social accounts to one CoreAuth account

CREATE TABLE IF NOT EXISTS user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Provider identification
    provider TEXT NOT NULL,           -- 'google', 'github', 'microsoft', etc.
    provider_user_id TEXT NOT NULL,   -- The user's ID from the provider

    -- Profile data from provider
    raw_profile JSONB DEFAULT '{}',   -- Full profile response from provider

    -- Tokens (optional, for refresh)
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_login_at TIMESTAMPTZ DEFAULT NOW(),

    -- Unique constraint: one provider identity per user
    UNIQUE(user_id, provider, provider_user_id),
    -- Index for looking up by provider identity
    UNIQUE(provider, provider_user_id)
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_identities_user_id ON user_identities(user_id);
CREATE INDEX IF NOT EXISTS idx_user_identities_provider ON user_identities(provider);

-- ============================================================================
-- UPDATE CONNECTIONS TABLE
-- ============================================================================
-- Add 'social' to allowed connection types

ALTER TABLE connections
    DROP CONSTRAINT IF EXISTS connections_type_check;

ALTER TABLE connections
    ADD CONSTRAINT connections_type_check
    CHECK (type = ANY (ARRAY['database'::text, 'oidc'::text, 'saml'::text, 'oauth2'::text, 'social'::text]));

-- ============================================================================
-- SOCIAL CONNECTION TEMPLATES
-- ============================================================================
-- Pre-configured templates for popular social providers

CREATE TABLE IF NOT EXISTS social_connection_templates (
    id TEXT PRIMARY KEY,              -- 'google', 'github', 'microsoft'
    name TEXT NOT NULL,
    logo_url TEXT,
    documentation_url TEXT,
    default_scopes TEXT[] NOT NULL,
    required_fields TEXT[] NOT NULL,  -- Fields needed in config
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Insert default templates
INSERT INTO social_connection_templates (id, name, logo_url, documentation_url, default_scopes, required_fields)
VALUES
    ('google', 'Google', 'https://www.google.com/favicon.ico',
     'https://developers.google.com/identity/protocols/oauth2',
     ARRAY['openid', 'email', 'profile'],
     ARRAY['client_id', 'client_secret']),

    ('github', 'GitHub', 'https://github.githubassets.com/favicons/favicon.svg',
     'https://docs.github.com/en/apps/oauth-apps',
     ARRAY['user:email', 'read:user'],
     ARRAY['client_id', 'client_secret']),

    ('microsoft', 'Microsoft', 'https://www.microsoft.com/favicon.ico',
     'https://learn.microsoft.com/en-us/entra/identity-platform/',
     ARRAY['openid', 'email', 'profile', 'User.Read'],
     ARRAY['client_id', 'client_secret']),

    ('facebook', 'Facebook', 'https://www.facebook.com/favicon.ico',
     'https://developers.facebook.com/docs/facebook-login/',
     ARRAY['email', 'public_profile'],
     ARRAY['client_id', 'client_secret']),

    ('apple', 'Apple', 'https://www.apple.com/favicon.ico',
     'https://developer.apple.com/sign-in-with-apple/',
     ARRAY['name', 'email'],
     ARRAY['client_id', 'client_secret', 'team_id', 'key_id']),

    ('linkedin', 'LinkedIn', 'https://www.linkedin.com/favicon.ico',
     'https://learn.microsoft.com/en-us/linkedin/shared/authentication/',
     ARRAY['openid', 'email', 'profile'],
     ARRAY['client_id', 'client_secret'])
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    logo_url = EXCLUDED.logo_url,
    documentation_url = EXCLUDED.documentation_url,
    default_scopes = EXCLUDED.default_scopes,
    required_fields = EXCLUDED.required_fields;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE user_identities IS 'Stores linked social/external identities for users';
COMMENT ON TABLE social_connection_templates IS 'Pre-configured templates for popular social providers';

COMMENT ON COLUMN user_identities.provider IS 'Social provider name (google, github, microsoft, etc.)';
COMMENT ON COLUMN user_identities.provider_user_id IS 'User ID from the social provider';
COMMENT ON COLUMN user_identities.raw_profile IS 'Full profile data from provider for future reference';
