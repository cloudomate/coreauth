-- Add groups_claim and group_role_mappings columns to oidc_providers
-- These are needed for mapping IdP groups to CoreAuth roles

-- Add groups_claim column (the claim name in the ID token that contains groups)
ALTER TABLE oidc_providers
ADD COLUMN IF NOT EXISTS groups_claim TEXT DEFAULT 'groups';

-- Add group_role_mappings column (maps IdP group names to CoreAuth role names)
ALTER TABLE oidc_providers
ADD COLUMN IF NOT EXISTS group_role_mappings JSONB DEFAULT '{}'::jsonb;

-- Add comment for documentation
COMMENT ON COLUMN oidc_providers.groups_claim IS 'The claim name in the ID token that contains user groups';
COMMENT ON COLUMN oidc_providers.group_role_mappings IS 'JSON mapping of IdP group names to CoreAuth role names';
