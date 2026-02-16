#!/bin/bash
# Suite: Authentication — Registration
section "AUTHENTICATION — REGISTRATION"

# Positive: register a fresh user (already done in bootstrap_tokens, so use another)
REG_EMAIL="reg_$(date +%s)@coreauth.test"

run_test_body "AUTH-REG-001" "Register with valid email/password" "201" "access_token" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${REG_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Reg\",\"last_name\":\"Test\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test_status_any "AUTH-REG-004" "Register with duplicate email" "409 400" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${REG_EMAIL}\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"Dup\",\"last_name\":\"Test\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test_status_any "AUTH-REG-005" "Register with invalid email format" "400 422 429" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"not-an-email\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"X\",\"last_name\":\"Y\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test_status_any "AUTH-REG-007" "Register with password too short" "400 422 429" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"short_$(date +%s)@test.com\",\"password\":\"ab\",\"first_name\":\"X\",\"last_name\":\"Y\",\"tenant_id\":\"${TENANT_ID}\"}"

run_test_status_any "AUTH-REG-009" "Register with empty body" "400 422 429" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{}"

run_test_status_any "AUTH-REG-010" "Register with non-existent tenant" "400 404 429" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"t_$(date +%s)@x.com\",\"password\":\"TestPass123!\",\"first_name\":\"X\",\"last_name\":\"Y\",\"tenant_id\":\"00000000-0000-0000-0000-000000000000\"}"

run_test_status_any "AUTH-REG-013" "Register with SQL injection in email" "400 422 429" \
    -X POST "${API}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"'; DROP TABLE users;--\",\"password\":\"${TEST_PASSWORD}\",\"first_name\":\"X\",\"last_name\":\"Y\",\"tenant_id\":\"${TENANT_ID}\"}"
