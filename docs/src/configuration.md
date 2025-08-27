# Configuration

BWS uses TOML configuration files to define server and site settings.

## Basic Structure

```toml
[server]
name = "BWS Server"

[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true
```

## Configuration Validation

Always validate your configuration before starting:

```bash
bws --config config.toml --dry-run
```

## Server Configuration

Global server settings:

```toml
[server]
name = "BWS Production Server"         # Server name
```

## Sites Configuration

Define websites using `[[sites]]` arrays:

### Required Fields

```toml
[[sites]]
name = "main"                          # Unique site identifier
hostname = "example.com"               # Domain name
port = 443                             # Port number
static_dir = "static"                  # Static files directory
```

### Optional Fields

```toml
default = true                         # Default site (catches all requests)
api_only = false                       # API-only mode (no static files)
hostnames = ["www.example.com"]        # Additional hostnames
```

## SSL/TLS Configuration

### Automatic SSL (Let's Encrypt)

```toml
[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false                        # Use false for production
challenge_dir = "./acme-challenges"
```

### Manual SSL

```toml
[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./certs/example.com.crt"
key_file = "./certs/example.com.key"
```

## Custom Headers

```toml
[sites.headers]
"X-Powered-By" = "BWS"
"Cache-Control" = "public, max-age=3600"
"Strict-Transport-Security" = "max-age=31536000"
```

## Reverse Proxy

```toml
[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"
weight = 1

[[sites.proxy.routes]]
path = "/api/"
upstream = "backend"
strip_prefix = false
websocket = false

[sites.proxy.load_balancing]
method = "round_robin"                 # round_robin, weighted, least_connections
```

## Management API

```toml
[management]
enabled = true                         # Disabled by default - must enable explicitly
host = "127.0.0.1"                     # Always localhost for security
port = 7654
api_key = "your-secure-key"            # Optional but recommended
```

Usage:
```bash
# Reload configuration
curl -X POST http://127.0.0.1:7654/api/config/reload \
  -H "X-API-Key: your-secure-key"
```

## Performance Settings

```toml
[performance]
worker_threads = 8
max_connections = 1000
keep_alive_timeout = 60
request_timeout = 30
```

## Security Settings

```toml
[security]
hide_server_header = true
max_request_size = "10MB"

[security.security_headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"X-XSS-Protection" = "1; mode=block"
```

## Logging

```toml
[logging]
level = "info"                         # debug, info, warn, error
format = "combined"                    # combined, compact, json
log_requests = true
```

## Complete Example

```toml
[server]
name = "BWS Production Server"

# Main HTTPS site
[[sites]]
name = "main"
hostname = "example.com"
port = 443
static_dir = "static"
default = true

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000"
"X-Frame-Options" = "DENY"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false

# API proxy site
[[sites]]
name = "api"
hostname = "api.example.com"
port = 443
static_dir = "static"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "api-backend"
url = "http://127.0.0.1:3001"

[[sites.proxy.routes]]
path = "/v1/"
upstream = "api-backend"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["api.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

# Management API
[management]
enabled = true
port = 7654
api_key = "secure-random-key"

# Performance
[performance]
worker_threads = 8
max_connections = 1000

# Security
[security]
hide_server_header = true

[security.security_headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"

# Logging
[logging]
level = "info"
log_requests = true
```

## File Locations

BWS looks for configuration files in this order:
1. File specified with `--config` flag
2. `config.toml` in current directory
3. `bws.toml` in current directory

## Environment Variables

Override configuration with environment variables:
- `BWS_CONFIG_FILE` - Configuration file path
- `BWS_LOG_LEVEL` - Logging level (debug, info, warn, error)
- `BWS_WORKERS` - Number of worker threads
