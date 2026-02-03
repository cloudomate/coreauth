# Testing Guide

## Overview

CoreAuth includes comprehensive testing coverage across unit tests, integration tests, and end-to-end tests.

## Running Tests

### Backend Tests

```bash
cd backend

# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests

# Run with logging
RUST_LOG=debug cargo test

# Run tests in Docker
docker compose exec backend cargo test
```

### Frontend Tests

```bash
cd frontend

# Run tests (when implemented)
npm test

# Run e2e tests
npm run test:e2e
```

## Test Structure

### Backend Tests

```
backend/
├── tests/
│   ├── integration/          # Integration tests
│   │   ├── auth_tests.rs     # Authentication flows
│   │   ├── mfa_tests.rs      # MFA enrollment & verification
│   │   ├── tenant_tests.rs   # Multi-tenant operations
│   │   └── hierarchy_tests.rs # Organizational hierarchy
│   └── unit/                 # Unit tests (in each module)
```

### Key Test Cases

1. **Authentication**
   - User registration
   - Login with email/password
   - MFA enrollment and verification
   - Token refresh
   - Logout

2. **Multi-Tenancy**
   - Tenant creation
   - User-tenant associations
   - Tenant isolation
   - Cross-tenant access prevention

3. **Authorization**
   - Role-based access control
   - Permission checks
   - Hierarchical permissions
   - Tuple-based authorization

4. **MFA**
   - TOTP enrollment
   - QR code generation
   - Code verification
   - Backup codes
   - Enrollment token flow

## Integration Testing with Docker

### Setup

```bash
# Start test database
docker compose up -d postgres redis

# Run migrations
docker compose exec backend ./migrations/run.sh

# Run integration tests
docker compose exec backend cargo test --test integration_tests
```

### Test Database

Tests use a separate database schema to avoid conflicts:

```sql
CREATE DATABASE coreauth_test;
```

Set `TEST_DATABASE_URL` in your environment for test isolation.

## Manual Testing Workflows

### 1. Complete User Signup & MFA Flow

```bash
# 1. Start services
./docker-start.sh

# 2. Create organization via API or UI
curl -X POST http://localhost:8000/api/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Company",
    "slug": "testco",
    "admin_email": "admin@testco.com",
    "admin_password": "SecurePass123!"
  }'

# 3. Enable MFA requirement in Security settings

# 4. Login - should redirect to MFA setup
# 5. Scan QR code with authenticator app
# 6. Enter verification code
# 7. Save backup codes
# 8. Successfully logged in
```

### 2. Multi-Tenant Isolation Test

```bash
# Create Tenant A
# Create Tenant B
# Create user in Tenant A
# Verify user cannot access Tenant B resources
```

### 3. Hierarchical Organization Test

```bash
# Create parent organization
# Create child organization
# Verify hierarchy relationships
# Test permission inheritance
```

## Test Coverage

Current coverage areas:
- ✅ Authentication flows
- ✅ MFA enrollment and verification
- ✅ Multi-tenant isolation
- ✅ Role-based access control
- ✅ JWT token validation
- ✅ Password hashing and validation
- ⚠️ Email delivery (mocked in tests)
- ⚠️ SMS delivery (mocked in tests)
- 🚧 WebAuthn (planned)
- 🚧 OAuth2 flows (planned)

## Debugging Tests

### View Test Logs

```bash
# Verbose test output
cargo test -- --nocapture

# Filter specific tests
cargo test test_mfa_enrollment -- --nocapture

# Show all logs
RUST_LOG=trace cargo test
```

### Database Inspection During Tests

```bash
# Connect to test database
docker compose exec postgres psql -U coreauth -d coreauth_test

# View test data
SELECT * FROM users;
SELECT * FROM mfa_methods;
```

## Performance Testing

### Load Testing

```bash
# Install Apache Bench
apt-get install apache2-utils

# Test login endpoint
ab -n 1000 -c 10 -p login.json -T application/json \
  http://localhost:8000/api/auth/login

# Test with authentication
ab -n 1000 -c 10 -H "Authorization: Bearer TOKEN" \
  http://localhost:8000/api/auth/me
```

### Benchmark Tests

```bash
cd backend
cargo bench
```

## Continuous Integration

Tests run automatically on:
- Pull requests
- Commits to main branch
- Nightly builds

See `.github/workflows/tests.yml` for CI configuration.
