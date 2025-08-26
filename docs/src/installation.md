# Installation

BWS can be installed in several ways. Choose the method that best fits your needs.

## From crates.io (Recommended for Development)

The easiest way to install BWS is using Cargo:

```bash
cargo install bws-web-server
```

This will install the latest stable version from [crates.io](https://crates.io/crates/bws-web-server). The package name is `bws-web-server`, but the installed binary is named `bws`.

### Prerequisites

- **Rust**: Version 1.89.0 or later
- **Cargo**: Comes with Rust installation

## From Docker (Recommended for Production)

Docker is the recommended deployment method for production environments:

```bash
# Pull and run the latest version
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest

# Or with custom configuration
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest

# Validate configuration before running
docker run --rm \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  ghcr.io/benliao/bws:latest \
  bws --config config.toml --dry-run
```

### Available Docker Tags

- `latest` - Latest stable release
- `v0.3.4` - Specific version (recommended for production)
- `main` - Latest development build

## From Source

For development or custom builds:

```bash
# Clone the repository
git clone https://github.com/benliao/bws.git
cd bws

# Validate the build configuration
cargo check

# Build in debug mode
cargo build --bin bws

# Build optimized release
cargo build --release --bin bws

# Test the binary
./target/release/bws --version
./target/release/bws --help

# Validate example configurations
./target/release/bws --config examples/basic-single-site.toml --dry-run

# The binary will be in target/release/bws
```

### Testing Your Build

After building from source, run the test suite:

```bash
# Run unit tests
cargo test

# Run configuration validation tests
./tests/scripts/validate-configs.sh

# Run integration tests (if server dependencies available)
./tests/scripts/test_headers.sh
```

## Pre-built Binaries

Download pre-built binaries from [GitHub Releases](https://github.com/benliao/bws/releases):

### Linux (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-linux-x86_64.tar.gz
tar -xzf bws-linux-x86_64.tar.gz
chmod +x bws
```

### macOS (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-macos-x86_64.tar.gz
tar -xzf bws-macos-x86_64.tar.gz
chmod +x bws
```

### macOS (ARM64/Apple Silicon)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-macos-aarch64.tar.gz
tar -xzf bws-macos-aarch64.tar.gz
chmod +x bws
```

### Windows (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-windows-x86_64.exe
# Note: Windows executable is directly usable
```

## Verification

After installation, verify BWS is working:

```bash
# Check version
bws --version

# Display help
bws --help
```

## Next Steps

Once installed, proceed to the [Quick Start](./quick-start.md) guide to set up your first BWS server!
