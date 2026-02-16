#!/bin/bash
# ============================================================
# CoreAuth Test Helpers — sourced by all test suites
# ============================================================

# Counters (exported so suites can increment)
export PASS=${PASS:-0}
export FAIL=${FAIL:-0}
export SKIP=${SKIP:-0}
export TOTAL=${TOTAL:-0}

# API base URL (internal to Docker)
export API="http://localhost:3000"
export PROXY="http://localhost:4000"

# Known tenant credentials (from bootstrap .env)
export KNOWN_TENANT_SLUG="acme"
export KNOWN_ADMIN_EMAIL="admin@acme.dev"
export KNOWN_ADMIN_PASSWORD="AcmeAdmin123!"

# Colors
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export CYAN='\033[0;36m'
export BOLD='\033[1m'
export NC='\033[0m'

# ── Curl wrapper: runs curl inside the backend container ─────
api() {
    docker exec corerun-backend curl -s -w "\n%{http_code}" "$@" 2>/dev/null
}

# ── Response parsing ─────────────────────────────────────────
get_status() { echo "$1" | tail -1; }
get_body()   { echo "$1" | sed '$d'; }
json_field()  { echo "$1" | python3 -c "import sys,json; print(json.load(sys.stdin).get('$2',''))" 2>/dev/null || echo ""; }

# ── Test runners ─────────────────────────────────────────────
# run_test ID DESC EXPECTED_HTTP curl_args...
run_test() {
    local id="$1" desc="$2" expected_status="$3"
    shift 3

    TOTAL=$((TOTAL + 1))
    local response status body
    response=$(api "$@" 2>/dev/null) || true
    status=$(get_status "$response")
    body=$(get_body "$response")

    if [[ "$status" == "$expected_status" ]]; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [%s] %s (HTTP %s)\n" "$id" "$desc" "$status"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [%s] %s (Expected %s, Got %s)\n" "$id" "$desc" "$expected_status" "$status"
        echo "  Response: $(echo "$body" | head -c 200)"
    fi
}

# run_test_body ID DESC EXPECTED_HTTP BODY_SUBSTRING curl_args...
run_test_body() {
    local id="$1" desc="$2" expected_status="$3" body_check="$4"
    shift 4

    TOTAL=$((TOTAL + 1))
    local response status body
    response=$(api "$@" 2>/dev/null) || true
    status=$(get_status "$response")
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

# run_test_status_any ID DESC "200|201|204" curl_args...
run_test_status_any() {
    local id="$1" desc="$2" expected_any="$3"
    shift 3

    TOTAL=$((TOTAL + 1))
    local response status body
    response=$(api "$@" 2>/dev/null) || true
    status=$(get_status "$response")
    body=$(get_body "$response")

    if echo "$expected_any" | grep -qw "$status"; then
        PASS=$((PASS + 1))
        printf "${GREEN}PASS${NC} [%s] %s (HTTP %s)\n" "$id" "$desc" "$status"
    else
        FAIL=$((FAIL + 1))
        printf "${RED}FAIL${NC} [%s] %s (Expected one of [%s], Got %s)\n" "$id" "$desc" "$expected_any" "$status"
        echo "  Response: $(echo "$body" | head -c 200)"
    fi
}

skip_test() {
    local id="$1" desc="$2" reason="$3"
    TOTAL=$((TOTAL + 1))
    SKIP=$((SKIP + 1))
    printf "${YELLOW}SKIP${NC} [%s] %s (%s)\n" "$id" "$desc" "$reason"
}

# ── Section header ───────────────────────────────────────────
section() {
    printf "\n${CYAN}=== %s ===${NC}\n" "$1"
}

# ── Require a variable or skip the rest of the suite ─────────
require_var() {
    local name="$1" value="$2"
    if [ -z "$value" ]; then
        echo "  WARNING: ${name} is empty — some tests will be skipped"
        return 1
    fi
    return 0
}

# ── Bootstrap: discover tenant + get tokens ──────────────────
bootstrap_tokens() {
    # 1. Discover tenant ID
    local resp body
    resp=$(api "${API}/api/organizations/by-slug/${KNOWN_TENANT_SLUG}" 2>/dev/null) || true
    body=$(get_body "$resp")
    TENANT_ID=$(json_field "$body" "id")

    if [ -z "$TENANT_ID" ]; then
        TENANT_ID=$(docker exec corerun-postgres psql -U coreauth -d coreauth -t \
            -c "SELECT id FROM tenants WHERE slug='${KNOWN_TENANT_SLUG}' LIMIT 1;" 2>/dev/null | tr -d ' ')
    fi
    export TENANT_ID
    echo "  Tenant ID: ${TENANT_ID:-UNKNOWN}"

    if [ -z "$TENANT_ID" ]; then
        echo "  FATAL: Could not determine tenant ID."
        return 1
    fi

    # 2. Register a throwaway test user
    TEST_EMAIL="testuser_$(date +%s)@coreauth.test"
    TEST_PASSWORD="TestPass123!@#"
    export TEST_EMAIL TEST_PASSWORD

    resp=$(api -X POST "${API}/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Test\",\"last_name\":\"User\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
    body=$(get_body "$resp")
    USER_TOKEN=$(json_field "$body" "access_token")
    USER_REFRESH=$(json_field "$body" "refresh_token")
    export USER_TOKEN USER_REFRESH

    # 3. Login as admin
    resp=$(api -X POST "${API}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"password\":\"${KNOWN_ADMIN_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
    body=$(get_body "$resp")
    ADMIN_TOKEN=$(json_field "$body" "access_token")
    export ADMIN_TOKEN

    echo "  Test user token: ${USER_TOKEN:+OK}${USER_TOKEN:-MISSING}"
    echo "  Admin token:     ${ADMIN_TOKEN:+OK}${ADMIN_TOKEN:-MISSING}"
}

# ── Summary printer ──────────────────────────────────────────
print_summary() {
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
        printf " ${RED}%d TEST(S) FAILED${NC}\n" "$FAIL"
    fi
    echo "============================================================"
}
