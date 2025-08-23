# Production Setup

This guide covers deploying BWS in production environments with security, performance, and reliability best practices.

## Production Architecture

### Recommended Infrastructure
```
Internet → Load Balancer → Reverse Proxy → BWS Instances
                                       ↓
                              Monitoring & Logging
```

### High Availability Setup
```
                    Load Balancer (HAProxy/Nginx)
                           /              \
                    BWS Instance 1    BWS Instance 2
                         |                    |
                    Static Files        Static Files
                    (NFS/S3)           (NFS/S3)
                         |                    |
                    Health Monitor     Health Monitor
```

## Security Configuration

### SSL/TLS Termination
BWS is typically deployed behind a reverse proxy that handles SSL termination:

```nginx
# /etc/nginx/sites-available/bws-production
server {
    listen 443 ssl http2;
    server_name example.com www.example.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/example.com.crt;
    ssl_certificate_key /etc/ssl/private/example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-Frame-Options DENY always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=static:10m rate=50r/s;

    location / {
        limit_req zone=static burst=20 nodelay;
        proxy_pass http://bws_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }

    location /api/ {
        limit_req zone=api burst=5 nodelay;
        proxy_pass http://bws_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /health {
        proxy_pass http://bws_backend/health;
        access_log off;
        allow 127.0.0.1;
        allow 10.0.0.0/8;
        deny all;
    }
}

upstream bws_backend {
    least_conn;
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8081 max_fails=3 fail_timeout=30s backup;
    keepalive 32;
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name example.com www.example.com;
    return 301 https://$server_name$request_uri;
}
```

### Firewall Configuration
```bash
# UFW (Ubuntu Firewall) example
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow from 10.0.0.0/8 to any port 8080  # Internal BWS access
sudo ufw enable

# iptables example
iptables -A INPUT -p tcp --dport 22 -j ACCEPT    # SSH
iptables -A INPUT -p tcp --dport 80 -j ACCEPT    # HTTP
iptables -A INPUT -p tcp --dport 443 -j ACCEPT   # HTTPS
iptables -A INPUT -s 10.0.0.0/8 -p tcp --dport 8080 -j ACCEPT  # BWS internal
iptables -A INPUT -j DROP
```

## Production Configuration

### Optimized BWS Configuration
```toml
# /etc/bws/production.toml
[daemon]
user = "bws"
group = "bws"
pid_file = "/var/run/bws.pid"
working_directory = "/opt/bws"

[logging]
level = "info"
output = "file"
file_path = "/var/log/bws/bws.log"
max_size = "100MB"
max_files = 10
compress = true
format = "json"

[monitoring]
enabled = true
health_endpoint = "/health"
detailed_endpoint = "/health/detailed"
metrics_endpoint = "/metrics"

[monitoring.checks]
disk_threshold = 85
memory_threshold = 80
response_time_threshold = 1000

[performance]
max_connections = 10000
worker_threads = 8
keep_alive_timeout = 30
request_timeout = 30
max_request_size = "10MB"

# Production site
[[sites]]
name = "production"
hostname = "127.0.0.1"  # Behind reverse proxy
port = 8080
static_dir = "/opt/bws/static"

[sites.headers]
"Cache-Control" = "public, max-age=31536000"
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
"X-Served-By" = "BWS-Production"
"Vary" = "Accept-Encoding"

# API site
[[sites]]
name = "api"
hostname = "127.0.0.1"
port = 8081
static_dir = "/opt/bws/api-docs"

[sites.headers]
"Cache-Control" = "no-cache, no-store, must-revalidate"
"Content-Type" = "application/json"
"Access-Control-Allow-Origin" = "https://example.com"
"X-API-Version" = "v1.0.0"
```

### Environment Variables
```bash
# /etc/environment.d/bws.conf
BWS_CONFIG=/etc/bws/production.toml
BWS_LOG_FILE=/var/log/bws/bws.log
BWS_PID_FILE=/var/run/bws.pid
RUST_LOG=info
RUST_BACKTRACE=0
BWS_ENV=production
```

## Performance Optimization

### System Tuning
```bash
# /etc/sysctl.d/99-bws.conf
# Network performance
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_keepalive_time = 600
net.ipv4.tcp_keepalive_intvl = 60
net.ipv4.tcp_keepalive_probes = 3

# File descriptor limits
fs.file-max = 1048576

# Apply settings
sysctl -p /etc/sysctl.d/99-bws.conf
```

### System Limits
```bash
# /etc/security/limits.d/bws.conf
bws soft nofile 65536
bws hard nofile 65536
bws soft nproc 4096
bws hard nproc 4096
```

### File System Optimization
```bash
# Mount options for static files (add to /etc/fstab)
/dev/sdb1 /opt/bws/static ext4 defaults,noatime,nodiratime 0 0

# For high-performance scenarios, consider tmpfs for cache
tmpfs /opt/bws/cache tmpfs defaults,size=1G,mode=755,uid=bws,gid=bws 0 0
```

## Monitoring and Alerting

### Prometheus Configuration
```yaml
# /etc/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "bws_rules.yml"

scrape_configs:
  - job_name: 'bws'
    static_configs:
      - targets: ['localhost:8080', 'localhost:8081']
    metrics_path: '/metrics'
    scrape_interval: 5s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alert Rules
```yaml
# /etc/prometheus/bws_rules.yml
groups:
- name: bws.rules
  rules:
  - alert: BWS_Down
    expr: up{job="bws"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "BWS instance is down"
      description: "BWS instance {{ $labels.instance }} has been down for more than 1 minute."

  - alert: BWS_HighResponseTime
    expr: histogram_quantile(0.95, bws_response_time_seconds) > 1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "BWS high response time"
      description: "95th percentile response time is {{ $value }}s"

  - alert: BWS_HighErrorRate
    expr: rate(bws_requests_total{status=~"5.."}[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "BWS high error rate"
      description: "Error rate is {{ $value }} errors per second"

  - alert: BWS_HighMemoryUsage
    expr: bws_memory_usage_bytes / bws_memory_limit_bytes > 0.8
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "BWS high memory usage"
      description: "Memory usage is {{ $value }}%"
```

### Grafana Dashboard
```json
{
  "dashboard": {
    "id": null,
    "title": "BWS Production Dashboard",
    "tags": ["bws", "production"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(bws_requests_total[5m])",
            "legendFormat": "{{ instance }}"
          }
        ],
        "yAxes": [
          {
            "label": "Requests/sec",
            "min": 0
          }
        ]
      },
      {
        "id": 2,
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, bws_response_time_seconds)",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, bws_response_time_seconds)",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "id": 3,
        "title": "Error Rate",
        "type": "singlestat",
        "targets": [
          {
            "expr": "rate(bws_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "5xx errors/sec"
          }
        ]
      }
    ]
  }
}
```

## Backup and Disaster Recovery

### Configuration Backup
```bash
#!/bin/bash
# /usr/local/bin/backup-bws-config.sh
BACKUP_DIR="/backup/bws"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="bws-config-$DATE.tar.gz"

mkdir -p "$BACKUP_DIR"

# Backup configuration
tar -czf "$BACKUP_DIR/$BACKUP_FILE" \
    /etc/bws/ \
    /opt/bws/static/ \
    /var/log/bws/ \
    /etc/systemd/system/bws.service

# Keep only last 30 backups
find "$BACKUP_DIR" -name "bws-config-*.tar.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/$BACKUP_FILE"
```

### Log Backup and Rotation
```bash
# /etc/logrotate.d/bws
/var/log/bws/*.log {
    daily
    missingok
    rotate 90
    compress
    delaycompress
    notifempty
    copytruncate
    postrotate
        systemctl reload bws
        # Archive to S3 or backup system
        aws s3 cp /var/log/bws/bws.log.1.gz s3://backup-bucket/logs/bws/$(date +%Y/%m/%d)/
    endscript
}
```

### Health Check and Recovery
```bash
#!/bin/bash
# /usr/local/bin/bws-recovery.sh
BWS_HEALTH_URL="http://localhost:8080/health"
LOG_FILE="/var/log/bws/recovery.log"
SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

send_alert() {
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"BWS Alert: $1\"}" \
        "$SLACK_WEBHOOK"
}

# Check BWS health
if ! curl -f -s "$BWS_HEALTH_URL" > /dev/null; then
    log_message "BWS health check failed, attempting recovery"
    send_alert "BWS is down, attempting automatic recovery"
    
    # Try graceful restart
    systemctl restart bws
    sleep 10
    
    # Check if restart was successful
    if curl -f -s "$BWS_HEALTH_URL" > /dev/null; then
        log_message "BWS recovered successfully"
        send_alert "BWS recovery successful"
    else
        log_message "BWS recovery failed"
        send_alert "BWS recovery FAILED - manual intervention required"
        exit 1
    fi
else
    log_message "BWS is healthy"
fi
```

## Deployment Strategies

### Blue-Green Deployment
```bash
#!/bin/bash
# /usr/local/bin/deploy-bws.sh
BLUE_PORT=8080
GREEN_PORT=8081
HEALTH_CHECK_URL="http://localhost"

deploy_to_port() {
    local PORT=$1
    local VERSION=$2
    
    echo "Deploying BWS $VERSION to port $PORT"
    
    # Stop existing service
    systemctl stop bws-$PORT
    
    # Update binary
    cp /tmp/bws-$VERSION /usr/local/bin/bws-$PORT
    
    # Update configuration
    sed "s/port = .*/port = $PORT/" /etc/bws/config.toml > /etc/bws/config-$PORT.toml
    
    # Start service
    systemctl start bws-$PORT
    
    # Health check
    sleep 5
    if curl -f -s "$HEALTH_CHECK_URL:$PORT/health" > /dev/null; then
        echo "Deployment to port $PORT successful"
        return 0
    else
        echo "Deployment to port $PORT failed"
        return 1
    fi
}

# Get current active port from load balancer
CURRENT_PORT=$(nginx -T 2>/dev/null | grep "server 127.0.0.1:" | head -1 | awk '{print $2}' | cut -d: -f2)

if [ "$CURRENT_PORT" = "$BLUE_PORT" ]; then
    DEPLOY_PORT=$GREEN_PORT
    SWITCH_FROM=$BLUE_PORT
else
    DEPLOY_PORT=$BLUE_PORT
    SWITCH_FROM=$GREEN_PORT
fi

echo "Deploying to $DEPLOY_PORT (current: $SWITCH_FROM)"

# Deploy to inactive port
if deploy_to_port $DEPLOY_PORT $1; then
    # Switch load balancer
    sed -i "s/server 127.0.0.1:$SWITCH_FROM/server 127.0.0.1:$DEPLOY_PORT/" /etc/nginx/sites-available/bws-production
    nginx -s reload
    
    echo "Switched load balancer to port $DEPLOY_PORT"
    
    # Stop old instance after delay
    sleep 30
    systemctl stop bws-$SWITCH_FROM
    
    echo "Deployment completed successfully"
else
    echo "Deployment failed"
    exit 1
fi
```

### Rolling Updates
```bash
#!/bin/bash
# /usr/local/bin/rolling-update.sh
INSTANCES=("server1" "server2" "server3")
VERSION=$1

for instance in "${INSTANCES[@]}"; do
    echo "Updating $instance..."
    
    # Remove from load balancer
    ssh $instance "nginx -s reload"  # Remove from upstream
    
    # Wait for connections to drain
    sleep 30
    
    # Deploy new version
    ssh $instance "systemctl stop bws && cp /tmp/bws-$VERSION /usr/local/bin/bws && systemctl start bws"
    
    # Health check
    if ssh $instance "curl -f -s http://localhost:8080/health"; then
        echo "$instance updated successfully"
        # Add back to load balancer
        ssh $instance "nginx -s reload"  # Add to upstream
    else
        echo "Update failed on $instance"
        exit 1
    fi
    
    sleep 10
done

echo "Rolling update completed"
```

## Security Best Practices

### Access Control
```bash
# Restrict access to BWS configuration
chmod 600 /etc/bws/config.toml
chown root:bws /etc/bws/config.toml

# Secure log files
chmod 640 /var/log/bws/*.log
chown bws:adm /var/log/bws/*.log

# Secure binary
chmod 755 /usr/local/bin/bws
chown root:root /usr/local/bin/bws
```

### Network Security
```bash
# Disable unnecessary services
systemctl disable telnet
systemctl disable ftp
systemctl disable rsh

# Configure fail2ban for SSH protection
cat > /etc/fail2ban/jail.local << EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
EOF

systemctl enable fail2ban
systemctl start fail2ban
```

### Regular Security Updates
```bash
#!/bin/bash
# /usr/local/bin/security-updates.sh
# Run via cron: 0 2 * * 0 /usr/local/bin/security-updates.sh

# Update system packages
apt update && apt upgrade -y

# Update BWS if new version available
CURRENT_VERSION=$(bws --version | awk '{print $2}')
LATEST_VERSION=$(curl -s https://api.github.com/repos/yourusername/bws/releases/latest | jq -r .tag_name)

if [ "$CURRENT_VERSION" != "$LATEST_VERSION" ]; then
    echo "BWS update available: $CURRENT_VERSION -> $LATEST_VERSION"
    # Implement update process
fi

# Restart services if needed
if [ -f /var/run/reboot-required ]; then
    echo "Reboot required after updates"
    # Schedule maintenance window reboot
fi
```

## Troubleshooting Production Issues

### Common Issues and Solutions

#### High CPU Usage
```bash
# Monitor CPU usage
top -p $(pgrep bws)
htop -p $(pgrep bws)

# Check system load
uptime
iostat 1

# Review configuration
grep -E "worker_threads|max_connections" /etc/bws/config.toml
```

#### Memory Leaks
```bash
# Monitor memory usage over time
while true; do
    ps -o pid,ppid,cmd,%mem,%cpu -p $(pgrep bws)
    sleep 60
done > /tmp/bws-memory.log

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full /usr/local/bin/bws
```

#### Network Issues
```bash
# Check port bindings
netstat -tulpn | grep bws

# Monitor connections
ss -tuln | grep :8080
lsof -i :8080

# Check network performance
iftop
nethogs
```

#### Disk Space Issues
```bash
# Check disk usage
df -h
du -sh /var/log/bws/
du -sh /opt/bws/

# Clean up logs
journalctl --vacuum-time=7d
logrotate -f /etc/logrotate.d/bws
```

## Maintenance Procedures

### Regular Maintenance Tasks
```bash
#!/bin/bash
# /usr/local/bin/bws-maintenance.sh
# Run weekly via cron

LOG_FILE="/var/log/bws/maintenance.log"

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

log_message "Starting BWS maintenance"

# Check disk space
DISK_USAGE=$(df /opt/bws | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 85 ]; then
    log_message "WARNING: Disk usage is ${DISK_USAGE}%"
fi

# Rotate logs
logrotate -f /etc/logrotate.d/bws

# Check configuration
if bws --config-check /etc/bws/config.toml; then
    log_message "Configuration is valid"
else
    log_message "ERROR: Configuration validation failed"
fi

# Update file permissions
chown -R bws:bws /opt/bws/static/
chmod -R 644 /opt/bws/static/*

# Clean temporary files
find /tmp -name "bws-*" -mtime +7 -delete

# Backup configuration
/usr/local/bin/backup-bws-config.sh

log_message "BWS maintenance completed"
```

### Update Procedures
```bash
#!/bin/bash
# /usr/local/bin/update-bws.sh
NEW_VERSION=$1

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Updating BWS to version $NEW_VERSION"

# Backup current version
cp /usr/local/bin/bws /usr/local/bin/bws.backup

# Download new version
wget "https://github.com/yourusername/bws/releases/download/$NEW_VERSION/bws-linux-amd64" -O /tmp/bws-$NEW_VERSION

# Verify checksum (if available)
# wget "https://github.com/yourusername/bws/releases/download/$NEW_VERSION/checksums.txt" -O /tmp/checksums.txt
# sha256sum -c /tmp/checksums.txt

# Test new version
chmod +x /tmp/bws-$NEW_VERSION
if /tmp/bws-$NEW_VERSION --version; then
    echo "New version validated"
else
    echo "New version validation failed"
    exit 1
fi

# Deploy using blue-green strategy
/usr/local/bin/deploy-bws.sh $NEW_VERSION

echo "BWS updated to version $NEW_VERSION"
```

## Best Practices Summary

### Configuration
- Use environment-specific configuration files
- Store sensitive data in environment variables or secret management systems
- Regularly validate configuration syntax
- Version control all configuration changes

### Security
- Run BWS behind a reverse proxy with SSL termination
- Implement proper firewall rules
- Regular security updates and patches
- Monitor for security vulnerabilities
- Use non-root user for BWS process

### Performance
- Tune system parameters for high-performance workloads
- Monitor resource usage continuously
- Implement proper caching strategies
- Use CDN for static assets when possible

### Reliability
- Implement comprehensive health checks
- Set up automated monitoring and alerting
- Use deployment strategies that minimize downtime
- Regular backups of configuration and data
- Document incident response procedures

### Monitoring
- Monitor both technical and business metrics
- Set up alerting with appropriate thresholds
- Regular review of logs and metrics
- Performance trend analysis
- Capacity planning based on growth projections

## Next Steps

- Learn about [Performance Tuning](./performance.md) for optimization
- Review [Configuration Schema](./config-schema.md) for advanced options
- Check [Troubleshooting](./troubleshooting.md) for common issues
