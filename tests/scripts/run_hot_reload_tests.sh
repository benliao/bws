#!/bin/bash

# BWS Test Runner - Hot Reload Testing Suite
# Runs both unit tests and integration tests for hot reload functionality

set -e

# Configuration
PROJECT_ROOT="$(dirname "$(dirname "$0")")"
TESTS_DIR="$(dirname "$0")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Test statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Run unit tests
run_unit_tests() {
    log_info "Running hot reload unit tests..."
    
    cd "$PROJECT_ROOT"
    
    if cargo test hot_reload --lib; then
        local unit_test_count=$(cargo test hot_reload --lib 2>&1 | grep -o '[0-9]\+ passed' | head -1 | cut -d' ' -f1)
        log_success "Unit tests passed ($unit_test_count tests)"
        TOTAL_TESTS=$((TOTAL_TESTS + unit_test_count))
        PASSED_TESTS=$((PASSED_TESTS + unit_test_count))
        return 0
    else
        log_error "Unit tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Run integration tests
run_integration_tests() {
    log_info "Running hot reload integration tests..."
    
    cd "$TESTS_DIR"
    
    # Make script executable
    chmod +x test_hot_reload.sh
    
    if ./test_hot_reload.sh; then
        log_success "Integration tests passed"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "Integration tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Build BWS binaries
build_bws() {
    log_info "Building BWS binaries..."
    
    cd "$PROJECT_ROOT"
    
    if cargo build; then
        log_success "BWS binaries built successfully"
        return 0
    else
        log_error "Failed to build BWS binaries"
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is not installed or not in PATH"
        return 1
    fi
    
    # Check if curl is available (needed for integration tests)
    if ! command -v curl &> /dev/null; then
        log_warning "curl is not installed - integration tests may fail"
    fi
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "Not in BWS project directory"
        return 1
    fi
    
    log_success "Prerequisites check passed"
    return 0
}

# Main function
main() {
    echo "=================================================="
    echo "BWS Hot Reload Test Runner"
    echo "=================================================="
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Build BWS
    if ! build_bws; then
        exit 1
    fi
    
    echo ""
    echo "Running test suite..."
    echo ""
    
    # Run unit tests
    run_unit_tests
    unit_result=$?
    
    echo ""
    
    # Run integration tests
    run_integration_tests
    integration_result=$?
    
    echo ""
    echo "=================================================="
    echo "Test Summary"
    echo "=================================================="
    
    if [ $unit_result -eq 0 ]; then
        echo -e "${GREEN}✓${NC} Unit Tests: PASSED"
    else
        echo -e "${RED}✗${NC} Unit Tests: FAILED"
    fi
    
    if [ $integration_result -eq 0 ]; then
        echo -e "${GREEN}✓${NC} Integration Tests: PASSED"
    else
        echo -e "${RED}✗${NC} Integration Tests: FAILED"
    fi
    
    echo ""
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All hot reload tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Check the output above for details.${NC}"
        exit 1
    fi
}

# Check if script is being executed (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
