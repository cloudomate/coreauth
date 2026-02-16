-- Add allowed_group_id column to oidc_providers
-- This restricts SSO access to only users who are members of the specified Azure AD group

ALTER TABLE oidc_providers
ADD COLUMN IF NOT EXISTS allowed_group_id TEXT;

COMMENT ON COLUMN oidc_providers.allowed_group_id IS 'Azure AD group ID - only users in this group can sign in via this connection';
