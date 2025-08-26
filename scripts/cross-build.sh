#!/bin/bash
# Cross-compilation build script for BWS
# This script builds BWS for multiple target platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build information
VERSION=$(cargo pkgid | cut -d'#' -f2)
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo -e "${BLUE}🔨 BWS Cross-Compilation Build Script${NC}"
echo -e "${BLUE}=====================================${NC}"
echo "Version: $VERSION"
echo "Build Date: $BUILD_DATE"
echo "Git Commit: $GIT_COMMIT"
echo

# Export build variables
export BWS_VERSION="$VERSION"
export BUILD_DATE="$BUILD_DATE"
export GIT_COMMIT="$GIT_COMMIT"

# Define targets
TARGETS=(
    "x86_64-unknown-linux-musl"      # Linux x64 static
    "aarch64-unknown-linux-musl"     # Linux ARM64 static
    "x86_64-unknown-linux-gnu"       # Linux x64 dynamic
    "aarch64-unknown-linux-gnu"      # Linux ARM64 dynamic
    "armv7-unknown-linux-musleabihf" # ARMv7 static
)

# Function to build for a specific target
build_target() {
    local target=$1
    echo -e "${YELLOW}🏗️  Building for $target...${NC}"
    
    # Check if cross tool should be used
    if command -v cross &> /dev/null && [[ "$target" != "x86_64-pc-windows-msvc" ]]; then
        echo "Using cross for $target"
        cross build --release --target "$target" --bin bws --verbose
        cross build --release --target "$target" --bin bws-ctl --verbose
    else
        echo "Using cargo for $target"
        cargo build --release --target "$target" --bin bws --verbose
        cargo build --release --target "$target" --bin bws-ctl --verbose
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully built for $target${NC}"
        
        # Get binary info
        local bws_binary_path="target/$target/release/bws"
        local ctl_binary_path="target/$target/release/bws-ctl"
        if [[ "$target" == *"windows"* ]]; then
            bws_binary_path="target/$target/release/bws.exe"
            ctl_binary_path="target/$target/release/bws-ctl.exe"
        fi
        
        if [ -f "$bws_binary_path" ]; then
            local size=$(du -h "$bws_binary_path" | cut -f1)
            echo "   📦 BWS binary size: $size"
            echo "   📍 BWS location: $bws_binary_path"
        fi
        
        if [ -f "$ctl_binary_path" ]; then
            local size=$(du -h "$ctl_binary_path" | cut -f1)
            echo "   📦 BWS-CTL binary size: $size"
            echo "   📍 BWS-CTL location: $ctl_binary_path"
        fi
        echo
    else
        echo -e "${RED}❌ Failed to build for $target${NC}"
        return 1
    fi
}

# Function to verify binary
verify_binary() {
    local target=$1
    local bws_binary_path="target/$target/release/bws"
    local ctl_binary_path="target/$target/release/bws-ctl"
    
    if [[ "$target" == *"windows"* ]]; then
        bws_binary_path="target/$target/release/bws.exe"
        ctl_binary_path="target/$target/release/bws-ctl.exe"
    fi
    
    if [ -f "$bws_binary_path" ] || [ -f "$ctl_binary_path" ]; then
        echo -e "${BLUE}🔍 Verifying $target binaries...${NC}"
        
        # Check BWS binary
        if [ -f "$bws_binary_path" ]; then
            echo "BWS Binary:"
            if command -v file &> /dev/null; then
                file "$bws_binary_path"
            fi
            
            # Check dependencies for Linux binaries
            if [[ "$target" == *"linux"* ]] && command -v objdump &> /dev/null; then
                echo "BWS Dependencies:"
                objdump -p "$bws_binary_path" 2>/dev/null | grep NEEDED || echo "   Static binary (no dependencies)"
            fi
        fi
        
        # Check BWS-CTL binary
        if [ -f "$ctl_binary_path" ]; then
            echo "BWS-CTL Binary:"
            if command -v file &> /dev/null; then
                file "$ctl_binary_path"
            fi
            
            # Check dependencies for Linux binaries
            if [[ "$target" == *"linux"* ]] && command -v objdump &> /dev/null; then
                echo "BWS-CTL Dependencies:"
                objdump -p "$ctl_binary_path" 2>/dev/null | grep NEEDED || echo "   Static binary (no dependencies)"
            fi
        fi
        
        echo
    fi
}

# Clean previous builds
echo -e "${YELLOW}🧹 Cleaning previous builds...${NC}"
cargo clean
echo

# Add required targets
echo -e "${YELLOW}🎯 Adding required targets...${NC}"
for target in "${TARGETS[@]}"; do
    echo "Adding target: $target"
    rustup target add "$target" 2>/dev/null || true
done
echo

# Main build loop
echo -e "${YELLOW}🚀 Starting cross-compilation builds...${NC}"
echo

SUCCESSFUL_BUILDS=()
FAILED_BUILDS=()

for target in "${TARGETS[@]}"; do
    if build_target "$target"; then
        SUCCESSFUL_BUILDS+=("$target")
        verify_binary "$target"
    else
        FAILED_BUILDS+=("$target")
    fi
done

# Build summary
echo -e "${BLUE}📊 Build Summary${NC}"
echo -e "${BLUE}===============${NC}"
echo

if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "${GREEN}✅ Successful builds (${#SUCCESSFUL_BUILDS[@]}):${NC}"
    for target in "${SUCCESSFUL_BUILDS[@]}"; do
        echo "   ✓ $target"
    done
    echo
fi

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo -e "${RED}❌ Failed builds (${#FAILED_BUILDS[@]}):${NC}"
    for target in "${FAILED_BUILDS[@]}"; do
        echo "   ✗ $target"
    done
    echo
fi

# Create distribution directory
if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "${YELLOW}📦 Creating distribution packages...${NC}"
    
    mkdir -p dist
    
    for target in "${SUCCESSFUL_BUILDS[@]}"; do
        local bws_binary_path="target/$target/release/bws"
        local ctl_binary_path="target/$target/release/bws-ctl"
        local dist_dir="dist/bws-$VERSION-$target"
        
        if [[ "$target" == *"windows"* ]]; then
            bws_binary_path="target/$target/release/bws.exe"
            ctl_binary_path="target/$target/release/bws-ctl.exe"
        fi
        
        if [ -f "$bws_binary_path" ]; then
            echo "Creating package for $target..."
            mkdir -p "$dist_dir"
            
            # Copy binaries
            cp "$bws_binary_path" "$dist_dir/"
            if [ -f "$ctl_binary_path" ]; then
                cp "$ctl_binary_path" "$dist_dir/"
            fi
            
            # Copy documentation
            cp README.md LICENSE "$dist_dir/" 2>/dev/null || true
            
            # Copy example configuration
            cp config.toml "$dist_dir/config.example.toml" 2>/dev/null || true
            
            # Create archive
            if command -v tar &> /dev/null; then
                tar -czf "dist/bws-$VERSION-$target.tar.gz" -C dist "bws-$VERSION-$target"
                echo "   📦 Created: dist/bws-$VERSION-$target.tar.gz"
            fi
            
            # Create zip for Windows targets
            if [[ "$target" == *"windows"* ]] && command -v zip &> /dev/null; then
                (cd dist && zip -r "bws-$VERSION-$target.zip" "bws-$VERSION-$target")
                echo "   📦 Created: dist/bws-$VERSION-$target.zip"
            fi
        fi
    done
    echo
fi

# Show final results
echo -e "${BLUE}🎉 Cross-compilation complete!${NC}"
echo "Built $((${#SUCCESSFUL_BUILDS[@]})) out of $((${#TARGETS[@]})) targets"

if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    echo -e "${GREEN}All builds successful! 🚀${NC}"
    exit 0
else
    echo -e "${YELLOW}Some builds failed. Check the output above for details.${NC}"
    exit 1
fi
