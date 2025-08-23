# Custom Headers Configuration

BWS allows you to configure custom HTTP headers for responses, providing control over caching, security, and client behavior.

## Header Configuration

Headers are configured per site in the `config.toml` file:

```toml
[[sites]]
name = "example"
hostname = "localhost"
port = 8080
static_dir = "static"

[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
```

## Common Header Types

### Security Headers

#### Content Security Policy (CSP)
```toml
[sites.headers]
"Content-Security-Policy" = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
```

#### Frame Protection
```toml
[sites.headers]
"X-Frame-Options" = "DENY"                    # Block all framing
"X-Frame-Options" = "SAMEORIGIN"              # Allow same-origin framing
"X-Frame-Options" = "ALLOW-FROM https://example.com"  # Allow specific origin
```

#### Content Type Protection
```toml
[sites.headers]
"X-Content-Type-Options" = "nosniff"          # Prevent MIME sniffing
```

#### XSS Protection
```toml
[sites.headers]
"X-XSS-Protection" = "1; mode=block"
```

#### HTTPS Enforcement
```toml
[sites.headers]
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
```

### Caching Headers

#### Standard Caching
```toml
[sites.headers]
"Cache-Control" = "public, max-age=3600"      # 1 hour
"Cache-Control" = "public, max-age=86400"     # 1 day
"Cache-Control" = "public, max-age=31536000"  # 1 year
```

#### No Caching
```toml
[sites.headers]
"Cache-Control" = "no-cache, no-store, must-revalidate"
"Pragma" = "no-cache"
"Expires" = "0"
```

#### ETag Support
```toml
[sites.headers]
"ETag" = ""custom-etag-value""
```

### CORS Headers

#### Basic CORS
```toml
[sites.headers]
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE, OPTIONS"
"Access-Control-Allow-Headers" = "Content-Type, Authorization"
```

#### Restricted CORS
```toml
[sites.headers]
"Access-Control-Allow-Origin" = "https://example.com"
"Access-Control-Allow-Credentials" = "true"
"Access-Control-Max-Age" = "86400"
```

### Custom Application Headers

#### API Versioning
```toml
[sites.headers]
"X-API-Version" = "v1.0.0"
"X-Service-Name" = "BWS"
```

#### Environment Information
```toml
[sites.headers]
"X-Environment" = "production"
"X-Deploy-Version" = "2024.01.15"
```

#### Rate Limiting Info
```toml
[sites.headers]
"X-RateLimit-Limit" = "1000"
"X-RateLimit-Window" = "3600"
```

## Site-Specific Configurations

### Development Environment
```toml
[[sites]]
name = "dev"
hostname = "localhost"
port = 8080
static_dir = "src"

[sites.headers]
"Cache-Control" = "no-cache, no-store, must-revalidate"
"X-Environment" = "development"
"X-Debug-Mode" = "enabled"
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE, OPTIONS"
```

### Production Environment
```toml
[[sites]]
name = "prod"
hostname = "example.com"
port = 8080
static_dir = "dist"

[sites.headers]
"Cache-Control" = "public, max-age=31536000"
"X-Environment" = "production"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
"Content-Security-Policy" = "default-src 'self'"
```

### API Server
```toml
[[sites]]
name = "api"
hostname = "api.example.com"
port = 8081
static_dir = "api-docs"

[sites.headers]
"Content-Type" = "application/json"
"X-API-Version" = "v2.1.0"
"Access-Control-Allow-Origin" = "https://app.example.com"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE"
"Access-Control-Allow-Headers" = "Content-Type, Authorization, X-Requested-With"
"Access-Control-Max-Age" = "86400"
"Cache-Control" = "no-cache"
```

### CDN/Asset Server
```toml
[[sites]]
name = "cdn"
hostname = "cdn.example.com"
port = 8082
static_dir = "assets"

[sites.headers]
"Cache-Control" = "public, max-age=31536000, immutable"
"Access-Control-Allow-Origin" = "*"
"X-Content-Type-Options" = "nosniff"
"Vary" = "Accept-Encoding"
```

## Advanced Header Patterns

### Multi-Value Headers
Some headers can have multiple values separated by commas:
```toml
[sites.headers]
"Vary" = "Accept-Encoding, Accept-Language, User-Agent"
"Access-Control-Expose-Headers" = "X-Total-Count, X-Page-Count"
```

### Conditional Headers
Different headers for different file types (handled by custom logic):
```toml
# Base headers for all files
[sites.headers]
"X-Served-By" = "BWS"
"X-Content-Type-Options" = "nosniff"

# Note: File-specific headers would require application logic
# This is the base configuration that applies to all responses
```

## Header Testing

### Check Headers with curl
```bash
# View all response headers
curl -I http://localhost:8080/

# Check specific header
curl -I http://localhost:8080/ | grep "Cache-Control"

# Verbose output with request/response
curl -v http://localhost:8080/
```

### Check Headers with HTTPie
```bash
# View headers
http HEAD localhost:8080

# Check specific response
http GET localhost:8080/api/health
```

### Browser Developer Tools
1. Open DevTools (F12)
2. Navigate to Network tab
3. Reload page
4. Click on any request
5. View Response Headers section

## Security Best Practices

### Minimal Security Headers
```toml
[sites.headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"X-XSS-Protection" = "1; mode=block"
"Referrer-Policy" = "strict-origin-when-cross-origin"
```

### Enhanced Security Headers
```toml
[sites.headers]
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains; preload"
"Content-Security-Policy" = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"X-XSS-Protection" = "1; mode=block"
"Referrer-Policy" = "strict-origin-when-cross-origin"
"Permissions-Policy" = "camera=(), microphone=(), geolocation=()"
```

### API Security Headers
```toml
[sites.headers]
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
"Cache-Control" = "no-store"
"Content-Security-Policy" = "default-src 'none'"
"X-Permitted-Cross-Domain-Policies" = "none"
```

## Performance Headers

### Optimal Caching Strategy
```toml
# Static assets (CSS, JS, images)
[sites.headers]
"Cache-Control" = "public, max-age=31536000, immutable"
"Vary" = "Accept-Encoding"

# HTML files
# Cache-Control = "public, max-age=3600"  # Shorter cache

# API responses
# Cache-Control = "no-cache"  # No caching
```

### Compression Headers
```toml
[sites.headers]
"Vary" = "Accept-Encoding"
"Content-Encoding" = "gzip"  # If serving pre-compressed files
```

## Troubleshooting

### Headers Not Appearing
1. Check TOML syntax in config file
2. Restart BWS after configuration changes
3. Verify header names are correct (case-sensitive)
4. Use browser dev tools to inspect

### CORS Issues
```toml
# Debug CORS headers
[sites.headers]
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE, OPTIONS"
"Access-Control-Allow-Headers" = "Content-Type, Authorization, X-Requested-With"
"Access-Control-Max-Age" = "86400"
```

### CSP Violations
1. Start with permissive policy
2. Use browser console to identify violations
3. Gradually restrict policy
4. Test thoroughly

### Cache Problems
```bash
# Force no-cache for debugging
[sites.headers]
"Cache-Control" = "no-cache, no-store, must-revalidate"
"Pragma" = "no-cache"
"Expires" = "0"
```

## Common Header Combinations

### Static Website
```toml
[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Frame-Options" = "SAMEORIGIN"
"X-Content-Type-Options" = "nosniff"
"Referrer-Policy" = "strict-origin-when-cross-origin"
```

### Single Page Application (SPA)
```toml
[sites.headers]
"Cache-Control" = "public, max-age=3600"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"Content-Security-Policy" = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
```

### Documentation Site
```toml
[sites.headers]
"Cache-Control" = "public, max-age=1800"
"X-Frame-Options" = "SAMEORIGIN"
"X-Content-Type-Options" = "nosniff"
"Content-Security-Policy" = "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:"
```

## Next Steps

- Learn about [Health Monitoring](./health.md)
- Configure [Docker Deployment](./docker.md)
- Explore [Performance Tuning](./performance.md)
