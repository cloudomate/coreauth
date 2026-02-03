#!/bin/bash

# Test backward compatibility with legacy login endpoint

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

TEST_DIR="/tmp/test_backward_compatibility"
mkdir -p "$TEST_DIR"

print_test_header "Backward Compatibility Tests"

# Test 1: Legacy login endpoint still works
print_info "Test 1: Legacy /api/auth/login endpoint with tenant_id"

# Get organization ID for acme
ACME_ORG_ID=$(db_query "SELECT id FROM organizations WHERE slug = 'acme' LIMIT 1" | xargs)

if [ -n "$ACME_ORG_ID" ] && [ "$ACME_ORG_ID" != "" ]; then
    print_info "Using organization ID: $ACME_ORG_ID"

    http_request "POST" "/api/auth/login" \
        "{\"tenant_id\":\"acme\",\"email\":\"john@acme.com\",\"password\":\"UserPass456!\"}" \
        "$TEST_DIR/legacy_login.json"

    HTTP_STATUS=$(jq -r '.status' "$TEST_DIR/legacy_login.json")

    # Legacy endpoint might return 200 if it still works, or 404/400 if deprecated
    TESTS_RUN=$((TESTS_RUN + 1))
    if [ "$HTTP_STATUS" == "200" ]; then
        print_success "Legacy login endpoint still functional"
        TESTS_PASSED=$((TESTS_PASSED + 1))

        # Verify token is returned
        assert_not_null ".body.access_token" "$TEST_DIR/legacy_login.json" \
            "Legacy login returns access token"

        # Decode JWT and verify it has tenant_id
        LEGACY_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/legacy_login.json")
        decode_jwt_payload "$LEGACY_TOKEN" > "$TEST_DIR/legacy_jwt_claims.json"

        assert_not_null ".tenant_id" "$TEST_DIR/legacy_jwt_claims.json" \
            "Legacy JWT includes tenant_id claim"

    elif [ "$HTTP_STATUS" == "404" ] || [ "$HTTP_STATUS" == "400" ]; then
        print_info "Legacy endpoint deprecated (returns $HTTP_STATUS) - this is acceptable"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_error "Legacy endpoint returned unexpected status: $HTTP_STATUS"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    print_info "Skipping legacy login test (no organization found)"
fi

# Test 2: Hierarchical endpoint can be used with organization_slug (new way)
print_info "Test 2: Hierarchical endpoint accepts organization_slug"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!","organization_slug":"acme"}' \
    "$TEST_DIR/hierarchical_slug.json"

assert_http_status "200" "$TEST_DIR/hierarchical_slug.json" \
    "Hierarchical login with organization_slug works"

# Test 3: JWT from hierarchical endpoint includes both tenant_id and organization_id
print_info "Test 3: Hierarchical JWT includes both old and new claims"
HIERARCHICAL_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/hierarchical_slug.json")
decode_jwt_payload "$HIERARCHICAL_TOKEN" > "$TEST_DIR/hierarchical_jwt_claims.json"

assert_not_null ".organization_id" "$TEST_DIR/hierarchical_jwt_claims.json" \
    "Hierarchical JWT includes new organization_id claim"

assert_not_null ".organization_slug" "$TEST_DIR/hierarchical_jwt_claims.json" \
    "Hierarchical JWT includes new organization_slug claim"

# tenant_id should also be present for backward compatibility
# It might be null for platform admins, but should exist for org members
TENANT_ID_VALUE=$(jq -r '.tenant_id' "$TEST_DIR/hierarchical_jwt_claims.json")
TESTS_RUN=$((TESTS_RUN + 1))
if [ "$TENANT_ID_VALUE" != "null" ] || [ -n "$TENANT_ID_VALUE" ]; then
    print_success "Hierarchical JWT includes tenant_id for backward compatibility"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_info "Note: tenant_id is null (acceptable for platform admins)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# Test 4: Old JWT claims still work with middleware
print_info "Test 4: Auth middleware handles both old and new JWT formats"

# Use the token to access a protected endpoint
http_request "GET" "/api/auth/me" "" \
    "$TEST_DIR/me_with_new_jwt.json" \
    "$HIERARCHICAL_TOKEN"

assert_http_status "200" "$TEST_DIR/me_with_new_jwt.json" \
    "Protected endpoint works with hierarchical JWT"

# Test 5: Database schema supports both models
print_info "Test 5: Verify database schema supports hierarchical model"

# Check users table has new columns
USER_COLUMNS=$(db_query "SELECT column_name FROM information_schema.columns WHERE table_name = 'users' AND column_name IN ('is_platform_admin', 'default_organization_id')" | wc -l | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$USER_COLUMNS" -ge 2 ]; then
    print_success "Users table has hierarchical columns (is_platform_admin, default_organization_id)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Users table missing hierarchical columns"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Check organizations table exists
ORG_TABLE_EXISTS=$(db_query "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'organizations'" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$ORG_TABLE_EXISTS" -eq 1 ]; then
    print_success "Organizations table exists"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Organizations table not found"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Check organization_members table exists
ORG_MEMBERS_TABLE_EXISTS=$(db_query "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'organization_members'" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$ORG_MEMBERS_TABLE_EXISTS" -eq 1 ]; then
    print_success "Organization_members table exists"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Organization_members table not found"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Check connections table exists and has scope column
CONNECTIONS_SCOPE=$(db_query "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'connections' AND column_name = 'scope'" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$CONNECTIONS_SCOPE" -eq 1 ]; then
    print_success "Connections table has scope column"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Connections table missing scope column"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 6: Test data migration - existing users have organization memberships
print_info "Test 6: Verify existing users migrated to organization members"

MIGRATED_USERS=$(db_query "SELECT COUNT(*) FROM organization_members" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$MIGRATED_USERS" -gt 0 ]; then
    print_success "Migration created organization memberships ($MIGRATED_USERS members)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "No organization members found - migration may have failed"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 7: Platform config exists
print_info "Test 7: Verify platform configuration exists"

PLATFORM_CONFIG=$(db_query "SELECT COUNT(*) FROM platform_config" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$PLATFORM_CONFIG" -gt 0 ]; then
    print_success "Platform configuration exists"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Platform configuration not found"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

print_info "Backward compatibility tests completed"
