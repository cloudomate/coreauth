#!/bin/bash

# Test helpers and utilities

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# API configuration
API_HOST="${API_HOST:-http://api:8000}"
POSTGRES_HOST="${POSTGRES_HOST:-postgres}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_DB="${POSTGRES_DB:-ciam}"
POSTGRES_USER="${POSTGRES_USER:-ciam}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-ciam_password}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Print functions
print_test_header() {
    echo ""
    echo "=========================================="
    echo "$1"
    echo "=========================================="
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Wait for API to be ready
wait_for_api() {
    print_info "Waiting for API to be ready..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -sf "${API_HOST}/health" > /dev/null 2>&1; then
            print_success "API is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    print_error "API failed to start after ${max_attempts} seconds"
    return 1
}

# Wait for database to be ready
wait_for_db() {
    print_info "Waiting for database to be ready..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if PGPASSWORD=$POSTGRES_PASSWORD psql -h $POSTGRES_HOST -p $POSTGRES_PORT -U $POSTGRES_USER -d $POSTGRES_DB -c "SELECT 1" > /dev/null 2>&1; then
            print_success "Database is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    print_error "Database failed to start after ${max_attempts} seconds"
    return 1
}

# Test assertion functions
assert_equals() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"

    TESTS_RUN=$((TESTS_RUN + 1))

    if [ "$expected" == "$actual" ]; then
        print_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        echo "  Expected: $expected"
        echo "  Actual:   $actual"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

assert_http_status() {
    local expected_status="$1"
    local response_file="$2"
    local test_name="$3"

    local actual_status=$(jq -r '.status' "$response_file")

    TESTS_RUN=$((TESTS_RUN + 1))

    if [ "$expected_status" == "$actual_status" ]; then
        print_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        echo "  Expected HTTP status: $expected_status"
        echo "  Actual HTTP status:   $actual_status"
        echo "  Response body:"
        jq '.' "$response_file" 2>/dev/null || cat "$response_file"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

assert_json_field() {
    local field_path="$1"
    local expected_value="$2"
    local response_file="$3"
    local test_name="$4"

    local actual_value=$(jq -r "$field_path" "$response_file")

    TESTS_RUN=$((TESTS_RUN + 1))

    if [ "$expected_value" == "$actual_value" ]; then
        print_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        echo "  Field: $field_path"
        echo "  Expected: $expected_value"
        echo "  Actual:   $actual_value"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

assert_not_null() {
    local field_path="$1"
    local response_file="$2"
    local test_name="$3"

    local actual_value=$(jq -r "$field_path" "$response_file")

    TESTS_RUN=$((TESTS_RUN + 1))

    if [ "$actual_value" != "null" ] && [ -n "$actual_value" ]; then
        print_success "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        echo "  Field: $field_path"
        echo "  Expected: non-null value"
        echo "  Actual:   $actual_value"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# HTTP request helper with status code capture
http_request() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local output_file="$4"
    local auth_token="$5"

    local status_code
    local headers=(-H "Content-Type: application/json")

    if [ -n "$auth_token" ]; then
        headers+=(-H "Authorization: Bearer $auth_token")
    fi

    if [ "$method" == "GET" ]; then
        status_code=$(curl -s -w "%{http_code}" -o "$output_file.body" \
            "${headers[@]}" \
            "${API_HOST}${endpoint}")
    else
        status_code=$(curl -s -w "%{http_code}" -o "$output_file.body" \
            -X "$method" \
            "${headers[@]}" \
            -d "$data" \
            "${API_HOST}${endpoint}")
    fi

    # Combine status and body into response file
    echo "{\"status\": $status_code, \"body\": $(cat "$output_file.body")}" > "$output_file"
    rm -f "$output_file.body"

    return 0
}

# Decode JWT payload (without verification)
decode_jwt_payload() {
    local jwt="$1"
    echo "$jwt" | cut -d'.' -f2 | base64 -d 2>/dev/null | jq '.'
}

# Print test summary
print_test_summary() {
    echo ""
    echo "=========================================="
    echo "Test Summary"
    echo "=========================================="
    echo "Total tests:  $TESTS_RUN"
    echo -e "Passed:       ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed:       ${RED}$TESTS_FAILED${NC}"
    echo "=========================================="

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

# Database query helper
db_query() {
    local query="$1"
    PGPASSWORD=$POSTGRES_PASSWORD psql -h $POSTGRES_HOST -p $POSTGRES_PORT -U $POSTGRES_USER -d $POSTGRES_DB -t -c "$query"
}
