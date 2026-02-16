# Testing Guide

## Overview

CoreAuth includes backend unit tests (Rust), integration test suites (bash scripts), and comprehensive test case documentation.

## Backend Tests (Rust)

```bash
cd coreauth-core

# Run all tests
cargo test

# Run a specific test with output
cargo test test_name -- --nocapture

# Run with logging
RUST_LOG=debug cargo test

# Offline mode (no database)
SQLX_OFFLINE=true cargo test
```

## Integration Tests (Bash)

The `tests/` directory contains bash-based integration tests that run against a live Docker stack.

### Running Integration Tests

```bash
# Start the full stack first
docker compose up -d

# Run all test suites
bash tests/run_tests.sh

# Run a specific suite
bash tests/suites/01_health.sh
bash tests/suites/02_auth_register.sh
bash tests/suites/03_auth_login.sh
```

### Test Suites

| Suite | Description |
| ----- | ----------- |
| `01_health.sh` | Health check endpoints |
| `02_auth_register.sh` | User registration flows |
| `03_auth_login.sh` | Login, token refresh, logout |

### Test Helpers

Tests use helper functions from `tests/helpers.sh`:

- `api()` - Make HTTP requests to the backend API
- `run_test()` - Run a test and check HTTP status code
- `run_test_body()` - Run a test and check response body content

## Test Coverage Areas

- Authentication flows (register, login, refresh, logout)
- MFA enrollment and verification (TOTP, SMS, backup codes)
- Passwordless authentication (magic links, OTP)
- Multi-tenant isolation
- Role-based access control
- JWT token validation
- Password hashing and policies
- OAuth2/OIDC flows
- FGA permission checks
- SCIM provisioning
- Webhook delivery
- Email/SMS delivery (mocked in tests)
- Session management
- Audit logging

## Comprehensive Test Cases

See `tests/COMPREHENSIVE_TEST_CASES.md` for the full test plan with detailed scenarios.

## Debugging Tests

```bash
# Verbose output
cargo test -- --nocapture

# Filter specific tests
cargo test test_mfa_enrollment -- --nocapture

# Full trace logging
RUST_LOG=trace cargo test
```

## Docker-Based Testing

```bash
# Build test image
docker build -f coreauth-core/Dockerfile.test -t coreauth-test coreauth-core/

# Run tests in container
docker run --rm coreauth-test
```
