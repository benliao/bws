# Frequently Asked Questions (FAQ)

## General Questions

### What is BWS?

BWS (Basic Web Server) is a high-performance, multi-site web server built with Rust and powered by Cloudflare's Pingora framework. It's designed for serving static files and provides enterprise-grade features including multi-site hosting, SSL/TLS support, and comprehensive monitoring.

### Why use BWS over other web servers?

**Performance**: Built on Pingora, BWS offers exceptional performance and low latency.

**Safety**: Written in Rust, BWS provides memory safety and prevents common security vulnerabilities.

**Simplicity**: Easy configuration through TOML files and straightforward deployment.

**Multi-tenancy**: Native support for hosting multiple sites from a single instance.

**Modern Features**: Built-in support for WASM, comprehensive MIME types, and modern web standards.

### What are the system requirements?

**Minimum Requirements**:
- CPU: 1 core (x86_64 or ARM64)
- RAM: 256MB
- Storage: 100MB for binary + static content
- OS: Linux, macOS, or Windows

**Recommended for Production**:
- CPU: 4+ cores
- RAM: 2GB+
- Storage: SSD with sufficient space for content
- OS: Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)

### Is BWS production-ready?

Yes, BWS is production-ready. It's built on Cloudflare's battle-tested Pingora framework and includes:
- Comprehensive error handling
- Security hardening
- Performance optimization
- Monitoring and logging
- Automated testing
- Documentation and support

## Installation & Setup

### How do I install BWS?

You have several options:

**From pre-built binaries**:
```bash
# Download from GitHub releases
wget https://github.com/yourusername/bws/releases/latest/download/bws-linux-x86_64.tar.gz
tar -xzf bws-linux-x86_64.tar.gz
```

**From source**:
```bash
git clone https://github.com/yourusername/bws.git
cd bws
cargo build --release
```

**Using Docker**:
```bash
docker pull ghcr.io/yourusername/bws:latest
```

See the [Installation Guide](./installation.md) for detailed instructions.

### Do I need root privileges to run BWS?

No, BWS can run as a non-root user. However:

- **Ports < 1024**: Require root privileges or `CAP_NET_BIND_SERVICE` capability
- **File access**: Ensure the user can read configuration and static files
- **Log files**: Ensure the user can write to log directories

**Best practice**: Run as a dedicated user with minimal privileges:
```bash
useradd -r -s /bin/false bws
sudo -u bws ./bws --config config.toml
```

### Can I run BWS on Windows?

Yes, BWS supports Windows. However, some features may behave differently:

- Use Windows paths (`C:\path	o\files`)
- Service management differs from Linux
- Performance may vary compared to Linux

For production, Linux is recommended.

## Configuration

### How do I configure multiple sites?

Use the `[[sites]]` array in your configuration:

```toml
[[sites]]
name = "main"
hostname = "example.com"
port = 8080
static_dir = "/var/www/main"

[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 8080
static_dir = "/var/www/blog"

[[sites]]
name = "api"
hostname = "api.example.com"
port = 8081
static_dir = "/var/www/api"
```

Each site can have different configurations while sharing the same BWS instance.

### Can I use environment variables in configuration?

Currently, BWS doesn't support environment variable substitution in TOML files. However, you can:

1. **Generate configuration programmatically**:
```bash
envsubst < config.template.toml > config.toml
```

2. **Use multiple config files**:
```bash
bws --config config-${ENVIRONMENT}.toml
```

3. **Override specific settings via command line** (if supported in your version)

### How do I enable SSL/TLS?

Add SSL configuration to your site:

```toml
[[sites]]
name = "secure"
hostname = "example.com"
port = 8443
static_dir = "/var/www/secure"

[sites.ssl]
enabled = true
cert_file = "/etc/ssl/certs/example.com.pem"
key_file = "/etc/ssl/private/example.com.key"
protocols = ["TLSv1.2", "TLSv1.3"]
```

See [SSL/TLS Configuration](./configuration.md#ssl-tls-configuration) for details.

### What's the difference between hostname and domain?

- **hostname**: The network interface and host header to match
- **domain**: An alias for hostname (legacy compatibility)

Both fields serve the same purpose. Use `hostname` for new configurations.

### How do I serve files from subdirectories?

BWS automatically serves files from subdirectories. For example, with:

```toml
static_dir = "/var/www/html"
```

These URLs work automatically:
- `http://example.com/` → `/var/www/html/index.html`
- `http://example.com/about/` → `/var/www/html/about/index.html`
- `http://example.com/assets/style.css` → `/var/www/html/assets/style.css`

## Performance

### How many concurrent connections can BWS handle?

BWS can handle thousands of concurrent connections. The exact number depends on:

- **System resources**: CPU cores, RAM, file descriptors
- **Configuration**: `max_connections`, `worker_threads`
- **Content type**: Static files vs. dynamic content
- **Network conditions**: Latency, bandwidth

**Typical performance**:
- Small VPS (2 cores, 2GB RAM): 1,000-5,000 concurrent connections
- Medium server (8 cores, 16GB RAM): 10,000-50,000 concurrent connections
- Large server (32 cores, 64GB RAM): 100,000+ concurrent connections

### How do I optimize BWS performance?

**Configuration tuning**:
```toml
[performance]
worker_threads = 8           # Match CPU cores * 2
max_connections = 10000      # Based on system capacity
read_buffer_size = "64KB"    # Larger for big files
write_buffer_size = "64KB"   # Larger for throughput

[caching]
enabled = true
max_memory = "1GB"           # Adjust based on available RAM
```

**System tuning**:
```bash
# Increase file descriptor limits
echo "bws soft nofile 65536" >> /etc/security/limits.conf
echo "bws hard nofile 65536" >> /etc/security/limits.conf

# Tune kernel parameters
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65535" >> /etc/sysctl.conf
sysctl -p
```

See [Performance Tuning](./performance.md) for comprehensive optimization.

### Does BWS support caching?

Yes, BWS includes built-in caching:

```toml
[caching]
enabled = true
max_memory = "512MB"         # Memory limit for cache
ttl = 3600                   # Time-to-live in seconds
compression = true           # Enable gzip compression
```

Caching improves performance by:
- Reducing disk I/O
- Enabling content compression
- Serving frequently requested files from memory

### Can BWS serve WASM files?

Yes, BWS includes native WASM support with proper MIME types:

- `.wasm` files are served with `application/wasm`
- Supports WebAssembly streaming compilation
- Enables modern web applications

No additional configuration required - WASM support is built-in.

## Security

### Is BWS secure?

BWS implements multiple security layers:

**Language Safety**: Written in Rust, preventing memory safety vulnerabilities.

**Input Validation**: Strict validation of requests and configuration.

**Path Traversal Protection**: Prevents access to files outside static directories.

**Security Headers**: Configurable security headers.

**Regular Updates**: Dependencies are regularly updated for security fixes.

### How do I secure BWS in production?

**Basic security**:
```toml
[security]
hide_server_header = true    # Don't reveal server information
max_request_size = "10MB"    # Prevent large request attacks

[headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"X-XSS-Protection" = "1; mode=block"
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
```

**Additional measures**:
- Run as non-root user
- Use firewall to restrict access
- Enable SSL/TLS with strong ciphers
- Regular security updates
- Monitor logs for suspicious activity

See [Security Best Practices](./production.md#security) for comprehensive guidance.

### Does BWS log sensitive information?

BWS is designed to avoid logging sensitive information:

- Passwords and API keys are not logged
- Request bodies are not logged by default
- IP addresses are logged for legitimate monitoring

**Log configuration**:
```toml
[logging]
level = "info"               # Avoid "debug" in production
access_log = "/var/log/bws/access.log"
error_log = "/var/log/bws/error.log"
```

Always review logs before sharing for troubleshooting.

### Can I restrict access to certain files?

Currently, BWS serves all files from the static directory. For access control:

**File system permissions**:
```bash
# Remove read permissions for sensitive files
chmod 600 sensitive-file.txt
```

**Reverse proxy approach**:
Use nginx or Apache as a reverse proxy for advanced access control:
```nginx
location /admin/ {
    auth_basic "Admin Area";
    auth_basic_user_file /etc/nginx/.htpasswd;
    proxy_pass http://localhost:8080;
}
```

**Separate directories**:
Use different BWS sites for different access levels.

## Deployment

### How do I deploy BWS in production?

**Systemd service** (recommended for Linux):
```bash
# Create service file
sudo tee /etc/systemd/system/bws.service > /dev/null <<EOF
[Unit]
Description=BWS Web Server
After=network.target

[Service]
Type=simple
User=bws
ExecStart=/usr/local/bin/bws --config /etc/bws/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable bws
sudo systemctl start bws
```

**Docker deployment**:
```bash
docker run -d 
  --name bws 
  -p 8080:8080 
  -v /path/to/config.toml:/app/config.toml:ro 
  -v /path/to/static:/app/static:ro 
  ghcr.io/yourusername/bws:latest
```

See [Production Deployment](./production.md) for complete instructions.

### Can I use BWS with a reverse proxy?

Yes, BWS works well behind reverse proxies:

**Nginx example**:
```nginx
upstream bws {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name example.com;
    
    location / {
        proxy_pass http://bws;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

**Benefits**:
- SSL termination at proxy
- Load balancing across multiple BWS instances
- Advanced routing and caching
- Security filtering

### How do I update BWS?

**Binary updates**:
```bash
# Stop service
sudo systemctl stop bws

# Backup current binary
cp /usr/local/bin/bws /usr/local/bin/bws.backup

# Replace binary
wget https://github.com/yourusername/bws/releases/latest/download/bws-linux-x86_64.tar.gz
tar -xzf bws-linux-x86_64.tar.gz
sudo cp bws /usr/local/bin/

# Restart service
sudo systemctl start bws
```

**Docker updates**:
```bash
# Pull new image
docker pull ghcr.io/yourusername/bws:latest

# Restart with new image
docker-compose down
docker-compose up -d
```

**Source updates**:
```bash
git pull origin main
cargo build --release
sudo cp target/release/bws /usr/local/bin/
sudo systemctl restart bws
```

### Can I run multiple BWS instances?

Yes, you can run multiple BWS instances:

**Same machine, different ports**:
```toml
# Instance 1: config1.toml
[[sites]]
name = "site1"
hostname = "localhost"
port = 8080
static_dir = "/var/www/site1"

# Instance 2: config2.toml
[[sites]]
name = "site2"
hostname = "localhost"
port = 8081
static_dir = "/var/www/site2"
```

**Load balancing**:
Use a load balancer to distribute traffic across instances.

**Different machines**:
Deploy separate BWS instances on different servers for horizontal scaling.

## Troubleshooting

### BWS won't start - what should I check?

**Common issues**:

1. **Configuration errors**:
```bash
bws --config config.toml --validate
```

2. **Port already in use**:
```bash
lsof -i :8080
```

3. **File permissions**:
```bash
ls -la config.toml
ls -la /path/to/static/
```

4. **Missing dependencies**:
```bash
ldd /usr/local/bin/bws  # Linux
otool -L /usr/local/bin/bws  # macOS
```

See [Troubleshooting Guide](./troubleshooting.md) for comprehensive diagnostics.

### High memory usage - is this normal?

Memory usage depends on:

**Configuration**:
- Cache size (`max_memory`)
- Worker threads (`worker_threads`)
- Connection pool size

**Workload**:
- Number of concurrent connections
- File sizes being served
- Request frequency

**Normal ranges**:
- Minimal configuration: 10-50MB
- Production configuration: 100-500MB
- Heavy caching: 1GB+

Monitor with:
```bash
ps aux | grep bws
top -p $(pgrep bws)
```

### Request timeouts - how to fix?

**Increase timeouts**:
```toml
[performance]
request_timeout = 60         # Default: 30 seconds
response_timeout = 60        # Default: 30 seconds
keep_alive_timeout = 120     # Default: 60 seconds
```

**Check system resources**:
```bash
iostat -x 1    # Disk I/O
free -h        # Memory usage
top            # CPU usage
```

**Network diagnostics**:
```bash
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/
```

Where `curl-format.txt` contains:
```
time_namelookup:  %{time_namelookup}

time_connect:     %{time_connect}

time_pretransfer: %{time_pretransfer}

time_redirect:    %{time_redirect}

time_starttransfer: %{time_starttransfer}

time_total:       %{time_total}

```

## Development

### How can I contribute to BWS?

We welcome contributions! Here's how to get started:

1. **Fork the repository** on GitHub
2. **Clone your fork**: `git clone https://github.com/yourusername/bws.git`
3. **Create a feature branch**: `git checkout -b feature/amazing-feature`
4. **Make your changes** and add tests
5. **Run tests**: `cargo test`
6. **Submit a pull request**

See [Contributing Guide](./contributing.md) for detailed instructions.

### How do I build BWS from source?

**Prerequisites**:
```bash
# Install Rust (version 1.89+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies (Ubuntu/Debian)
sudo apt install -y pkg-config libssl-dev build-essential
```

**Build process**:
```bash
git clone https://github.com/yourusername/bws.git
cd bws
cargo build --release

# Binary will be at target/release/bws
```

See [Building from Source](./building.md) for platform-specific instructions.

### Can I extend BWS with custom features?

BWS is designed to be extensible. Common extension points:

**Custom MIME types**:
```rust
// Add to src/lib.rs
pub fn get_mime_type(path: &str) -> &'static str {
    match path.split('.').last() {
        Some("myext") => "application/x-myformat",
        _ => get_default_mime_type(path),
    }
}
```

**Custom headers**:
```toml
[headers]
"X-Custom-Header" = "Custom Value"
"Cache-Control" = "max-age=3600"
```

**Middleware integration**:
BWS is built on Pingora, which supports middleware. Consider contributing middleware to the main project.

### How do I report bugs?

**Before reporting**:
1. Check existing issues on GitHub
2. Try the latest version
3. Read the documentation
4. Gather diagnostic information

**Bug report should include**:
- BWS version (`bws --version`)
- Operating system and version
- Configuration file (sanitized)
- Complete error messages
- Steps to reproduce
- Expected vs. actual behavior

Submit issues at: https://github.com/yourusername/bws/issues

## License & Support

### What license is BWS under?

BWS is released under the [MIT License](https://opensource.org/licenses/MIT), which allows:

- Commercial use
- Modification
- Distribution
- Private use

With requirements for:
- Including the license notice
- Including the copyright notice

### Where can I get support?

**Free support**:
- GitHub Issues (bug reports)
- GitHub Discussions (questions)
- Documentation (comprehensive guides)
- Community forums

**Commercial support**:
Contact us for:
- Priority support
- Custom development
- Training and consulting
- Enterprise licensing

### How often is BWS updated?

**Regular releases**:
- Patch releases: Monthly (bug fixes)
- Minor releases: Quarterly (new features)
- Major releases: Yearly (breaking changes)

**Security updates**:
- Critical security fixes: Within 24-48 hours
- Regular security updates: Weekly

**Dependencies**:
- Rust toolchain: Follow Rust stable releases
- Pingora framework: Updated with upstream releases

Stay updated by:
- Watching the GitHub repository
- Following release notes
- Subscribing to security advisories

## Migration

### Migrating from Apache/Nginx?

**Configuration mapping**:

Apache `.htaccess` → BWS configuration:
```apache
# Apache
DocumentRoot /var/www/html
Listen 80

# BWS
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "/var/www/html"
```

**Common features**:
- Virtual hosts → Multiple sites
- SSL configuration → SSL section
- Custom headers → Headers section
- Access logs → Logging configuration

### Migrating from other Rust web servers?

**From Actix-web**:
- Replace route handlers with static file serving
- Migrate middleware to configuration
- Update deployment scripts

**From Warp**:
- Convert filters to BWS configuration
- Replace custom handlers with static serving
- Update error handling

**From Rocket**:
- Replace routes with static file configuration
- Migrate state management to external systems
- Update launch configuration

### Migrating configuration formats?

**From JSON**:
```bash
# Convert JSON to TOML
pip install json2toml
json2toml config.json config.toml
```

**From YAML**:
```bash
# Convert YAML to TOML
pip install yq
yq -t config.yaml > config.toml
```

**Manual migration**:
Review and adapt configuration according to BWS schema.

## Future Roadmap

### What's planned for future versions?

**Short-term (next 3 months)**:
- Enhanced monitoring and metrics
- Additional security features
- Performance optimizations
- Configuration validation improvements

**Medium-term (next 6 months)**:
- Plugin system
- Advanced caching strategies
- HTTP/3 support
- Enhanced SSL/TLS configuration

**Long-term (next year)**:
- GUI configuration interface
- Distributed deployment support
- Advanced load balancing
- Enterprise features

### How can I influence the roadmap?

- Submit feature requests on GitHub
- Participate in discussions
- Contribute code
- Sponsor development
- Provide feedback and use cases

---

*Don't see your question? [Ask on GitHub Discussions](https://github.com/yourusername/bws/discussions) or [check the documentation](./README.md).*
