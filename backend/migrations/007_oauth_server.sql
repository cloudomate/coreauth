-- OAuth2/OIDC Authorization Server Tables
-- This migration adds support for CoreAuth to act as an OAuth2/OIDC provider

-- OAuth2 Authorization Codes (for authorization code flow)
CREATE TABLE oauth_authorization_codes (
    code TEXT PRIMARY KEY,
    client_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    scope TEXT,
    code_challenge TEXT,                    -- PKCE: base64url encoded challenge
    code_challenge_method TEXT,             -- 'S256' or 'plain'
    nonce TEXT,                             -- OIDC: for id_token replay protection
    state TEXT,                             -- OAuth2: state parameter
    response_type TEXT NOT NULL DEFAULT 'code',
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,                    -- Track if code was already used
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_auth_codes_client_id ON oauth_authorization_codes(client_id);
CREATE INDEX idx_oauth_auth_codes_user_id ON oauth_authorization_codes(user_id);
CREATE INDEX idx_oauth_auth_codes_expires_at ON oauth_authorization_codes(expires_at);

-- OAuth2 Refresh Tokens (persistent, revocable)
CREATE TABLE oauth_refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash TEXT UNIQUE NOT NULL,        -- SHA256 hash of the token
    client_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    scope TEXT,
    audience TEXT,                          -- API identifier
    expires_at TIMESTAMPTZ,                 -- NULL = never expires
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_refresh_tokens_token_hash ON oauth_refresh_tokens(token_hash);
CREATE INDEX idx_oauth_refresh_tokens_client_id ON oauth_refresh_tokens(client_id);
CREATE INDEX idx_oauth_refresh_tokens_user_id ON oauth_refresh_tokens(user_id);
CREATE INDEX idx_oauth_refresh_tokens_expires_at ON oauth_refresh_tokens(expires_at) WHERE expires_at IS NOT NULL;

-- OAuth2 Access Token Audit (optional, for introspection and debugging)
CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti TEXT UNIQUE NOT NULL,               -- JWT ID for tracking
    client_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,  -- NULL for client_credentials
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    scope TEXT,
    audience TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_access_tokens_jti ON oauth_access_tokens(jti);
CREATE INDEX idx_oauth_access_tokens_client_id ON oauth_access_tokens(client_id);
CREATE INDEX idx_oauth_access_tokens_user_id ON oauth_access_tokens(user_id);

-- OAuth2 Consent Records (remember user consent per client/scope)
CREATE TABLE oauth_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    scopes TEXT[] NOT NULL,                 -- Array of consented scopes
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    UNIQUE(user_id, client_id)
);

CREATE INDEX idx_oauth_consents_user_id ON oauth_consents(user_id);
CREATE INDEX idx_oauth_consents_client_id ON oauth_consents(client_id);

-- Login Sessions (for Universal Login)
-- Tracks authenticated sessions across the authorization server
CREATE TABLE login_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_token_hash TEXT UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    ip_address INET,
    user_agent TEXT,
    authenticated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    mfa_verified BOOLEAN NOT NULL DEFAULT false,
    mfa_verified_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_login_sessions_token_hash ON login_sessions(session_token_hash);
CREATE INDEX idx_login_sessions_user_id ON login_sessions(user_id);
CREATE INDEX idx_login_sessions_expires_at ON login_sessions(expires_at);

-- OAuth2 Authorization Requests (stores pending auth requests during login)
CREATE TABLE oauth_authorization_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id TEXT UNIQUE NOT NULL,        -- Random ID shown in URL
    client_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    response_type TEXT NOT NULL,
    scope TEXT,
    state TEXT,
    code_challenge TEXT,
    code_challenge_method TEXT,
    nonce TEXT,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    connection_hint TEXT,                   -- Pre-selected connection (e.g., 'google')
    login_hint TEXT,                        -- Pre-filled email
    prompt TEXT,                            -- 'none', 'login', 'consent', 'select_account'
    max_age INTEGER,                        -- Max auth age in seconds
    ui_locales TEXT,                        -- Preferred locale
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL         -- Usually 10 minutes
);

CREATE INDEX idx_oauth_auth_requests_request_id ON oauth_authorization_requests(request_id);
CREATE INDEX idx_oauth_auth_requests_expires_at ON oauth_authorization_requests(expires_at);

-- RSA Key Pairs for JWT signing (supports key rotation)
CREATE TABLE signing_keys (
    id TEXT PRIMARY KEY,                    -- Key ID (kid)
    algorithm TEXT NOT NULL DEFAULT 'RS256',
    public_key_pem TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,          -- Encrypted at rest in production
    is_current BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ                  -- For key rotation scheduling
);

CREATE INDEX idx_signing_keys_is_current ON signing_keys(is_current) WHERE is_current = true;

-- Function to clean up expired authorization codes
CREATE OR REPLACE FUNCTION cleanup_expired_oauth_codes()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM oauth_authorization_codes
    WHERE expires_at < NOW() OR used_at IS NOT NULL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired authorization requests
CREATE OR REPLACE FUNCTION cleanup_expired_auth_requests()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM oauth_authorization_requests
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired/revoked refresh tokens
CREATE OR REPLACE FUNCTION cleanup_expired_refresh_tokens()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM oauth_refresh_tokens
    WHERE (expires_at IS NOT NULL AND expires_at < NOW())
       OR revoked_at IS NOT NULL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Add OAuth2 specific columns to applications table if not exists
DO $$
BEGIN
    -- Token lifetimes configuration
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'access_token_ttl_seconds') THEN
        ALTER TABLE applications ADD COLUMN access_token_ttl_seconds INTEGER NOT NULL DEFAULT 3600;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'refresh_token_ttl_seconds') THEN
        ALTER TABLE applications ADD COLUMN refresh_token_ttl_seconds INTEGER DEFAULT 2592000; -- 30 days
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'id_token_ttl_seconds') THEN
        ALTER TABLE applications ADD COLUMN id_token_ttl_seconds INTEGER NOT NULL DEFAULT 3600;
    END IF;

    -- Grant types allowed for this application
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'grant_types') THEN
        ALTER TABLE applications ADD COLUMN grant_types TEXT[] NOT NULL DEFAULT ARRAY['authorization_code', 'refresh_token'];
    END IF;

    -- Response types allowed
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'response_types') THEN
        ALTER TABLE applications ADD COLUMN response_types TEXT[] NOT NULL DEFAULT ARRAY['code'];
    END IF;

    -- Token endpoint auth method
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'token_endpoint_auth_method') THEN
        ALTER TABLE applications ADD COLUMN token_endpoint_auth_method TEXT NOT NULL DEFAULT 'client_secret_post';
    END IF;

    -- Allowed logout URLs
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'allowed_logout_urls') THEN
        ALTER TABLE applications ADD COLUMN allowed_logout_urls TEXT[] DEFAULT ARRAY[]::TEXT[];
    END IF;

    -- Web origins (for CORS)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'allowed_web_origins') THEN
        ALTER TABLE applications ADD COLUMN allowed_web_origins TEXT[] DEFAULT ARRAY[]::TEXT[];
    END IF;

    -- Is this a first-party application (skip consent)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'applications' AND column_name = 'is_first_party') THEN
        ALTER TABLE applications ADD COLUMN is_first_party BOOLEAN NOT NULL DEFAULT false;
    END IF;
END $$;

-- Insert default signing key (DEVELOPMENT ONLY - generate proper keys in production)
-- This is a placeholder - the application should generate keys on first startup
INSERT INTO signing_keys (id, algorithm, public_key_pem, private_key_pem, is_current)
VALUES (
    'dev-key-001',
    'RS256',
    '-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA2Z3qX2BTLS4e0ek45B9k
5vLefD4KPz1xdEZMP6b+t5FWpLwBF6AKBeqhbMpoxGBGBxdKMEbLXLqmA6lQXAkl
jvNzNOs4t5s7f1LVsp9TqKMbQ6p0Rw1hbggRv/hbQZRAuGayLeCMoQ/IWWAQK0M4
1IkHm6kgadgBF3bZfcN8xe8xEhA7crlXpqVnDiI75z7K2Sp/U7WzbJcpw9qPwoHR
OQGF9mRMUfN8P4MhDBqB0YFJ1KrVjy1jGIH4Z5KrKsYRLNoz5mgFyjF8pT8DWkC4
wvrAhnT0+0gMBaM3vz1zH/V66pBPWKMt0EVWfhAYwIJJvLbW2PS4ISrHOjpOfSjA
PQIDAQAB
-----END PUBLIC KEY-----',
    '-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA2Z3qX2BTLS4e0ek45B9k5vLefD4KPz1xdEZMP6b+t5FWpLwB
F6AKBeqhbMpoxGBGBxdKMEbLXLqmA6lQXAkljvNzNOs4t5s7f1LVsp9TqKMbQ6p0
Rw1hbggRv/hbQZRAuGayLeCMoQ/IWWAQK0M41IkHm6kgadgBF3bZfcN8xe8xEhA7
crlXpqVnDiI75z7K2Sp/U7WzbJcpw9qPwoHROQGF9mRMUfN8P4MhDBqB0YFJ1KrV
jy1jGIH4Z5KrKsYRLNoz5mgFyjF8pT8DWkC4wvrAhnT0+0gMBaM3vz1zH/V66pBP
WKMt0EVWfhAYwIJJvLbW2PS4ISrHOjpOfSjAPQIDAQABAoIBAFwI2+7POPVgwVES
bFv7tdZdvGwHgbHRewlgh1E13V7Ui0N3cyKMlpLUTN+xgqFkADX97fRT4TQwlwBr
8lLwIvg2ZELX+qJtAqLCspYMFllxkzXqmFLiqyqfwBT8T3s7Ae0ybAatYt6UZNIw
xTJaV1E2PW4PmpVcNiQFrT8x0tkhzz4p+6gYwFQwPIUQ5XrFnTc5U8+RCzWP7LxA
v5RAMqLWKKHsC8h4+0rq4vtUfLPIu/f2XxsVPTAqFfVCDXwAkthqPcD4D7xMLqYL
zIECgYEA8F9MSEH+VoKrO/xUmJk4qlwMvY6q9p3qVVPu0Ww8v0IKtk05PEmTdgkd
yz8NUHD+2CU8rm/j0HY8t/PS8Mh1Q/7M6C3oAm5J5YRM+Y9P4shm2ZqfnqB/L2Ff
k5N9GIjuwkPNkPCr6IqcLXFwcOTFGEMw8a3BJPEF8jSjYmGJhbkCgYEA5w5V8IWz
u8QeT1qQ3Y3ckqQ6Qh+kjxpEwnlgh7EqI9VUJqhKBZfYCjCTi/VpnZMHBqE0C7fJ
1/gcXdkMTK3BDl13QGqPRF0VQSJ0UyJhHPq7u1VNXevfQm01C7qprAAX9H6NLwMB
sM0pByJDMKMZLo6CSr4VkJJcqvU0P0FdJoUCgYANmBxqGBdYY3rmZVBaLoc+U0OZ
D8KOmFphPbfDCa+ZRqHPdJa4V8QBt+6/yHWLyv6cX5cN7nk1XBrmYnLqua//FbCA
qM0v+msmZUX1Qbms8q1hPA0Z4pvHE7E2TPR+TJBhcl8T3BLfNRq8R3wZo/RQQJiP
PBHkLp41SbdNkiA0MQKBgEX3q8WUQK3o+V5sLFBfLFz2vVXbqHCqe5V9yqYUv4S0
7HaBtvFOPz7/+lDhCahDipqwOrpw6EFfMIz8cX/XCnjPo+kLj/SYpk5yWJmau9JP
gJo1V+Y+7jvJdsMVEuQrGRlTfv9i+b8YQz3Wz/GdhVhoIFx+4vZ+FFxJORnwTGpF
AoGBAMH6Ns7FlBa0F7K0R1jS8lDmwL7rP/wcAqdjhLzrASl87A9Qz8tPFy7HOc8f
Mb8GPGjYqvNsAAJyLa5P1rpx3OP0wIG1vPRbA2NnSxXTEFXMKZfZ0bCq9KsdMW1i
jyT4LmG//YK21ap6NfDxI/FqNLxKuA3E8QjckpUdQXlpvuJi
-----END RSA PRIVATE KEY-----',
    true
) ON CONFLICT (id) DO NOTHING;

-- Comments for documentation
COMMENT ON TABLE oauth_authorization_codes IS 'Stores OAuth2 authorization codes for the authorization code flow';
COMMENT ON TABLE oauth_refresh_tokens IS 'Stores OAuth2 refresh tokens for token refresh';
COMMENT ON TABLE oauth_access_tokens IS 'Audit log of issued access tokens for introspection';
COMMENT ON TABLE oauth_consents IS 'Stores user consent decisions per client application';
COMMENT ON TABLE login_sessions IS 'Tracks authenticated sessions in the Universal Login';
COMMENT ON TABLE oauth_authorization_requests IS 'Temporary storage for in-progress authorization requests';
COMMENT ON TABLE signing_keys IS 'RSA key pairs for JWT signing with rotation support';
