#!/bin/bash
# Seed test data via API endpoints

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

print_test_header "Seeding Test Data"

# Wait for API to be ready
wait_for_api

# Create ACME Corp tenant with admin user john@acme.com
print_info "Creating ACME Corp organization with admin user"
http_request "POST" "/api/tenants" \
    '{
        "slug": "acme",
        "name": "ACME Corp",
        "admin_email": "john@acme.com",
        "admin_password": "UserPass456!",
        "admin_full_name": "John Doe"
    }' \
    "/tmp/create_tenant.json"

STATUS=$(jq -r '.status' /tmp/create_tenant.json)
if [ "$STATUS" != "200" ] && [ "$STATUS" != "201" ]; then
    print_error "Failed to create ACME tenant (status: $STATUS)"
    cat /tmp/create_tenant.json
    exit 1
fi

print_success "ACME Corp created with admin john@acme.com"

# Create platform admin user directly via SQL (no tenant association)
print_info "Creating platform admin user"
db_query "
-- Create platform admin account using hash from john's account as template
INSERT INTO users (email, email_verified, password_hash, is_platform_admin, is_active, default_organization_id)
SELECT
    'admin@platform.com',
    true,
    u.password_hash,  -- Reuse the hash format from john's account as template
    true,
    true,
    NULL  -- No default organization for platform admin
FROM users u
WHERE u.email = 'john@acme.com'
LIMIT 1
ON CONFLICT (default_organization_id, email) DO NOTHING;
"

# Now update admin password by creating temp user and copying hash
print_info "Setting up platform admin password"

# Create temp tenant for password hashing
http_request "POST" "/api/tenants" \
    '{
        "slug": "temp-admin-setup",
        "name": "Temp Setup",
        "admin_email": "temp-admin@setup.local",
        "admin_password": "SecurePass123!",
        "admin_full_name": "Temp Admin"
    }' \
    "/tmp/temp_admin.json" || true

# Copy the password hash to platform admin
db_query "
UPDATE users
SET password_hash = (
    SELECT password_hash FROM users WHERE email = 'temp-admin@setup.local' LIMIT 1
)
WHERE email = 'admin@platform.com';

-- Clean up temp data (delete in correct order to respect foreign keys)
DELETE FROM user_roles WHERE user_id IN (SELECT id FROM users WHERE email = 'temp-admin@setup.local');
DELETE FROM organization_members WHERE user_id IN (SELECT id FROM users WHERE email = 'temp-admin@setup.local');
DELETE FROM users WHERE email = 'temp-admin@setup.local';
DELETE FROM roles WHERE tenant_id IN (SELECT id FROM organizations WHERE slug = 'temp-admin-setup');
DELETE FROM organizations WHERE slug = 'temp-admin-setup';
"

print_success "Platform admin admin@platform.com created"

# Verify users were created
print_info "Verifying test users"
USER_COUNT=$(PGPASSWORD=$POSTGRES_PASSWORD psql -h $POSTGRES_HOST -p $POSTGRES_PORT -U $POSTGRES_USER -d $POSTGRES_DB -A -t -c "SELECT COUNT(*) FROM users WHERE email IN ('admin@platform.com', 'john@acme.com');" | tr -d '[:space:]')

if [ "$USER_COUNT" = "2" ]; then
    print_success "Test data seeded successfully!"
    print_info "Test users:"
    print_info "  - admin@platform.com (password: SecurePass123!) - Platform Admin"
    print_info "  - john@acme.com (password: UserPass456!) - ACME Corp Admin"
else
    print_error "Failed to create all test users (found $USER_COUNT/2)"
    exit 1
fi
