# Multi-Hostname Site Configuration

BWS supports configuring multiple hostnames for a single site, allowing you to serve the same content and configuration across different domain names on the same port.

## Overview

The multi-hostname feature allows you to:

- **Serve multiple domains** from a single site configuration
- **Share the same port** across different hostnames
- **Maintain consistent configuration** across all domains
- **Simplify SSL certificate management** for multiple domains
- **Handle domain aliases** (e.g., www.example.com and example.com)

## Configuration

### Basic Multi-Hostname Setup

```toml
[[sites]]
name = "multi-domain-site"
hostname = "example.com"                    # Primary hostname
hostnames = ["www.example.com", "example.org"]  # Additional hostnames
port = 8080
static_dir = "./sites/main"
```

### Key Properties

- **`hostname`**: The primary hostname for the site
- **`hostnames`**: Array of additional hostnames that share the same configuration
- **`port`**: All hostnames will be served on this port
- **Configuration inheritance**: All hostnames inherit the same site configuration

## Use Cases

### 1. Domain Aliases and Redirects

Handle both www and non-www versions of your domain:

```toml
[[sites]]
name = "main-website"
hostname = "example.com"
hostnames = ["www.example.com"]
port = 80
static_dir = "./sites/main"
redirect_to_https = true
```

### 2. Multiple Brand Domains

Serve the same content across different branded domains:

```toml
[[sites]]
name = "brand-sites"
hostname = "brand-a.com"
hostnames = ["brand-b.com", "brand-c.com"]
port = 80
static_dir = "./sites/brands"
```

### 3. API Versioning

Handle multiple API endpoint names:

```toml
[[sites]]
name = "api-service"
hostname = "api.example.com"
hostnames = ["api-v1.example.com", "rest.example.com"]
port = 8080
static_dir = "./sites/api"
api_only = true
```

### 4. Development and Testing

Handle localhost and development aliases:

```toml
[[sites]]
name = "dev-site"
hostname = "localhost"
hostnames = ["127.0.0.1", "dev.local", "test.local"]
port = 3000
static_dir = "./sites/dev"
```

## SSL/TLS Support

### Automatic Certificate Management

When using ACME/Let's Encrypt, BWS will automatically request certificates for all hostnames:

```toml
[[sites]]
name = "secure-multi-domain"
hostname = "secure.example.com"
hostnames = ["ssl.example.com", "https.example.com"]
port = 443

[sites.ssl]
enabled = true
auto_cert = true
# SSL certificates will be requested for all hostnames automatically

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
```

### Additional SSL Domains

You can also specify additional domains beyond the hostnames:

```toml
[sites.ssl]
enabled = true
domains = ["extra.example.com", "cdn.example.com"]  # Additional SSL domains
```

Total SSL domains will include:
- Primary hostname: `secure.example.com`
- Additional hostnames: `ssl.example.com`, `https.example.com`
- Additional SSL domains: `extra.example.com`, `cdn.example.com`

## Request Routing

### Hostname Matching Priority

BWS matches incoming requests in this order:

1. **Exact hostname:port match** across all configured hostnames
2. **Port-only match** if hostname doesn't match exactly
3. **Default site** as fallback

### Example Request Flow

For configuration:
```toml
[[sites]]
name = "site-a"
hostname = "example.com"
hostnames = ["www.example.com"]
port = 8080

[[sites]]
name = "site-b"
hostname = "different.com"
port = 8080
default = true
```

Request routing:
- `example.com:8080` → Routes to `site-a`
- `www.example.com:8080` → Routes to `site-a`
- `unknown.com:8080` → Routes to `site-b` (default)

## Configuration Validation

BWS validates multi-hostname configurations:

### Duplicate Detection
```bash
Error: Duplicate hostname:port combination: example.com:8080
```

### Hostname Format Validation
```bash
Error: Invalid additional hostname format: invalid..hostname
```

### Empty Hostname Prevention
```bash
Error: Additional hostname 1 cannot be empty
```

## Best Practices

### 1. Use Primary Hostname for Main Domain
Set your main domain as the `hostname` and alternatives as `hostnames`:

```toml
hostname = "example.com"          # Main domain
hostnames = ["www.example.com"]   # Alternative
```

### 2. Group Related Domains
Keep related domains together in a single site configuration:

```toml
# ✅ Good: Related domains together
hostname = "api.example.com"
hostnames = ["api-v1.example.com", "rest.example.com"]

# ❌ Avoid: Unrelated domains together
hostname = "blog.example.com"
hostnames = ["shop.different.com"]  # Different purpose/brand
```

### 3. SSL Certificate Planning
Consider certificate limits when configuring many hostnames:

```toml
# Let's Encrypt allows up to 100 domains per certificate
hostname = "main.example.com"
hostnames = [
    "www.example.com",
    "blog.example.com",
    "api.example.com"
    # ... up to reasonable limits
]
```

### 4. Development vs Production
Use different configurations for development and production:

```toml
# Development
hostname = "localhost"
hostnames = ["127.0.0.1", "dev.local"]

# Production
hostname = "example.com"
hostnames = ["www.example.com"]
```

## Monitoring and Debugging

### Request Logging
BWS logs which site handles each request:

```
[INFO] Incoming request: GET / (site: multi-domain-site, host: www.example.com)
```

### Configuration Verification
Use the API to verify hostname handling:

```bash
curl http://localhost:8080/api/health
```

Response includes site information:
```json
{
  "status": "healthy",
  "site": "multi-domain-site",
  "hostname": "www.example.com"
}
```

## Migration Guide

### From Single to Multi-Hostname

**Before:**
```toml
[[sites]]
name = "site-www"
hostname = "www.example.com"
port = 80
static_dir = "./sites/main"

[[sites]]
name = "site-main"
hostname = "example.com"
port = 80
static_dir = "./sites/main"  # Same content
```

**After:**
```toml
[[sites]]
name = "main-site"
hostname = "example.com"
hostnames = ["www.example.com"]  # Combined
port = 80
static_dir = "./sites/main"
```

## Troubleshooting

### Common Issues

1. **Hostname conflicts**: Ensure no hostname appears in multiple sites
2. **Port conflicts**: Check that hostname:port combinations are unique
3. **SSL certificate errors**: Verify all hostnames are valid domain names
4. **Default site missing**: Ensure at least one site has `default = true`

### Debug Commands

```bash
# Test hostname resolution
curl -H "Host: www.example.com" http://localhost:8080/

# Check site configuration
curl http://localhost:8080/api/sites

# Verify SSL domains
curl http://localhost:8080/api/ssl/domains
```
