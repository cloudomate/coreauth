-- Add allowed_scopes column to applications table
-- This column stores the list of scopes this application is allowed to request

ALTER TABLE applications
ADD COLUMN IF NOT EXISTS allowed_scopes TEXT[] NOT NULL DEFAULT ARRAY['openid', 'profile', 'email'];

-- Add index for scope lookups
CREATE INDEX IF NOT EXISTS idx_applications_allowed_scopes ON applications USING GIN (allowed_scopes);

COMMENT ON COLUMN applications.allowed_scopes IS 'List of scopes this application is allowed to request';
COMMENT ON COLUMN applications.grant_types IS 'List of OAuth2 grant types this application can use';
