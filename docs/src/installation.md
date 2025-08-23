# Installation

BWS can be installed in several ways. Choose the method that best fits your needs.

## From crates.io (Recommended for Development)

The easiest way to install BWS is using Cargo:

```bash
cargo install bws-web-server
```

This will install the latest stable version from [crates.io](https://crates.io/crates/bws-web-server).

### Prerequisites

- **Rust**: Version 1.70 or later
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
```

### Available Docker Tags

- `latest` - Latest stable release
- `v0.1.5` - Specific version (recommended for production)
- `main` - Latest development build

## From Source

For development or custom builds:

```bash
# Clone the repository
git clone https://github.com/benliao/bws.git
cd bws

# Build in debug mode
cargo build

# Build optimized release
cargo build --release

# The binary will be in target/release/bws-web-server
```

## Pre-built Binaries

Download pre-built binaries from [GitHub Releases](https://github.com/benliao/bws/releases):

### Linux (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-web-server-linux-x86_64.tar.gz
tar -xzf bws-web-server-linux-x86_64.tar.gz
chmod +x bws-web-server
```

### macOS (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-web-server-macos-x86_64.tar.gz
tar -xzf bws-web-server-macos-x86_64.tar.gz
chmod +x bws-web-server
```

### macOS (ARM64/Apple Silicon)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-web-server-macos-aarch64.tar.gz
tar -xzf bws-web-server-macos-aarch64.tar.gz
chmod +x bws-web-server
```

### Windows (x86_64)
```bash
wget https://github.com/benliao/bws/releases/latest/download/bws-web-server-windows-x86_64.zip
unzip bws-web-server-windows-x86_64.zip
```

## Verification

After installation, verify BWS is working:

```bash
# Check version
bws-web-server --version

# Display help
bws-web-server --help
```

## Next Steps

Once installed, proceed to the [Quick Start](./quick-start.md) guide to set up your first BWS server!
