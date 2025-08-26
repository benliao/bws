# Production Setup

Deploy BWS in production with security, performance, and reliability best practices.

## Quick Production Setup

### 1. Install BWS
```bash
# Install from crates.io
cargo install bws-web-server

# Or use Docker
docker pull ghcr.io/benliao/bws:latest
```

### 2. Create Production Configuration
```toml
[server]
name = "BWS Production"

# Main HTTPS site
[[sites]]
name = "main"
hostname = "example.com"
port = 443
static_dir = "/var/www/html"
default = true

[sites.ssl]
enabled = true
auto_cert = true
domains = ["example.com", "www.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
staging = false

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"

# Management API
[management]
enabled = true
port = 7654
api_key = "secure-random-key-here"

# Performance tuning
[performance]
worker_threads = 8
max_connections = 2000

# Security
[security]
hide_server_header = true
max_request_size = "50MB"

# Logging
[logging]
level = "info"
log_requests = true
```

### 3. System Setup
```bash
# Create user and directories
sudo useradd -r -s /bin/false bws
sudo mkdir -p /var/www/html /var/log/bws /etc/bws
sudo chown -R bws:bws /var/www /var/log/bws

# Copy configuration
sudo cp config.toml /etc/bws/

# Set permissions
sudo chmod 600 /etc/bws/config.toml
sudo chown bws:bws /etc/bws/config.toml
```

## Systemd Service

### Create Service File
```ini
# /etc/systemd/system/bws.service
[Unit]
Description=BWS Web Server
After=network.target
Wants=network.target

[Service]
Type=forking
User=bws
Group=bws
ExecStart=/usr/local/bin/bws --config /etc/bws/config.toml --daemon
ExecReload=/bin/kill -HUP $MAINPID
PIDFile=/var/run/bws.pid
Restart=always
RestartSec=5
LimitNOFILE=65535

# Security
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/www /var/log/bws /var/run

[Install]
WantedBy=multi-user.target
```

### Service Management
```bash
# Enable and start service
sudo systemctl enable bws
sudo systemctl start bws

# Check status
sudo systemctl status bws

# View logs
sudo journalctl -u bws -f

# Reload configuration
sudo systemctl reload bws
```

## Docker Production Deployment

### Docker Compose
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  bws:
    image: ghcr.io/benliao/bws:v0.3.5
    container_name: bws-prod
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./static:/app/static:ro
      - ./acme-challenges:/app/acme-challenges
      - ./logs:/app/logs
    environment:
      - BWS_LOG_LEVEL=info
    networks:
      - bws-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  bws-network:
    driver: bridge
```

### Deploy with Docker
```bash
# Start production stack
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose -f docker-compose.prod.yml logs -f

# Update deployment
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

## Reverse Proxy Setup

### Nginx Frontend (Optional)
```nginx
# /etc/nginx/sites-available/bws
upstream bws_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name example.com www.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name example.com www.example.com;

    ssl_certificate /etc/letsencrypt/live/example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;

    location / {
        proxy_pass http://bws_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Security

### Firewall Configuration
```bash
# UFW setup
sudo ufw allow 22/tcp      # SSH
sudo ufw allow 80/tcp      # HTTP
sudo ufw allow 443/tcp     # HTTPS
sudo ufw enable

# Restrict management API
sudo ufw allow from 127.0.0.1 to any port 7654
```

### SSL/TLS Security
```toml
# Strong SSL configuration
[sites.ssl]
enabled = true
auto_cert = true
min_tls_version = "1.2"

[sites.headers]
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains; preload"
"Content-Security-Policy" = "default-src 'self'"
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"
"Referrer-Policy" = "strict-origin-when-cross-origin"
```

## Monitoring

### Health Checks
```bash
# Basic health check
curl -f http://localhost/api/health

# Detailed health information
curl http://localhost/api/health/detailed

# Check SSL certificate
echo | openssl s_client -connect example.com:443 2>/dev/null | openssl x509 -noout -dates
```

### Log Monitoring
```bash
# Monitor access logs
tail -f /var/log/bws/access.log

# Monitor error logs
tail -f /var/log/bws/error.log

# System resource monitoring
htop
iostat -x 1
```

### Alerting Setup
```bash
# Check certificate expiration
#!/bin/bash
# check-ssl.sh
DOMAIN="example.com"
EXPIRY=$(echo | openssl s_client -connect $DOMAIN:443 2>/dev/null | openssl x509 -noout -enddate | cut -d= -f2)
EXPIRY_DATE=$(date -d "$EXPIRY" +%s)
CURRENT_DATE=$(date +%s)
DAYS_UNTIL_EXPIRY=$(( ($EXPIRY_DATE - $CURRENT_DATE) / 86400 ))

if [ $DAYS_UNTIL_EXPIRY -lt 30 ]; then
    echo "SSL certificate for $DOMAIN expires in $DAYS_UNTIL_EXPIRY days!"
    # Send alert (email, Slack, etc.)
fi
```

## Performance Tuning

### System Limits
```bash
# /etc/security/limits.conf
bws soft nofile 65535
bws hard nofile 65535

# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.core.netdev_max_backlog = 5000
```

### BWS Configuration
```toml
[performance]
worker_threads = 16              # 2x CPU cores
max_connections = 10000          # Per worker
keep_alive_timeout = 75          # Seconds
request_timeout = 30             # Seconds
max_header_size = "16KB"
```

## Backup and Recovery

### Configuration Backup
```bash
# Backup script
#!/bin/bash
BACKUP_DIR="/backups/bws/$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# Backup configuration
cp /etc/bws/config.toml $BACKUP_DIR/
cp -r /var/www $BACKUP_DIR/
cp -r /etc/letsencrypt $BACKUP_DIR/

# Create archive
tar -czf $BACKUP_DIR.tar.gz $BACKUP_DIR
rm -rf $BACKUP_DIR
```

### Recovery Procedure
```bash
# Restore from backup
tar -xzf /backups/bws/20250827.tar.gz -C /

# Restore permissions
sudo chown -R bws:bws /var/www /etc/bws
sudo chmod 600 /etc/bws/config.toml

# Restart service
sudo systemctl restart bws
```

## Troubleshooting

### Common Issues
```bash
# Check BWS status
sudo systemctl status bws

# Validate configuration
bws --config /etc/bws/config.toml --dry-run

# Check logs
sudo journalctl -u bws --since "1 hour ago"

# Test configuration reload
curl -X POST http://127.0.0.1:7654/api/config/reload \
  -H "X-API-Key: your-api-key"

# Check process tree
pstree -p $(pgrep -f bws)
```

### Performance Debugging
```bash
# Check resource usage
top -p $(pgrep bws)

# Network connections
ss -tulpn | grep bws

# File descriptors
lsof -p $(pgrep bws) | wc -l
```

## Updates and Maintenance

### Rolling Updates
```bash
# Download new version
cargo install bws-web-server --force

# Validate new binary
bws --version
bws --config /etc/bws/config.toml --dry-run

# Hot reload (zero downtime)
sudo systemctl reload bws

# Or restart if needed
sudo systemctl restart bws
```

### Maintenance Window
```bash
# Graceful shutdown
sudo systemctl stop bws

# Perform maintenance
# - Update system packages
# - Update BWS configuration
# - Clean log files

# Start service
sudo systemctl start bws

# Verify operation
curl -f http://localhost/api/health
```
