#!/bin/bash

# Test error cases for hierarchical authentication

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

TEST_DIR="/tmp/test_error_cases"
mkdir -p "$TEST_DIR"

print_test_header "Error Cases Tests"

# Test 1: Non-admin user without organization context
print_info "Test 1: Non-admin user attempts login without organization"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!"}' \
    "$TEST_DIR/no_org.json"

assert_http_status "403" "$TEST_DIR/no_org.json" \
    "Non-admin without org returns 403 Forbidden"

assert_json_field ".body.error" "login_failed" "$TEST_DIR/no_org.json" \
    "Error response has correct error code"

# Test 2: Invalid credentials
print_info "Test 2: Login with invalid password"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"WrongPassword123!","organization_slug":"acme"}' \
    "$TEST_DIR/invalid_creds.json"

assert_http_status "401" "$TEST_DIR/invalid_creds.json" \
    "Invalid credentials return 401 Unauthorized"

assert_json_field ".body.error" "login_failed" "$TEST_DIR/invalid_creds.json" \
    "Invalid credentials error has correct error code"

# Test 3: Non-existent organization
print_info "Test 3: Login with non-existent organization slug"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!","organization_slug":"nonexistent-org-xyz"}' \
    "$TEST_DIR/org_not_found.json"

assert_http_status "404" "$TEST_DIR/org_not_found.json" \
    "Non-existent organization returns 404 Not Found"

assert_json_field ".body.error" "login_failed" "$TEST_DIR/org_not_found.json" \
    "Organization not found error has correct error code"

# Test 4: User not member of organization
print_info "Test 4: User attempts login to organization they're not a member of"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!","organization_slug":"imys"}' \
    "$TEST_DIR/not_member.json"

assert_http_status "403" "$TEST_DIR/not_member.json" \
    "User not a member returns 403 Forbidden"

assert_json_field ".body.error" "login_failed" "$TEST_DIR/not_member.json" \
    "Not a member error has correct error code"

# Test 5: Non-existent user
print_info "Test 5: Login with non-existent email"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"nonexistent@example.com","password":"SomePassword123!","organization_slug":"acme"}' \
    "$TEST_DIR/user_not_found.json"

assert_http_status "401" "$TEST_DIR/user_not_found.json" \
    "Non-existent user returns 401 Unauthorized"

# Test 6: Missing required fields
print_info "Test 6: Login request missing required email field"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"password":"UserPass456!","organization_slug":"acme"}' \
    "$TEST_DIR/missing_email.json"

assert_http_status "400" "$TEST_DIR/missing_email.json" \
    "Missing required field returns 400 Bad Request"

print_info "Test 7: Login request missing required password field"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","organization_slug":"acme"}' \
    "$TEST_DIR/missing_password.json"

assert_http_status "400" "$TEST_DIR/missing_password.json" \
    "Missing password returns 400 Bad Request"

# Test 8: Empty credentials
print_info "Test 8: Login with empty email"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"","password":"UserPass456!","organization_slug":"acme"}' \
    "$TEST_DIR/empty_email.json"

assert_http_status "400" "$TEST_DIR/empty_email.json" \
    "Empty email returns 400 Bad Request"

print_info "Test 9: Login with empty password"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"","organization_slug":"acme"}' \
    "$TEST_DIR/empty_password.json"

# Should return 401 (invalid credentials) or 400 (validation error)
HTTP_STATUS=$(jq -r '.status' "$TEST_DIR/empty_password.json")
TESTS_RUN=$((TESTS_RUN + 1))
if [ "$HTTP_STATUS" == "400" ] || [ "$HTTP_STATUS" == "401" ]; then
    print_success "Empty password returns 400 or 401"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Empty password should return 400 or 401, got $HTTP_STATUS"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 10: SQL injection attempt in organization_slug
print_info "Test 10: SQL injection attempt in organization_slug"
http_request "POST" "/api/auth/login-hierarchical" \
    '{"email":"john@acme.com","password":"UserPass456!","organization_slug":"acme'\'' OR 1=1--"}' \
    "$TEST_DIR/sql_injection.json"

# Should safely handle and return 404 or 403
HTTP_STATUS=$(jq -r '.status' "$TEST_DIR/sql_injection.json")
TESTS_RUN=$((TESTS_RUN + 1))
if [ "$HTTP_STATUS" == "404" ] || [ "$HTTP_STATUS" == "403" ] || [ "$HTTP_STATUS" == "400" ]; then
    print_success "SQL injection attempt safely handled"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "SQL injection attempt should be safely rejected, got $HTTP_STATUS"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 11: Invalid token for protected endpoint
print_info "Test 11: Access protected endpoint with invalid token"
http_request "GET" "/api/auth/me" "" \
    "$TEST_DIR/invalid_token.json" \
    "invalid.jwt.token"

assert_http_status "401" "$TEST_DIR/invalid_token.json" \
    "Invalid token returns 401 Unauthorized"

# Test 12: No token for protected endpoint
print_info "Test 12: Access protected endpoint without token"
http_request "GET" "/api/auth/me" "" \
    "$TEST_DIR/no_token.json"

assert_http_status "401" "$TEST_DIR/no_token.json" \
    "Missing token returns 401 Unauthorized"

# Test 13: Expired/invalid refresh token
print_info "Test 13: Refresh with invalid refresh token"
http_request "POST" "/api/auth/refresh" \
    '{"refresh_token":"invalid.refresh.token"}' \
    "$TEST_DIR/invalid_refresh.json"

assert_http_status "401" "$TEST_DIR/invalid_refresh.json" \
    "Invalid refresh token returns 401"

# Test 14: Check failed login audit logs
print_info "Test 14: Verify failed login attempts are logged"

FAILED_LOGIN_COUNT=$(db_query "SELECT COUNT(*) FROM audit_logs WHERE event_type = 'auth.user.login_failed' AND created_at > NOW() - INTERVAL '5 minutes'" | xargs)

TESTS_RUN=$((TESTS_RUN + 1))
if [ "$FAILED_LOGIN_COUNT" -gt 0 ]; then
    print_success "Failed login attempts are logged in audit_logs"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_error "Expected failed login audit logs"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

print_info "Error cases tests completed"
