#!/bin/bash
# Cross-compilation script for BWS Web Server
# This script builds BWS for multiple platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default targets
DEFAULT_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
    "aarch64-unknown-linux-gnu"
)

# All available targets
ALL_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabihf"
    "x86_64-pc-windows-gnu"
    "i686-pc-windows-gnu"
)

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] [TARGETS...]"
    echo ""
    echo "Cross-compile BWS Web Server for multiple platforms"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help      Show this help message"
    echo "  -a, --all       Build for all supported targets"
    echo "  -w, --windows   Build for Windows targets only"
    echo "  -l, --linux     Build for Linux targets only"
    echo "  --list          List all available targets"
    echo "  --clean         Clean before building"
    echo ""
    echo "TARGETS:"
    echo "  If no targets specified, builds for default targets:"
    for target in "${DEFAULT_TARGETS[@]}"; do
        echo "    - $target"
    done
    echo ""
    echo "EXAMPLES:"
    echo "  $0                                    # Build default targets"
    echo "  $0 --windows                          # Build Windows targets"
    echo "  $0 x86_64-pc-windows-gnu            # Build specific target"
    echo "  $0 --clean --all                     # Clean and build all targets"
}

# Function to list targets
list_targets() {
    echo "Available targets:"
    for target in "${ALL_TARGETS[@]}"; do
        echo "  - $target"
    done
}

# Function to check if target is supported
is_target_supported() {
    local target=$1
    for supported in "${ALL_TARGETS[@]}"; do
        if [[ "$supported" == "$target" ]]; then
            return 0
        fi
    done
    return 1
}

# Function to install target if needed
ensure_target() {
    local target=$1
    print_status "Ensuring target $target is installed..."
    
    if rustup target list --installed | grep -q "^$target$"; then
        print_status "Target $target already installed"
    else
        print_status "Installing target $target..."
        rustup target add "$target"
    fi
}

# Function to build for a specific target
build_target() {
    local target=$1
    print_status "Building for target: $target"
    
    # Ensure target is installed
    ensure_target "$target"
    
    # Build command
    local build_cmd
    if command -v cross >/dev/null 2>&1; then
        build_cmd="cross build --release --target $target"
        print_status "Using cross for $target"
    else
        build_cmd="cargo build --release --target $target"
        print_status "Using cargo for $target"
    fi
    
    # Execute build
    if $build_cmd; then
        print_success "Successfully built for $target"
        
        # Show output location
        local output_dir="target/$target/release"
        if [[ "$target" == *"windows"* ]]; then
            local exe_name="bws.exe"
        else
            local exe_name="bws"
        fi
        
        if [[ -f "$output_dir/$exe_name" ]]; then
            local file_size=$(du -h "$output_dir/$exe_name" | cut -f1)
            print_success "Binary: $output_dir/$exe_name ($file_size)"
        fi
    else
        print_error "Failed to build for $target"
        return 1
    fi
}

# Function to clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    cargo clean
    print_success "Clean completed"
}

# Parse command line arguments
TARGETS=()
CLEAN=false
BUILD_ALL=false
BUILD_WINDOWS=false
BUILD_LINUX=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -a|--all)
            BUILD_ALL=true
            shift
            ;;
        -w|--windows)
            BUILD_WINDOWS=true
            shift
            ;;
        -l|--linux)
            BUILD_LINUX=true
            shift
            ;;
        --list)
            list_targets
            exit 0
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        -*)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            if is_target_supported "$1"; then
                TARGETS+=("$1")
            else
                print_error "Unsupported target: $1"
                print_warning "Run '$0 --list' to see available targets"
                exit 1
            fi
            shift
            ;;
    esac
done

# Determine which targets to build
if [[ "$BUILD_ALL" == true ]]; then
    TARGETS=("${ALL_TARGETS[@]}")
elif [[ "$BUILD_WINDOWS" == true ]]; then
    TARGETS=("x86_64-pc-windows-gnu" "i686-pc-windows-gnu")
elif [[ "$BUILD_LINUX" == true ]]; then
    TARGETS=(
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-gnu"
        "aarch64-unknown-linux-musl"
        "armv7-unknown-linux-musleabihf"
    )
elif [[ ${#TARGETS[@]} -eq 0 ]]; then
    TARGETS=("${DEFAULT_TARGETS[@]}")
fi

# Print build plan
print_status "BWS Web Server Cross-Compilation"
print_status "================================="
echo ""
print_status "Targets to build:"
for target in "${TARGETS[@]}"; do
    echo "  - $target"
done
echo ""

# Clean if requested
if [[ "$CLEAN" == true ]]; then
    clean_build
    echo ""
fi

# Build each target
SUCCESSFUL_BUILDS=()
FAILED_BUILDS=()

for target in "${TARGETS[@]}"; do
    echo ""
    print_status "Building $target..."
    echo "----------------------------------------"
    
    if build_target "$target"; then
        SUCCESSFUL_BUILDS+=("$target")
    else
        FAILED_BUILDS+=("$target")
    fi
done

# Summary
echo ""
echo "========================================"
print_status "Build Summary"
echo "========================================"

if [[ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]]; then
    print_success "Successful builds (${#SUCCESSFUL_BUILDS[@]}):"
    for target in "${SUCCESSFUL_BUILDS[@]}"; do
        echo "  ✓ $target"
    done
fi

if [[ ${#FAILED_BUILDS[@]} -gt 0 ]]; then
    echo ""
    print_error "Failed builds (${#FAILED_BUILDS[@]}):"
    for target in "${FAILED_BUILDS[@]}"; do
        echo "  ✗ $target"
    done
    echo ""
    exit 1
fi

print_success "All builds completed successfully!"
