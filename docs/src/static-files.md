# Static File Serving

BWS provides efficient static file serving with automatic MIME type detection, caching headers, and subdirectory support. BWS offers two ways to serve static files: instant directory serving and configuration-based serving.

## Instant Directory Serving

The fastest way to serve static files is using BWS's built-in directory serving mode:

```bash
# Serve current directory
bws .

# Serve specific directory on custom port
bws /path/to/website --port 8080

# Windows example  
bws.exe C:\websites\mysite --port 8080
```

### Features

- **No Configuration Required**: Just point to a directory
- **Automatic Site Setup**: Creates temporary configuration with sensible defaults
- **Cross-Platform**: Handles Windows and Unix paths correctly
- **Default Index Files**: Automatically serves `index.html`, `index.htm`, or `default.html`
- **Clean Paths**: User-friendly path display on all platforms

### Example

```bash
# Create test directory
mkdir my-site
echo "<h1>Hello BWS!</h1>" > my-site/index.html

# Start serving
bws my-site --port 8080
```

**Output:**
```
🚀 Creating temporary web server:
   📁 Directory: /path/to/my-site
   🌐 Port: 8080
   🔗 URL: http://localhost:8080

🌐 Temporary web server ready!
📁 BWS Temporary Directory Server
```

## Configuration-Based Static Serving

For production deployments and advanced features, use configuration files.

## How Configuration-Based Static File Serving Works

BWS serves files from the `static_dir` configured for each site:

```toml
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"  # Files served from this directory
```

## Supported File Types

BWS automatically detects MIME types for common file formats:

### Web Files
- **HTML**: `.html`, `.htm` → `text/html`
- **CSS**: `.css` → `text/css`
- **JavaScript**: `.js`, `.mjs` → `application/javascript`
- **JSON**: `.json` → `application/json`

### Images
- **PNG**: `.png` → `image/png`
- **JPEG**: `.jpg`, `.jpeg` → `image/jpeg`
- **GIF**: `.gif` → `image/gif`
- **SVG**: `.svg` → `image/svg+xml`
- **WebP**: `.webp` → `image/webp`
- **AVIF**: `.avif` → `image/avif`
- **ICO**: `.ico` → `image/x-icon`

### Fonts
- **WOFF**: `.woff`, `.woff2` → `font/woff`
- **TTF**: `.ttf` → `font/ttf`
- **OTF**: `.otf` → `font/otf`
- **EOT**: `.eot` → `application/vnd.ms-fontobject`

### Media
- **MP4**: `.mp4` → `video/mp4`
- **WebM**: `.webm` → `video/webm`
- **MP3**: `.mp3` → `audio/mpeg`
- **WAV**: `.wav` → `audio/wav`
- **OGG**: `.ogg` → `audio/ogg`

### Documents
- **PDF**: `.pdf` → `application/pdf`
- **XML**: `.xml` → `application/xml`
- **Text**: `.txt` → `text/plain`
- **Markdown**: `.md` → `text/markdown`

### Archives
- **ZIP**: `.zip` → `application/zip`
- **Gzip**: `.gz` → `application/gzip`
- **Tar**: `.tar` → `application/x-tar`

### Configuration
- **TOML**: `.toml` → `application/toml`
- **YAML**: `.yaml`, `.yml` → `application/x-yaml`

### WebAssembly
- **WASM**: `.wasm` → `application/wasm`

## URL Patterns

BWS handles several URL patterns for static files:

### Direct File Access
```
http://localhost:8080/index.html
http://localhost:8080/styles.css
http://localhost:8080/script.js
```

### Static Directory Prefix
```
http://localhost:8080/static/css/main.css
http://localhost:8080/static/js/app.js
http://localhost:8080/static/images/logo.png
```

### Subdirectory Support
```
http://localhost:8080/assets/css/main.css
http://localhost:8080/docs/api.html
http://localhost:8080/images/gallery/photo1.jpg
```

### Index File Handling
BWS automatically serves `index.html` for directory requests:
```
http://localhost:8080/           → static/index.html
http://localhost:8080/docs/      → static/docs/index.html
http://localhost:8080/blog/      → static/blog/index.html
```

## Directory Structure Examples

### Basic Website
```
static/
├── index.html          # Main page
├── about.html          # About page
├── styles.css          # Stylesheet
├── script.js          # JavaScript
└── favicon.ico        # Site icon
```

### Advanced Structure
```
static/
├── index.html
├── assets/
│   ├── css/
│   │   ├── main.css
│   │   ├── theme.css
│   │   └── responsive.css
│   ├── js/
│   │   ├── app.js
│   │   ├── utils.js
│   │   └── components/
│   │       ├── header.js
│   │       └── footer.js
│   ├── images/
│   │   ├── logo.svg
│   │   ├── hero.webp
│   │   └── gallery/
│   │       ├── photo1.jpg
│   │       └── photo2.jpg
│   └── fonts/
│       ├── Inter-Regular.woff2
│       └── Inter-Bold.woff2
├── docs/
│   ├── index.html
│   ├── api.html
│   └── guide.html
└── downloads/
    ├── manual.pdf
    └── software.zip
```

## Caching and Performance

BWS automatically adds caching headers:

```http
Cache-Control: public, max-age=3600
Content-Type: text/css
Content-Length: 1234
```

### Cache Control
- **Static files**: 1 hour cache by default
- **HTML files**: Shorter cache for dynamic content
- **Assets**: Longer cache for images, fonts, etc.

## Security Features

### Path Traversal Protection
BWS prevents directory traversal attacks:
```
http://localhost:8080/../../../etc/passwd  ❌ Blocked
http://localhost:8080/..%2F..%2Fetc%2Fpasswd  ❌ Blocked
```

### File Type Restrictions
Only serves files from the configured `static_dir`:
```
http://localhost:8080/config.toml  ❌ Not in static_dir
http://localhost:8080/.env         ❌ Hidden files blocked
```

## Configuration Examples

### Single Site
```toml
[[sites]]
name = "website"
hostname = "localhost"
port = 8080
static_dir = "public"

[sites.headers]
"Cache-Control" = "public, max-age=86400"
"X-Content-Type-Options" = "nosniff"
```

### Multiple Asset Directories
```toml
# Main site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "dist"

# CDN-like asset server
[[sites]]
name = "assets"
hostname = "assets.localhost"
port = 8081
static_dir = "assets"

[sites.headers]
"Cache-Control" = "public, max-age=31536000"
"Access-Control-Allow-Origin" = "*"
```

### Development vs Production
```toml
# Development
[[sites]]
name = "dev"
hostname = "localhost"
port = 8080
static_dir = "src"

[sites.headers]
"Cache-Control" = "no-cache, no-store, must-revalidate"
"X-Environment" = "development"

# Production
[[sites]]
name = "prod"
hostname = "example.com"
port = 8080
static_dir = "build"

[sites.headers]
"Cache-Control" = "public, max-age=31536000"
"X-Environment" = "production"
```

## Testing Static Files

### Basic File Serving
```bash
# Test HTML file
curl -I http://localhost:8080/index.html

# Test CSS file
curl -I http://localhost:8080/styles.css

# Test JavaScript
curl -I http://localhost:8080/app.js
```

### MIME Type Verification
```bash
# Check MIME type headers
curl -I http://localhost:8080/image.png | grep "Content-Type"
curl -I http://localhost:8080/app.wasm | grep "Content-Type"
```

### Subdirectory Access
```bash
# Test nested files
curl -I http://localhost:8080/assets/css/main.css
curl -I http://localhost:8080/docs/api.html
```

### Index File Testing
```bash
# Directory with trailing slash
curl -I http://localhost:8080/docs/

# Directory without trailing slash
curl -I http://localhost:8080/docs
```

## Troubleshooting

### File Not Found (404)
- Check file exists in `static_dir`
- Verify file permissions (readable)
- Check path spelling and case sensitivity

### Wrong MIME Type
- File extension not recognized
- Add custom MIME type mapping if needed
- Verify file extension is correct

### Caching Issues
- Clear browser cache
- Check `Cache-Control` headers
- Use browser dev tools to verify requests

### Permission Errors
```bash
# Fix file permissions
chmod -R 644 static/*
chmod 755 static/

# Fix directory permissions
find static/ -type d -exec chmod 755 {} \;
find static/ -type f -exec chmod 644 {} \;
```

## Best Practices

### File Organization
- Use descriptive directory names
- Group related files together
- Keep deep nesting to minimum (max 3-4 levels)

### Performance
- Optimize images (WebP, AVIF for modern browsers)
- Minify CSS and JavaScript
- Use appropriate file formats
- Implement proper caching strategy

### Security
- Don't serve configuration files
- Avoid exposing sensitive data in static files
- Use proper file permissions
- Regular security audits

## Next Steps

- Configure [Custom Headers](./headers.md) for static files
- Set up [Health Monitoring](./health.md)
- Learn about [Performance Tuning](./performance.md)
