# Configuration Schema

This document provides a complete reference for all BWS configuration options in the `config.toml` file.

## Configuration File Structure

BWS uses TOML format for configuration. The basic structure is:

```toml
# Global settings
[daemon]
# Daemon configuration

[logging]
# Logging configuration

[performance]
# Performance tuning

[monitoring]
# Health monitoring

# Site definitions
[[sites]]
# First site configuration

[[sites]]
# Second site configuration
```

## Global Configuration Sections

### Daemon Configuration

Controls how BWS runs as a daemon process.

```toml
[daemon]
user = "bws"                           # User to run as (string)
group = "bws"                          # Group to run as (string)
pid_file = "/var/run/bws.pid"          # PID file location (string)
working_directory = "/opt/bws"         # Working directory (string)
daemonize = true                       # Run as daemon (boolean)
```

**Parameters:**
- `user` (string, optional): System user to run BWS as. Default: current user
- `group` (string, optional): System group to run BWS as. Default: current user's group
- `pid_file` (string, optional): Path to store process ID file. Default: no PID file
- `working_directory` (string, optional): Change to this directory on startup
- `daemonize` (boolean, optional): Fork and run in background. Default: `false`

### Management API Configuration

Controls the secure Management API for administrative operations.

```toml
[management]
enabled = true                         # Enable Management API (boolean)
host = "127.0.0.1"                    # Host to bind to (string)
port = 7654                           # Port number (integer)
api_key = "your-secure-api-key"       # API key for authentication (string)
```

**Parameters:**
- `enabled` (boolean, optional): Enable the Management API service. Default: `false`
- `host` (string, optional): Host address to bind to. Always `127.0.0.1` for security. Default: `"127.0.0.1"`
- `port` (integer, optional): Port number for the Management API. Default: `7654`
- `api_key` (string, optional): API key for authentication. If not set, no authentication required. Default: `null`

**Security Features:**
- Management API always binds to localhost only for security
- IP address validation ensures requests come from localhost
- Optional API key authentication for additional security
- All management operations are logged with client IP

**Available Endpoints:**
- `POST /api/config/reload`: Reload server configuration

### Logging Configuration

Controls logging behavior and output.

```toml
[logging]
level = "info"                         # Log level (string)
output = "stdout"                      # Output destination (string)
format = "json"                        # Log format (string)
file_path = "/var/log/bws/bws.log"    # Log file path (string)
max_size = "100MB"                     # Maximum log file size (string)
max_files = 10                         # Number of log files to keep (integer)
compress = true                        # Compress rotated logs (boolean)
include_fields = [                     # Fields to include in logs (array)
    "timestamp",
    "level", 
    "message",
    "request_id"
]
```

**Parameters:**
- `level` (string, optional): Log level. Values: `trace`, `debug`, `info`, `warn`, `error`. Default: `info`
- `output` (string, optional): Where to send logs. Values: `stdout`, `stderr`, `file`. Default: `stdout`
- `format` (string, optional): Log format. Values: `text`, `json`. Default: `text`
- `file_path` (string, required if output="file"): Path to log file
- `max_size` (string, optional): Maximum size before rotation. Examples: `10MB`, `1GB`. Default: `100MB`
- `max_files` (integer, optional): Number of rotated files to keep. Default: `10`
- `compress` (boolean, optional): Compress rotated log files. Default: `true`
- `include_fields` (array, optional): Fields to include in structured logs

### Performance Configuration

Tuning parameters for performance optimization.

```toml
[performance]
worker_threads = 8                     # Number of worker threads (integer)
max_blocking_threads = 512             # Max blocking threads (integer)
max_connections = 10000                # Maximum concurrent connections (integer)
keep_alive_timeout = 60                # Keep-alive timeout in seconds (integer)
request_timeout = 30                   # Request timeout in seconds (integer)
response_timeout = 30                  # Response timeout in seconds (integer)
read_buffer_size = "64KB"              # Read buffer size (string)
write_buffer_size = "64KB"             # Write buffer size (string)
max_request_size = "10MB"              # Maximum request size (string)
connection_pool_size = 1000            # Connection pool size (integer)
connection_pool_idle_timeout = 300     # Pool idle timeout in seconds (integer)
```

**Parameters:**
- `worker_threads` (integer, optional): Number of async worker threads. Default: number of CPU cores
- `max_blocking_threads` (integer, optional): Maximum blocking threads for file I/O. Default: `512`
- `max_connections` (integer, optional): Maximum concurrent connections. Default: `10000`
- `keep_alive_timeout` (integer, optional): HTTP keep-alive timeout in seconds. Default: `60`
- `request_timeout` (integer, optional): Request processing timeout in seconds. Default: `30`
- `response_timeout` (integer, optional): Response sending timeout in seconds. Default: `30`
- `read_buffer_size` (string, optional): Buffer size for reading requests. Default: `8KB`
- `write_buffer_size` (string, optional): Buffer size for writing responses. Default: `8KB`
- `max_request_size` (string, optional): Maximum allowed request size. Default: `1MB`
- `connection_pool_size` (integer, optional): Size of connection pool. Default: `100`
- `connection_pool_idle_timeout` (integer, optional): Idle timeout for pooled connections. Default: `300`

### Monitoring Configuration

Health monitoring and metrics collection.

```toml
[monitoring]
enabled = true                         # Enable monitoring (boolean)
health_endpoint = "/health"            # Health check endpoint (string)
detailed_endpoint = "/health/detailed" # Detailed health endpoint (string)
metrics_endpoint = "/metrics"          # Metrics endpoint for Prometheus (string)

[monitoring.checks]
disk_threshold = 90                    # Disk usage alert threshold (integer)
memory_threshold = 80                  # Memory usage alert threshold (integer)
response_time_threshold = 1000         # Response time alert threshold in ms (integer)

[monitoring.prometheus]
enabled = true                         # Enable Prometheus metrics (boolean)
endpoint = "/metrics"                  # Metrics endpoint path (string)
port = 9090                           # Metrics server port (integer)
```

**Parameters:**
- `enabled` (boolean, optional): Enable health monitoring. Default: `true`
- `health_endpoint` (string, optional): Path for basic health checks. Default: `/health`
- `detailed_endpoint` (string, optional): Path for detailed health info. Default: `/health/detailed`
- `metrics_endpoint` (string, optional): Path for Prometheus metrics. Default: `/metrics`

**Monitoring Checks:**
- `disk_threshold` (integer, optional): Alert when disk usage exceeds this percentage. Default: `90`
- `memory_threshold` (integer, optional): Alert when memory usage exceeds this percentage. Default: `80`
- `response_time_threshold` (integer, optional): Alert when response time exceeds this value in milliseconds. Default: `1000`

**Prometheus Integration:**
- `enabled` (boolean, optional): Enable Prometheus metrics export. Default: `false`
- `endpoint` (string, optional): Metrics endpoint path. Default: `/metrics`
- `port` (integer, optional): Port for metrics server. Default: same as main site

### Caching Configuration

Configure caching behavior for static files.

```toml
[caching]
enabled = true                         # Enable caching (boolean)
max_memory = "1GB"                     # Maximum memory for cache (string)
ttl_default = 3600                     # Default TTL in seconds (integer)
ttl_static = 86400                     # TTL for static files (integer)
max_file_size = "10MB"                 # Maximum cacheable file size (string)
cache_control_override = false         # Override Cache-Control headers (boolean)
```

**Parameters:**
- `enabled` (boolean, optional): Enable response caching. Default: `false`
- `max_memory` (string, optional): Maximum memory to use for cache. Default: `100MB`
- `ttl_default` (integer, optional): Default cache TTL in seconds. Default: `3600`
- `ttl_static` (integer, optional): TTL for static files in seconds. Default: `86400`
- `max_file_size` (string, optional): Maximum size of files to cache. Default: `1MB`
- `cache_control_override` (boolean, optional): Override existing Cache-Control headers. Default: `false`

### Compression Configuration

Configure response compression.

```toml
[compression]
enabled = true                         # Enable compression (boolean)
level = 6                              # Compression level 1-9 (integer)
min_size = 1024                        # Minimum size to compress (integer)
algorithms = ["gzip", "deflate"]       # Compression algorithms (array)
types = [                              # MIME types to compress (array)
    "text/html",
    "text/css",
    "application/javascript",
    "application/json"
]
```

**Parameters:**
- `enabled` (boolean, optional): Enable response compression. Default: `false`
- `level` (integer, optional): Compression level (1-9, higher = better compression). Default: `6`
- `min_size` (integer, optional): Minimum response size to compress in bytes. Default: `1024`
- `algorithms` (array, optional): Supported compression algorithms. Default: `["gzip"]`
- `types` (array, optional): MIME types to compress. Default: common text types

## Site Configuration

Each `[[sites]]` section defines a virtual host or site.

### Basic Site Configuration

```toml
[[sites]]
name = "example"                       # Site identifier (string)
hostname = "localhost"                 # Hostname to bind to (string)
port = 8080                           # Port to listen on (integer)
static_dir = "static"                 # Directory for static files (string)
index_file = "index.html"             # Default index file (string)
```

**Required Parameters:**
- `name` (string): Unique identifier for this site
- `hostname` (string): Hostname or IP address to bind to
- `port` (integer): TCP port number to listen on
- `static_dir` (string): Path to directory containing static files

**Optional Parameters:**
- `index_file` (string, optional): Default file to serve for directory requests. Default: `index.html`

### Advanced Site Configuration

```toml
[[sites]]
name = "advanced"
hostname = "example.com"
port = 8080
static_dir = "/var/www/example"
index_file = "index.html"
enable_directory_listing = false      # Allow directory browsing (boolean)
follow_symlinks = false               # Follow symbolic links (boolean)
case_sensitive = true                 # Case-sensitive file matching (boolean)
max_age = 3600                        # Default cache max-age (integer)
cors_enabled = true                   # Enable CORS headers (boolean)
```

**Additional Parameters:**
- `enable_directory_listing` (boolean, optional): Allow browsing directories without index files. Default: `false`
- `follow_symlinks` (boolean, optional): Follow symbolic links when serving files. Default: `false`
- `case_sensitive` (boolean, optional): Case-sensitive URL matching. Default: `true`
- `max_age` (integer, optional): Default Cache-Control max-age in seconds. Default: `3600`
- `cors_enabled` (boolean, optional): Enable CORS headers for cross-origin requests. Default: `false`

### Site Headers Configuration

Custom HTTP headers for responses from a site.

```toml
[[sites]]
name = "example"
hostname = "localhost"
port = 8080
static_dir = "static"

[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
"X-XSS-Protection" = "1; mode=block"
"Strict-Transport-Security" = "max-age=31536000"
"Content-Security-Policy" = "default-src 'self'"
"Referrer-Policy" = "strict-origin-when-cross-origin"
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE"
"X-Custom-Header" = "custom-value"
```

**Header Configuration:**
- Any valid HTTP header name can be used as a key
- Header values must be strings
- Headers are added to all responses from the site
- Case-insensitive header names (will be normalized)

### Site SSL/TLS Configuration

Configure SSL/TLS for HTTPS sites with automatic or manual certificates.

```toml
[[sites]]
name = "secure"
hostname = "secure.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true                         # Enable SSL/TLS (boolean)
auto_cert = true                       # Use automatic certificates (boolean)
domains = ["secure.example.com", "www.secure.example.com"] # Additional domains (array)
cert_file = "/etc/ssl/certs/site.crt" # Certificate file path (string)
key_file = "/etc/ssl/private/site.key" # Private key file path (string)

[sites.ssl.acme]
enabled = true                         # Enable ACME certificate generation (boolean)
email = "admin@example.com"            # Email for ACME registration (string)
staging = false                        # Use staging environment (boolean)
challenge_dir = "./acme-challenges"    # ACME challenge directory (string)
```

**SSL Parameters:**
- `enabled` (boolean, optional): Enable SSL/TLS for this site. Default: `false`
- `auto_cert` (boolean, optional): Use automatic certificate generation via ACME. Default: `false`
- `domains` (array, optional): Additional domains for the SSL certificate. Default: `[]`
- `cert_file` (string, required if auto_cert=false): Path to SSL certificate file
- `key_file` (string, required if auto_cert=false): Path to SSL private key file

**ACME Configuration:**
- `enabled` (boolean, optional): Enable ACME certificate generation. Default: `false`
- `email` (string, required if enabled): Email address for ACME registration
- `staging` (boolean, optional): Use Let's Encrypt staging environment for testing. Default: `false`
- `challenge_dir` (string, optional): Directory for HTTP-01 challenge files. Default: `"./acme-challenges"`

**SSL Configuration Examples:**

Automatic SSL with ACME (Let's Encrypt):
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

Manual SSL with custom certificates:
```toml
[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/example.com.crt"
key_file = "/etc/ssl/private/example.com.key"
```

### Site Rate Limiting

Configure rate limiting for requests.

```toml
[[sites]]
name = "rate-limited"
hostname = "api.example.com"
port = 8080
static_dir = "static"

[sites.rate_limit]
enabled = true                         # Enable rate limiting (boolean)
requests_per_minute = 60              # Requests per minute per IP (integer)
burst_size = 10                       # Burst allowance (integer)
block_duration = 300                  # Block duration in seconds (integer)
whitelist = ["127.0.0.1", "10.0.0.0/8"] # IP whitelist (array)
blacklist = ["192.168.1.100"]        # IP blacklist (array)
```

**Rate Limiting Parameters:**
- `enabled` (boolean, optional): Enable rate limiting. Default: `false`
- `requests_per_minute` (integer, optional): Maximum requests per minute per IP. Default: `60`
- `burst_size` (integer, optional): Allow burst of requests above the rate. Default: `10`
- `block_duration` (integer, optional): How long to block an IP after rate limit exceeded. Default: `300`
- `whitelist` (array, optional): IP addresses or CIDR blocks to exempt from rate limiting
- `blacklist` (array, optional): IP addresses or CIDR blocks to always block

### Site Access Control

Configure access control and authentication.

```toml
[[sites]]
name = "protected"
hostname = "internal.example.com"
port = 8080
static_dir = "static"

[sites.access]
allow_ips = ["10.0.0.0/8", "192.168.0.0/16"] # Allowed IP ranges (array)
deny_ips = ["192.168.1.100"]          # Denied IP addresses (array)
require_auth = true                    # Require authentication (boolean)
auth_type = "basic"                    # Authentication type (string)
auth_realm = "Protected Area"          # Basic auth realm (string)
auth_file = "/etc/bws/htpasswd"       # Password file for basic auth (string)
```

**Access Control Parameters:**
- `allow_ips` (array, optional): IP addresses or CIDR blocks allowed access
- `deny_ips` (array, optional): IP addresses or CIDR blocks denied access
- `require_auth` (boolean, optional): Require authentication for access. Default: `false`
- `auth_type` (string, optional): Authentication method. Values: `basic`, `digest`. Default: `basic`
- `auth_realm` (string, optional): Realm name for HTTP authentication. Default: `BWS`
- `auth_file` (string, required if require_auth=true): Path to password file

## Complete Configuration Example

```toml
# BWS Complete Configuration Example

# Daemon configuration
[daemon]
user = "bws"
group = "bws"
pid_file = "/var/run/bws.pid"
working_directory = "/opt/bws"

# Logging configuration
[logging]
level = "info"
output = "file"
format = "json"
file_path = "/var/log/bws/bws.log"
max_size = "100MB"
max_files = 10
compress = true

# Performance tuning
[performance]
worker_threads = 8
max_connections = 10000
keep_alive_timeout = 60
request_timeout = 30
read_buffer_size = "64KB"
write_buffer_size = "64KB"

# Monitoring
[monitoring]
enabled = true
health_endpoint = "/health"
detailed_endpoint = "/health/detailed"

[monitoring.checks]
disk_threshold = 90
memory_threshold = 80
response_time_threshold = 1000

# Caching
[caching]
enabled = true
max_memory = "1GB"
ttl_default = 3600
ttl_static = 86400

# Compression
[compression]
enabled = true
level = 6
min_size = 1024
types = ["text/html", "text/css", "application/javascript"]

# Main website (HTTP)
[[sites]]
name = "main"
hostname = "example.com"
port = 80
static_dir = "/var/www/main"
index_file = "index.html"

[sites.ssl]
enabled = false

[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "SAMEORIGIN"

# Main website (HTTPS with auto SSL)
[[sites]]
name = "main_https"
hostname = "example.com"
port = 443
static_dir = "/var/www/main"
index_file = "index.html"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "/var/www/acme-challenges"

[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "SAMEORIGIN"
"Strict-Transport-Security" = "max-age=31536000"

# API server (HTTPS with manual SSL)
[[sites]]
name = "api"
hostname = "api.example.com"
port = 443
static_dir = "/var/www/api-docs"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/api.example.com.crt"
key_file = "/etc/ssl/private/api.example.com.key"

[sites.headers]
"Content-Type" = "application/json"
"Access-Control-Allow-Origin" = "https://example.com"
"Cache-Control" = "no-cache"
"Strict-Transport-Security" = "max-age=31536000"

[sites.rate_limit]
enabled = true
requests_per_minute = 100
burst_size = 20

# Secure admin interface
[[sites]]
name = "admin"
hostname = "admin.example.com"
port = 8443
static_dir = "/var/www/admin"

[sites.ssl]
enabled = true
cert_file = "/etc/ssl/certs/admin.crt"
key_file = "/etc/ssl/private/admin.key"

[sites.access]
allow_ips = ["10.0.0.0/8"]
require_auth = true
auth_file = "/etc/bws/admin.htpasswd"

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000"
"X-Frame-Options" = "DENY"
```

## Data Types and Formats

### String Values
- Quoted strings: `"value"`
- Raw strings: `'value'` (no escape sequences)
- Multi-line strings: `"""value"""`

### Size Values
Size values can use suffixes:
- `B` - bytes
- `KB` - kilobytes (1024 bytes)
- `MB` - megabytes (1024 KB)
- `GB` - gigabytes (1024 MB)

Examples: `"1MB"`, `"512KB"`, `"2GB"`

### Duration Values
Duration values are integers representing seconds unless otherwise specified.

### Arrays
Arrays use square brackets: `["item1", "item2"]`

### IP Addresses and CIDR
- IPv4: `"192.168.1.1"`
- IPv6: `"2001:db8::1"`
- CIDR notation: `"192.168.0.0/16"`, `"10.0.0.0/8"`

## Configuration Validation

BWS validates configuration on startup. Common validation errors:

### Syntax Errors
```toml
# Invalid: missing quotes
name = value  # Should be name = "value"

# Invalid: missing comma in array
ports = [8080 8081]  # Should be ports = [8080, 8081]
```

### Type Errors
```toml
# Invalid: string instead of integer
port = "8080"  # Should be port = 8080

# Invalid: integer instead of boolean
enabled = 1  # Should be enabled = true
```

### Logical Errors
```toml
# Invalid: duplicate site names
[[sites]]
name = "main"

[[sites]]
name = "main"  # Error: duplicate name

# Invalid: missing required fields
[[sites]]
hostname = "localhost"  # Error: missing port and static_dir
```

## Environment Variable Overrides

Some configuration values can be overridden with environment variables:

```bash
BWS_CONFIG=/path/to/config.toml       # Configuration file path
BWS_LOG_FILE=/path/to/log/file        # Override logging.file_path
BWS_PID_FILE=/path/to/pid/file        # Override daemon.pid_file
BWS_STATIC_DIR=/path/to/static        # Override sites.static_dir (first site)
BWS_PORT=8080                         # Override sites.port (first site)
BWS_HOSTNAME=localhost                # Override sites.hostname (first site)
RUST_LOG=debug                        # Override logging.level
```

## Configuration Best Practices

### Security
- Set appropriate file permissions (600) for configuration files
- Use separate configuration files for different environments
- Store sensitive data in environment variables or external secret management
- Regularly review and audit configuration changes

### Performance
- Tune worker threads based on workload characteristics
- Configure appropriate timeouts for your use case
- Enable compression for text-based content
- Use caching for frequently accessed static files

### Monitoring
- Enable health endpoints for monitoring
- Configure appropriate thresholds for alerts
- Use structured logging for better analysis
- Monitor resource usage trends

### Maintenance
- Use version control for configuration files
- Document configuration changes
- Test configuration changes in non-production environments
- Implement configuration validation in CI/CD pipelines

## Example Configurations

BWS includes several example configurations for different use cases:

### Test Configurations

```bash
# Virtual hosting test (multiple sites on same port)
tests/test_multisite_shared_port.toml

# Multi-site test (different ports)
tests/config_test.toml

# Load balancing test
tests/test_load_balancing.toml

# WebSocket proxy test
tests/test_websocket_proxy.toml
```

### Testing Virtual Hosting

The virtual hosting test demonstrates multiple sites sharing port 8080:

```bash
# Run the virtual hosting test
./tests/test_multisite_shared_port.sh test

# View the configuration
cat tests/test_multisite_shared_port.toml
```

This test includes:
- **4 virtual hosts**: www.local.com, blog.local.com, api.local.com, dev.local.com
- **Shared port**: All sites use port 8080
- **Host-based routing**: Routes based on HTTP Host header
- **Site-specific content**: Each site serves different static files
- **Custom headers**: Site-specific response headers

### Development Configurations

```bash
# Basic single site
examples/basic-single-site.toml

# Multi-site setup
examples/basic-multi-site.toml

# SSL with ACME
examples/ssl-acme.toml

# Production setup
examples/production-multi-site.toml
```

### Configuration Templates

Create your own configuration based on these templates:

```bash
# Copy a template
cp tests/test_multisite_shared_port.toml my-config.toml

# Customize for your needs
# - Update hostnames to your domains
# - Change static_dir paths
# - Configure SSL settings
# - Add your custom headers

# Test the configuration
./target/release/bws --config my-config.toml
```

## Next Steps

- Review [Production Setup](./production.md) for deployment configurations
- Check [Performance Tuning](./performance.md) for optimization settings
- See [Troubleshooting](./troubleshooting.md) for configuration issues
