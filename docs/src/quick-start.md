# Quick Start

Get BWS up and running in just a few minutes with both HTTP and HTTPS sites!

## 1. Create Configuration

Create a `config.toml` file with both HTTP and HTTPS sites:

```toml
[server]
name = "BWS Multi-Site Server"

# HTTP site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.ssl]
enabled = false

[sites.headers]
"X-Site-Name" = "BWS Main Site"
"X-Powered-By" = "BWS/1.0"

# HTTPS site with automatic SSL
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
staging = true  # Use staging for testing
challenge_dir = "./acme-challenges"

[sites.headers]
"X-Site-Name" = "BWS Secure Site"
"X-Powered-By" = "BWS/1.0"
"Strict-Transport-Security" = "max-age=31536000"
```

## 2. Create Static Content

Create your static directory and add some content:

```bash
# Create directories
mkdir -p static acme-challenges

# Create a simple index.html
cat > static/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Welcome to BWS</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .secure { color: #28a745; }
    </style>
</head>
<body>
    <h1>🚀 Welcome to BWS!</h1>
    <p>Your multi-site web server is running!</p>
    
    <h2>Available Sites:</h2>
    <ul>
        <li><a href="http://localhost:8080">Main Site (HTTP)</a></li>
        <li><a href="https://secure.localhost:8443" class="secure">Secure Site (HTTPS)</a></li>
    </ul>
    
    <h2>API Endpoints:</h2>
    <ul>
        <li><a href="/api/health">Health Check</a></li>
        <li><a href="/api/sites">Sites Info</a></li>
    </ul>
</body>
</html>
EOF
```

## 3. Run the Server

Start BWS with your configuration:

```bash
# Using cargo install
bws --config config.toml

# Using Docker
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest

# From source
cargo run -- --config config.toml
```

## 4. Test Your Server

Open your browser or use curl to test both HTTP and HTTPS sites:

```bash
# Main HTTP site
curl http://localhost:8080/

# Health check
curl http://localhost:8080/api/health

# Sites information
curl http://localhost:8080/api/sites

# Test HTTPS site (may need to add hostname to /etc/hosts)
curl -k https://secure.localhost:8443/

# Check SSL certificate (if ACME is working)
openssl s_client -connect localhost:8443 -servername secure.localhost
```

### Setting up hostname resolution

For testing HTTPS sites with custom hostnames, add entries to `/etc/hosts`:

```bash
# Add to /etc/hosts
echo "127.0.0.1 secure.localhost" | sudo tee -a /etc/hosts
echo "127.0.0.1 ssl.localhost" | sudo tee -a /etc/hosts
```

### ACME Certificate Notes

- The example uses `staging = true` for Let's Encrypt staging environment
- Staging certificates are not trusted by browsers but are good for testing
- For production, set `staging = false` after testing
- ACME certificates require your domain to be publicly accessible

You should see:
- Your custom HTML page at `http://localhost:8080/`
- Health status at `http://localhost:8080/api/health`
- Site configuration at `http://localhost:8080/api/sites`
- SSL-secured content at `https://secure.localhost:8443/` (if configured)

## 5. Add WebSocket Proxy (Optional)

BWS can also proxy WebSocket connections with load balancing. Here's a simple example:

```toml
# Add to your existing configuration
[[sites]]
name = "websocket-proxy"
hostname = "ws.localhost"
port = 8090
static_dir = "static"

[sites.proxy]
enabled = true

# WebSocket upstream servers
[[sites.proxy.upstreams]]
name = "websocket_backend"
url = "http://localhost:3001"  # Will be converted to ws://localhost:3001
weight = 1

[[sites.proxy.upstreams]]
name = "websocket_backend"
url = "http://localhost:3002"  # Will be converted to ws://localhost:3002
weight = 1

# WebSocket routes
[[sites.proxy.routes]]
path = "/ws"
upstream = "websocket_backend"
strip_prefix = true
websocket = true  # Enable WebSocket proxying

# Load balancing for WebSocket connections
[sites.proxy.load_balancing]
method = "round_robin"
```

Test WebSocket proxying:
```bash
# Start simple WebSocket test servers (if you have Node.js)
npx ws ws://localhost:3001 &
npx ws ws://localhost:3002 &

# Connect through BWS proxy
# WebSocket connections to ws://ws.localhost:8090/ws will be load-balanced
```

## 7. Add More Sites (Optional)

Extend your configuration to host multiple sites with different SSL configurations:

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

[sites.ssl]
enabled = false

[sites.headers]
"X-Site-Name" = "BWS Main Site"

# Blog site with auto SSL
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
staging = true
challenge_dir = "./acme-challenges"

[sites.headers]
"X-Site-Name" = "BWS Blog"
"X-Content-Type" = "blog-content"
"Strict-Transport-Security" = "max-age=31536000"

# API documentation with manual SSL
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
"X-Site-Name" = "BWS Documentation"
"Access-Control-Allow-Origin" = "*"
"Strict-Transport-Security" = "max-age=31536000"
```

Create the corresponding directories and certificates:
```bash
# Create directories
mkdir static-blog static-docs certs

# Add content
echo "<h1>Blog Site (HTTPS)</h1>" > static-blog/index.html
echo "<h1>Documentation Site (HTTPS)</h1>" > static-docs/index.html

# Generate self-signed certificate for docs site
openssl req -x509 -newkey rsa:4096 -keyout certs/docs.localhost.key -out certs/docs.localhost.crt -days 365 -nodes -subj "/CN=docs.localhost"
chmod 600 certs/docs.localhost.key
```

Restart BWS and access:
- Main site: `http://localhost:8080`
- Blog: `https://blog.localhost:8443` (auto SSL)
- Docs: `https://docs.localhost:8444` (manual SSL)

## Command Line Options

BWS supports several command line options:

```bash
# Specify config file
bws --config /path/to/config.toml

# Enable verbose logging
bws --verbose

# Run as daemon (Unix only)
bws --daemon

# Custom PID and log files for daemon mode
bws --daemon \
  --pid-file /var/run/bws.pid \
  --log-file /var/log/bws.log
```

## Next Steps

- Learn more about [SSL/TLS Configuration](./ssl-tls.md) for HTTPS setup
- Explore [Configuration](./configuration.md) options
- Set up [Multi-Site hosting](./multi-site.md) for complex deployments
- Configure [Custom Headers](./headers.md) for security and functionality
- Deploy with [Docker](./docker.md) for production use
