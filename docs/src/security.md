# Security

BWS provides comprehensive security features to protect your web applications and server infrastructure.

## Management API Security

BWS includes a secure Management API for administrative operations like configuration reloading. This API is designed with security-first principles.

### Localhost-Only Access

The Management API runs on a separate port and binds exclusively to localhost (`127.0.0.1`) to prevent external access:

```toml
[management]
enabled = true
host = "127.0.0.1"    # Localhost only - cannot be changed
port = 7654           # Configurable port
```

**Security Benefits:**
- 🔒 **No External Access**: Only accessible from the server itself
- 🔒 **Isolated Service**: Separate from main web server ports
- 🔒 **IP Validation**: Double-checks request origin is localhost

### API Key Authentication

For additional security, you can enable API key authentication:

```toml
[management]
enabled = true
host = "127.0.0.1"
port = 7654
api_key = "your-secure-api-key-here"
```

**Usage with API Key:**
```bash
# Config reload with API key
curl -X POST http://127.0.0.1:7654/api/config/reload \
  -H "X-API-Key: your-secure-api-key-here"
```

**Without API Key:**
```bash
# Config reload (localhost only)
curl -X POST http://127.0.0.1:7654/api/config/reload
```

### Available Endpoints

| Endpoint | Method | Description | Authentication |
|----------|--------|-------------|----------------|
| `/api/config/reload` | POST | Reload configuration | Optional API Key |

### Security Logging

All Management API requests are logged with client IP addresses:

```
[INFO] Management API: Config reload requested
[INFO] Configuration reloaded successfully via management API
[INFO] Management API request: POST /api/config/reload from 127.0.0.1:35528
```

## General Security Features

### Request Size Limits

Protect against oversized requests:

```toml
[security]
max_request_size = "10MB"
```

### HTTPS/TLS Support

BWS supports modern TLS configurations:

```toml
[sites.ssl]
enabled = true
cert_path = "./certs/domain.crt"
key_path = "./certs/domain.key"
auto_cert = true  # ACME/Let's Encrypt support
```

### Custom Security Headers

Add security headers to responses:

```toml
[sites.headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"X-XSS-Protection" = "1; mode=block"
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
"Content-Security-Policy" = "default-src 'self'"
```

### Access Control

Configure CORS and access control:

```toml
[sites.access_control]
enabled = true
allow_origins = ["https://example.com", "https://app.example.com"]
allow_methods = ["GET", "POST", "PUT", "DELETE"]
allow_headers = ["Content-Type", "Authorization"]
max_age = 3600
```

## Production Security Checklist

### Management API
- [ ] Enable API key authentication in production
- [ ] Ensure port 7654 is not exposed externally via firewall
- [ ] Monitor management API access logs
- [ ] Rotate API keys regularly

### General Security
- [ ] Use HTTPS/TLS for all sites
- [ ] Configure appropriate security headers
- [ ] Set reasonable request size limits
- [ ] Enable access logging
- [ ] Regular security updates

### Network Security
- [ ] Configure firewall rules
- [ ] Use reverse proxy if needed
- [ ] Monitor unusual traffic patterns
- [ ] Implement rate limiting if required

## Security Best Practices

1. **Principle of Least Privilege**: Only enable features you need
2. **Defense in Depth**: Use multiple security layers
3. **Regular Updates**: Keep BWS and dependencies updated
4. **Monitoring**: Enable comprehensive logging
5. **Testing**: Regularly test security configurations

## Reporting Security Issues

If you discover a security vulnerability in BWS, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Contact the maintainers privately
3. Provide detailed reproduction steps
4. Allow time for the issue to be fixed before disclosure

Your security reports help make BWS safer for everyone.
