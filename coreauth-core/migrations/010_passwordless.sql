-- ============================================================
-- Passwordless Authentication: Magic Links and OTP codes
-- For headless IAM where clients build their own UIs
-- ============================================================

-- Passwordless tokens table (for both magic links and OTP)
CREATE TABLE passwordless_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    -- User email (may or may not exist yet)
    email VARCHAR(255) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,

    -- Token type: 'magic_link' or 'otp'
    token_type VARCHAR(20) NOT NULL CHECK (token_type IN ('magic_link', 'otp')),

    -- The actual token/code
    -- For magic_link: a long random string (hashed)
    -- For OTP: a 6-digit code (hashed)
    token_hash VARCHAR(255) NOT NULL,

    -- For rate limiting and security
    ip_address VARCHAR(45),
    user_agent TEXT,

    -- Expiration and usage
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,

    -- Attempt tracking (for OTP brute force protection)
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 5,

    -- Context (what happens after successful auth)
    redirect_uri TEXT,
    state TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_passwordless_tokens_email ON passwordless_tokens(tenant_id, email);
CREATE INDEX idx_passwordless_tokens_expires ON passwordless_tokens(expires_at) WHERE used_at IS NULL;

-- Cleanup old tokens (run periodically)
CREATE OR REPLACE FUNCTION cleanup_expired_passwordless_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM passwordless_tokens
    WHERE expires_at < NOW() - INTERVAL '1 day'
       OR used_at IS NOT NULL;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- WebAuthn/Passkey credentials for passwordless login
-- ============================================================

CREATE TABLE webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    -- Credential info from WebAuthn registration
    credential_id BYTEA NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,

    -- Counter for replay attack protection
    sign_count BIGINT DEFAULT 0,

    -- Credential metadata
    name VARCHAR(255),  -- User-friendly name like "MacBook Pro Touch ID"
    aaguid BYTEA,       -- Authenticator Attestation GUID

    -- Transports available (usb, nfc, ble, internal, hybrid)
    transports TEXT[],

    -- Device info
    device_type VARCHAR(50),  -- 'platform' (built-in) or 'cross-platform' (roaming)
    backed_up BOOLEAN DEFAULT false,

    -- Timestamps
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, credential_id)
);

CREATE INDEX idx_webauthn_user ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_credential ON webauthn_credentials(credential_id);

-- WebAuthn challenges (temporary storage during registration/authentication)
CREATE TABLE webauthn_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    -- Challenge data
    challenge BYTEA NOT NULL UNIQUE,

    -- What this challenge is for
    challenge_type VARCHAR(20) NOT NULL CHECK (challenge_type IN ('registration', 'authentication')),

    -- Associated user (for registration, may be NULL for authentication discovery)
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255),

    -- Expiration (challenges are short-lived)
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '5 minutes',

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_webauthn_challenges_expires ON webauthn_challenges(expires_at);

-- ============================================================
-- Token customization settings per application
-- ============================================================

-- Add columns to applications table for token customization
ALTER TABLE applications ADD COLUMN IF NOT EXISTS
    custom_claims JSONB DEFAULT '{}';

ALTER TABLE applications ADD COLUMN IF NOT EXISTS
    id_token_claims TEXT[] DEFAULT ARRAY['sub', 'email', 'name'];

ALTER TABLE applications ADD COLUMN IF NOT EXISTS
    access_token_claims TEXT[] DEFAULT ARRAY['sub', 'email', 'tenant_id'];

-- ============================================================
-- Rate limiting configuration per tenant
-- ============================================================

CREATE TABLE tenant_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    -- Endpoint category
    endpoint_category VARCHAR(50) NOT NULL,  -- 'login', 'register', 'passwordless', 'api'

    -- Rate limit settings
    requests_per_minute INTEGER NOT NULL DEFAULT 60,
    requests_per_hour INTEGER NOT NULL DEFAULT 1000,
    burst_limit INTEGER NOT NULL DEFAULT 10,

    -- Whether this limit is enabled
    is_enabled BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, endpoint_category)
);

CREATE TRIGGER tenant_rate_limits_updated_at
    BEFORE UPDATE ON tenant_rate_limits
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Insert default rate limits for all tenants
INSERT INTO tenant_rate_limits (tenant_id, endpoint_category, requests_per_minute, requests_per_hour, burst_limit)
SELECT t.id, ec.category, ec.rpm, ec.rph, ec.burst
FROM tenants t
CROSS JOIN (VALUES
    ('login', 10, 100, 5),
    ('register', 5, 50, 3),
    ('passwordless', 10, 100, 5),
    ('api', 100, 5000, 20)
) AS ec(category, rpm, rph, burst)
ON CONFLICT (tenant_id, endpoint_category) DO NOTHING;
