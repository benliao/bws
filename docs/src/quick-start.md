# Quick Start

Get BWS running in minutes with these simple options.

## Option 1: Instant Directory Server

Serve files immediately without configuration:

```bash
# Serve current directory on port 80
bws .

# Serve specific directory on custom port  
bws /path/to/website --port 8080

# Validate setup before starting
bws /path/to/website --port 8080 --dry-run
```

**Example:**
```bash
mkdir my-website
echo "<h1>Welcome to BWS!</h1>" > my-website/index.html
bws my-website --port 8080
```

## Option 2: Configuration File Setup

### 1. Create Configuration

Create `config.toml`:

```toml
[server]
name = "BWS Server"

# HTTP site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Powered-By" = "BWS"

# HTTPS site with auto SSL
[[sites]]
name = "secure"
hostname = "secure.localhost"
port = 8443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = true
challenge_dir = "./acme-challenges"
```

### 2. Create Content

```bash
mkdir -p static acme-challenges
cat > static/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>BWS Server</title></head>
<body>
    <h1>🚀 BWS is Running!</h1>
    <ul>
        <li><a href="http://localhost:8080">HTTP Site</a></li>
        <li><a href="https://secure.localhost:8443">HTTPS Site</a></li>
        <li><a href="/api/health">Health Check</a></li>
    </ul>
</body>
</html>
EOF
```

### 3. Start Server

```bash
# Validate first
bws --config config.toml --dry-run

# Start server
bws --config config.toml
```

## Test Your Setup

```bash
# Test HTTP
curl http://localhost:8080/

# Health check
curl http://localhost:8080/api/health

# Test HTTPS (add to /etc/hosts if needed)
curl -k https://secure.localhost:8443/
```

## Management API

BWS includes an optional secure management API for configuration reloads. To enable it, add to your config:

```toml
[management]
enabled = true
host = "127.0.0.1"
port = 7654
api_key = "your-secure-key"  # Optional but recommended
```

Usage:
```bash
# Hot reload configuration
curl -X POST http://127.0.0.1:7654/api/config/reload
```

## Command Options

```bash
# Common options
bws --config config.toml          # Use config file
bws --verbose                     # Enable verbose logging
bws --daemon                      # Run as daemon (Unix only)
bws --dry-run                     # Validate config only
```

## Next Steps

- [Configuration](./configuration.md) - Detailed configuration options
- [SSL/TLS Setup](./ssl-tls.md) - HTTPS configuration  
- [Multi-Site](./multi-site.md) - Host multiple websites
- [Docker](./docker.md) - Container deployment
