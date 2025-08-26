# SSL/TLS Configuration

BWS supports both automatic (Let's Encrypt) and manual SSL certificates with per-site configuration.

## Automatic SSL (Let's Encrypt)

### Basic Setup

```toml
[[sites]]
name = "secure"
hostname = "example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false                        # Use true for testing
challenge_dir = "./acme-challenges"
```

### ACME Options

| Field | Description | Default |
|-------|-------------|---------|
| `email` | Contact email for Let's Encrypt | Required |
| `staging` | Use staging environment for testing | `false` |
| `challenge_dir` | Directory for HTTP-01 challenges | `"./acme-challenges"` |

### Challenge Setup

```bash
# Create challenge directory
mkdir -p ./acme-challenges

# BWS automatically serves challenges at:
# http://yourdomain.com/.well-known/acme-challenge/
```

**Requirements:**
- Domain must be publicly accessible on port 80
- DNS must point to your server
- Challenge directory must be readable

## Manual SSL Certificates

### Configuration

```toml
[[sites]]
name = "manual-ssl"
hostname = "secure.local"
port = 8443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./certs/secure.local.crt"
key_file = "./certs/secure.local.key"
```

### Generate Self-Signed Certificate

```bash
# Create certificates directory
mkdir -p certs

# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout certs/secure.local.key \
  -out certs/secure.local.crt \
  -days 365 -nodes \
  -subj "/CN=secure.local"

# Set proper permissions
chmod 600 certs/secure.local.key
chmod 644 certs/secure.local.crt
```

### Use Existing Certificates

```bash
# Copy your existing certificates
cp /path/to/your.crt certs/
cp /path/to/your.key certs/

# Update permissions
chmod 600 certs/your.key
chmod 644 certs/your.crt
```

## Mixed HTTP/HTTPS Setup

Host different sites with different SSL configurations:

```toml
# HTTP site
[[sites]]
name = "http"
hostname = "example.com"
port = 80
static_dir = "static"

# HTTPS site with auto SSL
[[sites]]
name = "https-auto"
hostname = "secure.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

# HTTPS site with manual SSL
[[sites]]
name = "https-manual"
hostname = "internal.example.com"
port = 8443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./certs/internal.crt"
key_file = "./certs/internal.key"
```

## SSL Security Headers

Add security headers for HTTPS sites:

```toml
[sites.headers]
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
```

## Certificate Renewal

### Automatic Renewal
- Certificates are automatically renewed before expiration
- No manual intervention required
- Renewal status logged to server logs

### Manual Renewal
```bash
# Check certificate expiration
openssl x509 -in certs/your.crt -text -noout | grep "Not After"

# Replace expired certificates
cp /path/to/new.crt certs/
cp /path/to/new.key certs/

# Reload BWS configuration
curl -X POST http://127.0.0.1:7654/api/config/reload
```

## Testing SSL Configuration

### Verify Certificate
```bash
# Check certificate details
openssl x509 -in certs/your.crt -text -noout

# Test SSL connection
openssl s_client -connect yourdomain.com:443 -servername yourdomain.com

# Check certificate chain
curl -I https://yourdomain.com/
```

### SSL Labs Test
Use [SSL Labs](https://www.ssllabs.com/ssltest/) to test your SSL configuration for security issues.

## Troubleshooting

### Common Issues

**ACME Challenge Fails:**
- Verify domain points to your server
- Check port 80 is accessible
- Ensure challenge directory exists and is readable

**Certificate Not Loading:**
- Check file paths in configuration
- Verify file permissions (key file should be 600)
- Check BWS logs for error messages

**Mixed Content Warnings:**
- Ensure all resources use HTTPS URLs
- Add security headers to prevent mixed content

### Debug Commands

```bash
# Validate SSL configuration
bws --config config.toml --dry-run

# Check file permissions
ls -la certs/

# Test certificate
curl -v https://yourdomain.com/

# Check server logs
tail -f /var/log/bws.log
```
