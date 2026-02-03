-- Account Lockout (Brute Force Protection)
CREATE TABLE IF NOT EXISTS login_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    ip_address TEXT NOT NULL,
    successful BOOLEAN NOT NULL,
    failure_reason VARCHAR(100),
    attempted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_login_attempts_user ON login_attempts(user_id, attempted_at DESC);
CREATE INDEX IF NOT EXISTS idx_login_attempts_ip ON login_attempts(ip_address, attempted_at DESC);
CREATE INDEX IF NOT EXISTS idx_login_attempts_email ON login_attempts(tenant_id, email, attempted_at DESC);

CREATE TABLE IF NOT EXISTS account_lockouts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    locked_until TIMESTAMP WITH TIME ZONE NOT NULL,
    reason VARCHAR(255) NOT NULL,
    locked_by UUID REFERENCES users(id), -- NULL for automatic lockouts
    unlock_token VARCHAR(255) UNIQUE,
    unlocked_at TIMESTAMP WITH TIME ZONE,
    unlocked_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_account_lockouts_user ON account_lockouts(user_id, locked_until);
CREATE INDEX IF NOT EXISTS idx_account_lockouts_active ON account_lockouts(user_id, locked_until) WHERE unlocked_at IS NULL;

-- Email Verification Tokens
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_verification_user ON email_verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_expires ON email_verification_tokens(expires_at) WHERE used_at IS NULL;

-- Password Reset Tokens
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_password_reset_user ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_expires ON password_reset_tokens(expires_at) WHERE used_at IS NULL;

-- User Invitations
CREATE TABLE IF NOT EXISTS invitations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    invited_by UUID NOT NULL REFERENCES users(id),
    role_id UUID REFERENCES roles(id),
    metadata JSONB,
    accepted_at TIMESTAMP WITH TIME ZONE,
    accepted_by UUID REFERENCES users(id),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_invitations_tenant ON invitations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_invitations_email ON invitations(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_invitations_pending ON invitations(tenant_id, expires_at) WHERE accepted_at IS NULL;

-- Magic Link Tokens (Passwordless Authentication)
CREATE TABLE IF NOT EXISTS magic_link_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    device_fingerprint TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_magic_link_email ON magic_link_tokens(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_magic_link_expires ON magic_link_tokens(expires_at) WHERE used_at IS NULL;

-- User Bans
CREATE TABLE IF NOT EXISTS user_bans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ip_address TEXT,
    email VARCHAR(255),
    reason TEXT NOT NULL,
    banned_by UUID NOT NULL REFERENCES users(id),
    unbanned_at TIMESTAMP WITH TIME ZONE,
    unbanned_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- At least one identifier must be present
    CONSTRAINT user_bans_identifier_check CHECK (
        user_id IS NOT NULL OR ip_address IS NOT NULL OR email IS NOT NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_user_bans_user ON user_bans(user_id) WHERE unbanned_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_user_bans_ip ON user_bans(ip_address) WHERE unbanned_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_user_bans_email ON user_bans(tenant_id, email) WHERE unbanned_at IS NULL;

-- Disposable Email Domains Blocklist
CREATE TABLE IF NOT EXISTS blocked_email_domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE, -- NULL for global blocks
    domain VARCHAR(255) NOT NULL,
    reason TEXT,
    added_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(tenant_id, domain)
);

CREATE INDEX IF NOT EXISTS idx_blocked_domains_tenant ON blocked_email_domains(tenant_id);
CREATE INDEX IF NOT EXISTS idx_blocked_domains_lookup ON blocked_email_domains(domain);

-- Comments for documentation
COMMENT ON TABLE login_attempts IS 'Track all login attempts for security monitoring and brute force detection';
COMMENT ON TABLE account_lockouts IS 'Manage account lockouts due to failed login attempts or admin action';
COMMENT ON TABLE email_verification_tokens IS 'One-time tokens for verifying email addresses';
COMMENT ON TABLE password_reset_tokens IS 'Secure tokens for password reset flow';
COMMENT ON TABLE invitations IS 'User invitations with optional role assignment';
COMMENT ON TABLE magic_link_tokens IS 'Passwordless authentication via email magic links';
COMMENT ON TABLE user_bans IS 'Track banned users, IPs, and emails';
COMMENT ON TABLE blocked_email_domains IS 'Prevent registration from disposable/unwanted email domains';
