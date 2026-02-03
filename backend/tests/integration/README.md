# Integration Tests

This directory contains integration tests for the CIAM hierarchical authentication system.

## Quick Start

From the project root:

```bash
# Start test environment
make test-up

# Run all tests
make test-run

# Stop test environment
make test-down
```

## Test Files

- **`helpers.sh`** - Common test utilities, assertions, HTTP helpers
- **`test_hierarchical_login.sh`** - Tests for hierarchical authentication endpoint
- **`test_error_cases.sh`** - Error handling and security tests
- **`test_backward_compatibility.sh`** - Legacy endpoint compatibility tests
- **`run_all_tests.sh`** - Master test runner

## Running Individual Tests

```bash
# Hierarchical login tests only
make test-hierarchical

# Error cases only
make test-errors

# Backward compatibility only
make test-compat
```

## Test Assertions

The test framework provides these assertion functions:

```bash
# Assert HTTP status code
assert_http_status "200" "$response_file" "Test name"

# Assert JSON field value
assert_json_field ".body.email" "user@example.com" "$response_file" "Test name"

# Assert field is not null
assert_not_null ".body.access_token" "$response_file" "Test name"

# Assert equality
assert_equals "expected" "actual" "Test name"
```

## Helper Functions

```bash
# HTTP request with status capture
http_request "POST" "/api/endpoint" '{"key":"value"}' "$output_file" "$optional_auth_token"

# Decode JWT payload
decode_jwt_payload "$jwt_token" > claims.json

# Database query
db_query "SELECT * FROM users"

# Wait for services
wait_for_api
wait_for_db
```

## Writing New Tests

1. Create a new test file: `test_your_feature.sh`
2. Source the helpers: `source "$SCRIPT_DIR/helpers.sh"`
3. Create test directory: `mkdir -p "$TEST_DIR"`
4. Write test cases using assertions
5. Add to `run_all_tests.sh`

Example:

```bash
#!/bin/bash
set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

TEST_DIR="/tmp/test_your_feature"
mkdir -p "$TEST_DIR"

print_test_header "Your Feature Tests"

print_info "Test 1: Your test description"
http_request "POST" "/api/your-endpoint" \
    '{"data":"value"}' \
    "$TEST_DIR/response.json"

assert_http_status "200" "$TEST_DIR/response.json" \
    "Your endpoint returns 200"

assert_json_field ".body.status" "success" "$TEST_DIR/response.json" \
    "Response has success status"

print_info "Your feature tests completed"
```

## Environment

Tests run in Docker containers with:
- API: `http://api:8000`
- Database: `postgres:5432` (exposed as `localhost:5433`)
- Redis: `redis:6379` (exposed as `localhost:6380`)
- MailHog: `http://mailhog:8025` (exposed as `localhost:8025`)

## Debugging

```bash
# View test logs
make test-logs

# View API logs
make test-api-logs

# Open shell in test container
make test-shell

# Query database
make test-db
```

## See Also

- [TESTING_GUIDE.md](../../../TESTING_GUIDE.md) - Manual testing guide with curl examples
- [DOCKER_TESTING.md](../../../DOCKER_TESTING.md) - Docker Compose setup guide
