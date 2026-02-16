-- ============================================================
-- CoreAuth CIAM - Authentication
-- ============================================================
-- MFA, password reset, email verification, tokens
-- ============================================================

-- ============================================================
-- MFA Methods (Multiple per User)
-- ============================================================

CREATE TABLE mfa_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    method_type VARCHAR(20) NOT NULL CHECK (method_type IN ('totp', 'sms', 'email', 'webauthn')),
    secret VARCHAR(255),
    phone_number VARCHAR(50),
    email VARCHAR(255),

    -- WebAuthn
    credential_id TEXT,
    public_key TEXT,
    sign_count INTEGER DEFAULT 0,

    verified BOOLEAN DEFAULT false,
    is_primary BOOLEAN DEFAULT false,
    last_used_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_mfa_methods_user ON mfa_methods(user_id);
CREATE INDEX idx_mfa_methods_type ON mfa_methods(user_id, method_type);

CREATE TRIGGER mfa_methods_updated_at
    BEFORE UPDATE ON mfa_methods
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- Password Reset Tokens
-- ============================================================

CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,

    ip_address TEXT,
    user_agent TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_reset_user ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_expires ON password_reset_tokens(expires_at);

-- ============================================================
-- Email Verification Tokens
-- ============================================================

CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    token_hash VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_email_verification_user ON email_verification_tokens(user_id);
CREATE INDEX idx_email_verification_expires ON email_verification_tokens(expires_at);

-- ============================================================
-- Refresh Tokens (Legacy/Session-based)
-- ============================================================

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    token_hash VARCHAR(255) UNIQUE NOT NULL,
    family_id UUID NOT NULL,

    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    replaced_by UUID REFERENCES refresh_tokens(id),

    ip_address TEXT,
    user_agent TEXT,
    device_info JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens(expires_at);

-- ============================================================
-- Login Sessions (Universal Login)
-- ============================================================

CREATE TABLE login_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_token_hash VARCHAR(255) UNIQUE NOT NULL,

    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    ip_address TEXT,
    user_agent TEXT,

    authenticated_at TIMESTAMPTZ,
    last_active_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    mfa_verified BOOLEAN DEFAULT false,
    revoked_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_login_sessions_user ON login_sessions(user_id);
CREATE INDEX idx_login_sessions_expires ON login_sessions(expires_at);

-- ============================================================
-- User Identities (Social/External Providers)
-- ============================================================

CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    raw_profile JSONB DEFAULT '{}',

    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,

    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, provider, provider_user_id),
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX idx_user_identities_user ON user_identities(user_id);
CREATE INDEX idx_user_identities_provider ON user_identities(provider, provider_user_id);

-- ============================================================
-- Login Attempts (Security Audit)
-- ============================================================

CREATE TABLE login_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    ip_address TEXT NOT NULL,
    successful BOOLEAN NOT NULL,
    failure_reason VARCHAR(100),
    attempted_at TIMESTAMPTZ DEFAULT NOW(),
    user_agent TEXT
);

CREATE INDEX idx_login_attempts_user ON login_attempts(user_id, attempted_at DESC);
CREATE INDEX idx_login_attempts_ip ON login_attempts(ip_address, attempted_at DESC);
CREATE INDEX idx_login_attempts_email ON login_attempts(tenant_id, email, attempted_at DESC);

-- ============================================================
-- Account Lockouts
-- ============================================================

CREATE TABLE account_lockouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    locked_until TIMESTAMPTZ NOT NULL,
    failed_attempts INTEGER DEFAULT 0,
    last_attempt_at TIMESTAMPTZ DEFAULT NOW(),
    unlocked_at TIMESTAMPTZ,
    unlocked_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_account_lockouts_user ON account_lockouts(tenant_id, user_id);
CREATE INDEX idx_account_lockouts_lookup ON account_lockouts(tenant_id, user_id) WHERE unlocked_at IS NULL;

-- ============================================================
-- User Bans
-- ============================================================

CREATE TABLE user_bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ip_address TEXT,
    email VARCHAR(255),
    reason TEXT,
    expires_at TIMESTAMPTZ,
    unbanned_at TIMESTAMPTZ,
    unbanned_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_user_bans_tenant ON user_bans(tenant_id);
CREATE INDEX idx_user_bans_user ON user_bans(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_user_bans_ip ON user_bans(ip_address) WHERE ip_address IS NOT NULL;
CREATE INDEX idx_user_bans_email ON user_bans(email) WHERE email IS NOT NULL;

-- ============================================================
-- Magic Link Tokens
-- ============================================================

CREATE TABLE magic_link_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_magic_link_user ON magic_link_tokens(user_id);
CREATE INDEX idx_magic_link_token ON magic_link_tokens(token_hash);
CREATE INDEX idx_magic_link_email ON magic_link_tokens(tenant_id, email);
