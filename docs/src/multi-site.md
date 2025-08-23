# Multi-Site Setup

BWS excels at hosting multiple websites on different ports with individual configurations. This guide shows you how to set up and manage multiple sites.

## Basic Multi-Site Configuration

Here's a complete example with four different sites including SSL configurations:

```toml
[server]
name = "BWS Multi-Site Server"

# Main website (HTTP)
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.ssl]
enabled = false

[sites.headers]
"X-Site-Name" = "Main Website"
"X-Powered-By" = "BWS/1.0"
"X-Environment" = "production"

# Blog subdomain (HTTPS with auto SSL)
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8443
static_dir = "static-blog"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["blog.localhost", "www.blog.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

[sites.headers]
"X-Site-Name" = "Blog"
"X-Content-Type" = "blog-content"
"X-Author" = "BWS Team"
"Strict-Transport-Security" = "max-age=31536000"

# API documentation (HTTPS with manual SSL)
[[sites]]
name = "docs"
hostname = "docs.localhost"
port = 8444
static_dir = "static-docs"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./certs/docs.localhost.crt"
key_file = "./certs/docs.localhost.key"

[sites.headers]
"X-Site-Name" = "API Documentation"
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, OPTIONS"
"Strict-Transport-Security" = "max-age=31536000"

# Development environment (HTTP)
[[sites]]
name = "dev"
hostname = "localhost"
port = 8083
static_dir = "static-dev"

[sites.ssl]
enabled = false

[sites.headers]
"X-Site-Name" = "Development Environment"
"X-Environment" = "development"
"X-Debug-Mode" = "enabled"
```

## Directory Structure

Create the corresponding directory structure including SSL certificate directories:

```bash
mkdir -p static static-blog static-docs static-dev acme-challenges certs

# Main site
cat > static/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>Main Site</title></head>
<body>
    <h1>🌐 Main Website</h1>
    <p>Welcome to the main BWS site!</p>
    <nav>
        <a href="https://blog.localhost:8443">Blog (HTTPS)</a> |
        <a href="https://docs.localhost:8444">Docs (HTTPS)</a> |
        <a href="http://localhost:8083">Dev (HTTP)</a>
    </nav>
</body>
</html>
EOF

# Blog site
cat > static-blog/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>BWS Blog</title></head>
<body>
    <h1>📝 BWS Blog (HTTPS)</h1>
    <p>Latest news and updates - Secured with SSL</p>
    <article>
        <h2>Welcome to BWS Blog</h2>
        <p>This is our blog powered by BWS multi-site hosting with automatic SSL.</p>
    </article>
</body>
</html>
EOF

# Documentation site
cat > static-docs/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>BWS Documentation</title></head>
<body>
    <h1>📚 BWS Documentation (HTTPS)</h1>
    <p>Complete API and user documentation - Secured with manual SSL</p>
    <ul>
        <li><a href="/api/health">Health Check</a></li>
        <li><a href="/api/sites">Sites Info</a></li>
    </ul>
</body>
</html>
EOF

# Development site
cat > static-dev/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>BWS Development</title></head>
<body>
    <h1>🚧 BWS Development</h1>
    <p>Development and testing environment (HTTP only)</p>
    <p><strong>Debug Mode:</strong> Enabled</p>
</body>
</html>
EOF
```

## Site Types

### Default Site

One site should be marked as `default = true`. This site handles requests that don't match other hostnames:

```toml
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true  # This is the default site

[sites.ssl]
enabled = false
```

### HTTPS Sites

Sites can be configured with SSL/TLS for secure connections:

```toml
# Automatic SSL with ACME (Let's Encrypt)
[[sites]]
name = "secure_auto"
hostname = "secure.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.example.com", "www.secure.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

# Manual SSL certificates
[[sites]]
name = "secure_manual"
hostname = "manual.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/manual.example.com.crt"
key_file = "/etc/ssl/private/manual.example.com.key"
```

### API-Only Sites

For sites that only serve API endpoints without static files:

```toml
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8084
static_dir = "api-static"
api_only = true  # Only serve /api/* endpoints

[sites.ssl]
enabled = true
auto_cert = true
domains = ["api.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

[sites.headers]
"Access-Control-Allow-Origin" = "*"
"Content-Type" = "application/json"
```

### Hostname Patterns

BWS supports various hostname patterns:

```toml
# Localhost with different ports
[[sites]]
hostname = "localhost"
port = 8080

# Subdomain
[[sites]]
hostname = "blog.example.com"
port = 8081

# Different domain
[[sites]]
hostname = "api.myservice.com"
port = 8082

# IP address
[[sites]]
hostname = "192.168.1.100"
port = 8083
```

## Load Balancing

You can run multiple BWS instances and use a load balancer:

```nginx
# nginx.conf
upstream bws_main {
    server localhost:8080;
    server localhost:8090;  # Second BWS instance
}

upstream bws_blog {
    server localhost:8081;
    server localhost:8091;  # Second BWS instance
}

server {
    listen 80;
    server_name example.com;
    location / {
        proxy_pass http://bws_main;
    }
}

server {
    listen 80;
    server_name blog.example.com;
    location / {
        proxy_pass http://bws_blog;
    }
}
```

## Testing Multi-Site Setup

1. **Start BWS:**
```bash
bws --config config.toml
```

2. **Test each site:**
```bash
# HTTP Main site
curl http://localhost:8080/

# HTTPS Blog (requires host header or /etc/hosts entry)
curl -k -H "Host: blog.localhost" https://localhost:8443/

# HTTPS Docs (manual SSL)
curl -k -H "Host: docs.localhost" https://localhost:8444/

# HTTP Dev site
curl http://localhost:8083/
```

3. **Check SSL certificates:**
```bash
# Check auto SSL certificate
openssl s_client -connect localhost:8443 -servername blog.localhost

# Check manual SSL certificate
openssl s_client -connect localhost:8444 -servername docs.localhost
```

4. **Check site information:**
```bash
curl http://localhost:8080/api/sites
```

## Best Practices

### Port Management
- Use sequential ports for easy management
- Document port assignments
- Avoid common service ports (22, 80, 443, etc.)

### Directory Organization
```
project/
├── config.toml
├── static/           # Main site
├── static-blog/      # Blog content
├── static-docs/      # Documentation
├── static-api/       # API documentation
└── shared-assets/    # Common files (CSS, JS, images)
```

### Security Considerations
- Use different headers for different environments
- Implement CORS properly for API sites
- Consider IP restrictions for development sites

### Monitoring
Each site can be monitored independently:
```bash
# Check health of each site
curl http://localhost:8080/api/health
curl http://localhost:8081/api/health
curl http://localhost:8082/api/health
```

## Troubleshooting

### Port Conflicts
```bash
# Check if port is in use
lsof -i :8080

# Kill process using port
kill $(lsof -t -i:8080)
```

### Hostname Resolution
Add entries to `/etc/hosts` for testing:
```
127.0.0.1 blog.localhost
127.0.0.1 docs.localhost
127.0.0.1 api.localhost
```

Then access sites via:
- http://localhost:8080 (HTTP main site)
- https://blog.localhost:8443 (HTTPS blog with auto SSL)
- https://docs.localhost:8444 (HTTPS docs with manual SSL)

### SSL Certificate Setup
For manual SSL sites, generate test certificates:

```bash
# Create certificate directory
mkdir -p certs

# Generate self-signed certificate for docs.localhost
openssl req -x509 -newkey rsa:4096 -keyout certs/docs.localhost.key -out certs/docs.localhost.crt -days 365 -nodes -subj "/CN=docs.localhost"

# Set proper permissions
chmod 600 certs/docs.localhost.key
chmod 644 certs/docs.localhost.crt
```

### Configuration Validation
BWS validates configuration on startup:
- Duplicate site names → Error
- Port conflicts → Error
- Missing directories → Warning (created automatically)
- Invalid hostnames → Error

## Next Steps

- Configure [SSL/TLS Configuration](./ssl-tls.md) for HTTPS setup
- Set up [Custom Headers](./headers.md) for each site
- Configure [Health Monitoring](./health.md) for production monitoring
- Learn about [Static File Serving](./static-files.md) optimization
