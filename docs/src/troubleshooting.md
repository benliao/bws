# Troubleshooting

This guide helps you diagnose and resolve common issues with BWS installation, configuration, and operation.

## Common Issues

### Installation Problems

#### Rust Installation Issues

**Problem**: `cargo` command not found
```bash
cargo: command not found
```

**Solution**:
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Or add to shell profile
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Problem**: Outdated Rust version
```bash
error: package `bws v0.1.5` cannot be built because it requires rustc 1.89 or newer
```

**Solution**:
```bash
# Update Rust toolchain
rustup update stable
rustc --version  # Should show 1.89 or newer
```

#### Compilation Errors

**Problem**: Missing system dependencies
```bash
error: failed to run custom build command for `openssl-sys v0.9.xx`
```

**Solution**:
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y pkg-config libssl-dev build-essential

# CentOS/RHEL/Fedora
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkg-config openssl-devel

# macOS
brew install openssl pkg-config
```

**Problem**: Linker errors
```bash
error: linking with `cc` failed: exit status: 1
note: /usr/bin/ld: cannot find -lssl
```

**Solution**:
```bash
# Ubuntu/Debian
sudo apt install -y libssl-dev

# Set environment variables if needed
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig"
export OPENSSL_DIR="/usr"
```

### Configuration Issues

#### Invalid TOML Syntax

**Problem**: Configuration file parsing errors
```bash
Error: failed to parse config file
Caused by: invalid TOML syntax at line 5, column 10
```

**Solution**:
```bash
# Check TOML syntax
# Common issues:
# 1. Missing quotes around strings
name = example     # Wrong
name = "example"   # Correct

# 2. Incorrect array syntax
ports = [8080 8081]     # Wrong (missing comma)
ports = [8080, 8081]    # Correct

# 3. Misplaced sections
[[sites]]
name = "test"
[daemon]           # Wrong (should be before [[sites]])

# Validate TOML syntax online: https://www.toml-lint.com/
```

#### Missing Required Fields

**Problem**: Configuration validation errors
```bash
Error: missing required field `port` for site 'example'
```

**Solution**:
```toml
# Ensure all required fields are present
[[sites]]
name = "example"      # Required
hostname = "localhost" # Required
port = 8080           # Required
static_dir = "static" # Required
```

#### Port Binding Issues

**Problem**: Address already in use
```bash
Error: failed to bind to 127.0.0.1:8080
Caused by: Address already in use (os error 98)
```

**Solution**:
```bash
# Check what's using the port
lsof -i :8080
netstat -tulpn | grep :8080

# Kill the process using the port
kill -9 $(lsof -ti:8080)

# Or use a different port
[[sites]]
name = "example"
hostname = "localhost"
port = 8081  # Use different port
static_dir = "static"
```

#### File Permission Issues

**Problem**: Cannot access static files
```bash
Error: Permission denied (os error 13)
```

**Solution**:
```bash
# Check file permissions
ls -la static/

# Fix permissions
chmod 755 static/
chmod 644 static/*
find static/ -type d -exec chmod 755 {} \;
find static/ -type f -exec chmod 644 {} \;

# Check directory ownership
sudo chown -R $USER:$USER static/
```

### Runtime Issues

#### BWS Won't Start

**Problem**: Server fails to start silently
```bash
# No output, process exits immediately
```

**Solution**:
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/bws --config config.toml

# Check configuration
./target/release/bws --config config.toml --validate

# Check system resources
df -h        # Disk space
free -h      # Memory
ulimit -n    # File descriptors
```

#### High Memory Usage

**Problem**: BWS consuming excessive memory
```bash
# Process using > 1GB RAM for simple static serving
```

**Solution**:
```bash
# Monitor memory usage
ps aux | grep bws
top -p $(pgrep bws)

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full ./target/release/bws

# Optimize configuration
[performance]
max_connections = 1000      # Reduce if too high
worker_threads = 4          # Match CPU cores
connection_pool_size = 100  # Reduce pool size

[caching]
max_memory = "100MB"        # Limit cache size
```

#### High CPU Usage

**Problem**: BWS using 100% CPU
```bash
# CPU usage constantly high even with low traffic
```

**Solution**:
```bash
# Profile CPU usage
perf record -g ./target/release/bws --config config.toml
perf report

# Check configuration
[performance]
worker_threads = 4  # Don't exceed CPU cores
keep_alive_timeout = 30  # Reduce timeout

# Monitor system load
htop
iostat 1
```

#### Connection Issues

**Problem**: Cannot connect to BWS
```bash
curl: (7) Failed to connect to localhost port 8080: Connection refused
```

**Solution**:
```bash
# Check if BWS is running
ps aux | grep bws
systemctl status bws  # If using systemd

# Check port binding
netstat -tulpn | grep :8080
ss -tulpn | grep :8080

# Check firewall
sudo ufw status
sudo iptables -L

# Test locally first
curl -v http://127.0.0.1:8080/
```

### Hot Reload Issues

#### Hot Reload Not Working

**Problem**: Configuration changes not applied after SIGHUP
```bash
kill -HUP $(pgrep -f "bws.*master")
# No new worker spawned, configuration not updated
```

**Solution**:
```bash
# Check if master process exists
pgrep -f "bws.*master"

# Verify BWS is running in master-worker mode
ps aux | grep bws | grep -v grep

# Check logs for errors
tail -f /var/log/bws/bws.log | grep -E "(reload|error|worker|master)"

# Validate configuration before reload
bws --config-check /etc/bws/config.toml

# If master process not found, restart BWS
systemctl restart bws
```

#### Configuration Validation Failed

**Problem**: Invalid configuration preventing hot reload
```bash
tail -f /var/log/bws/bws.log
# ERROR: Configuration validation failed: ...
```

**Solution**:
```bash
# Test configuration syntax
bws --config-check /etc/bws/config.toml

# Common validation issues:
# - Invalid port numbers
# - Missing static directories
# - Invalid SSL certificate paths
# - Malformed TOML syntax

# Fix configuration and retry
vim /etc/bws/config.toml
bws --config-check /etc/bws/config.toml
systemctl reload bws
```

#### Worker Process Issues

**Problem**: New worker fails to start during reload
```bash
# Master spawns worker but worker exits immediately
```

**Solution**:
```bash
# Check worker process logs
journalctl -u bws -f | grep worker

# Common causes:
# 1. Port already in use by another process
netstat -tulpn | grep :8080

# 2. Permission issues
# Check file permissions for static directories
ls -la /var/www/html/

# 3. SSL certificate issues
# Verify certificate files exist and are readable
ls -la /etc/ssl/certs/example.com.crt
ls -la /etc/ssl/private/example.com.key

# 4. Resource limits
ulimit -n  # Check file descriptor limit
```

#### Master Process Not Responding

**Problem**: Master process exists but doesn't respond to signals
```bash
pgrep -f "bws.*master"  # Shows PID
kill -HUP <PID>         # No response
```

**Solution**:
```bash
# Check if process is stuck
ps aux | grep bws
top -p $(pgrep -f "bws.*master")

# Check system resources
free -h
df -h

# Force restart if unresponsive
systemctl stop bws
sleep 5
systemctl start bws

# Check process tree after restart
pstree -p $(pgrep -f "bws.*master")
```

#### Configuration Not Updating

**Problem**: Hot reload succeeds but changes not visible
```bash
# No errors in logs, new worker spawned, but config unchanged
```

**Solution**:
```bash
# Verify configuration file is correct
cat /etc/bws/config.toml

# Check if BWS is reading the right config file
ps aux | grep bws | grep -o -- '--config [^ ]*'

# Test specific changes
curl -I http://localhost:8080/ | grep "X-Custom-Header"

# Check if browser is caching
curl -H "Cache-Control: no-cache" http://localhost:8080/

# Verify worker process PID changed
# Before reload: note worker PID
ps aux | grep bws
# After reload: verify PID is different
```

### Performance Issues

#### Slow Response Times

**Problem**: High latency for static file serving
```bash
# Response times > 1 second for small files
```

**Solution**:
```bash
# Check disk I/O
iostat -x 1
iotop

# Optimize storage
# Use SSD for static files
# Enable file system caching
mount -o remount,noatime /path/to/static

# Tune BWS configuration
[performance]
read_buffer_size = "64KB"   # Increase buffer size
write_buffer_size = "64KB"
worker_threads = 8          # Increase workers

[caching]
enabled = true
max_memory = "1GB"          # Enable caching
```

#### Low Throughput

**Problem**: Cannot handle expected load
```bash
# Failing under moderate load (< 1000 req/s)
```

**Solution**:
```bash
# Increase system limits
# /etc/security/limits.conf
bws soft nofile 65536
bws hard nofile 65536

# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535

# Tune BWS
[performance]
max_connections = 10000
worker_threads = 16         # 2x CPU cores
keep_alive_timeout = 60

# Use load balancer for scaling
```

### Network Issues

#### Timeout Errors

**Problem**: Requests timing out
```bash
curl: (28) Operation timed out after 30000 milliseconds
```

**Solution**:
```bash
# Check network connectivity
ping hostname
traceroute hostname

# Increase timeouts
[performance]
request_timeout = 60      # Increase from 30
response_timeout = 60
keep_alive_timeout = 120

# Check for network congestion
iftop
nethogs
```

#### DNS Resolution Issues

**Problem**: Cannot resolve hostname
```bash
curl: (6) Could not resolve host: example.com
```

**Solution**:
```bash
# Test DNS resolution
nslookup example.com
dig example.com

# Check /etc/hosts
grep example.com /etc/hosts

# Use IP address instead
[[sites]]
name = "example"
hostname = "192.168.1.100"  # Use IP instead of hostname
port = 8080
static_dir = "static"
```

### SSL/TLS Issues

#### Certificate Problems

**Problem**: SSL certificate errors
```bash
Error: SSL certificate verification failed
```

**Solution**:
```bash
# Check certificate validity
openssl x509 -in cert.pem -text -noout
openssl verify -CAfile ca.pem cert.pem

# Check certificate permissions
ls -la /etc/ssl/certs/
chmod 644 /etc/ssl/certs/cert.pem
chmod 600 /etc/ssl/private/key.pem

# Verify certificate chain
openssl s_client -connect example.com:443 -showcerts
```

#### TLS Handshake Failures

**Problem**: TLS handshake errors
```bash
Error: TLS handshake failed
```

**Solution**:
```bash
# Check TLS configuration
[sites.ssl]
enabled = true
protocols = ["TLSv1.2", "TLSv1.3"]  # Ensure modern protocols
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"

# Test TLS connection
openssl s_client -connect localhost:8443 -tls1_2
```

### Docker Issues

#### Container Won't Start

**Problem**: Docker container exits immediately
```bash
docker run bws:latest
# Container exits with code 1
```

**Solution**:
```bash
# Check container logs
docker logs container_id

# Run interactively for debugging
docker run -it --entrypoint /bin/bash bws:latest

# Check file permissions in container
docker run --rm bws:latest ls -la /app/

# Mount configuration correctly
docker run -v $(pwd)/config.toml:/app/config.toml bws:latest
```

#### Volume Mount Issues

**Problem**: Cannot access mounted files
```bash
Error: No such file or directory: /app/static/index.html
```

**Solution**:
```bash
# Check volume mount syntax
docker run -v $(pwd)/static:/app/static bws:latest

# Verify host path exists
ls -la $(pwd)/static/

# Check file permissions
chmod -R 644 static/
chmod 755 static/

# Use absolute paths
docker run -v /full/path/to/static:/app/static bws:latest
```

## Diagnostic Commands

### System Information

```bash
#!/bin/bash
# diagnostic.sh - System diagnostic script

echo "=== BWS Diagnostic Information ==="
echo "Date: $(date)"
echo "Host: $(hostname)"
echo

echo "=== System Information ==="
uname -a
cat /etc/os-release 2>/dev/null || cat /etc/redhat-release 2>/dev/null
echo

echo "=== Resource Usage ==="
echo "CPU cores: $(nproc)"
echo "Memory:"
free -h
echo "Disk:"
df -h
echo "Load average:"
uptime
echo

echo "=== Network Configuration ==="
ip addr show
echo
netstat -tulpn | grep -E ":(8080|8081|8082|8083)"
echo

echo "=== BWS Process Information ==="
if pgrep -f bws > /dev/null; then
    echo "BWS is running:"
    ps aux | grep -v grep | grep bws
    echo
    echo "Open files:"
    lsof -p $(pgrep bws) | head -20
else
    echo "BWS is not running"
fi
echo

echo "=== Configuration Files ==="
find . -name "*.toml" -exec echo "File: {}" \; -exec head -20 {} \; -exec echo \;

echo "=== Log Files ==="
find /var/log -name "*bws*" 2>/dev/null | while read log; do
    echo "=== $log ==="
    tail -20 "$log" 2>/dev/null
    echo
done

echo "=== Recent System Logs ==="
journalctl -u bws --no-pager -n 20 2>/dev/null || echo "No systemd logs found"
```

### Network Diagnostics

```bash
#!/bin/bash
# network-test.sh - Network connectivity test

HOST=${1:-localhost}
PORT=${2:-8080}

echo "Testing connectivity to $HOST:$PORT"

# Test basic connectivity
echo "=== Ping Test ==="
ping -c 3 $HOST

echo "=== Port Test ==="
nc -zv $HOST $PORT 2>&1

echo "=== HTTP Test ==="
curl -v http://$HOST:$PORT/ 2>&1 | head -20

echo "=== DNS Resolution ==="
nslookup $HOST

echo "=== Route Trace ==="
traceroute $HOST 2>/dev/null | head -10
```

### Performance Monitoring

```bash
#!/bin/bash
# monitor.sh - Real-time BWS monitoring

BWS_PID=$(pgrep bws)

if [ -z "$BWS_PID" ]; then
    echo "BWS process not found"
    exit 1
fi

echo "Monitoring BWS process $BWS_PID"
echo "Press Ctrl+C to stop"

while true; do
    clear
    echo "=== BWS Monitoring - $(date) ==="
    echo
    
    # Process information
    echo "=== Process Information ==="
    ps -o pid,ppid,cmd,%mem,%cpu,time -p $BWS_PID
    echo
    
    # Memory usage
    echo "=== Memory Usage ==="
    cat /proc/$BWS_PID/status | grep -E "VmSize|VmRSS|VmData|VmStk"
    echo
    
    # Network connections
    echo "=== Network Connections ==="
    ss -tuln | grep ":8080\|:8081\|:8082\|:8083" | wc -l | xargs echo "Active connections:"
    echo
    
    # File descriptors
    echo "=== File Descriptors ==="
    ls /proc/$BWS_PID/fd/ | wc -l | xargs echo "Open file descriptors:"
    echo
    
    # System load
    echo "=== System Load ==="
    uptime
    
    sleep 5
done
```

## Log Analysis

### Error Log Patterns

```bash
# Common error patterns to look for

# Configuration errors
grep -i "config\|parse\|invalid" /var/log/bws/bws.log

# Network errors
grep -i "bind\|connection\|timeout" /var/log/bws/bws.log

# File system errors
grep -i "permission\|not found\|access" /var/log/bws/bws.log

# Performance issues
grep -i "slow\|timeout\|overload" /var/log/bws/bws.log

# Security issues
grep -i "attack\|malicious\|blocked" /var/log/bws/bws.log
```

### Log Analysis Script

```bash
#!/bin/bash
# analyze-logs.sh - BWS log analysis

LOG_FILE=${1:-/var/log/bws/bws.log}

if [ ! -f "$LOG_FILE" ]; then
    echo "Log file not found: $LOG_FILE"
    exit 1
fi

echo "Analyzing BWS logs: $LOG_FILE"
echo "Log file size: $(du -h $LOG_FILE | cut -f1)"
echo "Total lines: $(wc -l < $LOG_FILE)"
echo

echo "=== Error Summary ==="
grep -i error "$LOG_FILE" | wc -l | xargs echo "Total errors:"
grep -i warn "$LOG_FILE" | wc -l | xargs echo "Total warnings:"
echo

echo "=== Recent Errors ==="
grep -i error "$LOG_FILE" | tail -10
echo

echo "=== Top Error Messages ==="
grep -i error "$LOG_FILE" | sort | uniq -c | sort -rn | head -10
echo

echo "=== Request Statistics ==="
if grep -q "request" "$LOG_FILE"; then
    grep "request" "$LOG_FILE" | wc -l | xargs echo "Total requests:"
    
    # Status code distribution
    echo "Status codes:"
    grep -o '"status":[0-9]*' "$LOG_FILE" | cut -d: -f2 | sort | uniq -c | sort -rn
fi
```

## Recovery Procedures

### Service Recovery

```bash
#!/bin/bash
# recover-bws.sh - BWS service recovery

echo "Starting BWS recovery procedure..."

# Stop any running instances
echo "Stopping BWS..."
systemctl stop bws 2>/dev/null || pkill -f bws

# Clean up PID files
rm -f /var/run/bws.pid

# Check configuration
echo "Validating configuration..."
if ! bws --config /etc/bws/config.toml --validate; then
    echo "Configuration invalid, please fix and retry"
    exit 1
fi

# Check file permissions
echo "Checking file permissions..."
if [ ! -r /etc/bws/config.toml ]; then
    echo "Configuration file not readable"
    chmod 644 /etc/bws/config.toml
fi

# Check static directory
STATIC_DIR=$(grep static_dir /etc/bws/config.toml | head -1 | cut -d'"' -f2)
if [ ! -d "$STATIC_DIR" ]; then
    echo "Creating static directory: $STATIC_DIR"
    mkdir -p "$STATIC_DIR"
    echo "<h1>BWS Recovery Page</h1>" > "$STATIC_DIR/index.html"
fi

# Start service
echo "Starting BWS..."
if systemctl start bws; then
    echo "BWS started successfully"
    systemctl status bws
else
    echo "Failed to start BWS, check logs:"
    journalctl -u bws --no-pager -n 20
fi
```

### Database Recovery (if applicable)

```bash
#!/bin/bash
# recover-database.sh - Database recovery for BWS

echo "Database recovery procedure..."

# Check database connection
if ! pg_isready -h localhost -p 5432; then
    echo "Database not accessible"
    exit 1
fi

# Backup current database
pg_dump bws > /backup/bws-recovery-$(date +%Y%m%d).sql

# Run database maintenance
psql bws -c "VACUUM ANALYZE;"
psql bws -c "REINDEX DATABASE bws;"

echo "Database recovery completed"
```

## Prevention Strategies

### Monitoring Setup

```bash
# Set up basic monitoring
crontab -e

# Add monitoring jobs
*/5 * * * * /usr/local/bin/bws-health-check || /usr/bin/logger "BWS health check failed"
0 */6 * * * /usr/local/bin/cleanup-logs
0 2 * * * /usr/local/bin/backup-bws-config
```

### Automated Backups

```bash
#!/bin/bash
# backup-bws.sh - Automated BWS backup

BACKUP_DIR="/backup/bws"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup configuration
tar -czf "$BACKUP_DIR/config-$DATE.tar.gz" /etc/bws/

# Backup static files
tar -czf "$BACKUP_DIR/static-$DATE.tar.gz" /var/www/

# Backup logs
tar -czf "$BACKUP_DIR/logs-$DATE.tar.gz" /var/log/bws/

# Clean old backups (keep 30 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR"
```

### Health Monitoring

```bash
#!/bin/bash
# health-monitor.sh - Continuous health monitoring

ALERT_EMAIL="admin@example.com"
ALERT_WEBHOOK="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

check_health() {
    if ! curl -f -s http://localhost:8080/health > /dev/null; then
        return 1
    fi
    return 0
}

send_alert() {
    local message="$1"
    
    # Email alert
    echo "$message" | mail -s "BWS Alert" "$ALERT_EMAIL"
    
    # Slack alert
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"$message\"}" \
        "$ALERT_WEBHOOK"
}

# Main monitoring loop
while true; do
    if ! check_health; then
        send_alert "BWS health check failed at $(date)"
        
        # Attempt recovery
        systemctl restart bws
        sleep 30
        
        if check_health; then
            send_alert "BWS recovered successfully at $(date)"
        else
            send_alert "BWS recovery failed - manual intervention required"
        fi
    fi
    
    sleep 60
done
```

## Getting Help

### Information to Collect

When reporting issues, include:

1. **System Information**
   - Operating system and version
   - Rust version (`rustc --version`)
   - BWS version (`bws --version`)

2. **Configuration**
   - Configuration file content (sanitized)
   - Command line arguments used

3. **Error Information**
   - Complete error messages
   - Stack traces if available
   - Log file contents

4. **Environment**
   - Available memory and disk space
   - Network configuration
   - Other running services

### Support Channels

- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and get help
- **Documentation**: Check existing documentation
- **Community**: Join discussions with other users

### Before Reporting

1. Search existing issues
2. Try the latest version
3. Check documentation
4. Provide minimal reproduction case
5. Include diagnostic information

## Next Steps

After resolving issues:

1. Update your monitoring setup
2. Review [Performance Tuning](./performance.md) 
3. Check [Security Best Practices](./production.md#security)
4. Consider [High Availability Setup](./production.md#high-availability-setup)
