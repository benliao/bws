# Virtual Hosting - Multiple Sites on Same Port

BWS supports true virtual hosting, allowing you to run multiple completely different sites on the same port, distinguished by hostname. This is the standard way to host multiple websites on a single server.

## Overview

Virtual hosting in BWS allows you to:

- **Host multiple websites** on the same port (80 or 443)
- **Route by hostname** - each domain gets its own site configuration
- **Different content directories** for each site
- **Independent configurations** per site (headers, SSL, proxy, etc.)
- **Efficient resource usage** - share the same port across sites

## How It Works

When a request comes in, BWS:

1. **Extracts the Host header** from the HTTP request
2. **Matches against configured sites** by hostname
3. **Routes to the matching site** with its specific configuration
4. **Falls back to default site** if no hostname matches

## Configuration

### Basic Virtual Hosting

```toml
# Site 1: Main website
[[sites]]
name = "main-site"
hostname = "example.com"
port = 8080
static_dir = "./sites/main"
default = true

# Site 2: Blog (same port, different hostname)
[[sites]]
name = "blog-site"
hostname = "blog.example.com"
port = 8080                    # Same port!
static_dir = "./sites/blog"   # Different content!

# Site 3: API (same port, different hostname)
[[sites]]
name = "api-site"
hostname = "api.example.com"
port = 8080                    # Same port!
static_dir = "./sites/api"    # Different content!
api_only = true
```

### With Different Configurations

Each site can have completely different settings:

```toml
# Company website
[[sites]]
name = "company"
hostname = "company.com"
port = 80
static_dir = "./sites/company"
default = true

[sites.headers]
"X-Brand" = "Company Inc"

[sites.compression]
enabled = true

# Documentation site
[[sites]]
name = "docs"
hostname = "docs.company.com"
port = 80                      # Same port
static_dir = "./sites/docs"   # Different directory

[sites.headers]
"X-Site-Type" = "Documentation"
"Cache-Control" = "public, max-age=3600"

[sites.cache]
enabled = true
max_age_static = 7200

# API service
[[sites]]
name = "api"
hostname = "api.company.com"
port = 80                      # Same port
static_dir = "./sites/api"    # Different directory
api_only = true

[sites.headers]
"Content-Type" = "application/json"
"Access-Control-Allow-Origin" = "*"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"

[[sites.proxy.routes]]
path = "/api/"
upstream = "backend"
```

## SSL/HTTPS Virtual Hosting

### Automatic SSL for Multiple Sites

```toml
# Secure main site
[[sites]]
name = "secure-main"
hostname = "secure.example.com"
port = 443
static_dir = "./sites/secure"

[sites.ssl]
enabled = true
auto_cert = true

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

# Secure API (same HTTPS port)
[[sites]]
name = "secure-api"
hostname = "api.secure.example.com"
port = 443                     # Same HTTPS port
static_dir = "./sites/api"    # Different content

[sites.ssl]
enabled = true
auto_cert = true

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
```

### Mixed HTTP/HTTPS Setup

```toml
# HTTP sites on port 80
[[sites]]
name = "public"
hostname = "public.example.com"
port = 80
static_dir = "./sites/public"

[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 80
static_dir = "./sites/blog"

# HTTPS sites on port 443
[[sites]]
name = "secure"
hostname = "secure.example.com"
port = 443
static_dir = "./sites/secure"

[sites.ssl]
enabled = true
auto_cert = true

[[sites]]
name = "admin"
hostname = "admin.example.com"
port = 443
static_dir = "./sites/admin"

[sites.ssl]
enabled = true
auto_cert = true
```

## Advanced Use Cases

### 1. Multi-Tenant Application

Host separate customer portals:

```toml
[[sites]]
name = "customer-a"
hostname = "customer-a.myapp.com"
port = 80
static_dir = "./tenants/customer-a"

[sites.headers]
"X-Tenant" = "customer-a"

[[sites]]
name = "customer-b"
hostname = "customer-b.myapp.com"
port = 80
static_dir = "./tenants/customer-b"

[sites.headers]
"X-Tenant" = "customer-b"
```

### 2. Environment Separation

```toml
# Production
[[sites]]
name = "prod"
hostname = "app.company.com"
port = 80
static_dir = "./environments/prod"

[sites.headers]
"X-Environment" = "production"

# Staging
[[sites]]
name = "staging"
hostname = "staging.company.com"
port = 80
static_dir = "./environments/staging"

[sites.headers]
"X-Environment" = "staging"

# Development
[[sites]]
name = "dev"
hostname = "dev.company.com"
port = 80
static_dir = "./environments/dev"

[sites.headers]
"X-Environment" = "development"
```

### 3. Microservices Gateway

```toml
# Main application
[[sites]]
name = "app"
hostname = "app.company.com"
port = 80
static_dir = "./sites/app"

# User service
[[sites]]
name = "users"
hostname = "users.company.com"
port = 80

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "user-service"
url = "http://127.0.0.1:3001"

[[sites.proxy.routes]]
path = "/"
upstream = "user-service"

# Payment service
[[sites]]
name = "payments"
hostname = "payments.company.com"
port = 80

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "payment-service"
url = "http://127.0.0.1:3002"

[[sites.proxy.routes]]
path = "/"
upstream = "payment-service"
```

## Request Routing Logic

### Hostname Matching Priority

1. **Exact hostname match**: `example.com` matches site with `hostname = "example.com"`
2. **Additional hostnames match**: Checks `hostnames` array for each site
3. **Port fallback**: If no hostname matches, uses any site on the same port
4. **Default site**: Final fallback to site with `default = true`

### Example Flow

Configuration:
```toml
[[sites]]
name = "main"
hostname = "example.com"
hostnames = ["www.example.com"]
port = 80
default = true

[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 80

[[sites]]
name = "api"
hostname = "api.example.com"
port = 80
```

Request routing:
- `Host: example.com` → Routes to "main" site
- `Host: www.example.com` → Routes to "main" site (via hostnames)
- `Host: blog.example.com` → Routes to "blog" site
- `Host: api.example.com` → Routes to "api" site
- `Host: unknown.com` → Routes to "main" site (default)

## Directory Structure Example

```
project/
├── config.toml
├── sites/
│   ├── main/           # example.com content
│   │   ├── index.html
│   │   └── static/
│   ├── blog/           # blog.example.com content
│   │   ├── index.html
│   │   └── posts/
│   ├── api/            # api.example.com content
│   │   └── swagger.html
│   └── docs/           # docs.example.com content
│       ├── index.html
│       └── guides/
└── logs/
```

## Testing Virtual Hosting

### Using curl with Host Headers

```bash
# Test main site
curl -H "Host: example.com" http://localhost:8080/

# Test blog site
curl -H "Host: blog.example.com" http://localhost:8080/

# Test API site
curl -H "Host: api.example.com" http://localhost:8080/

# Test with unknown host (should go to default)
curl -H "Host: unknown.com" http://localhost:8080/
```

### Local Development with /etc/hosts

Add to `/etc/hosts`:
```
127.0.0.1 example.com
127.0.0.1 blog.example.com
127.0.0.1 api.example.com
127.0.0.1 docs.example.com
```

Then access directly:
- http://example.com:8080
- http://blog.example.com:8080
- http://api.example.com:8080

## Monitoring and Debugging

### Request Logging

BWS logs which site handles each request:

```
[INFO] Incoming request: GET / (site: main, host: example.com)
[INFO] Incoming request: GET /posts (site: blog, host: blog.example.com)
[INFO] Incoming request: GET /api/users (site: api, host: api.example.com)
```

### Health Check Endpoints

Each site can have its own health endpoint:
```bash
# Check main site
curl -H "Host: example.com" http://localhost:8080/api/health

# Check blog site
curl -H "Host: blog.example.com" http://localhost:8080/api/health
```

### Configuration Verification

```bash
# List all sites
curl http://localhost:8080/api/sites

# Get site-specific info
curl -H "Host: blog.example.com" http://localhost:8080/api/health
```

## Best Practices

### 1. Use Descriptive Site Names
```toml
# ✅ Good
name = "company-blog"
name = "customer-portal"
name = "api-v1"

# ❌ Avoid
name = "site1"
name = "test"
```

### 2. Organize Content Directories
```
sites/
├── company-main/
├── company-blog/
├── company-api/
└── company-docs/
```

### 3. Consistent Naming Conventions
```toml
hostname = "company.com"          # Main site
hostname = "blog.company.com"     # Blog subdomain
hostname = "api.company.com"      # API subdomain
hostname = "docs.company.com"     # Docs subdomain
```

### 4. Set One Default Site
```toml
[[sites]]
hostname = "example.com"
default = true                    # Only one site should be default

[[sites]]
hostname = "blog.example.com"
# default = false (implicit)
```

### 5. SSL Certificate Planning
```toml
# Let's Encrypt has rate limits - plan your domains
[[sites]]
hostname = "secure.example.com"
[sites.ssl]
enabled = true
auto_cert = true

[[sites]]
hostname = "api.secure.example.com"
[sites.ssl]
enabled = true
auto_cert = true
```

## Troubleshooting

### Common Issues

1. **Wrong site responding**: Check hostname configuration and Host header
2. **404 errors**: Verify static_dir paths exist and contain content
3. **SSL errors**: Ensure domains are resolvable for ACME challenges
4. **Default site issues**: Verify exactly one site has `default = true`

### Debug Steps

1. **Check configuration**: `cargo run -- --validate-config`
2. **Test routing**: Use curl with different Host headers
3. **Check logs**: Look for "Incoming request" log entries
4. **Verify DNS**: Ensure domains resolve to your server
5. **Test ports**: Verify nothing else is using your ports

### Migration from Single Site

**Before** (single site):
```toml
[[sites]]
hostname = "example.com"
port = 80
static_dir = "./static"
```

**After** (multiple sites):
```toml
[[sites]]
name = "main"
hostname = "example.com"
port = 80
static_dir = "./sites/main"
default = true

[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 80
static_dir = "./sites/blog"
```

## Testing Virtual Hosting

BWS includes comprehensive testing tools for virtual hosting configurations.

### Automated Testing

```bash
# Run the complete virtual hosting test suite
./tests/test_multisite_shared_port.sh test

# Start server for manual testing
./tests/test_multisite_shared_port.sh start

# Check server status
./tests/test_multisite_shared_port.sh status

# Stop the test server
./tests/test_multisite_shared_port.sh stop
```

### Manual Testing with curl

```bash
# Test each virtual host using Host headers
curl -H "Host: www.local.com" http://127.0.0.1:8080
curl -H "Host: blog.local.com" http://127.0.0.1:8080
curl -H "Host: api.local.com" http://127.0.0.1:8080
curl -H "Host: dev.local.com" http://127.0.0.1:8080

# Check site-specific response headers
curl -I -H "Host: www.local.com" http://127.0.0.1:8080
curl -I -H "Host: blog.local.com" http://127.0.0.1:8080
```

### Browser Testing

For browser testing, add test domains to `/etc/hosts`:

```bash
sudo bash -c 'echo "127.0.0.1 www.local.com blog.local.com api.local.com dev.local.com" >> /etc/hosts'
```

Then access the virtual hosts in your browser:
- http://www.local.com:8080 (Main site with virtual hosting demo)
- http://blog.local.com:8080 (Blog with red theme)
- http://api.local.com:8080 (API documentation with dark theme)
- http://dev.local.com:8080 (Development site with gradient background)

### Test Configuration

The test uses this configuration (`tests/test_multisite_shared_port.toml`):

```toml
[server]
name = "BWS Multi-Site Shared Port Test Server"

# All sites share port 8080 but have different hostnames
[[sites]]
name = "main"
hostname = "www.local.com"
port = 8080
static_dir = "examples/sites/static"
default = true

[sites.headers]
"X-Site-Name" = "Main Site"
"X-Port-Sharing" = "enabled"

[[sites]]
name = "blog"
hostname = "blog.local.com"
port = 8080
static_dir = "examples/sites/static-blog"

[sites.headers]
"X-Site-Name" = "Blog Site"
"X-Port-Sharing" = "enabled"

# Additional sites...
```

### What to Verify

✅ **Host-based Routing**: Each hostname serves different content  
✅ **Shared Port**: All sites use port 8080  
✅ **Site Headers**: Each site returns custom `X-Site-Name` header  
✅ **Static Content**: Each site serves from separate directories  
✅ **Default Site**: Fallback works for unmatched hostnames  

This testing framework ensures your virtual hosting configuration works correctly before deployment.

Virtual hosting in BWS provides the flexibility to serve multiple websites efficiently while maintaining clean separation of concerns and configurations.
