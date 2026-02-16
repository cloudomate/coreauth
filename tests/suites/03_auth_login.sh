#!/bin/bash
# Suite: Authentication — Login, Refresh, Profile, Password, Logout
section "AUTHENTICATION — LOGIN"

# Positive login (use admin since test user may be unverified)
run_test_body "AUTH-LOGIN-001" "Login with valid admin credentials" "200" "access_token" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"password\":\"${KNOWN_ADMIN_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-002" "Login with wrong password" "401" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"password\":\"WrongPassword\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test "AUTH-LOGIN-003" "Login with non-existent email (no enumeration)" "401" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"noone_$(date +%s)@void.dev\",\"password\":\"TestPass123!\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test_status_any "AUTH-LOGIN-006" "Login with empty body" "400 422 429" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{}"

run_test_status_any "AUTH-LOGIN-012" "Login with SQL injection in email" "401 429" \
    -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"' OR 1=1 --\",\"password\":\"x\",\"tenant_id\":\"${TENANT_ID}\"}"

# Check no password leak
LOGIN_RESP=$(api -X POST "${API}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${KNOWN_ADMIN_EMAIL}\",\"password\":\"${KNOWN_ADMIN_PASSWORD}\",\"tenant_id\":\"${TENANT_ID}\"}" 2>/dev/null) || true
LOGIN_BODY=$(get_body "$LOGIN_RESP")

TOTAL=$((TOTAL + 1))
if echo "$LOGIN_BODY" | grep -q "password_hash\|password_digest"; then
    FAIL=$((FAIL + 1))
    printf "${RED}FAIL${NC} [AUTH-LOGIN-013] Login response leaks password hash\n"
else
    PASS=$((PASS + 1))
    printf "${GREEN}PASS${NC} [AUTH-LOGIN-013] Login response does not leak password hash\n"
fi

# ── Token Refresh ────────────────────────────────────────────
section "AUTHENTICATION — TOKEN REFRESH"

if [ -n "$USER_REFRESH" ]; then
    run_test_body "AUTH-REFRESH-001" "Refresh with valid token" "200" "access_token" \
        -X POST "${API}/api/auth/refresh" \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\":\"${USER_REFRESH}\"}"
else
    skip_test "AUTH-REFRESH-001" "Refresh with valid token" "No refresh token"
fi

run_test "AUTH-REFRESH-003" "Refresh with invalid token" "401" \
    -X POST "${API}/api/auth/refresh" \
    -H "Content-Type: application/json" \
    -d '{"refresh_token":"invalid_garbage_token"}'

run_test "AUTH-REFRESH-004" "Refresh with empty token" "401" \
    -X POST "${API}/api/auth/refresh" \
    -H "Content-Type: application/json" \
    -d '{"refresh_token":""}'

# ── Profile (me) ─────────────────────────────────────────────
section "AUTHENTICATION — PROFILE"

if [ -n "$ADMIN_TOKEN" ]; then
    run_test_body "AUTH-ME-001" "Get profile with valid token" "200" "email" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${API}/api/auth/me"

    # No password leak
    ME_RESP=$(api -H "Authorization: Bearer ${ADMIN_TOKEN}" "${API}/api/auth/me" 2>/dev/null) || true
    TOTAL=$((TOTAL + 1))
    if get_body "$ME_RESP" | grep -q "password_hash"; then
        FAIL=$((FAIL + 1)); printf "${RED}FAIL${NC} [AUTH-ME-008] Profile exposes password hash\n"
    else
        PASS=$((PASS + 1)); printf "${GREEN}PASS${NC} [AUTH-ME-008] Profile does not expose password\n"
    fi
else
    skip_test "AUTH-ME-001" "Get profile" "No token"
    skip_test "AUTH-ME-008" "Password leak check" "No token"
fi

run_test "AUTH-ME-002" "Get profile without auth" "401" "${API}/api/auth/me"
run_test "AUTH-ME-003" "Get profile with bad token" "401" \
    -H "Authorization: Bearer invalid" "${API}/api/auth/me"
run_test "AUTH-ME-004" "Get profile with malformed JWT" "401" \
    -H "Authorization: Bearer not.a.jwt" "${API}/api/auth/me"

# ── Change Password ──────────────────────────────────────────
section "AUTHENTICATION — CHANGE PASSWORD"

run_test "AUTH-CHPWD-005" "Change password without auth" "401" \
    -X POST "${API}/api/auth/change-password" \
    -H "Content-Type: application/json" \
    -d '{"current_password":"x","new_password":"y"}'

if [ -n "$ADMIN_TOKEN" ]; then
    run_test_status_any "AUTH-CHPWD-002" "Change password with wrong current" "400 401 403" \
        -X POST "${API}/api/auth/change-password" \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"current_password":"WrongPassword!@#","new_password":"NewPass123!@#"}'
fi

# ── Logout ───────────────────────────────────────────────────
section "AUTHENTICATION — LOGOUT"

run_test "AUTH-LOGOUT-004" "Logout without token" "401" \
    -X POST "${API}/api/auth/logout"
