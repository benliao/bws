# SSL/TLS Configuration

BWS provides comprehensive per-site SSL/TLS support, allowing each site to have its own HTTPS configuration. This enables you to run mixed HTTP and HTTPS sites, use different SSL certificates for different domains, and configure automatic certificate management.

## Overview

BWS supports two types of SSL/TLS configuration:

1. **Automatic SSL Certificates** - Using ACME (Let's Encrypt) for automatic certificate generation and renewal
2. **Manual SSL Certificates** - Using your own SSL certificate files

Each site can independently choose its SSL configuration, enabling flexible deployment scenarios.

## Automatic SSL Certificates (ACME)

### Basic ACME Configuration

```toml
[[sites]]
name = "auto_ssl_site"
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
staging = false
challenge_dir = "./acme-challenges"
```

### ACME Configuration Options

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `enabled` | Boolean | Enable ACME certificate generation | `false` |
| `email` | String | Email address for ACME registration | Required |
| `staging` | Boolean | Use Let's Encrypt staging environment | `false` |
| `challenge_dir` | String | Directory for ACME HTTP-01 challenges | `"./acme-challenges"` |

### ACME Challenge Handling

BWS automatically handles ACME HTTP-01 challenges:

1. **Challenge Directory**: Configure `challenge_dir` where ACME challenges will be served
2. **Automatic Routing**: BWS automatically serves files from `/.well-known/acme-challenge/` 
3. **Directory Creation**: The challenge directory is created automatically if it doesn't exist

```bash
# Create challenge directory
mkdir -p ./acme-challenges

# BWS will automatically serve challenges from:
# http://yourdomain.com/.well-known/acme-challenge/TOKEN
```

### Production vs Staging

**Staging Environment** (`staging = true`):
- Use for testing and development
- Higher rate limits
- Certificates are not trusted by browsers
- Recommended for initial setup

**Production Environment** (`staging = false`):
- Use for live websites
- Lower rate limits (5 certificates per domain per week)
- Certificates are trusted by browsers
- Use only after testing with staging

```toml
# Testing configuration
[sites.ssl.acme]
enabled = true
email = "test@example.com"
staging = true  # Use staging environment
challenge_dir = "./acme-challenges"

# Production configuration
[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false  # Use production environment
challenge_dir = "./acme-challenges"
```

## Manual SSL Certificates

### Basic Manual SSL Configuration

```toml
[[sites]]
name = "manual_ssl_site"
hostname = "secure.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/secure.example.com.crt"
key_file = "/etc/ssl/private/secure.example.com.key"
```

### Certificate File Requirements

**Certificate File** (`cert_file`):
- Must contain the SSL certificate in PEM format
- Can include intermediate certificates (certificate chain)
- File must be readable by the BWS process

**Private Key File** (`key_file`):
- Must contain the private key in PEM format
- Should be protected with appropriate file permissions (600)
- Must correspond to the certificate

### Certificate Generation

You can generate certificates using various methods:

#### Self-Signed Certificates (Development)

```bash
# Generate private key
openssl genrsa -out server.key 2048

# Generate certificate
openssl req -new -x509 -key server.key -out server.crt -days 365 -subj "/CN=localhost"

# Use in configuration
[sites.ssl]
enabled = true
auto_cert = false
cert_file = "./server.crt"
key_file = "./server.key"
```

#### Commercial SSL Certificates

```bash
# Generate private key
openssl genrsa -out example.com.key 2048

# Generate certificate signing request
openssl req -new -key example.com.key -out example.com.csr

# Submit CSR to certificate authority
# Download certificate and intermediate certificates

# Combine certificate with intermediate certificates
cat example.com.crt intermediate.crt > example.com-chain.crt

# Use in configuration
[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/example.com-chain.crt"
key_file = "/etc/ssl/private/example.com.key"
```

## Per-Site SSL Configuration

### Configuration Structure

Each site has its own SSL configuration section:

```toml
[[sites]]
name = "site_name"
hostname = "example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true           # Enable SSL for this site
auto_cert = true         # Use automatic certificates
domains = ["example.com", "www.example.com"]  # Additional domains
cert_file = "path/to/cert.pem"    # Manual certificate file
key_file = "path/to/key.pem"      # Manual private key file

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"
```

### SSL Configuration Options

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `enabled` | Boolean | Enable SSL for this site | Yes |
| `auto_cert` | Boolean | Use ACME for automatic certificates | Yes |
| `domains` | Array | Additional domains for the certificate | No |
| `cert_file` | String | Path to certificate file (manual SSL) | If `auto_cert = false` |
| `key_file` | String | Path to private key file (manual SSL) | If `auto_cert = false` |

## Multi-Site SSL Examples

### Mixed HTTP and HTTPS Sites

```toml
[server]
name = "Mixed Protocol Server"

# HTTP site on port 80
[[sites]]
name = "http_site"
hostname = "example.com"
port = 80
static_dir = "static"

[sites.ssl]
enabled = false

# HTTPS site on port 443 with auto SSL
[[sites]]
name = "https_site"
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
staging = false
challenge_dir = "./acme-challenges"

# API site with manual SSL
[[sites]]
name = "api_site"
hostname = "api.example.com"
port = 8443
static_dir = "api-static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/api.example.com.crt"
key_file = "/etc/ssl/private/api.example.com.key"
```

### Different SSL Configurations per Site

```toml
# Main website with Let's Encrypt
[[sites]]
name = "main"
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
staging = false
challenge_dir = "./acme-challenges"

# Corporate subdomain with commercial certificate
[[sites]]
name = "corporate"
hostname = "corp.example.com"
port = 443
static_dir = "corp-static"

[sites.ssl]
enabled = true
auto_cert = false
cert_file = "/etc/ssl/certs/corp.example.com.crt"
key_file = "/etc/ssl/private/corp.example.com.key"

# Development site without SSL
[[sites]]
name = "dev"
hostname = "dev.example.com"
port = 8080
static_dir = "dev-static"

[sites.ssl]
enabled = false
```

## Security Best Practices

### File Permissions

Ensure proper file permissions for SSL files:

```bash
# Certificate files can be world-readable
chmod 644 /etc/ssl/certs/*.crt

# Private keys should be readable only by the owner
chmod 600 /etc/ssl/private/*.key

# Ensure BWS can read the files
chown bws:bws /etc/ssl/private/*.key
```

### Security Headers

Use security headers with HTTPS sites:

```toml
[[sites]]
name = "secure_site"
hostname = "example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false
challenge_dir = "./acme-challenges"

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains; preload"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"Referrer-Policy" = "strict-origin-when-cross-origin"
"Content-Security-Policy" = "default-src 'self'; script-src 'self' 'unsafe-inline'"
```

### Certificate Monitoring

Monitor certificate expiration:

```bash
# Check certificate expiration
openssl x509 -in /etc/ssl/certs/example.com.crt -noout -dates

# For ACME certificates, BWS handles renewal automatically
# Check ACME certificate status in logs
tail -f /var/log/bws.log | grep -i acme
```

## Troubleshooting

### Common Issues

#### ACME Challenge Failures

```toml
# Ensure challenge directory is accessible
[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = true  # Use staging for debugging
challenge_dir = "./acme-challenges"
```

Check that:
1. Domain resolves to your server
2. Port 80 is accessible for HTTP-01 challenges
3. Challenge directory exists and is writable
4. No firewall blocking HTTP traffic

#### Certificate File Errors

```bash
# Verify certificate file format
openssl x509 -in cert.pem -text -noout

# Verify private key format
openssl rsa -in key.pem -check

# Check if certificate and key match
openssl x509 -noout -modulus -in cert.pem | openssl md5
openssl rsa -noout -modulus -in key.pem | openssl md5
```

#### Port Binding Issues

```bash
# Check if port is already in use
sudo lsof -i :443

# Ensure BWS has permission to bind to privileged ports
sudo setcap CAP_NET_BIND_SERVICE=+eip /usr/local/bin/bws-web-server
```

### Debug Mode

Enable verbose logging for SSL debugging:

```bash
# Start BWS with verbose logging
RUST_LOG=debug bws-web-server --config config.toml --verbose
```

### Validation

Test SSL configuration:

```bash
# Test HTTPS connectivity
curl -I https://example.com

# Check SSL certificate
openssl s_client -connect example.com:443 -servername example.com

# Test with specific protocol versions
curl --tlsv1.2 -I https://example.com
curl --tlsv1.3 -I https://example.com
```

## Next Steps

- Configure [Custom Headers](./headers.md) for enhanced security
- Set up [Health Monitoring](./health.md) for SSL sites
- Learn about [Production Deployment](./production.md) with SSL
- Review [Multi-Site Setup](./multi-site.md) for complex configurations
