# BWS (Ben's Web Server)

[![CI](https://github.com/benliao/bws/workflows/CI/badge.svg)](https://github.com/benliao/bws/actions)
[![Security](https://github.com/benliao/bws/workflows/Security/badge.svg)](https://github.com/benliao/bws/actions)
[![Crates.io](https://img.shields.io/crates/v/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![Downloads](https://img.shields.io/crates/d/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-mdbook-blue.svg)](https://benliao.github.io/bws/)

A high-performance, multi-site web server built with [Pingora](https://github.com/cloudflare/pingora), Cloudflare's battle-tested proxy framework.

## 📖 Documentation

**[Complete Documentation →](https://benliao.github.io/bws/)**

The full documentation includes:
- 📚 **User Guide** - Installation, configuration, and deployment
- 🚀 **Quick Start** - Get running in minutes
- 🔧 **API Reference** - Complete REST API documentation
- 🐳 **Docker Guide** - Container deployment options
- 💡 **Examples** - Real-world use cases and configurations

## 🚀 Features

- **Multi-Site Support**: Host multiple websites on different ports with individual configurations
- **Configurable Headers**: Set custom HTTP headers per site via TOML configuration
- **High Performance**: Built on Pingora for enterprise-grade performance and reliability
- **Health Monitoring**: Built-in health check endpoints for monitoring
- **Security Focused**: Comprehensive security auditing and dependency management
- **Easy Configuration**: Simple TOML-based configuration system

## 📦 Installation

### From crates.io

```bash
cargo install bws-web-server
```

### From Docker (Recommended for Production)

```bash
# Pull and run the latest version
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest
```

### From Source

```bash
git clone https://github.com/benliao/bws.git
cd bws
cargo build --release
```

## 🔧 Quick Start

1. **Create a configuration file** (`config.toml`):

```toml
[server]
name = "BWS Multi-Site Server"

[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "BWS Main Site"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "main"

[[sites]]
name = "api"
hostname = "api.localhost"
port = 8081
static_dir = "static-api"
api_only = true

[sites.headers]
"X-API-Version" = "v1"
"X-Service" = "api"
```

2. **Create static directories and files**:

```bash
# Create directories for each site
mkdir -p static static-api

# Add content to main site
echo "<h1>Welcome to BWS Main Site</h1>" > static/index.html

# Add content to API site
echo "<h1>API Documentation</h1>" > static-api/index.html
```

3. **Run the server**:

```bash
# Run with default config.toml
bws-web-server

# Specify a custom config file
bws-web-server --config my-config.toml

# Enable verbose logging
bws-web-server --verbose

# Show help
bws-web-server --help

# Show version
bws-web-server --version
```

4. **Test your setup**:

```bash
# Test main site
curl -I http://localhost:8080

# Test API site  
curl -I http://localhost:8081

# Health check
curl http://localhost:8080/api/health
```

## 💻 Command Line Options

BWS supports the following command line options:

- `-c, --config <FILE>`: Specify configuration file path (default: `config.toml`)
- `-v, --verbose`: Enable verbose logging with debug information
- `-d, --daemon`: Run as daemon (background process) - **Unix only**
- `--pid-file <FILE>`: PID file path when running as daemon (default: `/tmp/bws-web-server.pid`) - **Unix only**
- `--log-file <FILE>`: Log file path when running as daemon (default: `/tmp/bws-web-server.log`) - **Unix only**
- `-h, --help`: Show help information
- `-V, --version`: Show version information

> **Note**: Daemon functionality is only available on Unix-like systems (Linux, macOS, etc.). On Windows, the server runs in foreground mode only.

### Examples

```bash
# Use default config.toml
bws-web-server

# Use custom configuration file
bws-web-server --config /path/to/my-sites.toml

# Enable verbose logging to see headers and debug info
bws-web-server --verbose

# Combine options
bws-web-server --config prod.toml --verbose

# Run as daemon (background process)
bws-web-server --daemon

# Run as daemon with custom log and PID files
bws-web-server --daemon --log-file /var/log/bws.log --pid-file /var/run/bws.pid
```

## 🔧 Daemon Management (Unix Only)

BWS can run as a daemon (background process) for production deployments on Unix-like systems (Linux, macOS, etc.). A management script is provided for easy daemon control:

> **Windows Note**: Daemon functionality is not supported on Windows. Use your system's service manager or run in the foreground with a process manager like PM2 or NSSM.

```bash
# Start the daemon
./bws-daemon.sh start

# Stop the daemon
./bws-daemon.sh stop

# Restart the daemon
./bws-daemon.sh restart

# Check daemon status
./bws-daemon.sh status
```

### Daemon Features

- **Background Operation**: Runs detached from the terminal
- **PID File Management**: Tracks process ID for management
- **Log File Output**: All output redirected to log files
- **Graceful Shutdown**: Handles termination signals properly
- **Status Monitoring**: Built-in health checks and status reporting

### Manual Daemon Control

```bash
# Start daemon manually
bws-web-server --daemon --config config.toml --log-file bws.log --pid-file bws.pid

# Stop daemon using PID file
kill $(cat bws.pid)

# Check if daemon is running
ps aux | grep bws-web-server
```

## 🐳 Docker Deployment

BWS provides Docker images for easy deployment and scaling.

### Quick Start with Docker

```bash
# Pull the latest image from GitHub Container Registry
docker pull ghcr.io/benliao/bws:latest

# Run with default configuration
docker run -d \
  --name bws-server \
  -p 8080:8080 \
  -p 8081:8081 \
  -p 8082:8082 \
  -p 8083:8083 \
  ghcr.io/benliao/bws:latest

# Run with custom configuration
docker run -d \
  --name bws-server \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest
```

### Using Docker Compose

```bash
# Start with the provided docker-compose.yml
docker-compose up -d

# Start with daemon mode profile
docker-compose --profile daemon up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Available Docker Images

- **Latest**: `ghcr.io/benliao/bws:latest` - Built from main branch
- **Versioned**: `ghcr.io/benliao/bws:v0.1.2` - Specific version releases
- **Platform Support**: Available for `linux/amd64` and `linux/arm64`

### Docker Environment Variables

- `BWS_CONFIG`: Configuration file path (default: `/app/config.toml`)
- `BWS_LOG_FILE`: Log file path for daemon mode (default: `/app/logs/bws.log`)
- `BWS_PID_FILE`: PID file path for daemon mode (default: `/app/run/bws.pid`)
- `RUST_LOG`: Logging level (`error`, `warn`, `info`, `debug`, `trace`)

### Docker Volumes

- `/app/config.toml`: Mount your configuration file
- `/app/static*`: Mount your static content directories
- `/app/logs`: Persistent log storage
- `/app/run`: Persistent runtime files (PID files, etc.)

```toml
[server]
name = "BWS Multi-Site Server"

# Main site with production headers
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "BWS Main Site"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "main"
"X-Environment" = "production"

# Blog site with custom headers
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8081
static_dir = "static-blog"

[sites.headers]
"X-Site-Name" = "BWS Blog"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "blog"
"X-Content-Type" = "blog-content"
"X-Author" = "BWS Team"

# API documentation site with CORS headers
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8082
static_dir = "static-api"
api_only = true

[sites.headers]
"X-Site-Name" = "BWS API Documentation"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "api-docs"
"X-API-Version" = "v1.0"
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE, OPTIONS"

# Development site with debug headers
[[sites]]
name = "dev"
hostname = "localhost"
port = 8083
static_dir = "static-dev"

[sites.headers]
"X-Site-Name" = "BWS Development Site"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "development"
"X-Environment" = "development"
"X-Debug-Mode" = "enabled"
```

### Configuration Options

- **server.name**: Display name for the server
- **sites**: Array of site configurations
- **sites.name**: Unique identifier for the site
- **sites.hostname**: Hostname for virtual host routing
- **sites.port**: Port number for the site
- **sites.static_dir**: Directory containing static files
- **sites.default**: Mark as default site (optional)
- **sites.api_only**: API-only site, no static files (optional)
- **sites.headers**: Custom HTTP headers to include in all responses for this site

### Configurable Headers

Each site can define custom HTTP headers that will be included in **all responses** from that site:

- **Security Headers**: CORS, CSP, security policies
- **Custom Headers**: Site identification, versioning, environment info
- **API Headers**: API versioning, CORS for API sites
- **Debug Headers**: Development flags, debug information
- **Branding Headers**: Custom site branding and identification

# Development site
[[sites]]
name = "dev"
hostname = "localhost"
port = 8083
static_dir = "static-dev"
```

### Configuration Options

- **name**: Unique identifier for the site
- **hostname**: Hostname for virtual host matching
- **port**: Port number to listen on
- **static_dir**: Directory containing static files for this site
- **default**: (optional) Set to true for the default/fallback site
- **api_only**: (optional) Set to true for API-only sites

## Available Endpoints

### Multi-Site Support
The server can host multiple websites simultaneously:
- **Main Site** (localhost:8080) - Original BWS website
- **Blog Site** (blog.localhost:8081) - Blog content with different styling
- **API Docs** (api.localhost:8082) - API documentation with dark theme
- **Dev Site** (localhost:8083) - Development environment

### Static Website Endpoints
Each site serves its own static content:
- **GET /** - Serves the site's `index.html` from its static directory
- **GET /about.html** - About page (if available in static directory)
- **GET /contact.html** - Contact page (if available in static directory)  
- **GET /static/*** - Serves static assets (CSS, JS, images, etc.) with cache headers

### API Endpoints

#### Sites Information
- **GET /api/sites** - Returns information about all configured sites
```json
{
  "server": "BWS Multi-Site Server",
  "sites": [
    {
      "name": "main",
      "hostname": "localhost", 
      "port": 8080,
      "static_dir": "static",
      "url": "http://localhost:8080"
    }
  ],
  "total_sites": 4
}
```

#### Health Check
- **GET /api/health** - Returns server health status (available on all sites)
```json
{
  "status": "ok",
  "timestamp": "2025-08-23T01:27:37.952799+00:00",
  "service": "bws-web-server"
}
```

#### File Content API
- **GET /api/file?path=filename** - Returns the content of the specified file
```json
{
  "file_path": "config.toml",
  "content": "[server]\nname = \"BWS Multi-Site Server\"...",
  "size": 567
}
```

### Error Handling
- **404 Not Found** - Returns JSON error for non-existent endpoints
```json
{
  "error": "Not Found",
  "message": "The requested endpoint does not exist",
  "available_endpoints": ["/", "/api/health", "/api/file"]
}
```

### Getting Started

#### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo

#### Dependencies
- `pingora` - High-performance proxy framework
- `async-trait` - Async trait support
- `serde` & `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `tokio` - Async runtime
- `env_logger` & `log` - Logging
- `toml` - Configuration file parsing

#### Building
```bash
cargo build
```

#### Configuration
1. Copy and modify `config.toml` to configure your sites
2. Create static directories for each site (e.g., `static-blog/`, `static-api/`)
3. Add HTML, CSS, JS files to each static directory

#### Running
```bash
# Start the multi-site server
RUST_LOG=info cargo run
```

The server will start multiple services based on your configuration.

### Testing Multiple Sites
```bash
# Run comprehensive multi-site test
./test_multisite.sh

# Test configurable headers functionality
./test_headers.sh

# Test individual sites
curl http://localhost:8080          # Main site
curl http://localhost:8081          # Blog site  
curl http://localhost:8082          # API docs site
curl http://localhost:8083          # Dev site

# Test with virtual host headers
curl -H "Host: blog.localhost:8081" http://localhost:8081

# Check sites configuration (includes header config)
curl http://localhost:8080/api/sites

# Test site-specific headers
curl -I http://localhost:8080/      # Main site headers
curl -I http://localhost:8083/      # Dev site headers (X-Debug-Mode: enabled)
curl -H "Host: api.localhost:8082" -I http://localhost:8082/  # API site CORS headers
```

#### Virtual Host Setup (Optional)
Add entries to `/etc/hosts` for easier testing:
```
127.0.0.1 blog.localhost
127.0.0.1 api.localhost
```

Then access sites via:
- http://blog.localhost:8081
- http://api.localhost:8082

## Project Structure

```
├── src/
│   ├── bin/
│   │   └── main.rs               # Server entry point and multi-site setup
│   └── lib.rs                    # Web server implementation with config support
├── static/                       # Main site files
│   ├── index.html               # Main homepage
│   ├── about.html               # About page
│   ├── contact.html             # Contact page
│   ├── styles.css               # Website styles
│   └── script.js                # Website JavaScript
├── static-blog/                  # Blog site files
│   └── index.html               # Blog homepage
├── static-api/                   # API documentation site
│   └── index.html               # API docs homepage
├── static-dev/                   # Development site files
│   └── index.html               # Dev homepage
├── config.toml                   # Multi-site configuration with headers
├── Cargo.toml                    # Project dependencies
├── test_multisite.sh             # Multi-site test script
├── test_headers.sh               # Configurable headers test script
├── test_static_server.sh         # Static website test script
└── README.md                     # This file
```

## API Implementation Details

### WebServerService
The main service implements the `ProxyHttp` trait from Pingora:
- `request_filter()`: Routes incoming requests to appropriate handlers
- `upstream_peer()`: Returns error since we handle requests locally

### Request Routing
The server intelligently routes requests based on URL patterns:
- `/` → `static/index.html`
- `/static/*` → Static assets with cache headers
- `/*.html` → HTML files from static directory
- `/api/*` → API endpoints
- Everything else → 404 error

### Request Handlers
- `handle_static_file()`: Serves static files with proper MIME types and cache headers
- `handle_health()`: Returns JSON health status
- `handle_file_content()`: Reads and returns file contents
- `handle_404()`: Returns JSON error for unknown endpoints

### Static File Features
- **MIME Type Detection**: Automatic content-type detection for:
  - HTML, CSS, JavaScript
  - Images (PNG, JPEG, GIF, SVG, ICO)
  - Fonts (WOFF, WOFF2, TTF)
  - Documents (PDF, XML, TXT)
- **Cache Headers**: `Cache-Control: public, max-age=3600` for static assets
- **Error Handling**: Graceful 404 responses for missing files

### Features
- **Content-Type Headers**: Proper MIME types for all file types
- **Content-Length**: Accurate response size calculation
- **Error Handling**: Graceful error responses with helpful messages
- **Query Parameter Parsing**: Manual parsing for file path parameter
- **Logging**: Request logging with method and URI

## Extending the Server

To add new endpoints:

1. Add a new pattern match in `request_filter()`
2. Create a new handler method following the pattern of existing handlers
3. Ensure proper error handling and response headers

Example:
```rust
"/api/new-endpoint" => {
    self.handle_new_endpoint(session).await?;
    Ok(true)
}
```

## Performance Characteristics

- **Memory Efficient**: Uses `Vec<u8>` for response bodies to avoid borrowing issues
- **Async**: Fully asynchronous request handling
- **Zero-Copy**: Efficient byte handling where possible
- **Concurrent**: Supports multiple simultaneous connections

## Security Considerations

- File reading is currently unrestricted - consider adding path validation
- No authentication/authorization implemented
- Consider rate limiting for production use
- HTTPS support can be added through Pingora configuration

### Security Status

The project uses automated security scanning via GitHub Actions. The current security status:

- **✅ Active Monitoring**: Weekly security audits via `cargo audit`
- **✅ Dependency Review**: Automated dependency review on pull requests
- **⚠️ Known Issue**: One accepted vulnerability (RUSTSEC-2024-0437) - see `SECURITY.md`

**Security Documentation**: See `SECURITY.md` for detailed security status and known issues.

**Monitoring Script**: Run `./scripts/monitor-deps.sh` to check for dependency updates and security status.

**Security Workflow**: The CI pipeline automatically scans for new vulnerabilities while ignoring documented accepted risks.

## 📦 Publishing & Releases

BWS uses automated publishing to multiple platforms:

### Automated Release Process

When you create a new version tag (e.g., `v0.1.3`), GitHub Actions automatically:

1. **📦 Publishes to crates.io** - Available via `cargo install bws-web-server`
2. **🏗️ Builds cross-platform binaries** - Linux, macOS, Windows
3. **🐳 Publishes Docker images** - To GitHub Container Registry 
4. **📝 Creates GitHub Release** - With downloadable assets

### Publishing to crates.io

**Setup Required** (one-time):
1. Get API token from [crates.io](https://crates.io/) → Account Settings → API Tokens
2. Add token as repository secret: `CARGO_REGISTRY_TOKEN`

**To publish a new version**:
```bash
# 1. Update version in Cargo.toml
sed -i 's/version = "0.1.2"/version = "0.1.3"/' Cargo.toml

# 2. Commit and tag
git add Cargo.toml
git commit -m "Bump version to 0.1.3"
git tag v0.1.3
git push origin main --tags
```

**Manual Publishing** (if needed):
- Go to **Actions** → **Publish to crates.io** workflow
- Click **Run workflow** and choose dry-run option for testing

### Release Artifacts

Each release includes:
- 📦 **crates.io package** - `cargo install bws-web-server`
- 🐧 **Linux binaries** - x86_64 (glibc + musl)
- 🍎 **macOS binaries** - x86_64 + ARM64 (Apple Silicon)
- 🪟 **Windows binaries** - x86_64 (no daemon support)
- 🐳 **Docker images** - Multi-arch (amd64/arm64)

For detailed setup instructions, see `PUBLISHING.md`.
