# Quick Start

Get BWS up and running in just a few minutes!

## 1. Create Configuration

Create a `config.toml` file:

```toml
[server]
name = "BWS Multi-Site Server"

# Main site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "BWS Main Site"
"X-Powered-By" = "BWS/1.0"
```

## 2. Create Static Content

Create your static directory and add some content:

```bash
# Create static directory
mkdir static

# Create a simple index.html
cat > static/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Welcome to BWS</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
    </style>
</head>
<body>
    <h1>🚀 Welcome to BWS!</h1>
    <p>Your multi-site web server is running!</p>
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
bws-web-server --config config.toml

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

Open your browser or use curl to test:

```bash
# Main site
curl http://localhost:8080/

# Health check
curl http://localhost:8080/api/health

# Sites information
curl http://localhost:8080/api/sites
```

You should see:
- Your custom HTML page at `http://localhost:8080/`
- Health status at `http://localhost:8080/api/health`
- Site configuration at `http://localhost:8080/api/sites`

## 5. Add More Sites

Extend your configuration to host multiple sites:

```toml
[server]
name = "BWS Multi-Site Server"

# Main site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "BWS Main Site"

# Blog site
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8081
static_dir = "static-blog"

[sites.headers]
"X-Site-Name" = "BWS Blog"
"X-Content-Type" = "blog-content"

# API documentation
[[sites]]
name = "docs"
hostname = "docs.localhost"
port = 8082
static_dir = "static-docs"

[sites.headers]
"X-Site-Name" = "BWS Documentation"
"Access-Control-Allow-Origin" = "*"
```

Create the corresponding directories:
```bash
mkdir static-blog static-docs
echo "<h1>Blog Site</h1>" > static-blog/index.html
echo "<h1>Documentation Site</h1>" > static-docs/index.html
```

Restart BWS and access:
- Main site: `http://localhost:8080`
- Blog: `http://blog.localhost:8081`  
- Docs: `http://docs.localhost:8082`

## Command Line Options

BWS supports several command line options:

```bash
# Specify config file
bws-web-server --config /path/to/config.toml

# Enable verbose logging
bws-web-server --verbose

# Run as daemon (Unix only)
bws-web-server --daemon

# Custom PID and log files for daemon mode
bws-web-server --daemon \
  --pid-file /var/run/bws.pid \
  --log-file /var/log/bws.log
```

## Next Steps

- Learn more about [Configuration](./configuration.md)
- Set up [Multi-Site hosting](./multi-site.md)
- Configure [Custom Headers](./headers.md)
- Deploy with [Docker](./docker.md)
