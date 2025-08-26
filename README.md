# BWS (Blazing Web Server)

[![CI](https://github.com/benliao/bws/workflows/CI/badge.svg)](https://github.com/benliao/bws/actions)
[![Security](https://github.com/benliao/bws/workflows/Security/badge.svg)](https://github.com/benliao/bws/actions)
[![Crates.io](https://img.shields.io/crates/v/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![Downloads](https://img.shields.io/crates/d/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, memory-safe web server and reverse proxy built with Rust and Cloudflare's Pingora framework.

## Features

- **Multi-Site Hosting** - Multiple websites with individual configurations
- **Multi-Hostname Support** - Handle multiple domains per site
- **Automatic SSL/TLS** - Let's Encrypt integration with auto-renewal
- **Load Balancing** - Round-robin, weighted, and least-connections algorithms
- **WebSocket Proxy** - Full WebSocket support with load balancing
- **HTTP Compression** - Gzip, Brotli, and Deflate compression support
- **Health Monitoring** - Built-in health checks and metrics
- **Memory Safety** - Rust eliminates buffer overflows and memory leaks
- **Hot Reload** - Zero-downtime configuration updates
- **Enterprise Management** - Production-ready process management

## Quick Start

### Instant Directory Server
```bash
# Serve current directory on port 80
bws .

# Serve specific directory on custom port
bws /path/to/website --port 8080
```

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
Create `config.toml` for production:
```toml
[server]
name = "BWS Server"

[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.ssl]
enabled = false

# Multiple sites on same port (virtual hosting)
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8080
static_dir = "static-blog"

# Reverse proxy setup
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8080

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
# Quick start - serve directory
bws static --port 8080

# With configuration file
bws -c config.toml

# Validate configuration first
bws -c config.toml --dry-run
```

## Architecture

BWS uses a modular, enterprise-grade architecture:

```
src/
├── core/              # Foundation: types, error handling
├── config/            # Configuration management
├── handlers/          # Request processing (static, API, proxy, WebSocket)
├── middleware/        # CORS, security headers, rate limiting
├── monitoring/        # Health checks, metrics, certificate monitoring
├── server/            # Server infrastructure
└── ssl/               # SSL/TLS and certificate management
```

## CLI Options

```bash
bws                                  # Use config.toml  
bws --config custom.toml             # Custom config file
bws /path/to/directory               # Serve directory directly
bws /path/to/directory --port 8080   # Custom port
bws --verbose                        # Enable debug logging
bws --daemon                         # Background process (Unix only)
bws --dry-run                        # Validate configuration
```

## Configuration Validation

Validate before deployment:

```bash
# Validate configuration
bws --config config.toml --dry-run

# Validate before starting
bws --config production.toml --dry-run && bws --config production.toml
```

The validator checks:
- TOML syntax and schema compliance
- Static directories and index files
- SSL certificates and ACME settings
- Proxy upstream configurations
- Port conflicts and virtual hosting setup

## Hot Reload

Update configuration without restarting:

```bash
# Reload via API
curl -X POST http://localhost:8080/api/reload

# Using signals (Unix)
kill -HUP $(pgrep -f "bws.*master")
```

What can be reloaded:
- Site configurations and hostnames
- SSL certificates and ACME settings  
- Proxy routes and upstreams
- Static file directories
- Security headers

**Note**: Server ports require restart.

## API Endpoints

BWS provides monitoring and management APIs:

- `GET /api/health` - Server health status
- `GET /api/health/detailed` - Detailed system information  
- `GET /api/sites` - List configured sites
- `POST /api/reload` - Hot reload configuration

Example:
```bash
curl http://localhost:8080/api/health
curl http://localhost:8080/api/sites | jq
curl -X POST http://localhost:8080/api/reload
```

## Docker

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

## Security

- **Memory Safety**: Rust prevents buffer overflows and memory leaks
- **Zero Panics**: Comprehensive error handling throughout
- **Security Headers**: HSTS, CSP, XSS protection built-in
- **Path Traversal Protection**: Secure static file serving
- **Rate Limiting**: Configurable request throttling

## Documentation

- [Quick Start Guide](docs/src/quick-start.md)
- [Configuration Guide](docs/src/configuration.md)
- [Hot Reload Guide](docs/src/hot-reload.md)
- [Architecture Guide](docs/architecture/README.md)
- [API Documentation](docs/src/api.md)
- [Security Guide](SECURITY.md)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/name`)
3. Commit your changes (`git commit -m 'Add feature'`)
4. Push to the branch (`git push origin feature/name`)
5. Open a Pull Request

## License

Licensed under the [MIT License](LICENSE).

---

**BWS** - Enterprise-grade web serving, simplified.
