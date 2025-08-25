# BWS Compression Documentation

BWS (Blazing Web Server) now supports advanced HTTP response compression using multiple algorithms including Gzip, Deflate, and Brotli compression.

## Features

- **Multiple Compression Algorithms**: Supports Gzip, Deflate, and Brotli compression
- **Automatic Algorithm Selection**: Automatically selects the best compression method based on client's Accept-Encoding header
- **Smart Content-Type Detection**: Only compresses appropriate file types (text/html, text/css, application/javascript, etc.)
- **Configurable Compression Levels**: Balance between compression speed and ratio (0-9)
- **Minimum Size Threshold**: Avoids compressing small files that don't benefit from compression
- **Per-Site Configuration**: Each site can have its own compression settings

## Configuration

Add compression settings to your site configuration in `config.toml`:

```toml
[[sites]]
name = "example"
hostname = "example.com"
port = 8080
static_dir = "public"

[sites.compression]
enabled = true                                    # Enable/disable compression for this site
types = [                                        # MIME types to compress
    "text/html",
    "text/css", 
    "application/javascript",
    "application/json",
    "text/xml",
    "application/xml",
    "text/plain"
]
level = 6                                        # Compression level (0-9, higher = better compression)
min_size = 1024                                  # Minimum file size to compress (bytes)
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable or disable compression for the site |
| `types` | array of strings | See below | MIME types that should be compressed |
| `level` | integer (0-9) | `6` | Compression level (0=fastest, 9=best compression) |
| `min_size` | integer | `1024` | Minimum file size in bytes to compress |

### Default Compressible Types

```toml
types = [
    "text/html",
    "text/css",
    "text/javascript",
    "application/javascript", 
    "application/json",
    "text/xml",
    "application/xml",
    "text/plain"
]
```

## Algorithm Priority

BWS automatically selects the best compression algorithm based on the client's `Accept-Encoding` header:

1. **Brotli (`br`)** - Highest priority, best compression ratio
2. **Gzip (`gzip`)** - Good compression, widely supported
3. **Deflate (`deflate`)** - Basic compression, legacy support
4. **None** - No compression if client doesn't support any

## Performance Impact

### Compression Ratios (Typical)

- **Text files (HTML, CSS, JS)**: 60-80% reduction
- **JSON/XML data**: 70-90% reduction
- **Plain text**: 50-70% reduction

### Example Results

Testing on a 9,528-byte HTML file:

- **No compression**: 9,528 bytes
- **Gzip compression**: 3,458 bytes (63% reduction)
- **Brotli compression**: 3,198 bytes (66% reduction)

### CPU vs Bandwidth Trade-off

Higher compression levels use more CPU but provide better compression:

- **Level 1-3**: Fast compression, moderate ratio
- **Level 4-6**: Balanced speed/ratio (recommended)
- **Level 7-9**: Best compression, higher CPU usage

## HTTP Headers

When compression is applied, BWS adds these headers:

```http
Content-Encoding: gzip|br|deflate
Vary: Accept-Encoding
Content-Length: [compressed-size]
```

## Testing Compression

### Manual Testing

```bash
# Test with gzip support
curl -H "Accept-Encoding: gzip" -I http://localhost:8080/page.html

# Test with brotli support  
curl -H "Accept-Encoding: br" -I http://localhost:8080/page.html

# Test without compression
curl -H "Accept-Encoding: identity" -I http://localhost:8080/page.html
```

### Expected Response Headers

**With compression:**
```http
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Content-Encoding: gzip
Vary: Accept-Encoding
Content-Length: 3458
```

**Without compression:**
```http
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Content-Length: 9528
```

## Best Practices

### When to Enable Compression

- ✅ **Text-based content**: HTML, CSS, JavaScript, JSON, XML
- ✅ **Large files**: Files larger than 1KB benefit most
- ✅ **Dynamic content**: API responses, generated HTML
- ❌ **Already compressed**: Images (JPEG, PNG), videos, ZIP files
- ❌ **Very small files**: Files under 1KB may not benefit

### Recommended Settings

**Production (performance-focused):**
```toml
[sites.compression]
enabled = true
level = 6
min_size = 1024
```

**Development (speed-focused):**
```toml
[sites.compression]
enabled = false    # Or use level = 1 for minimal compression
```

**High-traffic sites:**
```toml
[sites.compression]
enabled = true
level = 4          # Lower level for faster compression
min_size = 2048    # Higher threshold to reduce CPU load
```

## Troubleshooting

### Compression Not Working

1. **Check Accept-Encoding header**: Client must send supported encoding
2. **Verify content type**: File must be in the configured `types` list
3. **Check file size**: File must be larger than `min_size`
4. **Confirm enabled**: Ensure `enabled = true` in configuration

### Debug Logging

Enable debug logging to see compression details:

```bash
RUST_LOG=debug cargo run -- --config config.toml
```

Look for log messages like:
```
Compressed 9528 bytes to 3458 bytes using Gzip (63% reduction)
```

### Performance Issues

If compression is causing performance problems:

1. **Lower compression level**: Reduce from 6 to 3-4
2. **Increase minimum size**: Raise `min_size` to 2KB or higher
3. **Limit content types**: Only compress the most important types
4. **Disable for development**: Set `enabled = false` for dev sites

## Security Considerations

- **BREACH attack**: Be cautious compressing sensitive data with user input
- **CPU DoS**: Consider rate limiting for high-compression endpoints
- **Memory usage**: Large files may use significant memory during compression

## Browser Compatibility

- **Gzip**: Supported by all modern browsers
- **Deflate**: Supported by all modern browsers
- **Brotli**: Supported by Chrome 50+, Firefox 44+, Safari 11+

BWS automatically falls back to older algorithms for older browsers.
