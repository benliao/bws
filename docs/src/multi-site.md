# Multi-Site Setup

BWS hosts multiple websites with individual configurations on different ports or hostnames.

## Basic Multi-Site Configuration

```toml
[server]
name = "BWS Multi-Site Server"

# Main HTTP site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "Main Website"

# HTTPS blog site
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8443
static_dir = "static-blog"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["blog.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

# API proxy site
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8080
static_dir = "static"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"

[[sites.proxy.routes]]
path = "/v1/"
upstream = "backend"
```

## Setup Multi-Site Content

Create directories and content for each site:

```bash
# Create directories
mkdir -p static static-blog acme-challenges

# Main site content
cat > static/index.html << 'EOF'
<h1>Main Website</h1>
<p>Welcome to the main site</p>
<a href="https://blog.localhost:8443">Visit Blog</a>
EOF

# Blog content
cat > static-blog/index.html << 'EOF'
<h1>Blog Site</h1>
<p>This is the blog running on HTTPS</p>
<a href="http://localhost:8080">Back to Main</a>
EOF
```

## Virtual Hosting (Same Port)

Host multiple sites on the same port using hostname routing:

```toml
[server]
name = "Virtual Hosting Server"

# Main site
[[sites]]
name = "main"
hostname = "www.example.com"
port = 80
static_dir = "sites/main"
default = true

# Blog site (same port, different hostname)
[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 80
static_dir = "sites/blog"

# API site (same port, different hostname)
[[sites]]
name = "api"
hostname = "api.example.com"
port = 80
static_dir = "sites/api"
```

### Testing Virtual Hosting

Add entries to `/etc/hosts` for local testing:

```bash
# Add to /etc/hosts
echo "127.0.0.1 www.example.com blog.example.com api.example.com" | sudo tee -a /etc/hosts
```

Test with curl:
```bash
curl -H "Host: www.example.com" http://localhost/
curl -H "Host: blog.example.com" http://localhost/
curl -H "Host: api.example.com" http://localhost/
```

## Multi-Port Configuration

Run different services on different ports:

```toml
# HTTP on port 80
[[sites]]
name = "web"
hostname = "example.com"
port = 80
static_dir = "web"

# HTTPS on port 443
[[sites]]
name = "secure"
hostname = "example.com"
port = 443
static_dir = "web"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

# API on port 8080
[[sites]]
name = "api"
hostname = "api.example.com"
port = 8080
static_dir = "api"

# Admin panel on port 9000
[[sites]]
name = "admin"
hostname = "admin.example.com"
port = 9000
static_dir = "admin"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["admin.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
```

## Site-Specific Configuration

Each site can have unique settings:

### Headers
```toml
# Different headers per site
[[sites]]
name = "api"
hostname = "api.example.com"
port = 80
static_dir = "api"

[sites.headers]
"Access-Control-Allow-Origin" = "*"
"X-API-Version" = "v1"

[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 80
static_dir = "blog"

[sites.headers]
"X-Content-Type" = "blog"
"Cache-Control" = "public, max-age=3600"
```

### Security Settings
```toml
# Secure admin site
[[sites]]
name = "admin"
hostname = "admin.example.com"
port = 443
static_dir = "admin"

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["admin.example.com"]
```

## Start Multi-Site Server

```bash
# Validate configuration
bws --config multi-site.toml --dry-run

# Start server
bws --config multi-site.toml
```

## Testing Multiple Sites

```bash
# Test different sites
curl http://localhost:8080/                    # Main site
curl https://blog.localhost:8443/              # Blog site  
curl http://api.localhost:8080/v1/health       # API site

# Check site information
curl http://localhost:8080/api/sites

# Test virtual hosting
curl -H "Host: www.example.com" http://localhost/
curl -H "Host: blog.example.com" http://localhost/
```

## Management

### Hot Reload
Update configuration without stopping the server:

```bash
# Edit configuration file
nano multi-site.toml

# Reload configuration
curl -X POST http://127.0.0.1:7654/api/config/reload
```

### Monitoring
```bash
# Check all sites health
curl http://localhost:8080/api/health

# Monitor logs for all sites
tail -f /var/log/bws.log | grep -E "(site:|error:|ssl:)"
```

## Production Deployment

### Directory Structure
```
/var/www/
├── sites/
│   ├── main/
│   ├── blog/
│   ├── api/
│   └── admin/
├── certs/
├── acme-challenges/
└── config/
    └── multi-site.toml
```

### Service Configuration
```toml
[server]
name = "Production Multi-Site"

[[sites]]
name = "main"
hostname = "example.com"
port = 443
static_dir = "/var/www/sites/main"
default = true

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
challenge_dir = "/var/www/acme-challenges"

[[sites]]
name = "api"
hostname = "api.example.com"
port = 443
static_dir = "/var/www/sites/api"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["api.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
challenge_dir = "/var/www/acme-challenges"
```

### Firewall Setup
```bash
# Allow HTTP and HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow management API (localhost only)
sudo ufw allow from 127.0.0.1 to any port 7654
```
