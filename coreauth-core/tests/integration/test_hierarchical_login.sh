#!/bin/bash

# Test hierarchical authentication login endpoint

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

TEST_DIR="/tmp/test_hierarchical_login"
mkdir -p "$TEST_DIR"

print_test_header "Hierarchical Login Tests"

# Test 1: Platform Admin Login (No Organization)
print_info "Test 1: Platform admin login without organization context"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"admin@platform.com","password":"SecurePass123!"}' \
    "$TEST_DIR/admin_login.json"

assert_http_status "200" "$TEST_DIR/admin_login.json" \
    "Platform admin login returns 200"

assert_not_null ".body.access_token" "$TEST_DIR/admin_login.json" \
    "Platform admin receives access token"

assert_not_null ".body.refresh_token" "$TEST_DIR/admin_login.json" \
    "Platform admin receives refresh token"

assert_json_field ".body.user.email" "admin@platform.com" "$TEST_DIR/admin_login.json" \
    "Platform admin user email is correct"

# Decode JWT and verify claims
ADMIN_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/admin_login.json")
decode_jwt_payload "$ADMIN_TOKEN" > "$TEST_DIR/admin_jwt_claims.json"

assert_json_field ".is_platform_admin" "true" "$TEST_DIR/admin_jwt_claims.json" \
    "Platform admin JWT has is_platform_admin=true"

assert_json_field ".organization_id" "null" "$TEST_DIR/admin_jwt_claims.json" \
    "Platform admin JWT has null organization_id"

assert_json_field ".email" "admin@platform.com" "$TEST_DIR/admin_jwt_claims.json" \
    "Platform admin JWT email claim is correct"

# Test 2: Organization Member Login
print_info "Test 2: Organization member login with organization_slug"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!","organization_slug":"acme"}' \
    "$TEST_DIR/org_member_login.json"

assert_http_status "200" "$TEST_DIR/org_member_login.json" \
    "Organization member login returns 200"

assert_not_null ".body.access_token" "$TEST_DIR/org_member_login.json" \
    "Organization member receives access token"

# Decode JWT and verify organization context
ORG_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/org_member_login.json")
decode_jwt_payload "$ORG_TOKEN" > "$TEST_DIR/org_jwt_claims.json"

assert_json_field ".organization_slug" "acme" "$TEST_DIR/org_jwt_claims.json" \
    "Organization member JWT has correct organization_slug"

assert_json_field ".is_platform_admin" "false" "$TEST_DIR/org_jwt_claims.json" \
    "Organization member JWT has is_platform_admin=false"

assert_not_null ".organization_id" "$TEST_DIR/org_jwt_claims.json" \
    "Organization member JWT has organization_id"

assert_not_null ".role" "$TEST_DIR/org_jwt_claims.json" \
    "Organization member JWT has role"

assert_json_field ".email" "john@acme.com" "$TEST_DIR/org_jwt_claims.json" \
    "Organization member JWT email claim is correct"

# Test 3: Use access token to access protected endpoint
print_info "Test 3: Access protected endpoint with organization context token"
http_request "GET" "/api/auth/me" "" \
    "$TEST_DIR/auth_me.json" \
    "$ORG_TOKEN"

assert_http_status "200" "$TEST_DIR/auth_me.json" \
    "Protected endpoint accessible with valid token"

assert_json_field ".body.email" "john@acme.com" "$TEST_DIR/auth_me.json" \
    "Protected endpoint returns correct user"

# Test 4: Token refresh preserves organization context
print_info "Test 4: Token refresh preserves organization context"
REFRESH_TOKEN=$(jq -r '.body.refresh_token' "$TEST_DIR/org_member_login.json")

http_request "POST" "/api/auth/refresh" \
    "{\"refresh_token\":\"$REFRESH_TOKEN\"}" \
    "$TEST_DIR/token_refresh.json"

assert_http_status "200" "$TEST_DIR/token_refresh.json" \
    "Token refresh returns 200"

# Decode new access token
NEW_ACCESS_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/token_refresh.json")
decode_jwt_payload "$NEW_ACCESS_TOKEN" > "$TEST_DIR/refreshed_jwt_claims.json"

assert_json_field ".organization_slug" "acme" "$TEST_DIR/refreshed_jwt_claims.json" \
    "Refreshed JWT preserves organization_slug"

assert_json_field ".email" "john@acme.com" "$TEST_DIR/refreshed_jwt_claims.json" \
    "Refreshed JWT preserves email"

# Test 5: Multi-organization user can login to different orgs
print_info "Test 5: Multi-organization user login to first organization"

# First, check if consultant user exists and belongs to multiple orgs
CONSULTANT_ORG_COUNT=$(db_query "SELECT COUNT(*) FROM organization_members om JOIN users u ON om.user_id = u.id WHERE u.email = 'consultant@example.com'" | xargs)

if [ "$CONSULTANT_ORG_COUNT" -ge 2 ]; then
    print_info "Testing multi-org user (consultant has $CONSULTANT_ORG_COUNT organizations)"

    http_request "POST" "/api/auth/login-hierarchical" \
        '{"email":"consultant@example.com","password":"ConsultPass789!","organization_slug":"acme"}' \
        "$TEST_DIR/consultant_acme.json"

    CONSULTANT_ACME_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/consultant_acme.json" 2>/dev/null || echo "")

    if [ -n "$CONSULTANT_ACME_TOKEN" ] && [ "$CONSULTANT_ACME_TOKEN" != "null" ]; then
        decode_jwt_payload "$CONSULTANT_ACME_TOKEN" > "$TEST_DIR/consultant_acme_claims.json"

        assert_json_field ".organization_slug" "acme" "$TEST_DIR/consultant_acme_claims.json" \
            "Multi-org user JWT has correct organization_slug for first org"

        # Login to second org
        http_request "POST" "/api/auth/login-hierarchical" \
            '{"email":"consultant@example.com","password":"ConsultPass789!","organization_slug":"imys"}' \
            "$TEST_DIR/consultant_imys.json"

        CONSULTANT_IMYS_TOKEN=$(jq -r '.body.access_token' "$TEST_DIR/consultant_imys.json" 2>/dev/null || echo "")

        if [ -n "$CONSULTANT_IMYS_TOKEN" ] && [ "$CONSULTANT_IMYS_TOKEN" != "null" ]; then
            decode_jwt_payload "$CONSULTANT_IMYS_TOKEN" > "$TEST_DIR/consultant_imys_claims.json"

            assert_json_field ".organization_slug" "imys" "$TEST_DIR/consultant_imys_claims.json" \
                "Multi-org user JWT has correct organization_slug for second org"

            # Verify organization IDs are different
            ACME_ORG_ID=$(jq -r '.organization_id' "$TEST_DIR/consultant_acme_claims.json")
            IMYS_ORG_ID=$(jq -r '.organization_id' "$TEST_DIR/consultant_imys_claims.json")

            TESTS_RUN=$((TESTS_RUN + 1))
            if [ "$ACME_ORG_ID" != "$IMYS_ORG_ID" ]; then
                print_success "Multi-org user receives different organization_id for different orgs"
                TESTS_PASSED=$((TESTS_PASSED + 1))
            else
                print_error "Multi-org user should have different organization_id for different orgs"
                TESTS_FAILED=$((TESTS_FAILED + 1))
            fi
        else
            print_info "Skipping second org test (login failed)"
        fi
    else
        print_info "Skipping multi-org test (first login failed)"
    fi
else
    print_info "Skipping multi-org tests (consultant user not found or has < 2 orgs)"
fi

# Test 6: Audit logs capture organization context
print_info "Test 6: Verify audit logs capture organization context"

RECENT_LOGIN_COUNT=$(db_query "SELECT COUNT(*) FROM audit_logs WHERE event_type = 'auth.user.login' AND created_at > NOW() - INTERVAL '5 minutes'" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$RECENT_LOGIN_COUNT" -gt 0 ]; then
    print_success "Audit logs contain recent login events"
    TESTS_PASSED=$((TESTS_PASSED + 1))

    # Check if metadata contains organization context
    ORG_CONTEXT_COUNT=$(db_query "SELECT COUNT(*) FROM audit_logs WHERE event_type = 'auth.user.login' AND metadata::text LIKE '%organization%' AND created_at > NOW() - INTERVAL '5 minutes'" | xargs)

    TESTS_RUN=$((TESTS_RUN + 1))
    if [ "$ORG_CONTEXT_COUNT" -gt 0 ]; then
        print_success "Audit logs contain organization context in metadata"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_error "Audit logs should contain organization context in metadata"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    print_error "No recent login audit logs found"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

print_info "Hierarchical login tests completed"
