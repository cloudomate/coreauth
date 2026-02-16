-- ============================================================
-- SMS MFA: SMS-based One-Time Password for Multi-Factor Authentication
-- ============================================================

-- Table to store SMS OTP codes temporarily (similar to email verification)
CREATE TABLE IF NOT EXISTS sms_otp_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    phone_number VARCHAR(50) NOT NULL,
    code_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 5,
    verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_sms_otp_user ON sms_otp_codes(user_id);
CREATE INDEX idx_sms_otp_expires ON sms_otp_codes(expires_at) WHERE verified = false;

-- Cleanup function for expired OTP codes
CREATE OR REPLACE FUNCTION cleanup_expired_sms_otp()
RETURNS void AS $$
BEGIN
    DELETE FROM sms_otp_codes
    WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;

-- Add phone_verified column to users table if not exists
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_verified_at TIMESTAMPTZ;

-- Update mfa_methods table to store phone number for SMS methods
-- (The phone is already stored in users.phone, but we may want to allow multiple)
ALTER TABLE mfa_methods ADD COLUMN IF NOT EXISTS phone_number VARCHAR(50);
