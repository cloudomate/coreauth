
#!/bin/bash
# CoreAuth CIAM - Comprehensive Test Suite
# Tests run against the live Docker stack
# Backend: docker exec corerun-backend curl ...
# Proxy: curl http://localhost:4000/...

set -euo pipefail

API="http://localhost:3000"
PROXY="http://localhost:4000"
PASS=0
FAIL=0
SKIP=0
TOTAL=0
RESULTS=""

# Known tenant credentials (from bootstrap)
KNOWN_TENANT_SLUG="acme"
KNOWN_ADMIN_EMAIL="admin@acme.dev"
KNOWN_ADMIN_PASSWORD="AcmeAdmin123!"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Helper: run curl inside backend container
api() {
    docker exec corerun-backend curl -s -w "\n%{http_code}" "$@" 2>/dev/null
}

# Helper: extract HTTP status code from response
get_status() {
    echo "$1" | tail -1
}

# Helper: extract body from response
get_body() {
    echo "$1" | sed '$d'
}

# Test runner
run_test() {
    local id="$1"
    local desc="$2"
    local expected_status="$3"
    shift 3
    local curl_args=("$@")

    TOTAL=$((TOTAL + 1))
    local response
    response=$(api "${curl_args[@]}" 2>/dev/null) || true
    local status
    status=$(get_status "$response")
    local body
    body=$(get_body "$response")

    if [[ "$status" == "$expected_status" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [%s] %s (HTTP %s)\n" "$id" "$desc" "$status"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [%s] %s (Expected %s, Got %s)\n" "$id" "$desc" "$expected_status" "$status"
        # Show truncated body on failure
        echo "  Response: $(echo "$body" | head -c 200)"
    fi
}

# Test with body check (substring match)
run_test_body() {
    local id="$1"
    local desc="$2"
    local expected_status="$3"
    local body_check="$4"
    shift 4
    local curl_args=("$@")

    TOTAL=$((TOTAL + 1))
    local response
    response=$(api "${curl_args[@]}" 2>/dev/null) || true
    local status
    status=$(get_status "$response")
    local body
    body=$(get_body "$response")

    if [[ "$status" == "$expected_status" ]] && echo "$body" | grep -q "$body_check"; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [%s] %s (HTTP %s, body contains '%s')\n" "$id" "$desc" "$status" "$body_check"
    elif [[ "$status" != "$expected_status" ]]; then
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [%s] %s (Expected HTTP %s, Got %s)\n" "$id" "$desc" "$expected_status" "$status"
        echo "  Response: $(echo "$body" | head -c 200)"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [%s] %s (HTTP %s OK, but body missing '%s')\n" "$id" "$desc" "$status" "$body_check"
        echo "  Response: $(echo "$body" | head -c 300)"
    fi
}

skip_test() {
    local id="$1"
    local desc="$2"
    local reason="$3"
    TOTAL=$((TOTAL + 1))
    SKIP=$((SKIP + 1))
    printf "${YELLOW}SKIP${NC} [%s] %s (%s)\n" "$id" "$desc" "$reason"
}

echo "============================================================"
echo " CoreAuth CIAM - Comprehensive Test Suite"
echo " $(date)"
echo "============================================================"
echo ""

# ============================================================
# 0. HEALTH & CONNECTIVITY
# ============================================================
printf "\n${CYAN}=== HEALTH & CONNECTIVITY ===${NC}\n"

run_test_body "HEALTH-001" "Backend health check" "200" "healthy" \
    "${API}/health"

run_test "HEALTH-002" "Test connectivity endpoint" "200" \
    "${API}/api/test/connectivity"

# ============================================================
# 1. AUTHENTICATION - REGISTRATION
# ============================================================
printf "\n${CYAN}=== 1. AUTHENTICATION - REGISTRATION ===${NC}\n"

# Discover tenant from known slug
TENANT_RESP=$(api "${API}/api/organizations/by-slug/${KNOWN_TENANT_SLUG}" 2>/dev/null) || true
TENANT_BODY=$(get_body "$TENANT_RESP")
TENANT_STATUS=$(get_status "$TENANT_RESP")
TENANT_ID=$(echo "$TENANT_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

if [ -z "$TENANT_ID" ]; then
    # Try to get from DB directly
    TENANT_ID=$(docker exec corerun-postgres psql -U coreauth -d coreauth -t -c "SELECT id FROM tenants WHERE slug='${KNOWN_TENANT_SLUG}' LIMIT 1;" 2>/dev/null | tr -d ' ')
fi

echo "Tenant ID: ${TENANT_ID:-UNKNOWN}"
if [ -z "$TENANT_ID" ]; then
    echo "FATAL: Could not determine tenant ID. Exiting."
    exit 1
fi

# Generate unique email for test user
TEST_EMAIL="testuser_$(date +%s)@corerun.dev"
TEST_PASSWORD="TestPass123!@#"

run_test_body "AUTH-REG-001" "Register with valid email/password" "201" "access_token" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

# Capture tokens for this user
REG_RESP=$(api -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"testuser2_$(date +%s)@corerun.dev\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test2\",\"last_name\":\"User2\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
REG_BODY=$(get_body "$REG_RESP")

run_test "AUTH-REG-004" "Register with duplicate email" "409" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-REG-005" "Register with invalid email format" "400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"not-an-email\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-REG-006" "Register with empty email" "400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-REG-007" "Register with password too short" "400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"short_$(date +%s)@test.com\",\"password\":\"123\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-REG-009" "Register with empty body" "400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{}"

run_test "AUTH-REG-013" "Register with SQL injection in email" "400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"'; DROP TABLE users;--\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}"

# ============================================================
# 2. AUTHENTICATION - LOGIN
# ============================================================
printf "\n${CYAN}=== 2. AUTHENTICATION - LOGIN ===${NC}\n"

# Login and capture tokens
LOGIN_RESP=$(api -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
LOGIN_BODY=$(get_body "$LOGIN_RESP")
LOGIN_STATUS=$(get_status "$LOGIN_RESP")
ACCESS_TOKEN=$(echo "$LOGIN_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null || echo "")
REFRESH_TOKEN=$(echo "$LOGIN_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('refresh_token',''))" 2>/dev/null || echo "")

TOTAL=$((TOTAL + 1))
if [[ "$LOGIN_STATUS" == "200" ]] && [[ -n "$ACCESS_TOKEN" ]]; then
    PASS=$((PASS + 1))
    printf "${GREEN}PASS${NC} [AUTH-LOGIN-001] Login with valid credentials (HTTP 200, token received)\n"
else
    FAIL=$((FAIL + 1))
    printf "${RED}FAIL${NC} [AUTH-LOGIN-001] Login with valid credentials (HTTP %s)\n" "$LOGIN_STATUS"
    echo "  Response: $(echo "$LOGIN_BODY" | head -c 200)"
fi

run_test "AUTH-LOGIN-002" "Login with wrong password" "401" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"WrongPassword123\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-003" "Login with non-existent email" "401" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"nonexistent_$(date +%s)@test.com\",\"password\":\"${TEST_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-004" "Login with empty email" "400" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"\",\"password\":\"${TEST_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-005" "Login with empty password" "400" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-006" "Login with empty body" "400" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{}"

run_test "AUTH-LOGIN-012" "Login with SQL injection in email" "401" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"' OR 1=1 --\",\"password\":\"${TEST_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}"

# Check no password in login response
TOTAL=$((TOTAL + 1))
if echo "$LOGIN_BODY" | grep -q "password_hash\|password_digest"; then
    FAIL=$((FAIL + 1))
    printf "${RED}FAIL${NC} [AUTH-LOGIN-013] Login response leaks password hash!\n"
else
    PASS=$((PASS + 1))
    printf "${GREEN}PASS${NC} [AUTH-LOGIN-013] Login response does not leak password hash\n"
fi

# ============================================================
# 3. AUTHENTICATION - TOKEN REFRESH
# ============================================================
printf "\n${CYAN}=== 3. AUTHENTICATION - TOKEN REFRESH ===${NC}\n"

if [ -n "$REFRESH_TOKEN" ]; then
    run_test_body "AUTH-REFRESH-001" "Refresh with valid refresh_token" "200" "access_token" \
        -X POST "${API}/api/auth/refresh" \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\":\"${REFRESH_TOKEN}\"}"
else
    skip_test "AUTH-REFRESH-001" "Refresh with valid refresh_token" "No refresh token available"
fi

run_test "AUTH-REFRESH-003" "Refresh with invalid refresh_token" "401" \
    -X POST "${API}/api/auth/refresh" \
    -H "Content-Type: application/json" \
    -d '{"refresh_token":"invalid_token_here"}'

run_test "AUTH-REFRESH-004" "Refresh with empty token" "401" \
    -X POST "${API}/api/auth/refresh" \
    -H "Content-Type: application/json" \
    -d '{"refresh_token":""}'

# ============================================================
# 4. AUTHENTICATION - PROFILE (ME)
# ============================================================
printf "\n${CYAN}=== 4. AUTHENTICATION - PROFILE ===${NC}\n"

if [ -n "$ACCESS_TOKEN" ]; then
    run_test_body "AUTH-ME-001" "Get current user profile" "200" "email" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/auth/me"

    # Check no password in profile
    ME_RESP=$(api -H "Authorization: Bearer ${ACCESS_TOKEN}" "${API}/api/auth/me" 2>/dev/null) || true
    ME_BODY=$(get_body "$ME_RESP")
    TOTAL=$((TOTAL + 1))
    if echo "$ME_BODY" | grep -q "password_hash\|password_digest"; then
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [AUTH-ME-008] Profile exposes password hash!\n"
    else
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [AUTH-ME-008] Profile does not expose password hash\n"
    fi

    run_test_body "AUTH-ME-005" "Update profile" "200" "Updated" \
        -X PATCH "${API}/api/auth/me" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"first_name":"Updated","last_name":"Name"}'
else
    skip_test "AUTH-ME-001" "Get profile" "No access token"
    skip_test "AUTH-ME-008" "Password hash check" "No access token"
    skip_test "AUTH-ME-005" "Update profile" "No access token"
fi

run_test "AUTH-ME-002" "Get profile without auth" "401" \
    "${API}/api/auth/me"

run_test "AUTH-ME-003" "Get profile with expired/bad token" "401" \
    -H "Authorization: Bearer invalid_token_here" \
    "${API}/api/auth/me"

run_test "AUTH-ME-004" "Get profile with malformed JWT" "401" \
    -H "Authorization: Bearer not.a.jwt" \
    "${API}/api/auth/me"

# ============================================================
# 5. PASSWORD RESET
# ============================================================
printf "\n${CYAN}=== 5. PASSWORD RESET ===${NC}\n"

run_test "PWRESET-001" "Request password reset for existing email" "200" \
    -X POST "${API}/api/auth/forgot-password" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"tenant_id\":\"${TENANT_ID}\"}"

# Should return 200 even for non-existent email (no enumeration)
run_test "PWRESET-002" "Request reset for non-existent email (no enumeration)" "200" \
    -X POST "${API}/api/auth/forgot-password" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"nobody_$(date +%s)@test.com\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "PWRESET-006" "Verify invalid reset token" "400" \
    "${API}/api/auth/verify-reset-token?token=invalid_fake_token"

run_test "PWRESET-008" "Reset password with invalid token" "400" \
    -X POST "${API}/api/auth/reset-password" \
    -H "Content-Type: application/json" \
    -d '{"token":"invalid_token","password":"NewPass123!@#"}'

# ============================================================
# 6. OAUTH2/OIDC DISCOVERY
# ============================================================
printf "\n${CYAN}=== 6. OAUTH2/OIDC DISCOVERY ===${NC}\n"

run_test_body "OAUTH-DISC-001" "OpenID Configuration" "200" "authorization_endpoint" \
    "${API}/.well-known/openid-configuration"

run_test_body "OAUTH-DISC-003" "JWKS endpoint" "200" "keys" \
    "${API}/.well-known/jwks.json"

# ============================================================
# 7. UNIVERSAL LOGIN PAGES
# ============================================================
printf "\n${CYAN}=== 7. UNIVERSAL LOGIN PAGES ===${NC}\n"

run_test "UL-001" "Login page renders" "200" \
    "${API}/login"

run_test "UL-004" "Signup page renders" "200" \
    "${API}/signup"

run_test "UL-011" "Logged-out page renders" "200" \
    "${API}/logged-out"

# ============================================================
# 8. SELF-SERVICE FLOWS
# ============================================================
printf "\n${CYAN}=== 8. SELF-SERVICE FLOWS ===${NC}\n"

run_test "SS-002" "Create login flow (API)" "200" \
    "${API}/self-service/login/api"

run_test "SS-007" "Create registration flow (API)" "200" \
    "${API}/self-service/registration/api"

run_test "SS-011" "Whoami without session" "401" \
    "${API}/sessions/whoami"

# ============================================================
# 9. OIDC PROVIDERS & SSO
# ============================================================
printf "\n${CYAN}=== 9. OIDC PROVIDERS & SSO ===${NC}\n"

run_test_body "OIDC-TPL-001" "List OIDC provider templates" "200" "azuread" \
    "${API}/api/oidc/templates"

run_test_body "OIDC-TPL-002" "Get Azure AD template" "200" "microsoft" \
    "${API}/api/oidc/templates/azuread"

run_test "SSO-DISC-004" "SSO check with missing email" "400" \
    "${API}/api/oidc/sso-check"

run_test "OIDC-MGT-005" "Create provider without admin" "401" \
    -X POST "${API}/api/oidc/providers" \
    -H "Content-Type: application/json" \
    -d '{"name":"test"}'

# List public providers
run_test "OIDC-MGT-007" "List public OIDC providers" "200" \
    "${API}/api/oidc/providers/public?tenant_id=${TENANT_ID}"

# ============================================================
# 10. TENANT & ORGANIZATION
# ============================================================
printf "\n${CYAN}=== 10. TENANT & ORGANIZATION ===${NC}\n"

run_test_body "TENANT-005" "Get org by slug (corerun)" "200" "corerun" \
    "${API}/api/organizations/by-slug/corerun"

run_test "TENANT-006" "Get org by non-existent slug" "404" \
    "${API}/api/organizations/by-slug/nonexistent-org-$(date +%s)"

run_test "TENANT-004" "Create tenant with invalid slug" "400" \
    -X POST "${API}/api/tenants" \
    -H "Content-Type: application/json" \
    -d '{"name":"Test","slug":"INVALID SLUG!!!","admin_email":"x@test.com","admin_password":"Pass123!@#","admin_first_name":"A","admin_last_name":"B"}'

# ============================================================
# 11. ADMIN LOGIN - Get admin token for protected endpoints
# ============================================================
printf "\n${CYAN}=== 11. ADMIN ACCESS ===${NC}\n"

# Login as admin with known credentials
ADMIN_LOGIN_RESP=$(api -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"password\":\"${KNOWN_ADMIN_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
ADMIN_LOGIN_BODY=$(get_body "$ADMIN_LOGIN_RESP")
ADMIN_LOGIN_STATUS=$(get_status "$ADMIN_LOGIN_RESP")
ADMIN_TOKEN=$(echo "$ADMIN_LOGIN_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null || echo "")

if [ -z "$ADMIN_TOKEN" ]; then
    echo "WARNING: Could not get admin token (HTTP ${ADMIN_LOGIN_STATUS}). Some tests will be skipped."
    echo "  Response: $(echo "$ADMIN_LOGIN_BODY" | head -c 200)"
fi

# ============================================================
# 12. TENANT USERS (Admin)
# ============================================================
printf "\n${CYAN}=== 12. TENANT USER MANAGEMENT ===${NC}\n"

if [ -n "$ADMIN_TOKEN" ]; then
    run_test_body "TUSER-001" "List tenant users (admin)" "200" "email" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/users"
else
    skip_test "TUSER-001" "List tenant users" "No admin token"
fi

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "TUSER-002" "List tenant users (regular user)" "403" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/users"
else
    skip_test "TUSER-002" "List tenant users (regular user)" "No regular user token"
fi

run_test "TUSER-008" "List users without auth" "401" \
    "${API}/api/tenants/${TENANT_ID}/users"

# ============================================================
# 13. INVITATIONS
# ============================================================
printf "\n${CYAN}=== 13. INVITATIONS ===${NC}\n"

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "INV-003" "Create invitation without admin" "403" \
        -X POST "${API}/api/tenants/${TENANT_ID}/invitations" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"email":"invite@test.com"}'
else
    skip_test "INV-003" "Create invitation without admin" "No regular user token"
fi

run_test "INV-008" "Verify invalid invitation token" "400" \
    "${API}/api/invitations/verify?token=invalid_fake_token"

if [ -n "$ADMIN_TOKEN" ]; then
    INV_EMAIL="invite_$(date +%s)@test.com"
    run_test "INV-001" "Create invitation (admin)" "201" \
        -X POST "${API}/api/tenants/${TENANT_ID}/invitations" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"${INV_EMAIL}\",\"expires_in_days\":7}"

    run_test_body "INV-005" "List invitations (admin)" "200" "email" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/invitations"
else
    skip_test "INV-001" "Create invitation" "No admin token"
    skip_test "INV-005" "List invitations" "No admin token"
fi

# ============================================================
# 14. GROUPS
# ============================================================
printf "\n${CYAN}=== 14. GROUPS MANAGEMENT ===${NC}\n"

run_test "GRP-003" "Create group without admin" "403" \
    -X POST "${API}/api/tenants/${TENANT_ID}/groups" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"name":"test-group"}'

if [ -n "$ADMIN_TOKEN" ]; then
    GROUP_NAME="test-group-$(date +%s)"
    GRP_CREATE_RESP=$(api -X POST "${API}/api/tenants/${TENANT_ID}/groups" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"${GROUP_NAME}\",\"description\":\"Test group\"}" 2>/dev/null) || true
    GRP_STATUS=$(get_status "$GRP_CREATE_RESP")
    GRP_BODY=$(get_body "$GRP_CREATE_RESP")
    GROUP_ID=$(echo "$GRP_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

    TOTAL=$((TOTAL + 1))
    if [[ "$GRP_STATUS" == "201" ]] && [[ -n "$GROUP_ID" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [GRP-001] Create group (HTTP 201)\n"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [GRP-001] Create group (Expected 201, Got %s)\n" "$GRP_STATUS"
        echo "  Response: $(echo "$GRP_BODY" | head -c 200)"
    fi

    run_test_body "GRP-004" "List groups (admin)" "200" "name" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/groups"

    if [ -n "$GROUP_ID" ]; then
        run_test_body "GRP-005" "Get group by ID" "200" "$GROUP_NAME" \
            -H "Authorization: Bearer ${ADMIN_TOKEN}" \
            "${API}/api/tenants/${TENANT_ID}/groups/${GROUP_ID}"
    fi

    run_test "GRP-006" "Get non-existent group" "404" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/groups/00000000-0000-0000-0000-000000000000"
else
    skip_test "GRP-001" "Create group" "No admin token"
    skip_test "GRP-004" "List groups" "No admin token"
fi

# ============================================================
# 15. SESSION MANAGEMENT
# ============================================================
printf "\n${CYAN}=== 15. SESSION MANAGEMENT ===${NC}\n"

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "SESS-001" "List active sessions" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/sessions"

    run_test "SESS-007" "Get login history" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/login-history"

    run_test "SESS-008" "Get security audit logs" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/security/audit-logs"
fi

run_test "SESS-006" "List sessions without auth" "401" \
    "${API}/api/sessions"

run_test "SESS-005" "Revoke non-existent session" "404" \
    -X DELETE \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    "${API}/api/sessions/00000000-0000-0000-0000-000000000000"

# ============================================================
# 16. OAUTH APPLICATIONS
# ============================================================
printf "\n${CYAN}=== 16. OAUTH APPLICATIONS ===${NC}\n"

run_test "OAPP-003" "Create app without admin" "403" \
    -X POST "${API}/api/organizations/${TENANT_ID}/applications" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"name":"test-app","type":"webapp"}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test_body "OAPP-004" "List applications (admin)" "200" "" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/applications"

    APP_NAME="test-app-$(date +%s)"
    APP_RESP=$(api -X POST "${API}/api/organizations/${TENANT_ID}/applications" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"${APP_NAME}\",\"type\":\"webapp\",\"redirect_uris\":[\"http://localhost:3000/callback\"]}" 2>/dev/null) || true
    APP_STATUS=$(get_status "$APP_RESP")
    APP_BODY=$(get_body "$APP_RESP")
    APP_ID=$(echo "$APP_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',json.load(sys.stdin) if isinstance(json.load(sys.stdin),str) else ''))" 2>/dev/null || echo "")

    TOTAL=$((TOTAL + 1))
    if [[ "$APP_STATUS" == "201" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [OAPP-001] Create OAuth application (HTTP 201)\n"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [OAPP-001] Create OAuth application (Expected 201, Got %s)\n" "$APP_STATUS"
        echo "  Response: $(echo "$APP_BODY" | head -c 200)"
    fi
fi

# ============================================================
# 17. CONNECTIONS
# ============================================================
printf "\n${CYAN}=== 17. CONNECTIONS ===${NC}\n"

run_test "CONN-015" "List connections without auth" "401" \
    "${API}/api/organizations/${TENANT_ID}/connections"

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "CONN-001" "List org connections (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/connections"

    run_test "CONN-012" "Get auth methods" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/connections/auth-methods"

    run_test "CONN-013" "List all platform connections (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/admin/connections"
fi

# ============================================================
# 18. WEBHOOKS
# ============================================================
printf "\n${CYAN}=== 18. WEBHOOKS ===${NC}\n"

run_test_body "WH-012" "List webhook event types (public)" "200" "user" \
    "${API}/api/webhooks/event-types"

run_test "WH-003" "Create webhook without admin" "403" \
    -X POST "${API}/api/organizations/${TENANT_ID}/webhooks" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"url":"https://example.com/webhook","events":["user.created"]}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "WH-005" "List webhooks (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/webhooks"

    WH_RESP=$(api -X POST "${API}/api/organizations/${TENANT_ID}/webhooks" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"url":"https://httpbin.org/post","events":["user.created","user.updated"],"description":"Test webhook"}' 2>/dev/null) || true
    WH_STATUS=$(get_status "$WH_RESP")
    TOTAL=$((TOTAL + 1))
    if [[ "$WH_STATUS" == "201" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [WH-001] Create webhook (HTTP 201)\n"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [WH-001] Create webhook (Expected 201, Got %s)\n" "$WH_STATUS"
        echo "  Response: $(echo "$(get_body "$WH_RESP")" | head -c 200)"
    fi
fi

# ============================================================
# 19. ACTIONS/HOOKS
# ============================================================
printf "\n${CYAN}=== 19. ACTIONS/HOOKS ===${NC}\n"

run_test "ACT-003" "Create action without admin" "403" \
    -X POST "${API}/api/organizations/${TENANT_ID}/actions" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"name":"test-action"}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "ACT-004" "List actions (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/actions"
fi

# ============================================================
# 20. SCIM DISCOVERY
# ============================================================
printf "\n${CYAN}=== 20. SCIM 2.0 ===${NC}\n"

run_test_body "SCIM-DISC-001" "SCIM Service Provider Config" "200" "patch" \
    "${API}/scim/v2/ServiceProviderConfig"

run_test_body "SCIM-DISC-002" "SCIM Resource Types" "200" "User" \
    "${API}/scim/v2/ResourceTypes"

run_test_body "SCIM-DISC-003" "SCIM Schemas" "200" "schema" \
    "${API}/scim/v2/Schemas"

# SCIM without auth
run_test "SCIM-USR-015" "SCIM Users without auth" "401" \
    "${API}/scim/v2/Users"

run_test "SCIM-USR-016" "SCIM Users with invalid token" "401" \
    -H "Authorization: Bearer invalid_scim_token" \
    "${API}/scim/v2/Users"

# ============================================================
# 21. AUDIT LOGS
# ============================================================
printf "\n${CYAN}=== 21. AUDIT LOGS ===${NC}\n"

run_test "AUDIT-011" "Audit logs without auth" "401" \
    "${API}/api/audit/logs"

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "AUDIT-001" "Query audit logs" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/audit/logs"

    run_test "AUDIT-007" "Get security events" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/audit/security-events"

    run_test "AUDIT-010" "Get audit stats" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/audit/stats"
fi

# ============================================================
# 22. EMAIL TEMPLATES
# ============================================================
printf "\n${CYAN}=== 22. EMAIL TEMPLATES ===${NC}\n"

run_test "ETPL-009" "Update template without admin" "403" \
    -X PUT "${API}/api/organizations/${TENANT_ID}/email-templates/verification" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"subject":"Test","html_body":"<p>Test</p>"}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "ETPL-001" "List email templates (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/email-templates"
fi

# ============================================================
# 23. BRANDING & SECURITY SETTINGS
# ============================================================
printf "\n${CYAN}=== 23. BRANDING & SECURITY ===${NC}\n"

run_test "BRAND-003" "Update branding without admin" "403" \
    -X PUT "${API}/api/organizations/${TENANT_ID}/branding" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"app_name":"Test"}'

run_test "SEC-006" "Update security without admin" "403" \
    -X PUT "${API}/api/organizations/${TENANT_ID}/security" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"mfa_required":true}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test_body "BRAND-001" "Get branding (admin)" "200" "" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/branding"

    run_test "SEC-001" "Get security settings (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/security"
fi

# ============================================================
# 24. FGA STORES
# ============================================================
printf "\n${CYAN}=== 24. FGA STORES ===${NC}\n"

run_test "STORE-003" "Create store without auth" "401" \
    -X POST "${API}/api/fga/stores" \
    -H "Content-Type: application/json" \
    -d '{"name":"test-store"}'

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "STORE-004" "List FGA stores" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/fga/stores"
fi

# ============================================================
# 25. AUTHZ TUPLES
# ============================================================
printf "\n${CYAN}=== 25. AUTHORIZATION TUPLES ===${NC}\n"

run_test "FGA-TUP-010" "Create tuple without auth" "401" \
    -X POST "${API}/api/authz/tuples" \
    -H "Content-Type: application/json" \
    -d '{"namespace":"test","object_id":"1","relation":"viewer","subject_type":"user","subject_id":"1"}'

run_test "FGA-CHK-007" "Check permission without auth" "401" \
    -X POST "${API}/api/authz/check" \
    -H "Content-Type: application/json" \
    -d '{"namespace":"test","object_id":"1","relation":"viewer","subject_type":"user","subject_id":"1"}'

# ============================================================
# 26. TENANT REGISTRY (ADMIN)
# ============================================================
printf "\n${CYAN}=== 26. TENANT REGISTRY ===${NC}\n"

run_test "TREG-014" "Registry API without auth" "401" \
    "${API}/api/admin/tenants"

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "TREG-001" "List all tenants (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/admin/tenants"

    run_test "TREG-006" "Get router stats" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/admin/tenants/stats"
fi

# ============================================================
# 27. SCIM TOKEN MANAGEMENT
# ============================================================
printf "\n${CYAN}=== 27. SCIM TOKEN MANAGEMENT ===${NC}\n"

run_test "SCIM-TOK-004" "Create SCIM token without admin" "403" \
    -X POST "${API}/api/organizations/${TENANT_ID}/scim/tokens" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"description":"test token"}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "SCIM-TOK-002" "List SCIM tokens (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/organizations/${TENANT_ID}/scim/tokens"
fi

# ============================================================
# 28. RATE LIMITING CONFIG
# ============================================================
printf "\n${CYAN}=== 28. RATE LIMITING ===${NC}\n"

run_test "RATE-003" "Update rate limit without admin" "403" \
    -X PUT "${API}/api/tenants/${TENANT_ID}/rate-limits" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"login":{"max_attempts":10,"window_seconds":60}}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test "RATE-001" "Get rate limit config (admin)" "200" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/tenants/${TENANT_ID}/rate-limits"
fi

# ============================================================
# 29. PASSWORDLESS AUTH
# ============================================================
printf "\n${CYAN}=== 29. PASSWORDLESS ===${NC}\n"

run_test "PWLESS-008" "Start passwordless with fake tenant" "404" \
    -X POST "${API}/api/tenants/00000000-0000-0000-0000-000000000000/passwordless/start" \
    -H "Content-Type: application/json" \
    -d '{"email":"test@test.com","method":"magic_link"}'

run_test "PWLESS-009" "Start passwordless with invalid email" "400" \
    -X POST "${API}/api/tenants/${TENANT_ID}/passwordless/start" \
    -H "Content-Type: application/json" \
    -d '{"email":"not-an-email","method":"magic_link"}'

run_test "PWLESS-006" "Verify passwordless with expired token" "400" \
    -X POST "${API}/api/tenants/${TENANT_ID}/passwordless/verify" \
    -H "Content-Type: application/json" \
    -d '{"token":"expired_fake_token"}'

# ============================================================
# 30. MFA ENDPOINTS
# ============================================================
printf "\n${CYAN}=== 30. MFA ===${NC}\n"

run_test "MFA-TOTP-005" "Enroll TOTP without auth" "401" \
    -X POST "${API}/api/mfa/enroll/totp"

run_test "MFA-SMS-006" "Enroll SMS without auth" "401" \
    -X POST "${API}/api/mfa/enroll/sms"

run_test "MFA-MGT-006" "Regenerate backup codes without auth" "401" \
    -X POST "${API}/api/mfa/backup-codes/regenerate"

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "MFA-MGT-001" "List MFA methods" "200" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        "${API}/api/mfa/methods"
fi

# ============================================================
# 31. FORWARD AUTH
# ============================================================
printf "\n${CYAN}=== 31. FORWARD AUTH ===${NC}\n"

run_test "FGA-FWD-004" "Forward auth with missing headers" "400" \
    -X POST "${API}/authz/forward-auth" \
    -H "Content-Type: application/json" \
    -d '{}'

# ============================================================
# 32. PROXY INTEGRATION
# ============================================================
printf "\n${CYAN}=== 32. PROXY INTEGRATION ===${NC}\n"

# Test proxy without session redirects to login
PROXY_RESP=$(curl -s -o /dev/null -w "%{http_code}" -L --max-redirs 0 http://localhost:4000/dashboard 2>/dev/null) || true
TOTAL=$((TOTAL + 1))
if [[ "$PROXY_RESP" == "302" ]] || [[ "$PROXY_RESP" == "303" ]]; then
    PASS=$((PASS + 1))
    printf "${GREEN}PASS${NC} [PROXY-002] Protected route without session redirects (HTTP %s)\n" "$PROXY_RESP"
else
    FAIL=$((FAIL + 1))
    printf "${RED}FAIL${NC} [PROXY-002] Protected route without session (Expected 302/303, Got %s)\n" "$PROXY_RESP"
fi

# ============================================================
# 33. CHANGE PASSWORD
# ============================================================
printf "\n${CYAN}=== 33. CHANGE PASSWORD ===${NC}\n"

run_test "AUTH-CHPWD-005" "Change password without auth" "401" \
    -X POST "${API}/api/auth/change-password" \
    -H "Content-Type: application/json" \
    -d '{"current_password":"old","new_password":"new"}'

if [ -n "$ACCESS_TOKEN" ]; then
    run_test "AUTH-CHPWD-002" "Change password with wrong current" "401" \
        -X POST "${API}/api/auth/change-password" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"current_password":"WrongPassword!@#","new_password":"NewPass123!@#"}'

    run_test "AUTH-CHPWD-003" "Change password with short new password" "400" \
        -X POST "${API}/api/auth/change-password" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"current_password\":\"${TEST_PASSWORD}\",\"new_password\":\"123\"}"
fi

# ============================================================
# 34. LOGOUT
# ============================================================
printf "\n${CYAN}=== 34. LOGOUT ===${NC}\n"

run_test "AUTH-LOGOUT-004" "Logout without token" "401" \
    -X POST "${API}/api/auth/logout"

# Logout should be one of the last tests since it invalidates the token
if [ -n "$ACCESS_TOKEN" ]; then
    run_test "AUTH-LOGOUT-001" "Logout with valid token" "200" \
        -X POST "${API}/api/auth/logout" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\":\"${REFRESH_TOKEN}\"}"

    # Verify token is invalid after logout
    sleep 1
    POSTLOGOUT_RESP=$(api -H "Authorization: Bearer ${ACCESS_TOKEN}" "${API}/api/auth/me" 2>/dev/null) || true
    POSTLOGOUT_STATUS=$(get_status "$POSTLOGOUT_RESP")
    TOTAL=$((TOTAL + 1))
    if [[ "$POSTLOGOUT_STATUS" == "401" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [AUTH-LOGOUT-002] Token invalid after logout (HTTP 401)\n"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [AUTH-LOGOUT-002] Token should be invalid after logout (Got HTTP %s)\n" "$POSTLOGOUT_STATUS"
    fi
fi

# ============================================================
# SUMMARY
# ============================================================
echo ""
echo "============================================================"
echo " TEST RESULTS SUMMARY"
echo "============================================================"
printf " ${GREEN}PASSED:  %d${NC}\n" "$PASS"
printf " ${RED}FAILED:  %d${NC}\n" "$FAIL"
printf " ${YELLOW}SKIPPED: %d${NC}\n" "$SKIP"
echo " TOTAL:   $TOTAL"
echo ""
if [ "$FAIL" -eq 0 ]; then
    printf " ${GREEN}ALL TESTS PASSED!${NC}\n"
else
    printf " ${RED}%d TESTS FAILED${NC}\n" "$FAIL"
fi
echo "============================================================"
exit $FAIL
