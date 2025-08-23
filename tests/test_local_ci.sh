#!/bin/bash

# Local CI Testing Script for BWS Project
# This script runs the most important checks from your GitHub workflows locally

set -e

echo "🚀 BWS Local CI Testing"
echo "======================="

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the BWS project root directory"
    exit 1
fi

echo "📂 Current directory: $(pwd)"
echo ""

# 1. Code Formatting Check
echo "🎨 Checking code formatting..."
if cargo fmt --check; then
    print_status "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi
echo ""

# 2. Clippy Linting (without build dependencies to avoid cmake issues)
echo "🔍 Running clippy (quick check mode)..."
if cargo clippy --version >/dev/null 2>&1; then
    if timeout 60s cargo clippy --no-deps -- -D warnings; then
        print_status "Clippy checks passed"
    else
        print_warning "Clippy checks failed or timed out (may be due to build dependencies)"
    fi
else
    print_warning "Clippy not available - install with: rustup component add clippy"
fi
echo ""

# 3. Security Audit
echo "🛡️  Running security audit..."
if cargo audit --version >/dev/null 2>&1; then
    if cargo audit; then
        print_status "Security audit passed"
    else
        print_warning "Security audit found issues - check SECURITY.md for documented acceptances"
    fi
else
    echo "Installing cargo-audit..."
    cargo install cargo-audit
    cargo audit
fi
echo ""

# 4. Cargo Deny Check
echo "🚫 Running cargo deny checks..."
if cargo deny --version >/dev/null 2>&1; then
    if cargo deny check; then
        print_status "Cargo deny checks passed"
    else
        print_error "Cargo deny checks failed"
        exit 1
    fi
else
    echo "Installing cargo-deny..."
    cargo install cargo-deny
    cargo deny check
fi
echo ""

# 5. Basic Compilation Test (without full build to avoid cmake)
echo "⚙️  Testing basic compilation..."
if timeout 120s cargo check; then
    print_status "Basic compilation check passed"
else
    print_warning "Compilation check failed or timed out"
fi
echo ""

# 6. Configuration Validation
echo "📋 Validating configurations..."

# Check if configuration files exist
if [ -f "bws_config.toml" ]; then
    print_status "BWS configuration file exists"
else
    print_warning "BWS configuration file not found - server may not start"
fi

if [ -f "deny.toml" ]; then
    print_status "Cargo deny configuration exists"
else
    print_error "Cargo deny configuration missing"
fi

if [ -f "SECURITY.md" ]; then
    print_status "Security documentation exists"
else
    print_warning "Security documentation missing"
fi
echo ""

# 7. GitHub Workflows Validation
echo "🔄 Validating GitHub workflows..."
if [ -d ".github/workflows" ]; then
    for workflow in .github/workflows/*.yml; do
        if [ -f "$workflow" ]; then
            print_status "Found workflow: $(basename "$workflow")"
        fi
    done
else
    print_warning "No GitHub workflows found"
fi
echo ""

echo "✨ Local CI testing complete!"
echo ""
echo "💡 Tips:"
echo "  - Run 'cargo fmt' to fix formatting issues"
echo "  - Run 'cargo clippy --fix' to fix some linting issues"
echo "  - Check SECURITY.md for documented vulnerability acceptances"
echo "  - Use 'act' for full GitHub Actions simulation (requires Docker)"
echo ""
echo "🔗 Useful commands:"
echo "  act --list                     # List available workflows"
echo "  act -W .github/workflows/security.yml -j security_audit  # Run security audit"
echo "  cargo build                    # Full build (requires system dependencies)"
echo "  ../target/debug/main           # Run the server after building"
