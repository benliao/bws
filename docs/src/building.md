# Building from Source

This guide covers building BWS from source code, including development setup, build options, and cross-compilation.

## Prerequisites

### System Requirements

#### Minimum Requirements
- **RAM**: 2GB available memory
- **Disk Space**: 1GB free space for build artifacts
- **CPU**: Any modern x64 or ARM64 processor

#### Supported Platforms
- **Linux**: Ubuntu 18.04+, Debian 10+, RHEL 8+, Alpine Linux
- **macOS**: 10.15+ (Catalina)
- **Windows**: Windows 10+ (with WSL2 recommended)
- **FreeBSD**: 12.0+

### Dependencies

#### Rust Toolchain
```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Update to latest stable
rustup update stable
```

#### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    curl
```

**CentOS/RHEL/Fedora:**
```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y \
    pkg-config \
    openssl-devel \
    cmake \
    git \
    curl
```

**macOS:**
```bash
# Install Xcode command line tools
xcode-select --install

# Or install via Homebrew
brew install cmake pkg-config openssl
```

**Windows (WSL2):**
```bash
# In WSL2 Ubuntu
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    curl
```

#### Optional Dependencies
```bash
# For enhanced compression support
sudo apt install -y libbrotli-dev zlib1g-dev

# For performance profiling
cargo install cargo-flamegraph

# For security auditing
cargo install cargo-audit

# For benchmarking
cargo install cargo-bench
```

## Getting the Source Code

### Clone Repository
```bash
# Clone the main repository
git clone https://github.com/benliao/bws.git
cd bws

# Or clone your fork
git clone https://github.com/yourusername/bws.git
cd bws

# Check available branches/tags
git branch -a
git tag -l
```

### Download Source Archive
```bash
# Download specific version
wget https://github.com/benliao/bws/archive/refs/tags/v0.1.5.tar.gz
tar -xzf v0.1.5.tar.gz
cd bws-0.1.5
```

## Build Configuration

### Cargo.toml Overview
```toml
[package]
name = "bws"
version = "0.1.5"
edition = "2021"
rust-version = "1.89"

[dependencies]
pingora = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
anyhow = "1.0"
tracing = "0.1"

[features]
default = ["compression", "metrics"]
compression = ["brotli", "gzip"]
metrics = ["prometheus"]
tls = ["openssl"]
jemalloc = ["jemallocator"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Feature Flags

#### Available Features
- `compression`: Enable response compression (gzip, brotli)
- `metrics`: Enable Prometheus metrics export
- `tls`: Enable TLS/SSL support
- `jemalloc`: Use jemalloc allocator for better performance
- `mimalloc`: Use mimalloc allocator (alternative to jemalloc)

#### Building with Specific Features
```bash
# Build with all features
cargo build --all-features

# Build with specific features
cargo build --features "compression,metrics"

# Build without default features
cargo build --no-default-features

# Build with custom feature combination
cargo build --no-default-features --features "compression,tls"
```

## Build Commands

### Development Build
```bash
# Standard debug build
cargo build

# With specific features
cargo build --features "compression,metrics"

# With verbose output
cargo build --verbose

# Check without building
cargo check
```

### Release Build
```bash
# Optimized release build
cargo build --release

# Release with all features
cargo build --release --all-features

# Strip debug symbols
cargo build --release
strip target/release/bws  # Linux/macOS
```

### Custom Profiles

#### Performance Profile
```toml
# Add to Cargo.toml
[profile.performance]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
```

```bash
# Build with performance profile
cargo build --profile performance
```

#### Size-Optimized Profile
```toml
[profile.min-size]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

```bash
# Build for minimum size
cargo build --profile min-size
```

## Build Optimization

### Rust Compiler Flags

#### Environment Variables
```bash
# Enable link-time optimization
export RUSTFLAGS="-C link-arg=-s"

# Use specific target CPU
export RUSTFLAGS="-C target-cpu=native"

# Optimize for size
export RUSTFLAGS="-C opt-level=z"

# Build with optimizations
cargo build --release
```

#### Target-Specific Optimization
```bash
# Build for specific CPU architecture
RUSTFLAGS="-C target-cpu=skylake" cargo build --release

# Build with all CPU features
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Build for compatibility
RUSTFLAGS="-C target-cpu=x86-64" cargo build --release
```

### Memory Allocator Optimization

#### Using jemalloc
```toml
# Add to Cargo.toml
[dependencies]
jemallocator = "0.5"

# Add to src/main.rs
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

#### Using mimalloc
```toml
# Add to Cargo.toml
[dependencies]
mimalloc = { version = "0.1", default-features = false }

# Add to src/main.rs
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

### Link-Time Optimization

```toml
# Cargo.toml release profile
[profile.release]
lto = true              # Enable LTO
codegen-units = 1       # Use single codegen unit
opt-level = 3           # Maximum optimization
```

## Cross-Compilation

### Setup Cross-Compilation Targets

```bash
# Add common targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-gnu

# List installed targets
rustup target list --installed
```

### Cross-Compilation Examples

#### Linux to musl (static linking)
```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Install musl tools (Ubuntu/Debian)
sudo apt install musl-tools

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Verify static linking
ldd target/x86_64-unknown-linux-musl/release/bws
# Should show "not a dynamic executable"
```

#### Linux to ARM64
```bash
# Install ARM64 target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
sudo apt install gcc-aarch64-linux-gnu

# Set up linker
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu
```

#### macOS Universal Binary
```bash
# Install both targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
    target/x86_64-apple-darwin/release/bws \
    target/aarch64-apple-darwin/release/bws \
    -output target/release/bws-universal
```

#### Windows Cross-Compilation (from Linux)
```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW
sudo apt install mingw-w64

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

### Using cross Tool

```bash
# Install cross
cargo install cross

# Build for different targets using Docker
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Build Scripts and Automation

### Makefile
```makefile
# Makefile
.PHONY: build build-release test clean install

# Default target
all: build

# Development build
build:
	cargo build

# Release build
build-release:
	cargo build --release

# Build with all features
build-all:
	cargo build --release --all-features

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Install locally
install:
	cargo install --path .

# Cross-compilation targets
build-linux-musl:
	cargo build --release --target x86_64-unknown-linux-musl

build-arm64:
	cargo build --release --target aarch64-unknown-linux-gnu

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

# All targets
build-all-targets: build-linux-musl build-arm64 build-windows

# Package release
package:
	mkdir -p dist
	cp target/release/bws dist/
	cp README.md LICENSE dist/
	tar -czf dist/bws-$(shell cargo pkgid | cut -d'#' -f2).tar.gz -C dist .
```

### Build Script
```bash
#!/bin/bash
# build.sh

set -e

echo "Building BWS from source..."

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    exit 1
fi

# Get version
VERSION=$(cargo pkgid | cut -d'#' -f2)
echo "Building BWS version $VERSION"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build release version
echo "Building release version..."
cargo build --release --all-features

# Run tests
echo "Running tests..."
cargo test --release

# Check binary
if [ -f "target/release/bws" ]; then
    echo "Build successful!"
    echo "Binary location: target/release/bws"
    echo "Binary size: $(du -h target/release/bws | cut -f1)"
    
    # Test the binary
    ./target/release/bws --version
else
    echo "Build failed: Binary not found"
    exit 1
fi

echo "Build completed successfully!"
```

### GitHub Actions Build

```yaml
# .github/workflows/build.yml
name: Build

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    name: Build
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Build
      run: cargo build --release --all-features

    - name: Run tests
      run: cargo test --release

    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: bws-${{ matrix.os }}
        path: target/release/bws*
```

## Testing the Build

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_config

# Run tests with output
cargo test -- --nocapture

# Run tests in single thread
cargo test -- --test-threads=1

# Run ignored tests
cargo test -- --ignored
```

### Integration Tests
```bash
# Run integration tests only
cargo test --test integration

# Run specific integration test
cargo test --test integration -- test_server_startup
```

### Benchmarks
```bash
# Install criterion for benchmarking
cargo install criterion

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_name
```

### Manual Testing
```bash
# Build and test the binary
cargo build --release

# Test basic functionality
./target/release/bws --version
./target/release/bws --help

# Test with sample configuration
echo '
[[sites]]
name = "test"
hostname = "127.0.0.1"
port = 8080
static_dir = "static"
' > test-config.toml

mkdir -p static
echo "Hello, World!" > static/index.html

# Start server in background
./target/release/bws --config test-config.toml &
SERVER_PID=$!

# Test HTTP request
sleep 2
curl http://127.0.0.1:8080/

# Clean up
kill $SERVER_PID
rm -rf test-config.toml static/
```

## Troubleshooting Build Issues

### Common Build Errors

#### 1. Linker Errors
```bash
# Error: linker `cc` not found
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # RHEL/Fedora

# Error: could not find system library 'openssl'
sudo apt install libssl-dev pkg-config  # Ubuntu/Debian
sudo dnf install openssl-devel pkgconf  # RHEL/Fedora
```

#### 2. Memory Issues
```bash
# Reduce parallel builds if running out of memory
cargo build --jobs 1

# Or set permanently
echo 'jobs = 1' >> ~/.cargo/config.toml
```

#### 3. Network Issues
```bash
# Use git instead of HTTPS for dependencies
git config --global url."git://github.com/".insteadOf "https://github.com/"

# Or use offline mode with vendored dependencies
cargo vendor
cargo build --offline
```

#### 4. Permission Issues
```bash
# Fix cargo directory permissions
sudo chown -R $(whoami) ~/.cargo

# Clean and rebuild
cargo clean
cargo build
```

### Debugging Build Issues

#### Verbose Output
```bash
# Build with verbose output
cargo build --verbose

# Show build timing
cargo build --timings

# Show why dependencies are being rebuilt
cargo build --verbose --explain
```

#### Environment Debugging
```bash
# Show cargo configuration
cargo config list

# Show target information
rustc --print target-list
rustc --print cfg

# Show toolchain information
rustup show
```

## Installation

### Local Installation
```bash
# Install from current directory
cargo install --path .

# Install with specific features
cargo install --path . --features "compression,metrics"

# Force reinstall
cargo install --path . --force
```

### System-Wide Installation
```bash
# Copy binary to system path
sudo cp target/release/bws /usr/local/bin/

# Make executable
sudo chmod +x /usr/local/bin/bws

# Verify installation
bws --version
```

### Package Creation

#### DEB Package (Ubuntu/Debian)
```bash
# Install cargo-deb
cargo install cargo-deb

# Add metadata to Cargo.toml
[package.metadata.deb]
maintainer = "Your Name <your.email@example.com>"
copyright = "2024, Your Name <your.email@example.com>"
license-file = ["LICENSE", "5"]
extended-description = "BWS is a high-performance multi-site web server"
depends = "$auto"
section = "utility"
priority = "optional"
assets = [
    ["target/release/bws", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/bws/README", "644"],
]

# Build DEB package
cargo deb
```

#### RPM Package (RHEL/Fedora)
```bash
# Install cargo-rpm
cargo install cargo-rpm

# Initialize RPM spec
cargo rpm init

# Build RPM package
cargo rpm build
```

#### Windows Installer
```bash
# Install cargo-wix
cargo install cargo-wix

# Initialize WiX configuration
cargo wix init

# Build MSI installer
cargo wix
```

## Optimization Tips

### Build Performance
- Use `cargo check` during development instead of `cargo build`
- Enable incremental compilation
- Use `sccache` for shared build cache
- Consider using `mold` linker on Linux for faster linking

### Binary Size Optimization
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
strip = true         # Strip symbols
panic = "abort"      # Smaller panic handling
```

### Runtime Performance
```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"          # Full LTO
codegen-units = 1    # Single codegen unit
panic = "abort"      # Abort on panic
```

## Next Steps

After building BWS:

1. Read the [Configuration Guide](./configuration.md) to set up your server
2. Follow the [Quick Start](./quick-start.md) to get running
3. Check [Performance Tuning](./performance.md) for optimization
4. Review [Production Setup](./production.md) for deployment

For development:
1. Read the [Contributing Guide](./contributing.md)
2. Set up your development environment
3. Run the test suite
4. Start contributing!
