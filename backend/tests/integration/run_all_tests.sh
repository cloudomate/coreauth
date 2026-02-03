#!/bin/bash

# Master test runner for hierarchical authentication integration tests

set -e

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/helpers.sh"

print_test_header "Hierarchical Authentication Integration Tests"

# Wait for services to be ready
wait_for_db || exit 1
wait_for_api || exit 1

print_info "All services are ready, starting tests..."
echo ""

# Seed test data via API
print_info "Seeding test data..."
bash "$SCRIPT_DIR/seed_test_data.sh" || {
    print_error "Failed to seed test data"
    exit 1
}
echo ""

# Initialize test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Run test suites
TEST_SUITES=(
    "test_hierarchical_login.sh"
    "test_error_cases.sh"
    "test_backward_compatibility.sh"
)

for suite in "${TEST_SUITES[@]}"; do
    if [ -f "$SCRIPT_DIR/$suite" ]; then
        print_info "Running test suite: $suite"

        # Run test suite and capture exit code
        if bash "$SCRIPT_DIR/$suite"; then
            print_success "Test suite $suite completed"
        else
            print_error "Test suite $suite had failures"
        fi

        echo ""
    else
        print_error "Test suite not found: $suite"
    fi
done

# Print final summary
print_test_summary

# Exit with appropriate code
if [ $TESTS_FAILED -eq 0 ]; then
    exit 0
else
    exit 1
fi
