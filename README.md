# BWS Multi-Site Web Server

A high-performance, configurable web server built with Rust using Pingora framework by Cloudflare. Supports multiple websites on different ports and hostnames from a single server instance.

## Features

- **High Performance**: Built on Pingora, Cloudflare's high-performance proxy framework
- **Multi-Site Support**: Host multiple websites on different ports or hostnames
- **TOML Configuration**: Easy configuration via config.toml files
- **Virtual Hosting**: Support for hostname-based virtual hosts
- **Port-based Sites**: Different sites on different ports
- **Static Website Serving**: Serves static HTML, CSS, JS, and other assets with proper MIME types
- **HTTP/HTTPS Support**: Handles HTTP requests with proper response headers
- **JSON API**: RESTful endpoints returning JSON responses
- **File Reading**: API endpoint to read and return file contents
- **Caching**: Built-in cache headers for static assets
- **MIME Type Detection**: Automatic content-type detection for various file types
- **Logging**: Structured logging with configurable levels
- **Health Monitoring**: Built-in health check endpoint

## Configuration

The server is configured via `config.toml`:

```toml
[server]
name = "BWS Multi-Site Server"

# Main site with production headers
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "BWS Main Site"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "main"
"X-Environment" = "production"

# Blog site with custom headers
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8081
static_dir = "static-blog"

[sites.headers]
"X-Site-Name" = "BWS Blog"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "blog"
"X-Content-Type" = "blog-content"
"X-Author" = "BWS Team"

# API documentation site with CORS headers
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8082
static_dir = "static-api"
api_only = true

[sites.headers]
"X-Site-Name" = "BWS API Documentation"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "api-docs"
"X-API-Version" = "v1.0"
"Access-Control-Allow-Origin" = "*"
"Access-Control-Allow-Methods" = "GET, POST, PUT, DELETE, OPTIONS"

# Development site with debug headers
[[sites]]
name = "dev"
hostname = "localhost"
port = 8083
static_dir = "static-dev"

[sites.headers]
"X-Site-Name" = "BWS Development Site"
"X-Powered-By" = "BWS/1.0"
"X-Site-Type" = "development"
"X-Environment" = "development"
"X-Debug-Mode" = "enabled"
```

### Configuration Options

- **server.name**: Display name for the server
- **sites**: Array of site configurations
- **sites.name**: Unique identifier for the site
- **sites.hostname**: Hostname for virtual host routing
- **sites.port**: Port number for the site
- **sites.static_dir**: Directory containing static files
- **sites.default**: Mark as default site (optional)
- **sites.api_only**: API-only site, no static files (optional)
- **sites.headers**: Custom HTTP headers to include in all responses for this site

### Configurable Headers

Each site can define custom HTTP headers that will be included in **all responses** from that site:

- **Security Headers**: CORS, CSP, security policies
- **Custom Headers**: Site identification, versioning, environment info
- **API Headers**: API versioning, CORS for API sites
- **Debug Headers**: Development flags, debug information
- **Branding Headers**: Custom site branding and identification

# Development site
[[sites]]
name = "dev"
hostname = "localhost"
port = 8083
static_dir = "static-dev"
```

### Configuration Options

- **name**: Unique identifier for the site
- **hostname**: Hostname for virtual host matching
- **port**: Port number to listen on
- **static_dir**: Directory containing static files for this site
- **default**: (optional) Set to true for the default/fallback site
- **api_only**: (optional) Set to true for API-only sites

## Available Endpoints

### Multi-Site Support
The server can host multiple websites simultaneously:
- **Main Site** (localhost:8080) - Original BWS website
- **Blog Site** (blog.localhost:8081) - Blog content with different styling
- **API Docs** (api.localhost:8082) - API documentation with dark theme
- **Dev Site** (localhost:8083) - Development environment

### Static Website Endpoints
Each site serves its own static content:
- **GET /** - Serves the site's `index.html` from its static directory
- **GET /about.html** - About page (if available in static directory)
- **GET /contact.html** - Contact page (if available in static directory)  
- **GET /static/*** - Serves static assets (CSS, JS, images, etc.) with cache headers

### API Endpoints

#### Sites Information
- **GET /api/sites** - Returns information about all configured sites
```json
{
  "server": "BWS Multi-Site Server",
  "sites": [
    {
      "name": "main",
      "hostname": "localhost", 
      "port": 8080,
      "static_dir": "static",
      "url": "http://localhost:8080"
    }
  ],
  "total_sites": 4
}
```

#### Health Check
- **GET /api/health** - Returns server health status (available on all sites)
```json
{
  "status": "ok",
  "timestamp": "2025-08-23T01:27:37.952799+00:00",
  "service": "bws-web-server"
}
```

#### File Content API
- **GET /api/file?path=filename** - Returns the content of the specified file
```json
{
  "file_path": "config.toml",
  "content": "[server]\nname = \"BWS Multi-Site Server\"...",
  "size": 567
}
```

### Error Handling
- **404 Not Found** - Returns JSON error for non-existent endpoints
```json
{
  "error": "Not Found",
  "message": "The requested endpoint does not exist",
  "available_endpoints": ["/", "/api/health", "/api/file"]
}
```

### Getting Started

#### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo

#### Dependencies
- `pingora` - High-performance proxy framework
- `async-trait` - Async trait support
- `serde` & `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `tokio` - Async runtime
- `env_logger` & `log` - Logging
- `toml` - Configuration file parsing

#### Building
```bash
cargo build
```

#### Configuration
1. Copy and modify `config.toml` to configure your sites
2. Create static directories for each site (e.g., `static-blog/`, `static-api/`)
3. Add HTML, CSS, JS files to each static directory

#### Running
```bash
# Start the multi-site server
RUST_LOG=info cargo run
```

The server will start multiple services based on your configuration.

### Testing Multiple Sites
```bash
# Run comprehensive multi-site test
./test_multisite.sh

# Test configurable headers functionality
./test_headers.sh

# Test individual sites
curl http://localhost:8080          # Main site
curl http://localhost:8081          # Blog site  
curl http://localhost:8082          # API docs site
curl http://localhost:8083          # Dev site

# Test with virtual host headers
curl -H "Host: blog.localhost:8081" http://localhost:8081

# Check sites configuration (includes header config)
curl http://localhost:8080/api/sites

# Test site-specific headers
curl -I http://localhost:8080/      # Main site headers
curl -I http://localhost:8083/      # Dev site headers (X-Debug-Mode: enabled)
curl -H "Host: api.localhost:8082" -I http://localhost:8082/  # API site CORS headers
```

#### Virtual Host Setup (Optional)
Add entries to `/etc/hosts` for easier testing:
```
127.0.0.1 blog.localhost
127.0.0.1 api.localhost
```

Then access sites via:
- http://blog.localhost:8081
- http://api.localhost:8082

## Project Structure

```
├── src/
│   ├── bin/
│   │   └── main.rs               # Server entry point and multi-site setup
│   └── lib.rs                    # Web server implementation with config support
├── static/                       # Main site files
│   ├── index.html               # Main homepage
│   ├── about.html               # About page
│   ├── contact.html             # Contact page
│   ├── styles.css               # Website styles
│   └── script.js                # Website JavaScript
├── static-blog/                  # Blog site files
│   └── index.html               # Blog homepage
├── static-api/                   # API documentation site
│   └── index.html               # API docs homepage
├── static-dev/                   # Development site files
│   └── index.html               # Dev homepage
├── config.toml                   # Multi-site configuration with headers
├── Cargo.toml                    # Project dependencies
├── test_multisite.sh             # Multi-site test script
├── test_headers.sh               # Configurable headers test script
├── test_static_server.sh         # Static website test script
└── README.md                     # This file
```

## API Implementation Details

### WebServerService
The main service implements the `ProxyHttp` trait from Pingora:
- `request_filter()`: Routes incoming requests to appropriate handlers
- `upstream_peer()`: Returns error since we handle requests locally

### Request Routing
The server intelligently routes requests based on URL patterns:
- `/` → `static/index.html`
- `/static/*` → Static assets with cache headers
- `/*.html` → HTML files from static directory
- `/api/*` → API endpoints
- Everything else → 404 error

### Request Handlers
- `handle_static_file()`: Serves static files with proper MIME types and cache headers
- `handle_health()`: Returns JSON health status
- `handle_file_content()`: Reads and returns file contents
- `handle_404()`: Returns JSON error for unknown endpoints

### Static File Features
- **MIME Type Detection**: Automatic content-type detection for:
  - HTML, CSS, JavaScript
  - Images (PNG, JPEG, GIF, SVG, ICO)
  - Fonts (WOFF, WOFF2, TTF)
  - Documents (PDF, XML, TXT)
- **Cache Headers**: `Cache-Control: public, max-age=3600` for static assets
- **Error Handling**: Graceful 404 responses for missing files

### Features
- **Content-Type Headers**: Proper MIME types for all file types
- **Content-Length**: Accurate response size calculation
- **Error Handling**: Graceful error responses with helpful messages
- **Query Parameter Parsing**: Manual parsing for file path parameter
- **Logging**: Request logging with method and URI

## Extending the Server

To add new endpoints:

1. Add a new pattern match in `request_filter()`
2. Create a new handler method following the pattern of existing handlers
3. Ensure proper error handling and response headers

Example:
```rust
"/api/new-endpoint" => {
    self.handle_new_endpoint(session).await?;
    Ok(true)
}
```

## Performance Characteristics

- **Memory Efficient**: Uses `Vec<u8>` for response bodies to avoid borrowing issues
- **Async**: Fully asynchronous request handling
- **Zero-Copy**: Efficient byte handling where possible
- **Concurrent**: Supports multiple simultaneous connections

## Security Considerations

- File reading is currently unrestricted - consider adding path validation
- No authentication/authorization implemented
- Consider rate limiting for production use
- HTTPS support can be added through Pingora configuration

### Security Status

The project uses automated security scanning via GitHub Actions. The current security status:

- **✅ Active Monitoring**: Weekly security audits via `cargo audit`
- **✅ Dependency Review**: Automated dependency review on pull requests
- **⚠️ Known Issue**: One accepted vulnerability (RUSTSEC-2024-0437) - see `SECURITY.md`

**Security Documentation**: See `SECURITY.md` for detailed security status and known issues.

**Monitoring Script**: Run `./scripts/monitor-deps.sh` to check for dependency updates and security status.

**Security Workflow**: The CI pipeline automatically scans for new vulnerabilities while ignoring documented accepted risks.
