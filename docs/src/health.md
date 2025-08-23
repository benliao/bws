# Health Monitoring

BWS provides built-in health monitoring endpoints and logging capabilities to help you monitor your server's status and performance.

## Health Endpoints

BWS automatically provides health check endpoints for monitoring:

### Basic Health Check
```
GET /health
```

Response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.1.5",
  "uptime": 3600
}
```

### Detailed Health Status
```
GET /health/detailed
```

Response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.1.5",
  "uptime": 3600,
  "sites": [
    {
      "name": "main",
      "hostname": "localhost",
      "port": 8080,
      "status": "active",
      "requests_served": 1542,
      "last_request": "2024-01-15T10:29:45Z"
    }
  ],
  "system": {
    "memory_usage": "45.2 MB",
    "cpu_usage": "12.5%",
    "disk_usage": "78.3%"
  }
}
```

## Monitoring Configuration

### Health Check Settings
```toml
[monitoring]
enabled = true
health_endpoint = "/health"
detailed_endpoint = "/health/detailed"
metrics_endpoint = "/metrics"
```

### Custom Health Checks
```toml
[monitoring.checks]
disk_threshold = 90    # Alert if disk usage > 90%
memory_threshold = 80  # Alert if memory usage > 80%
response_time_threshold = 1000  # Alert if response time > 1000ms
```

## Logging Configuration

### Basic Logging Setup
```toml
[logging]
level = "info"
format = "json"
output = "stdout"
```

### File Logging
```toml
[logging]
level = "info"
format = "json"
output = "file"
file_path = "/var/log/bws/bws.log"
max_size = "100MB"
max_files = 10
compress = true
```

### Structured Logging
```toml
[logging]
level = "debug"
format = "json"
include_fields = [
    "timestamp",
    "level",
    "message",
    "request_id",
    "client_ip",
    "user_agent",
    "response_time",
    "status_code"
]
```

## Log Levels

### Available Levels
- **ERROR**: Error conditions and failures
- **WARN**: Warning conditions
- **INFO**: General informational messages
- **DEBUG**: Detailed debugging information
- **TRACE**: Very detailed tracing information

### Log Level Examples
```toml
# Production
[logging]
level = "info"

# Development
[logging]
level = "debug"

# Troubleshooting
[logging]
level = "trace"
```

## Metrics Collection

### Basic Metrics
BWS automatically collects:
- Request count
- Response times
- Error rates
- Active connections
- Memory usage
- CPU usage

### Prometheus Integration
```toml
[monitoring.prometheus]
enabled = true
endpoint = "/metrics"
port = 9090
```

Example metrics output:
```
# HELP bws_requests_total Total number of requests
# TYPE bws_requests_total counter
bws_requests_total{site="main",method="GET",status="200"} 1542

# HELP bws_response_time_seconds Response time in seconds
# TYPE bws_response_time_seconds histogram
bws_response_time_seconds_bucket{site="main",le="0.1"} 1200
bws_response_time_seconds_bucket{site="main",le="0.5"} 1500
bws_response_time_seconds_bucket{site="main",le="1.0"} 1540
```

## Alerting

### Health Check Monitoring
```bash
#!/bin/bash
# health_check.sh
HEALTH_URL="http://localhost:8080/health"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $HEALTH_URL)

if [ $RESPONSE -eq 200 ]; then
    echo "BWS is healthy"
    exit 0
else
    echo "BWS health check failed: HTTP $RESPONSE"
    exit 1
fi
```

### Uptime Monitoring Script
```bash
#!/bin/bash
# uptime_check.sh
while true; do
    if curl -f -s http://localhost:8080/health > /dev/null; then
        echo "$(date): BWS is running"
    else
        echo "$(date): BWS is DOWN" | mail -s "BWS Alert" admin@example.com
    fi
    sleep 60
done
```

## System Integration

### Systemd Integration
```ini
# /etc/systemd/system/bws.service
[Unit]
Description=BWS Web Server
After=network.target

[Service]
Type=simple
User=bws
ExecStart=/usr/local/bin/bws --config /etc/bws/config.toml
Restart=always
RestartSec=5

# Health check
ExecHealthCheck=/usr/local/bin/health_check.sh
HealthCheckInterval=30s

[Install]
WantedBy=multi-user.target
```

### Docker Health Checks
```dockerfile
# Dockerfile
FROM rust:1.89-slim

# ... build steps ...

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080
CMD ["bws", "--config", "/app/config.toml"]
```

### Docker Compose Health Check
```yaml
# docker-compose.yml
version: '3.8'
services:
  bws:
    build: .
    ports:
      - "8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped
```

## Monitoring Tools Integration

### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "BWS Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(bws_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, bws_response_time_seconds)"
          }
        ]
      }
    ]
  }
}
```

### Nagios Check
```bash
#!/bin/bash
# check_bws.sh for Nagios
HOST="localhost"
PORT="8080"
WARNING_TIME=1000
CRITICAL_TIME=2000

RESPONSE_TIME=$(curl -o /dev/null -s -w "%{time_total}" http://$HOST:$PORT/health)
RESPONSE_TIME_MS=$(echo "$RESPONSE_TIME * 1000" | bc)

if (( $(echo "$RESPONSE_TIME_MS > $CRITICAL_TIME" | bc -l) )); then
    echo "CRITICAL - Response time: ${RESPONSE_TIME_MS}ms"
    exit 2
elif (( $(echo "$RESPONSE_TIME_MS > $WARNING_TIME" | bc -l) )); then
    echo "WARNING - Response time: ${RESPONSE_TIME_MS}ms"
    exit 1
else
    echo "OK - Response time: ${RESPONSE_TIME_MS}ms"
    exit 0
fi
```

## Log Analysis

### Log Parsing with jq
```bash
# Extract error logs
cat bws.log | jq 'select(.level == "ERROR")'

# Count requests by status code
cat bws.log | jq -r '.status_code' | sort | uniq -c

# Average response time
cat bws.log | jq -r '.response_time' | awk '{sum+=$1; count++} END {print sum/count}'
```

### Log Aggregation
```bash
# Tail logs in real-time
tail -f /var/log/bws/bws.log | jq '.'

# Filter by log level
tail -f /var/log/bws/bws.log | jq 'select(.level == "ERROR" or .level == "WARN")'

# Monitor specific endpoints
tail -f /var/log/bws/bws.log | jq 'select(.path == "/api/users")'
```

## Performance Monitoring

### Request Tracking
```toml
[monitoring.requests]
track_response_times = true
track_status_codes = true
track_user_agents = true
track_client_ips = true
sample_rate = 1.0  # 100% sampling
```

### Memory Monitoring
```toml
[monitoring.memory]
track_usage = true
alert_threshold = 80  # Alert at 80% usage
gc_metrics = true
```

### Connection Monitoring
```toml
[monitoring.connections]
track_active = true
track_total = true
max_connections = 1000
timeout = 30  # seconds
```

## Troubleshooting Health Issues

### Common Health Check Failures

#### Service Unavailable
```bash
# Check if BWS is running
ps aux | grep bws

# Check port binding
netstat -tulpn | grep :8080

# Check configuration
bws --config-check
```

#### High Response Times
```bash
# Check system load
top
htop

# Check disk usage
df -h

# Check memory usage
free -h
```

#### Memory Leaks
```bash
# Monitor memory over time
while true; do
    ps -o pid,ppid,cmd,%mem,%cpu -p $(pgrep bws)
    sleep 10
done

# Generate memory dump (if available)
kill -USR1 $(pgrep bws)
```

### Health Check Debugging
```bash
# Test health endpoint
curl -v http://localhost:8080/health

# Check detailed health
curl -s http://localhost:8080/health/detailed | jq '.'

# Monitor health over time
while true; do
    echo "$(date): $(curl -s http://localhost:8080/health | jq -r '.status')"
    sleep 30
done
```

## Best Practices

### Health Check Configuration
- Set appropriate timeouts (3-5 seconds)
- Use consistent intervals (30-60 seconds)
- Include multiple health indicators
- Monitor both application and system metrics

### Logging Strategy
- Use structured logging (JSON format)
- Include correlation IDs for request tracking
- Log at appropriate levels
- Rotate logs to prevent disk space issues

### Alerting Guidelines
- Set realistic thresholds
- Avoid alert fatigue
- Include actionable information
- Test alert mechanisms regularly

### Monitoring Coverage
- Monitor all critical endpoints
- Track business metrics, not just technical
- Set up both reactive and proactive monitoring
- Document monitoring procedures

## Next Steps

- Configure [Docker Deployment](./docker.md) with health checks
- Set up [Production Environment](./production.md) monitoring
- Learn about [Performance Tuning](./performance.md)
