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

- **[Hot Reload Guide](docs/src/hot-reload.md)** - Zero-downtime configuration updates
- **[Architecture Guide](docs/architecture/README.md)** - System design and modules
- **[Configuration Examples](examples/)** - Ready-to-use configs
- **[Security Guide](SECURITY.md)** - Security features and best practices

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
bws                              # Use config.toml
bws --config custom.toml         # Custom config
bws --verbose                    # Debug logging
bws --daemon                     # Background process (Unix)
```

## 🔄 True Hot Reload & Process Management

BWS implements a production-grade master-worker architecture for true hot reloading without service interruption, inspired by enterprise proxies like HAProxy and nginx.

### Master-Worker Architecture

BWS operates with a master process that spawns and manages worker processes:

- **Master Process**: Monitors configuration changes and manages worker lifecycle
- **Worker Processes**: Handle actual HTTP traffic and serve requests
- **Zero-Downtime Reloads**: New workers serve requests while old workers gracefully finish existing connections

### Hot Configuration Reload

Update configuration without restarting or dropping connections:

```bash
# Send reload signal to master process
kill -HUP $(pgrep -f "bws.*master")

# Or using process management
systemctl reload bws
```

**Hot Reload Process:**
1. Master process receives SIGHUP signal
2. Loads and validates new configuration
3. Spawns new worker process with updated config
4. New worker starts serving requests
5. Old worker gracefully finishes existing connections
6. Old worker process terminates

**What can be hot reloaded:**
- ✅ Site configurations and hostnames
- ✅ SSL certificates and ACME settings  
- ✅ Proxy routes and upstreams
- ✅ Static file directories
- ✅ Security headers and middleware
- ✅ Logging configuration
- ✅ Multi-hostname configurations
- ❌ Server ports (requires restart)
- ❌ Worker count (requires restart)

### Process Management

```bash
# Check BWS processes
ps aux | grep bws

# View master and worker processes
pgrep -a bws

# Monitor process tree
pstree -p $(pgrep -f "bws.*master")

# Graceful shutdown (stops all workers)
kill -TERM $(pgrep -f "bws.*master")
```

### Production Example

```bash
# Start BWS with hot reload capability
bws --config /etc/bws/config.toml

# Edit configuration
vim /etc/bws/config.toml

# Hot reload configuration
kill -HUP $(pgrep -f "bws.*master")

# Verify new configuration is active
curl -I http://localhost:8080/ | grep "X-Config-Version"
```

See [Hot Reload Guide](docs/src/hot-reload.md) for detailed documentation.

## 📊 API Endpoints

- `GET /api/health` - Server health status
- `GET /api/health/detailed` - Detailed system information
- `GET /` - Static content (when configured)

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
