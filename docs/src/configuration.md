# Configuration

BWS uses TOML configuration files to define server behavior and site settings.

## Configuration File Lkey_file = "./certs/manual.crt"
```

### Reverse Proxy Configuration

Each site can be configured as a reverse proxy with load balancing:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `proxy.enabled` | Boolean | Enable reverse proxy for this site | `false` |
| `proxy.upstreams` | Array | Backend servers for proxying | `[]` |
| `proxy.upstreams.name` | String | Upstream group name | Required |
| `proxy.upstreams.url` | String | Backend server URL | Required |
| `proxy.upstreams.weight` | Integer | Server weight for load balancing | `1` |
| `proxy.routes` | Array | Proxy route configurations | `[]` |
| `proxy.routes.path` | String | Path pattern to match | Required |
| `proxy.routes.upstream` | String | Upstream group to proxy to | Required |
| `proxy.routes.strip_prefix` | Boolean | Remove path prefix when forwarding | `false` |
| `proxy.routes.websocket` | Boolean | Enable WebSocket proxying for this route | `false` |
| `proxy.load_balancing.method` | String | Load balancing algorithm | `"round_robin"` |
| `proxy.timeout.read` | Integer | Read timeout in seconds | `30` |
| `proxy.timeout.write` | Integer | Write timeout in seconds | `30` |
| `proxy.headers.add_x_forwarded` | Boolean | Add X-Forwarded-* headers | `true` |
| `proxy.headers.add_forwarded` | Boolean | Add Forwarded header | `true` |
| `proxy.headers.add` | Table | Custom headers to add | `{}` |
| `proxy.headers.remove` | Table | Headers to remove | `{}` |

### Example Proxy Configuration

```toml
# Reverse proxy site with load balancing
[[sites]]
name = "api"
hostname = "api.example.com"
port = 80

[sites.proxy]
enabled = true

# Multiple backend servers (same name for load balancing)
[[sites.proxy.upstreams]]
name = "api-servers"
url = "http://127.0.0.1:3001"
weight = 2

[[sites.proxy.upstreams]]
name = "api-servers"
url = "http://127.0.0.1:3002"
weight = 1

# Route configuration
[[sites.proxy.routes]]
path = "/v1/"
upstream = "api-servers"
strip_prefix = false

# Load balancing method (round_robin, weighted, least_connections)
[sites.proxy.load_balancing]
method = "weighted"

# Timeout configuration
[sites.proxy.timeout]
read = 30
write = 30

# Header management
[sites.proxy.headers]
add_x_forwarded = true
add_forwarded = true

[sites.proxy.headers.add]
"X-Proxy-Version" = "BWS/1.0"

[sites.proxy.headers.remove]
"X-Internal-Token" = true
```

key_file = "./certs/manual.key"ation

By default, BWS looks for `config.toml` in the current directory. You can specify a different location:

```bash
bws --config /path/to/your/config.toml
```

## Basic Configuration Structure

```toml
[server]
name = "BWS Multi-Site Server"

[[sites]]
name = "example"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Custom-Header" = "value"
```

## Server Section

The `[server]` section contains global server settings:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `name` | String | Server identification name | "BWS Server" |

```toml
[server]
name = "My Production BWS Server"
```

## Sites Configuration

Sites are defined using `[[sites]]` array tables. Each site represents a separate web service with its own SSL configuration.

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Unique identifier for the site |
| `hostname` | String | Hostname to bind to |
| `port` | Integer | Port number to listen on |
| `static_dir` | String | Directory containing static files |

### Optional Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `default` | Boolean | Whether this is the default site (automatically set for single sites) | `false` |
| `api_only` | Boolean | Only serve API endpoints, no static files | `false` |

### SSL Configuration

Each site can have its own SSL/TLS configuration:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `ssl.enabled` | Boolean | Enable SSL for this site | `false` |
| `ssl.auto_cert` | Boolean | Use automatic certificates (ACME) | `false` |
| `ssl.domains` | Array | Additional domains for the certificate | `[]` |
| `ssl.cert_file` | String | Path to certificate file (manual SSL) | `null` |
| `ssl.key_file` | String | Path to private key file (manual SSL) | `null` |
| `ssl.acme.enabled` | Boolean | Enable ACME certificate generation | `false` |
| `ssl.acme.email` | String | Email for ACME registration | `null` |
| `ssl.acme.staging` | Boolean | Use staging environment | `false` |
| `ssl.acme.challenge_dir` | String | Directory for ACME challenges | `"./acme-challenges"` |

### Example Site Configuration

```toml
# HTTP Site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.ssl]
enabled = false

# HTTPS Site with Auto SSL
[[sites]]
name = "secure"
hostname = "secure.localhost"
port = 8443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.localhost", "ssl.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

# HTTPS Site with Manual SSL
[[sites]]
name = "manual_ssl"
hostname = "manual.localhost"
port = 8444
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./certs/manual.crt"
key_file = "./certs/manual.key"
```

## Complete Example

Here's a comprehensive configuration example with SSL:

```toml
[server]
name = "BWS Production Server"

# Main website with HTTP
[[sites]]
name = "main"
hostname = "example.com"
port = 80
static_dir = "static"
default = true

[sites.ssl]
enabled = false

[sites.headers]
"X-Site-Name" = "Main Website"
"X-Powered-By" = "BWS/1.0"

# Main website with HTTPS (automatic SSL)
[[sites]]
name = "main_https"
hostname = "example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

[sites.headers]
"X-Site-Name" = "Main Website (HTTPS)"
"X-Powered-By" = "BWS/1.0"
"Strict-Transport-Security" = "max-age=31536000"

# Blog subdomain with manual SSL
[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 443
static_dir = "blog-static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/blog.example.com.crt"
key_file = "/etc/ssl/private/blog.example.com.key"

[sites.headers]
"X-Site-Name" = "Blog"
"X-Content-Type" = "blog-content"
"Strict-Transport-Security" = "max-age=31536000"
```

## Next Steps

- Learn about [SSL/TLS Configuration](./ssl-tls.md) for HTTPS setup
- Configure [Multi-Site Setup](./multi-site.md) for hosting multiple websites
- Set up [Reverse Proxy](./reverse-proxy.md) for backend service integration
- Configure [Load Balancing](./load-balancing.md) for high availability
- Set up [Custom Headers](./headers.md) for enhanced functionality
- Configure [Health Monitoring](./health.md) for production use
