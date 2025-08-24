# BWS (Ben's Web Server)

[![CI](https://github.com/benliao/bws/workflows/CI/badge.svg)](https://github.com/benliao/bws/actions)
[![Security](https://github.com/benliao/bws/workflows/Security/badge.svg)](https://github.com/benliao/bws/actions)
[![Crates.io](https://img.shields.io/crates/v/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![Downloads](https://img.shields.io/crates/d/bws-web-server.svg)](https://crates.io/crates/bws-web-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-mdbook-blue.svg)](https://benliao.github.io/bws/)

**A memory-safe, high-performance web server and reverse proxy built with Rust.**

Built on [Pingora](https://github.com/cloudflare/pingora), Cloudflare's production-tested proxy framework, BWS delivers enterprise-grade performance with Rust's memory safety guarantees. No buffer overflows, no use-after-free vulnerabilities, no data races – just reliable, fast web serving.

## 🎯 Why BWS?

**Memory Safety Meets Performance**: BWS eliminates entire classes of security vulnerabilities through Rust's type system. No more segmentation faults, buffer overflows, or memory leaks in production.

**Cloudflare-Grade Infrastructure**: Built on the same Pingora framework that handles 20% of the internet's traffic, BWS brings enterprise-level performance and reliability to your applications.

**SSL Made Simple**: Automatic certificate management means your sites stay secure without manual intervention. Let's Encrypt integration handles renewals, monitoring, and deployment seamlessly.

**Developer Experience**: Hot configuration reloading, comprehensive monitoring, and clear error messages make BWS a joy to operate in production environments.

## ✨ Key Features

### 🛡️ **Memory Safety First**
- **Zero Buffer Overflows**: Rust's ownership system prevents memory corruption at compile time
- **Eliminated Use-After-Free**: No dangling pointers or memory access violations
- **Data Race Protection**: Safe concurrent access guaranteed by Rust's type system
- **Predictable Performance**: No garbage collection pauses or unpredictable latency spikes

### ⚡ **Enterprise Performance**
- **Production-Grade Foundation**: Built on Pingora, serving billions of requests at Cloudflare
- **Zero-Copy Operations**: Efficient memory handling with minimal allocations
- **Async-First Architecture**: Non-blocking I/O for maximum throughput
- **Optimized Binary**: Native compilation with aggressive optimizations

### 🔒 **SSL/TLS Excellence**
- **Automatic Certificate Management**: Let's Encrypt integration with zero-downtime renewal
- **Perfect Forward Secrecy**: Modern TLS configurations for maximum security
- **SNI Support**: Multiple certificates per instance for different domains
- **ACME Protocol**: Industry-standard automatic certificate provisioning

### 🌐 **Advanced Networking**
- **Multi-Site Hosting**: Host multiple websites with independent configurations
- **Intelligent Load Balancing**: Round-robin, weighted, and least-connections algorithms
- **WebSocket Proxying**: Full WebSocket support with load balancing
- **Header Management**: Flexible request/response header manipulation
- **Path-Based Routing**: Sophisticated URL routing and rewriting

### 🔧 **Operations Excellence**
- **Hot Configuration Reload**: Update settings without service interruption
- **Comprehensive Monitoring**: Built-in health checks and metrics endpoints
- **Docker Ready**: Multi-architecture container images for easy deployment
- **Cross-Platform**: Native binaries for Linux, macOS, and Windows

## 📦 Installation

### From crates.io
```bash
cargo install bws-web-server
```

### From Docker (Recommended for Production)
```bash
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest
```

### From Source
```bash
git clone https://github.com/benliao/bws.git
cd bws
cargo build --release
```

## 🚀 Quick Start

1. **Create a configuration file** (`config.toml`):
```toml
[server]
name = "BWS Server"

# Basic HTTP site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

# Reverse proxy site with load balancing
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8090

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"
weight = 1

[[sites.proxy.routes]]
path = "/api/"
upstream = "backend"

[sites.proxy.load_balancing]
method = "round_robin"

# HTTPS site with automatic SSL
[[sites]]
name = "secure"
hostname = "secure.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
challenge_dir = "./acme-challenges"
```

2. **Create static content**:
```bash
mkdir -p static acme-challenges
echo "<h1>Welcome to BWS!</h1>" > static/index.html
```

3. **Run the server**:
```bash
bws
```

4. **Test your setup**:
```bash
curl http://localhost:8080
curl http://localhost:8080/api/health
```

## 🔄 Reverse Proxy & Load Balancing

BWS provides comprehensive reverse proxy functionality with intelligent load balancing:

### Load Balancing Algorithms
- **Round Robin**: Distributes requests evenly across servers
- **Weighted**: Routes based on server capacity weights
- **Least Connections**: Routes to server with fewest active connections

### Configuration Example
```toml
[[sites]]
name = "proxy"
hostname = "api.example.com"
port = 8080

[sites.proxy]
enabled = true

# Backend server pool
[[sites.proxy.upstreams]]
name = "api-cluster"
url = "http://api1.internal:8080"
weight = 3

[[sites.proxy.upstreams]]
name = "api-cluster"
url = "http://api2.internal:8080"
weight = 2

# Routing configuration
[[sites.proxy.routes]]
path = "/api/v1/"
upstream = "api-cluster"
strip_prefix = false

[sites.proxy.load_balancing]
method = "weighted"

# Header management
[sites.proxy.headers]
add_x_forwarded = true
add_forwarded = true

[sites.proxy.headers.add]
"X-Proxy-Server" = "BWS"
```

### Testing
```bash
# Test load balancing
./tests/test_load_balance.sh

# Verify proxy headers
curl -v http://api.example.com/api/v1/users
```

## 🔒 SSL/TLS Configuration

### Automatic Certificates (Let's Encrypt)
```toml
[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"
```

### Manual Certificates
```toml
[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/example.com.crt"
key_file = "/etc/ssl/private/example.com.key"
```

## 🐳 Docker Deployment

### Quick Start
```bash
# Basic deployment
docker run -d -p 8080:8080 ghcr.io/benliao/bws:latest

# With custom configuration
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest
```

### Docker Compose
```yaml
version: '3.8'
services:
  bws:
    image: ghcr.io/benliao/bws:latest
    ports:
      - "8080:8080"
      - "443:443"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./static:/app/static:ro
      - ./certs:/app/certs:ro
    environment:
      - RUST_LOG=info
```

## 💻 Command Line Options

```bash
# Basic usage
bws                                    # Use config.toml
bws --config custom.toml              # Custom config
bws --verbose                         # Debug logging

# Daemon mode (Unix only)
bws --daemon                          # Background process
bws --daemon --pid-file /var/run/bws.pid --log-file /var/log/bws.log

# Help and version
bws --help
bws --version
```

## 📊 API Endpoints

### Health & Monitoring
- `GET /api/health` - Server health status
- `GET /api/sites` - Site configuration overview

### Static Content
- `GET /` - Serves index.html
- `GET /static/*` - Static assets with cache headers
- `GET /*.html` - HTML pages

### Example Response
```json
{
  "status": "ok",
  "timestamp": "2025-08-24T12:00:00Z",
  "service": "bws-web-server"
}
```

## 🧪 Testing

```bash
# Comprehensive testing
./tests/test_multisite.sh
./tests/test_load_balance.sh
./tests/test_websocket_proxy.sh

# Individual tests
curl http://localhost:8080/api/health
curl -I http://localhost:8080          # Check headers
python3 ./tests/test_websocket_client.py
```

## 🔧 Configuration Reference

### Core Configuration
- `server.name`: Server display name
- `sites[]`: Array of site configurations
- `sites[].hostname`: Virtual host matching
- `sites[].port`: Listen port
- `sites[].static_dir`: Static content directory

### SSL Configuration
- `sites[].ssl.enabled`: Enable HTTPS
- `sites[].ssl.auto_cert`: Automatic certificates
- `sites[].ssl.acme.*`: Let's Encrypt settings

### Proxy Configuration
- `sites[].proxy.enabled`: Enable reverse proxy
- `sites[].proxy.upstreams[]`: Backend servers
- `sites[].proxy.routes[]`: URL routing rules
- `sites[].proxy.load_balancing.method`: Load balancing algorithm

## 📚 Documentation

**[Complete Documentation →](https://benliao.github.io/bws/)**

- 🚀 [Quick Start Guide](https://benliao.github.io/bws/quick-start/)
- 🔧 [Configuration Reference](https://benliao.github.io/bws/configuration/)
- 🔄 [Load Balancing Guide](docs/load-balancing.md)
- 🐳 [Docker Deployment](https://benliao.github.io/bws/docker/)
- 📖 [API Reference](https://benliao.github.io/bws/api/)

## 🏗️ Project Structure

```
├── src/
│   ├── bin/main.rs           # Server entry point
│   ├── config/site.rs        # Configuration structures
│   ├── handlers/             # Request handlers
│   ├── server/               # Core server logic
│   └── ssl/                  # SSL/TLS management
├── tests/                    # Test scripts and configs
├── static/                   # Example static content
├── docs/                     # Documentation
└── config.toml              # Example configuration
```

## 🔒 Security

BWS uses automated security scanning:
- ✅ **Weekly Security Audits** via `cargo audit`
- ✅ **Dependency Review** on pull requests
- ✅ **Memory Safety** guaranteed by Rust

See `SECURITY.md` for detailed security information.

## 📦 Release Management

### Automated Releases
Creating a version tag (e.g., `v0.2.1`) automatically:
1. Publishes to crates.io
2. Builds cross-platform binaries
3. Creates Docker images
4. Generates GitHub releases

### Manual Release
```bash
# Update version
sed -i 's/version = "0.2.0"/version = "0.2.1"/' Cargo.toml

# Tag and push
git add Cargo.toml
git commit -m "Bump version to 0.2.1"
git tag v0.2.1
git push origin main --tags
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
