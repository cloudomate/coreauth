#!/bin/bash
# Suite: Health & Connectivity
section "HEALTH & CONNECTIVITY"

run_test_body "HEALTH-001" "Backend health check" "200" "healthy" \
    "${API}/health"

run_test_body "OAUTH-DISC-001" "OpenID Configuration" "200" "authorization_endpoint" \
    "${API}/.well-known/openid-configuration"

run_test_body "OAUTH-DISC-003" "JWKS endpoint" "200" "keys" \
    "${API}/.well-known/jwks.json"
