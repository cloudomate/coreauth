-- MFA Backup Codes for account recovery
CREATE TABLE IF NOT EXISTS mfa_backup_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash VARCHAR(255) NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mfa_backup_codes_user ON mfa_backup_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_mfa_backup_codes_unused ON mfa_backup_codes(user_id, used_at) WHERE used_at IS NULL;

-- MFA Challenges for tracking verification attempts
CREATE TABLE IF NOT EXISTS mfa_challenges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge_token VARCHAR(255) NOT NULL UNIQUE,
    method_id UUID REFERENCES mfa_methods(id) ON DELETE CASCADE,
    code_hash VARCHAR(255),
    verified BOOLEAN DEFAULT false,
    ip_address TEXT,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mfa_challenges_user ON mfa_challenges(user_id);
CREATE INDEX IF NOT EXISTS idx_mfa_challenges_token ON mfa_challenges(challenge_token);
CREATE INDEX IF NOT EXISTS idx_mfa_challenges_expires ON mfa_challenges(expires_at) WHERE verified = false;

-- Add MFA requirement flag to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS mfa_enforced_at TIMESTAMP WITH TIME ZONE;

-- Add index for users with MFA enabled
CREATE INDEX IF NOT EXISTS idx_users_mfa_enabled ON users(tenant_id, mfa_enabled) WHERE mfa_enabled = true;

-- Comments for documentation
COMMENT ON TABLE mfa_backup_codes IS 'Recovery codes for MFA account access when primary method unavailable';
COMMENT ON TABLE mfa_challenges IS 'Temporary MFA verification challenges during login flow';
COMMENT ON COLUMN users.mfa_enabled IS 'Whether user has successfully enrolled in MFA';
COMMENT ON COLUMN users.mfa_enforced_at IS 'When MFA was enforced for this user (tenant policy)';
