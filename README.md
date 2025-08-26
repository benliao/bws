# <img src="assets/logo.svg" alt="BWS Logo" width="32" height="32"> BWS (Blazing Web Server)

[![CI](https://github.com/benliao/bws/workflows/CI/badge.svg)](https://github.com/benliao/bws/actions)
[![Security](https://github.com/benliao/bws/workflows/Security/badge.svg)](https://github.com/benliao/bws/actions)
[![Crates.io](https://img.shields.io/crates/v/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![Downloads](https://img.shields.io/crates/d/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, memory-safe web server and reverse proxy built with Rust and Cloudflare's Pingora framework.

## ✨ Features

- 🌐 **Multi-Site Hosting** - Multiple websites with individual configurations
- 🎯 **Multi-Hostname Support** - Handle multiple domains for single site
- 🔒 **Automatic SSL/TLS** - Let's Encrypt integration with auto-renewal
- ⚡ **Load Balancing** - Round-robin, weighted, and least-connections algorithms
- 🔌 **WebSocket Proxy** - Full WebSocket support with load balancing
- 🗜️ **HTTP Compression** - Gzip, Brotli, and Deflate compression support
- 📊 **Health Monitoring** - Built-in health checks and metrics
- 🛡️ **Memory Safety** - Rust eliminates buffer overflows and memory leaks
- 🔧 **True Hot Reload** - Master-worker architecture for zero-downtime configuration updates
- 🚀 **Zero-Downtime Operations** - Configuration and binary updates without dropping connections
- 🛠️ **Enterprise-Grade Management** - Production-ready process management and monitoring

## 🚀 Quick Start

### Instant Directory Server
Start serving any directory immediately:
```bash
# Serve current directory on port 80
bws .

# Serve specific directory on custom port
bws /path/to/website --port 8080

# Windows example
bws.exe C:\websites\mysite --port 8080
```
No configuration file needed! Perfect for development and quick file sharing.

### Installation
```bash
# From crates.io
cargo install bws-web-server

# From Docker
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest

# From source
git clone https://github.com/benliao/bws.git && cd bws
cargo build --release
```

### Configuration-Based Setup
For production deployments, create `config.toml`:
```toml
[server]
name = "BWS Server"

# Virtual hosting: Multiple sites on same port
[[sites]]
name = "company-main"
hostname = "company.com"
hostnames = ["www.company.com"]  # Multi-hostname for same site
port = 8080
static_dir = "examples/sites/static"
default = true

[[sites]]
name = "company-blog"
hostname = "blog.company.com"    # Different site, same port
port = 8080
static_dir = "examples/sites/static-blog"        # Different content

[[sites]]
name = "company-api"
hostname = "api.company.com"     # Another site, same port
port = 8080
static_dir = "examples/sites/static-api"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"

[[sites.proxy.routes]]
path = "/api/"
upstream = "backend"

# HTTPS with automatic certificates
[[sites]]
name = "secure"
hostname = "example.com"
port = 443

[sites.ssl]
enabled = true
auto_cert = true

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
```

### Run
```bash
# Quick start - serve directory directly
bws static --port 8080

# Or with configuration file
mkdir static && echo "<h1>Hello BWS!</h1>" > static/index.html
bws -c config.toml
```

## 📖 Documentation

- **[Quick Start Guide](docs/src/quick-start.md)** - Get up and running in minutes
- **[Configuration Guide](docs/src/configuration.md)** - Comprehensive configuration reference
- **[Hot Reload Guide](docs/src/hot-reload.md)** - Zero-downtime configuration updates
- **[Architecture Guide](docs/architecture/README.md)** - System design and modules
- **[Testing Guide](docs/src/testing.md)** - Testing methodology and scripts
- **[Configuration Examples](examples/)** - Ready-to-use configurations
- **[Security Guide](SECURITY.md)** - Security features and best practices
- **[API Documentation](docs/src/api.md)** - REST API reference

## 🏗️ Architecture

BWS uses a modular, enterprise-grade architecture:

```
src/
├── core/              # Foundation: types, error handling, utilities
├── config/            # Configuration management
├── handlers/          # Request processing (static, API, proxy, WebSocket)
├── middleware/        # CORS, security headers, rate limiting
├── monitoring/        # Health checks, metrics, certificate monitoring
├── server/            # Server infrastructure
└── ssl/               # SSL/TLS and certificate management
```

## 🔧 CLI Options

```bash
bws                                  # Use config.toml  
bws --config custom.toml             # Custom config file
bws /path/to/directory               # Serve directory directly
bws /path/to/directory --port 8080   # Custom port for directory serving
bws --verbose                        # Enable debug logging
bws --daemon                         # Run as background process (Unix only)
bws --dry-run                        # Validate configuration without starting server
bws --help                           # Show all available options
bws --version                        # Show version information
```

### Configuration Validation

BWS now includes comprehensive configuration validation with the `--dry-run` flag:

```bash
# Validate configuration file without starting server
bws --config config.toml --dry-run

# Validate before starting server  
bws --config production.toml --dry-run && bws --config production.toml

# Check example configurations
bws --config examples/basic-single-site.toml --dry-run
```

The validator performs comprehensive checks:
- ✅ **TOML Syntax**: Validates configuration file format
- ✅ **Required Fields**: Ensures all necessary configuration sections exist
- ✅ **Static Directories**: Verifies that specified directories exist
- ✅ **SSL Certificates**: Checks certificate file availability
- ✅ **Proxy Configuration**: Validates upstream configurations
- ✅ **Port Conflicts**: Warns about potential virtual hosting issues
- ✅ **Schema Compliance**: Ensures configuration matches current schema

## 🔄 Configuration Reload

BWS supports real-time configuration reloading through a simple API endpoint, allowing you to update configurations without restarting the server.

### API-Based Reload

Update configuration without restarting:

```bash
# Reload configuration via API
curl -X POST http://localhost:8080/api/reload

# Or using your configured port
curl -X POST http://localhost:8081/api/reload
```

**Reload Process:**
1. Send POST request to `/api/reload` endpoint
2. BWS validates new configuration file
3. If valid, applies new configuration immediately
4. Returns success/error status

**What can be reloaded:**
- ✅ Site configurations and hostnames
- ✅ SSL certificates and ACME settings  
- ✅ Proxy routes and upstreams
- ✅ Static file directories
- ✅ Security headers and middleware
- ✅ Multi-hostname configurations
- ❌ Server ports (requires restart)

### Production Example

```bash
# Start BWS
bws --config /etc/bws/config.toml

# Edit configuration
vim /etc/bws/config.toml

# Validate configuration before applying
bws --config /etc/bws/config.toml --dry-run

# Reload configuration
curl -X POST http://localhost:8080/api/reload

# Verify new configuration is active
curl -I http://localhost:8080/ | grep "Server:"
```

### Configuration Validation

BWS includes a built-in configuration validator that checks your configuration files without starting the server:

```bash
# Validate configuration file
bws --config config.toml --dry-run

# Validate directory serving setup
bws /path/to/website --port 8080 --dry-run
```

The validator checks for:
- ✅ **TOML Syntax**: Validates configuration file format
- ✅ **Required Fields**: Ensures all necessary configuration sections exist  
- ✅ **Static Directories**: Verifies that specified directories exist
- ✅ **SSL Certificate Files**: Checks certificate file availability
- ✅ **Proxy Configuration**: Validates upstream configurations
- ✅ **Schema Compliance**: Ensures configuration matches expected structure
- ⚠️  **Port Conflicts**: Warns about potential virtual hosting setup issues
- ⚠️  **Missing Files**: Reports missing index files and referenced paths

**Example validation output:**
```
🔍 BWS Configuration Validation (Dry Run Mode)
==========================================
✅ Configuration file 'config.toml' loaded successfully

📊 Configuration Summary:
   Server: BWS Multi-Site Server v0.3.4
   Sites: 4

🌐 Site 1: main
   Hostname: localhost
   Port: 8080
   Static directory: examples/sites/static
   ✅ Static directory exists
   📋 Custom headers: 4

==========================================
           VALIDATION RESULTS
==========================================
✅ Configuration validation passed!
🚀 Configuration is ready for deployment
```

See [Hot Reload Guide](docs/src/hot-reload.md) for detailed documentation.

## 📊 API Endpoints

BWS provides a RESTful API for monitoring and management:

- `GET /api/health` - Basic server health status
- `GET /api/health/detailed` - Detailed system information  
- `GET /api/sites` - List all configured sites
- `POST /api/reload` - Hot reload configuration without restart
- `GET /` - Static content (when configured)

### API Examples

```bash
# Check server health
curl http://localhost:8080/api/health

# Get detailed system information
curl http://localhost:8080/api/health/detailed | jq

# List all configured sites
curl http://localhost:8080/api/sites | jq

# Hot reload configuration
curl -X POST http://localhost:8080/api/reload
```

## 🐳 Docker

```bash
# Quick start
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest

# With custom config
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest
```

## 🛡️ Security

- **Memory Safety**: Rust's type system prevents entire classes of vulnerabilities
- **Zero Panics**: Comprehensive error handling throughout
- **Security Headers**: HSTS, CSP, XSS protection built-in
- **Path Traversal Protection**: Secure static file serving
- **Rate Limiting**: Configurable request throttling

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

Licensed under the [MIT License](LICENSE).

---

**BWS** - Enterprise-grade web serving, simplified. Built with ❤️ in Rust.
