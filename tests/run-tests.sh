#!/bin/bash

# BWS Test Runner - Comprehensive test suite for BWS Web Server
# Usage: ./run-tests.sh [options]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$TEST_DIR/.." && pwd)"
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Default options
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_SCRIPTS=true
VERBOSE=false
QUICK=false

# Print usage information
usage() {
    cat << EOF
BWS Test Runner

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -q, --quick         Run only quick tests (skip integration)
    --unit-only         Run only unit tests
    --integration-only  Run only integration tests
    --scripts-only      Run only script tests
    --no-unit          Skip unit tests
    --no-integration   Skip integration tests
    --no-scripts       Skip script tests

EXAMPLES:
    $0                  # Run all tests
    $0 --quick          # Quick test run
    $0 --unit-only      # Only unit tests
    $0 --verbose        # Verbose output
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -q|--quick)
                QUICK=true
                RUN_INTEGRATION=false
                shift
                ;;
            --unit-only)
                RUN_UNIT=true
                RUN_INTEGRATION=false
                RUN_SCRIPTS=false
                shift
                ;;
            --integration-only)
                RUN_UNIT=false
                RUN_INTEGRATION=true
                RUN_SCRIPTS=false
                shift
                ;;
            --scripts-only)
                RUN_UNIT=false
                RUN_INTEGRATION=false
                RUN_SCRIPTS=true
                shift
                ;;
            --no-unit)
                RUN_UNIT=false
                shift
                ;;
            --no-integration)
                RUN_INTEGRATION=false
                shift
                ;;
            --no-scripts)
                RUN_SCRIPTS=false
                shift
                ;;
            *)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_skip() {
    echo -e "${YELLOW}⏭️  $1${NC}"
    ((TESTS_SKIPPED++))
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for required commands
    local missing=()
    command -v cargo >/dev/null 2>&1 || missing+=("cargo")
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required commands: ${missing[*]}"
        echo "Please install the missing commands and try again."
        exit 1
    fi
    
    # Check if BWS can be built
    if ! cargo check --bin bws >/dev/null 2>&1; then
        log_error "BWS failed to compile. Please fix compilation errors first."
        exit 1
    fi
    
    # Validate configurations using dry-run
    log_info "Validating configurations..."
    local config_files=(
        "config.toml"
        "tests/configs/basic.toml"
        "tests/configs/config_test.toml"
    )
    
    for config_file in "${config_files[@]}"; do
        if [[ -f "$config_file" ]]; then
            if ! cargo run --bin bws -- --config "$config_file" --dry-run >/dev/null 2>&1; then
                log_error "Configuration validation failed for $config_file"
                exit 1
            fi
        fi
    done
    
    log_success "Prerequisites check passed"
}

# Build BWS
build_bws() {
    log_info "Building BWS..."
    
    cd "$PROJECT_ROOT"
    if $VERBOSE; then
        cargo build --bin bws
    else
        cargo build --bin bws >/dev/null 2>&1
    fi
    
    if [[ $? -eq 0 ]]; then
        log_success "BWS build completed"
    else
        log_error "BWS build failed"
        exit 1
    fi
}

# Run unit tests
run_unit_tests() {
    if ! $RUN_UNIT; then
        return 0
    fi
    
    log_info "Running unit tests..."
    
    cd "$PROJECT_ROOT"
    if $VERBOSE; then
        cargo test --lib
    else
        cargo test --lib >/dev/null 2>&1
    fi
    
    if [[ $? -eq 0 ]]; then
        log_success "Unit tests passed"
    else
        log_error "Unit tests failed"
    fi
}

# Run integration tests
run_integration_tests() {
    if ! $RUN_INTEGRATION; then
        return 0
    fi
    
    log_info "Running integration tests..."
    
    cd "$PROJECT_ROOT"
    if $VERBOSE; then
        cargo test --test '*'
    else
        cargo test --test '*' >/dev/null 2>&1
    fi
    
    if [[ $? -eq 0 ]]; then
        log_success "Integration tests passed"
    else
        log_error "Integration tests failed"
    fi
}

# Run script tests
run_script_tests() {
    if ! $RUN_SCRIPTS; then
        return 0
    fi
    
    log_info "Running script tests..."
    
    # List of script tests to run
    local script_tests=(
        "test_server.sh"
        "test_static_server.sh"
        "test_headers.sh"
        "test_multisite.sh"
    )
    
    if ! $QUICK; then
        script_tests+=(
            "run_hot_reload_tests.sh"
            "test_load_balance.sh"
            "simple_websocket_test.sh"
        )
    fi
    
    cd "$TEST_DIR/scripts"
    
    # Make scripts executable
    chmod +x *.sh 2>/dev/null || true
    
    # Run each script test
    for script in "${script_tests[@]}"; do
        if [[ -f "$script" ]]; then
            log_info "Running $script..."
            
            if $VERBOSE; then
                if bash "$script"; then
                    log_success "$script passed"
                else
                    log_error "$script failed"
                fi
            else
                if bash "$script" >/dev/null 2>&1; then
                    log_success "$script passed"
                else
                    log_error "$script failed"
                fi
            fi
        else
            log_skip "$script not found"
        fi
    done
}

# Print test summary
print_summary() {
    echo
    echo "=========================================="
    echo "             TEST SUMMARY"
    echo "=========================================="
    echo -e "✅ Tests Passed:  ${GREEN}$TESTS_PASSED${NC}"
    echo -e "❌ Tests Failed:  ${RED}$TESTS_FAILED${NC}"
    echo -e "⏭️  Tests Skipped: ${YELLOW}$TESTS_SKIPPED${NC}"
    echo "=========================================="
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "${RED}❌ Some tests failed. Please check the output above.${NC}"
        exit 1
    else
        echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
        exit 0
    fi
}

# Main execution
main() {
    echo "🚀 BWS Test Runner v2.0.0"
    echo "=========================================="
    
    parse_args "$@"
    
    if $VERBOSE; then
        echo "Configuration:"
        echo "  Unit tests: $RUN_UNIT"
        echo "  Integration tests: $RUN_INTEGRATION"
        echo "  Script tests: $RUN_SCRIPTS"
        echo "  Quick mode: $QUICK"
        echo "  Verbose: $VERBOSE"
        echo
    fi
    
    check_prerequisites
    build_bws
    run_unit_tests
    run_integration_tests
    run_script_tests
    print_summary
}

# Run main function with all arguments
main "$@"
