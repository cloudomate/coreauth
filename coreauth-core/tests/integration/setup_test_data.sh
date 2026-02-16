#!/bin/bash
# Setup test data via API calls

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

print_info "Setting up test data..."

# Create platform admin via direct SQL (bypass password for now)
# We'll use a special marker password that the tests can handle
ADMIN_HASH='$argon2id$v=19$m=19456,t=2,p=1$cGxhdGZvcm1hZG1pbnNhbHQ$+Hw7qLG+FqPLDc8M3ERnAZzF1j6n1xS3K2mS+Wm9vCM'

db_query "
DELETE FROM organization_members;
DELETE FROM users;
DELETE FROM organizations;

INSERT INTO organizations (slug, name, isolation_mode)
VALUES ('acme', 'ACME Corp', 'pool');

INSERT INTO users (email, email_verified, password_hash, is_platform_admin, is_active)
VALUES ('admin@platform.com', true, '$ADMIN_HASH', true, true);

INSERT INTO users (email, email_verified, password_hash, default_organization_id, is_platform_admin, is_active)
SELECT 'john@acme.com', true, '$argon2id$v=19$m=19456,t=2,p=1$am9obmFjbWVzYWx0$VHw7qLG+FqPLDc8M3ERnAZzF1j6n1xS3K2mS+Wm9vCM', o.id, false, true
FROM organizations o WHERE o.slug = 'acme';

INSERT INTO organization_members (user_id, organization_id, role)
SELECT u.id, o.id, 'admin'
FROM users u
CROSS JOIN organizations o
WHERE u.email = 'john@acme.com' AND o.slug = 'acme';
"

print_info "Test data setup complete"
