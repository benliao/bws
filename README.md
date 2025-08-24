# 🚀 BWS (Ben's Web Server)

[![CI](https://github.com/benliao/bws/workflows/CI/badge.svg)](https://github.com/benliao/bws/actions)
[![Security](https://github.com/benliao/bws/workflows/Security/badge.svg)](https://github.com/benliao/bws/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, memory-safe web server and reverse proxy built with Rust and Cloudflare's Pingora framework.

## ✨ Features

- 🌐 **Multi-Site Hosting** - Multiple websites with individual configurations
- 🔒 **Automatic SSL/TLS** - Let's Encrypt integration with auto-renewal
- ⚡ **Load Balancing** - Round-robin, weighted, and least-connections algorithms
- 🔌 **WebSocket Proxy** - Full WebSocket support with load balancing
- 📊 **Health Monitoring** - Built-in health checks and metrics
- 🛡️ **Memory Safety** - Rust eliminates buffer overflows and memory leaks
- 🔧 **Hot Reload** - Update configuration without downtime

## 🚀 Quick Start

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

### Basic Configuration
Create `config.toml`:
```toml
[server]
name = "BWS Server"

# Static website
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

# Reverse proxy with load balancing
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8090

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
mkdir static && echo "<h1>Hello BWS!</h1>" > static/index.html
bws
```

## 📖 Documentation

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
