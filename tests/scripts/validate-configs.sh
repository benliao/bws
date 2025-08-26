#!/bin/bash

# BWS Configuration Validator
# Validates all test configurations for syntax and common issues

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_ROOT="$(cd "$TEST_DIR/.." && pwd)"
CONFIGS_PASSED=0
CONFIGS_FAILED=0

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((CONFIGS_PASSED++))
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    ((CONFIGS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Validate a single TOML configuration file
validate_config() {
    local config_file="$1"
    local config_name=$(basename "$config_file")
    
    log_info "Validating $config_name..."
    
    # Check if file exists and is readable
    if [[ ! -f "$config_file" ]]; then
        log_error "$config_name: File does not exist"
        return 1
    fi
    
    if [[ ! -r "$config_file" ]]; then
        log_error "$config_name: File is not readable"
        return 1
    fi
    
    # Check TOML syntax and configuration validity using BWS dry-run
    cd "$PROJECT_ROOT"
    if cargo run --bin bws -- --config "$config_file" --dry-run >/dev/null 2>&1; then
        log_success "$config_name: Configuration is valid"
    else
        log_error "$config_name: Configuration validation failed"
        return 1
    fi
    
    # Check for common configuration issues
    check_config_content "$config_file" "$config_name"
}

# Check configuration content for common issues
check_config_content() {
    local config_file="$1"
    local config_name="$2"
    
    # Read the config file
    local content=$(cat "$config_file")
    
    # Check for required sections
    if ! echo "$content" | grep -q "^\s*\[server\]"; then
        log_warning "$config_name: Missing [server] section"
    fi
    
    # Check for port conflicts (common test issue)
    local ports=$(echo "$content" | grep -o 'port\s*=\s*[0-9]\+' | grep -o '[0-9]\+' | sort -n)
    local unique_ports=$(echo "$ports" | uniq)
    
    if [[ "$ports" != "$unique_ports" ]]; then
        log_warning "$config_name: Potential port conflicts detected"
    fi
    
    # Check for localhost bindings (should be 127.0.0.1 or 0.0.0.0 for tests)
    if echo "$content" | grep -q 'host\s*=\s*"localhost"'; then
        log_warning "$config_name: Using 'localhost' instead of '127.0.0.1' (may cause issues)"
    fi
    
    # Check for SSL configuration in test files
    if echo "$content" | grep -q 'ssl_cert\|ssl_key' && [[ "$config_name" != *"ssl"* ]]; then
        log_warning "$config_name: SSL configuration in test file (ensure test certificates exist)"
    fi
    
    # Check for file paths that might not exist
    local paths=$(echo "$content" | grep -o '"[^"]*\.\(html\|css\|js\|png\|jpg\|gif\)"' | tr -d '"')
    for path in $paths; do
        if [[ "$path" == /* ]]; then
            # Absolute path - check if exists
            if [[ ! -f "$path" ]]; then
                log_warning "$config_name: Referenced file may not exist: $path"
            fi
        else
            # Relative path - check relative to project root and config dir
            if [[ ! -f "$PROJECT_ROOT/$path" ]] && [[ ! -f "$(dirname "$config_file")/$path" ]]; then
                log_warning "$config_name: Referenced file may not exist: $path"
            fi
        fi
    done
}

# Main validation function
main() {
    echo "🔍 BWS Configuration Validator"
    echo "=========================================="
    
    # Find all TOML files in configs directory
    local config_files=()
    if [[ -d "$TEST_DIR/configs" ]]; then
        while IFS= read -r -d '' file; do
            config_files+=("$file")
        done < <(find "$TEST_DIR/configs" -name "*.toml" -print0)
    fi
    
    # Also check main config files
    [[ -f "$PROJECT_ROOT/config.toml" ]] && config_files+=("$PROJECT_ROOT/config.toml")
    
    if [[ ${#config_files[@]} -eq 0 ]]; then
        log_warning "No TOML configuration files found"
        exit 0
    fi
    
    log_info "Found ${#config_files[@]} configuration files to validate"
    echo
    
    # Validate each configuration file
    for config_file in "${config_files[@]}"; do
        validate_config "$config_file"
        echo
    done
    
    # Print summary
    echo "=========================================="
    echo "          VALIDATION SUMMARY"
    echo "=========================================="
    echo -e "✅ Configs Passed: ${GREEN}$CONFIGS_PASSED${NC}"
    echo -e "❌ Configs Failed: ${RED}$CONFIGS_FAILED${NC}"
    echo "=========================================="
    
    if [[ $CONFIGS_FAILED -gt 0 ]]; then
        echo -e "${RED}❌ Some configurations failed validation.${NC}"
        exit 1
    else
        echo -e "${GREEN}🎉 All configurations are valid!${NC}"
        exit 0
    fi
}

main "$@"
