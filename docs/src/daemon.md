# Daemon Mode Configuration

BWS can run as a system daemon (background service) for production deployments, providing automatic startup, monitoring, and management capabilities.

## Master-Worker Architecture

BWS uses a master-worker process model for production deployments:

### Process Model
- **Master Process**: Manages configuration, handles signals, spawns workers
- **Worker Processes**: Handle HTTP traffic, SSL termination, proxying
- **Hot Reload**: Zero-downtime configuration updates via worker replacement
- **Graceful Shutdown**: Coordinated shutdown with connection draining

### Signal Handling
BWS master process responds to standard Unix signals:

```bash
# Hot reload configuration (spawn new worker)
sudo kill -HUP $(pgrep -f "bws.*master")

# Graceful shutdown (stop all processes)
sudo kill -TERM $(pgrep -f "bws.*master")

# Check process tree
pstree -p $(pgrep -f "bws.*master")
```

## Systemd Configuration

### Service File Creation
Create a systemd service file for BWS:

```ini
# /etc/systemd/system/bws.service
[Unit]
Description=BWS Multi-Site Web Server
Documentation=https://github.com/yourusername/bws
After=network.target
Wants=network.target

[Service]
Type=simple
User=bws
Group=bws
WorkingDirectory=/opt/bws
ExecStart=/usr/local/bin/bws --config /etc/bws/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
TimeoutStartSec=60
TimeoutStopSec=30

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/bws /var/lib/bws

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Environment
Environment=RUST_LOG=info
Environment=BWS_CONFIG=/etc/bws/config.toml
Environment=BWS_LOG_FILE=/var/log/bws/bws.log
Environment=BWS_PID_FILE=/var/run/bws.pid

[Install]
WantedBy=multi-user.target
```

### Installing the Service
```bash
# Copy service file
sudo cp bws.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable service (auto-start on boot)
sudo systemctl enable bws

# Start service
sudo systemctl start bws

# Check status
sudo systemctl status bws
```

### Service Management
```bash
# Start the service
sudo systemctl start bws

# Stop the service
sudo systemctl stop bws

# Restart the service
sudo systemctl restart bws

# Reload configuration
sudo systemctl reload bws

# Check service status
sudo systemctl status bws

# View logs
sudo journalctl -u bws -f

# Check if service is enabled
sudo systemctl is-enabled bws
```

## User and Directory Setup

### Creating BWS User
```bash
# Create system user for BWS
sudo useradd -r -s /bin/false -d /opt/bws bws

# Create necessary directories
sudo mkdir -p /opt/bws
sudo mkdir -p /etc/bws
sudo mkdir -p /var/log/bws
sudo mkdir -p /var/lib/bws

# Set ownership
sudo chown -R bws:bws /opt/bws
sudo chown -R bws:bws /var/log/bws
sudo chown -R bws:bws /var/lib/bws
sudo chown root:bws /etc/bws

# Set permissions
sudo chmod 755 /opt/bws
sudo chmod 750 /etc/bws
sudo chmod 755 /var/log/bws
sudo chmod 755 /var/lib/bws
```

### File Structure
```
/opt/bws/                 # BWS home directory
├── static/               # Static files
├── sites/                # Multi-site configurations
└── bin/                  # BWS binary (optional)

/etc/bws/                 # Configuration directory
├── config.toml           # Main configuration
├── sites/                # Site-specific configs
└── ssl/                  # SSL certificates

/var/log/bws/             # Log directory
├── bws.log               # Main log file
├── access.log            # Access logs
└── error.log             # Error logs

/var/lib/bws/             # Runtime data
├── cache/                # Cache files
└── temp/                 # Temporary files
```

## Configuration Files

### Main Configuration
```toml
# /etc/bws/config.toml
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

[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "/opt/bws/static"

[sites.headers]
"X-Served-By" = "BWS"
"Cache-Control" = "public, max-age=3600"
```

### Environment Configuration
```bash
# /etc/bws/environment
BWS_CONFIG=/etc/bws/config.toml
BWS_LOG_FILE=/var/log/bws/bws.log
BWS_PID_FILE=/var/run/bws.pid
RUST_LOG=info
RUST_BACKTRACE=1
```

## Process Management

### Signal Handling
BWS master process responds to standard Unix signals:

```bash
# Hot reload configuration (spawn new worker)
sudo kill -HUP $(pgrep -f "bws.*master")

# Graceful shutdown (stop all processes)
sudo kill -TERM $(pgrep -f "bws.*master")

# Check process tree  
pstree -p $(pgrep -f "bws.*master")
```

### Master-Worker Operations
```bash
# View all BWS processes
ps aux | grep bws

# Check master process
pgrep -f "bws.*master"

# Monitor worker processes
pgrep -f "bws.*worker"

# Hot reload without downtime
systemctl reload bws
```

### Process Monitoring
```bash
# Check if BWS is running
pgrep -f bws

# Monitor BWS process
ps aux | grep bws

# Check open files
sudo lsof -p $(cat /var/run/bws.pid)

# Monitor resource usage
top -p $(cat /var/run/bws.pid)
```

## Log Management

### Log Rotation Configuration
```bash
# /etc/logrotate.d/bws
/var/log/bws/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
    postrotate
        systemctl reload bws
    endscript
}
```

### Log Monitoring
```bash
# Follow main log
tail -f /var/log/bws/bws.log

# Follow with filtering
tail -f /var/log/bws/bws.log | grep ERROR

# Search logs
grep "error" /var/log/bws/bws.log

# Count log entries by level
grep -c "INFO\|WARN\|ERROR" /var/log/bws/bws.log
```

## Monitoring and Health Checks

### Health Check Script
```bash
#!/bin/bash
# /usr/local/bin/bws-health-check
BWS_PID_FILE="/var/run/bws.pid"
BWS_HEALTH_URL="http://localhost:8080/health"

# Check if PID file exists
if [ ! -f "$BWS_PID_FILE" ]; then
    echo "ERROR: PID file not found"
    exit 1
fi

# Check if process is running
PID=$(cat "$BWS_PID_FILE")
if ! kill -0 "$PID" 2>/dev/null; then
    echo "ERROR: BWS process not running"
    exit 1
fi

# Check health endpoint
if ! curl -f -s "$BWS_HEALTH_URL" > /dev/null; then
    echo "ERROR: Health check failed"
    exit 1
fi

echo "OK: BWS is healthy"
exit 0
```

### Monitoring with Cron
```bash
# Add to crontab for user bws
*/5 * * * * /usr/local/bin/bws-health-check || /usr/bin/logger "BWS health check failed"
```

### Systemd Timer for Health Checks
```ini
# /etc/systemd/system/bws-health.service
[Unit]
Description=BWS Health Check
After=bws.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/bws-health-check
User=bws
```

```ini
# /etc/systemd/system/bws-health.timer
[Unit]
Description=BWS Health Check Timer
Requires=bws-health.service

[Timer]
OnCalendar=*:0/5
Persistent=true

[Install]
WantedBy=timers.target
```

## Auto-Recovery and Restart

### Automatic Restart Configuration
```ini
# Enhanced systemd service with restart logic
[Service]
Type=simple
Restart=always
RestartSec=5
StartLimitInterval=60
StartLimitBurst=3

# Restart conditions
RestartPreventExitStatus=1 2 3 4 6 SIGTERM
```

### Recovery Script
```bash
#!/bin/bash
# /usr/local/bin/bws-recovery
LOG_FILE="/var/log/bws/recovery.log"
PID_FILE="/var/run/bws.pid"

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE"
}

# Check if BWS is running
if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") 2>/dev/null; then
    if curl -f -s http://localhost:8080/health > /dev/null; then
        log_message "BWS is healthy"
        exit 0
    fi
fi

log_message "BWS appears to be down, attempting restart"

# Stop any existing processes
systemctl stop bws
sleep 5

# Clean up PID file if exists
[ -f "$PID_FILE" ] && rm -f "$PID_FILE"

# Start BWS
if systemctl start bws; then
    log_message "BWS restarted successfully"
    exit 0
else
    log_message "Failed to restart BWS"
    exit 1
fi
```

## Security Considerations

### Service Security
```ini
# Enhanced security in systemd service
[Service]
# Run as non-root user
User=bws
Group=bws

# Security restrictions
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectHome=true
ProtectSystem=strict
ReadWritePaths=/var/log/bws /var/lib/bws

# Capability restrictions
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

# Network restrictions
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX

# File system restrictions
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
```

### File Permissions
```bash
# Set secure permissions
chmod 600 /etc/bws/config.toml
chmod 755 /etc/bws
chmod 644 /usr/local/bin/bws
chmod 755 /usr/local/bin/bws

# Verify permissions
ls -la /etc/bws/
ls -la /var/log/bws/
ls -la /opt/bws/
```

## Integration Examples

### With Nginx
```nginx
# /etc/nginx/sites-available/bws
upstream bws {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://bws;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /health {
        proxy_pass http://bws/health;
        access_log off;
    }
}
```

### With Load Balancer
```yaml
# HAProxy configuration
global
    daemon
    maxconn 4096

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend web_frontend
    bind *:80
    default_backend bws_servers

backend bws_servers
    balance roundrobin
    option httpchk GET /health
    server bws1 127.0.0.1:8080 check
    server bws2 127.0.0.1:8081 check
```

## Troubleshooting

### Service Won't Start
```bash
# Check service status
systemctl status bws

# View detailed logs
journalctl -u bws -xe

# Check configuration
bws --config-check /etc/bws/config.toml

# Verify permissions
ls -la /etc/bws/config.toml
ls -la /usr/local/bin/bws
```

### Permission Errors
```bash
# Fix ownership
sudo chown -R bws:bws /opt/bws /var/log/bws

# Fix permissions
sudo chmod 755 /opt/bws
sudo chmod 644 /etc/bws/config.toml

# Check SELinux (if applicable)
sestatus
setsebool -P httpd_can_network_connect 1
```

### Performance Issues
```bash
# Check resource limits
systemctl show bws | grep Limit

# Monitor system resources
htop
iotop
netstat -tulpn
```

## Best Practices

### Configuration Management
- Store configurations in version control
- Use configuration templates for different environments
- Validate configurations before deployment
- Document all configuration changes

### Monitoring
- Set up comprehensive logging
- Monitor service health continuously
- Configure alerting for service failures
- Regular log analysis and cleanup

### Security
- Run with minimal privileges
- Regular security updates
- Secure file permissions
- Network security (firewall rules)

### Maintenance
- Regular backup of configurations
- Monitor disk space for logs
- Plan for service updates
- Document operational procedures

## Next Steps

- Configure [Production Environment](./production.md)
- Set up [Performance Monitoring](./performance.md)
- Learn about [Troubleshooting](./troubleshooting.md)
