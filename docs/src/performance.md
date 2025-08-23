# Performance Tuning

This guide covers optimizing BWS for maximum performance, throughput, and efficiency in production environments.

## Performance Overview

BWS is built on Pingora, providing excellent performance characteristics:
- High-performance async I/O
- Minimal memory footprint
- Efficient connection handling
- Built-in caching capabilities
- Horizontal scaling support

## System-Level Optimization

### Operating System Tuning

#### Network Stack Optimization
```bash
# /etc/sysctl.d/99-bws-performance.conf

# TCP/IP stack tuning
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_max_tw_buckets = 1440000
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15

# Buffer sizes
net.core.rmem_default = 262144
net.core.rmem_max = 16777216
net.core.wmem_default = 262144
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# Connection tracking
net.netfilter.nf_conntrack_max = 1048576
net.netfilter.nf_conntrack_tcp_timeout_established = 300

# Apply settings
sysctl -p /etc/sysctl.d/99-bws-performance.conf
```

#### File Descriptor Limits
```bash
# /etc/security/limits.d/bws-performance.conf
bws soft nofile 1048576
bws hard nofile 1048576
bws soft nproc 32768
bws hard nproc 32768

# For systemd services
# /etc/systemd/system/bws.service.d/limits.conf
[Service]
LimitNOFILE=1048576
LimitNPROC=32768
```

#### CPU and Memory Optimization
```bash
# CPU governor for performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable swap for consistent performance
swapoff -a
# Comment out swap in /etc/fstab

# NUMA optimization (for multi-socket systems)
echo 0 > /proc/sys/kernel/numa_balancing
```

### Storage Optimization

#### File System Tuning
```bash
# Mount options for static files (add to /etc/fstab)
/dev/sdb1 /opt/bws/static ext4 defaults,noatime,nodiratime,data=writeback 0 0

# For high-performance scenarios
/dev/nvme0n1 /opt/bws/cache xfs defaults,noatime,largeio,inode64 0 0

# Temporary files in memory
tmpfs /tmp tmpfs defaults,size=2G,mode=1777 0 0
tmpfs /var/tmp tmpfs defaults,size=1G,mode=1777 0 0
```

#### I/O Scheduler Optimization
```bash
# For SSDs
echo noop > /sys/block/sda/queue/scheduler

# For HDDs
echo deadline > /sys/block/sda/queue/scheduler

# Make persistent (add to /etc/rc.local or systemd service)
echo 'noop' > /sys/block/sda/queue/scheduler
```

## BWS Configuration Optimization

### Performance-Focused Configuration
```toml
# /etc/bws/performance.toml
[performance]
# Worker thread configuration
worker_threads = 16  # 2x CPU cores for I/O bound workloads
max_blocking_threads = 512

# Connection settings
max_connections = 100000
keep_alive_timeout = 60
request_timeout = 30
response_timeout = 30

# Buffer sizes
read_buffer_size = "64KB"
write_buffer_size = "64KB"
max_request_size = "10MB"

# Connection pooling
connection_pool_size = 1000
connection_pool_idle_timeout = 300

[caching]
enabled = true
max_memory = "1GB"
ttl_default = 3600
ttl_static = 86400

[compression]
enabled = true
level = 6  # Balance between CPU and compression ratio
min_size = 1024  # Don't compress small files

[[sites]]
name = "high-performance"
hostname = "0.0.0.0"
port = 8080
static_dir = "/opt/bws/static"

# Optimized headers for caching
[sites.headers]
"Cache-Control" = "public, max-age=31536000, immutable"
"Vary" = "Accept-Encoding"
"X-Content-Type-Options" = "nosniff"
```

### Memory Management
```toml
[memory]
# Garbage collection tuning
gc_threshold = 0.8  # Trigger GC at 80% memory usage
max_heap_size = "4GB"

# Buffer pool settings
buffer_pool_size = "512MB"
buffer_pool_max_buffers = 10000

# Static file caching
file_cache_size = "2GB"
file_cache_max_files = 100000
```

### CPU Optimization
```toml
[cpu]
# Thread affinity (if supported)
pin_threads = true
cpu_affinity = [0, 1, 2, 3, 4, 5, 6, 7]  # Pin to specific cores

# Async runtime tuning
io_uring = true  # Use io_uring if available (Linux 5.1+)
event_loop_threads = 4
```

## Load Testing and Benchmarking

### Benchmarking Tools

#### wrk - HTTP Benchmarking
```bash
# Install wrk
sudo apt install wrk

# Basic load test
wrk -t12 -c400 -d30s --latency http://localhost:8080/

# Custom script for complex scenarios
cat > test-script.lua << 'EOF'
wrk.method = "GET"
wrk.headers["User-Agent"] = "wrk-benchmark"

request = function()
    local path = "/static/file" .. math.random(1, 1000) .. ".jpg"
    return wrk.format(nil, path)
end
EOF

wrk -t12 -c400 -d30s -s test-script.lua http://localhost:8080/
```

#### Apache Bench (ab)
```bash
# Simple test
ab -n 10000 -c 100 http://localhost:8080/

# With keep-alive
ab -n 10000 -c 100 -k http://localhost:8080/

# POST requests
ab -n 1000 -c 10 -p post_data.json -T application/json http://localhost:8080/api/test
```

#### hey - Modern Load Testing
```bash
# Install hey
go install github.com/rakyll/hey@latest

# Basic test
hey -n 10000 -c 100 http://localhost:8080/

# With custom headers
hey -n 10000 -c 100 -H "Accept: application/json" http://localhost:8080/api/health

# Rate limited test
hey -n 10000 -q 100 http://localhost:8080/
```

### Performance Testing Strategy

#### Baseline Testing
```bash
#!/bin/bash
# performance-baseline.sh

SERVER_URL="http://localhost:8080"
RESULTS_DIR="/tmp/performance-results"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$RESULTS_DIR"

echo "Running BWS performance baseline tests - $DATE"

# Test 1: Static file serving
echo "Testing static file serving..."
wrk -t4 -c50 -d60s --latency "$SERVER_URL/static/test.html" > "$RESULTS_DIR/static-$DATE.log"

# Test 2: API endpoints
echo "Testing API endpoints..."
wrk -t4 -c50 -d60s --latency "$SERVER_URL/health" > "$RESULTS_DIR/api-$DATE.log"

# Test 3: High concurrency
echo "Testing high concurrency..."
wrk -t12 -c1000 -d60s --latency "$SERVER_URL/" > "$RESULTS_DIR/concurrency-$DATE.log"

# Test 4: Sustained load
echo "Testing sustained load..."
wrk -t8 -c200 -d300s --latency "$SERVER_URL/" > "$RESULTS_DIR/sustained-$DATE.log"

echo "Performance tests completed. Results in $RESULTS_DIR"
```

#### Stress Testing
```bash
#!/bin/bash
# stress-test.sh

# Gradually increase load
for connections in 100 500 1000 2000 5000; do
    echo "Testing with $connections connections..."
    wrk -t12 -c$connections -d30s --latency http://localhost:8080/ > "stress-$connections.log"
    
    # Check if server is still responsive
    if ! curl -f -s http://localhost:8080/health > /dev/null; then
        echo "Server failed at $connections connections"
        break
    fi
    
    # Cool down period
    sleep 10
done
```

### Performance Monitoring

#### Real-time Monitoring Script
```bash
#!/bin/bash
# monitor-performance.sh

LOG_FILE="/var/log/bws/performance.log"
INTERVAL=5

log_metrics() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    local memory_usage=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
    local connections=$(ss -tuln | grep :8080 | wc -l)
    local load_avg=$(uptime | awk -F'load average:' '{print $2}' | cut -d',' -f1 | xargs)
    
    echo "$timestamp,CPU:$cpu_usage%,Memory:$memory_usage%,Connections:$connections,Load:$load_avg" >> "$LOG_FILE"
}

echo "Starting performance monitoring (interval: ${INTERVAL}s)"
echo "Timestamp,CPU,Memory,Connections,LoadAvg" > "$LOG_FILE"

while true; do
    log_metrics
    sleep $INTERVAL
done
```

#### Performance Dashboard Script
```bash
#!/bin/bash
# performance-dashboard.sh

# Real-time performance dashboard
watch -n 1 '
echo "=== BWS Performance Dashboard ==="
echo "Time: $(date)"
echo ""
echo "=== System Resources ==="
echo "CPU Usage: $(top -bn1 | grep "Cpu(s)" | awk "{print \$2}")"
echo "Memory: $(free -h | grep Mem | awk "{printf \"Used: %s/%s (%.1f%%)\", \$3, \$2, \$3/\$2*100}")"
echo "Load Average: $(uptime | awk -F"load average:" "{print \$2}")"
echo ""
echo "=== Network ==="
echo "Active Connections: $(ss -tuln | grep :8080 | wc -l)"
echo "TCP Connections: $(ss -s | grep TCP | head -1)"
echo ""
echo "=== BWS Process ==="
BWS_PID=$(pgrep bws)
if [ ! -z "$BWS_PID" ]; then
    echo "Process ID: $BWS_PID"
    echo "Memory Usage: $(ps -o pid,ppid,cmd,%mem,%cpu -p $BWS_PID | tail -1)"
    echo "File Descriptors: $(lsof -p $BWS_PID | wc -l)"
else
    echo "BWS process not found"
fi
'
```

## Optimization Strategies

### Static File Optimization

#### File Compression
```bash
#!/bin/bash
# precompress-static-files.sh

STATIC_DIR="/opt/bws/static"

# Pre-compress static files
find "$STATIC_DIR" -type f \( -name "*.css" -o -name "*.js" -o -name "*.html" -o -name "*.svg" \) | while read file; do
    # Gzip compression
    if [ ! -f "$file.gz" ] || [ "$file" -nt "$file.gz" ]; then
        gzip -k -9 "$file"
        echo "Compressed: $file"
    fi
    
    # Brotli compression (if available)
    if command -v brotli > /dev/null; then
        if [ ! -f "$file.br" ] || [ "$file" -nt "$file.br" ]; then
            brotli -k -q 11 "$file"
            echo "Brotli compressed: $file"
        fi
    fi
done

# Optimize images
find "$STATIC_DIR" -name "*.jpg" -o -name "*.jpeg" | while read file; do
    if command -v jpegoptim > /dev/null; then
        jpegoptim --strip-all --max=85 "$file"
    fi
done

find "$STATIC_DIR" -name "*.png" | while read file; do
    if command -v optipng > /dev/null; then
        optipng -o2 "$file"
    fi
done
```

#### CDN Integration
```toml
# Configure BWS for CDN usage
[[sites]]
name = "cdn-optimized"
hostname = "localhost"
port = 8080
static_dir = "/opt/bws/static"

[sites.headers]
"Cache-Control" = "public, max-age=31536000, immutable"
"Access-Control-Allow-Origin" = "*"
"Vary" = "Accept-Encoding"
"X-CDN-Cache" = "MISS"

# Separate configuration for CDN edge
[[sites]]
name = "edge"
hostname = "edge.example.com"
port = 8081
static_dir = "/opt/bws/edge-cache"

[sites.headers]
"Cache-Control" = "public, max-age=604800"  # 1 week for edge cache
"X-CDN-Cache" = "HIT"
```

### Database and Cache Optimization

#### Redis Integration (if using caching)
```bash
# Redis performance tuning
# /etc/redis/redis.conf

# Memory settings
maxmemory 2gb
maxmemory-policy allkeys-lru

# Persistence settings (adjust based on needs)
save 900 1
save 300 10
save 60 10000

# Network settings
tcp-keepalive 60
timeout 0

# Performance settings
tcp-backlog 511
databases 16
```

#### Memcached Configuration
```bash
# /etc/memcached.conf
-m 1024  # 1GB memory
-c 1024  # Max connections
-t 4     # Number of threads
-l 127.0.0.1  # Listen address
-p 11211 # Port
```

### Reverse Proxy Optimization

#### Nginx Performance Configuration
```nginx
# /etc/nginx/nginx.conf
user nginx;
worker_processes auto;
worker_rlimit_nofile 100000;

events {
    worker_connections 4000;
    use epoll;
    multi_accept on;
}

http {
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 30;
    keepalive_requests 100000;
    reset_timedout_connection on;
    client_body_timeout 10;
    send_timeout 2;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml
        image/svg+xml;
    
    # Buffer sizes
    client_max_body_size 10m;
    client_body_buffer_size 128k;
    client_header_buffer_size 1k;
    large_client_header_buffers 4 4k;
    output_buffers 1 32k;
    postpone_output 1460;
    
    # Proxy settings
    proxy_buffering on;
    proxy_buffer_size 128k;
    proxy_buffers 4 256k;
    proxy_busy_buffers_size 256k;
    
    upstream bws_backend {
        least_conn;
        server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
        server 127.0.0.1:8081 max_fails=3 fail_timeout=30s;
        keepalive 300;
    }
    
    server {
        listen 80 default_server;
        
        location / {
            proxy_pass http://bws_backend;
            proxy_http_version 1.1;
            proxy_set_header Connection "";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            
            # Caching
            proxy_cache_valid 200 1h;
            proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
        }
    }
}
```

## Performance Monitoring and Analysis

### Key Performance Metrics

#### Response Time Monitoring
```bash
#!/bin/bash
# response-time-monitor.sh

URL="http://localhost:8080"
LOG_FILE="/var/log/bws/response-times.log"

while true; do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    RESPONSE_TIME=$(curl -o /dev/null -s -w "%{time_total}" "$URL/health")
    RESPONSE_CODE=$(curl -o /dev/null -s -w "%{http_code}" "$URL/health")
    
    echo "$TIMESTAMP,$RESPONSE_TIME,$RESPONSE_CODE" >> "$LOG_FILE"
    sleep 1
done
```

#### Throughput Measurement
```bash
#!/bin/bash
# throughput-monitor.sh

# Monitor requests per second
LOG_FILE="/var/log/bws/throughput.log"
ACCESS_LOG="/var/log/nginx/access.log"

while true; do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    RPS=$(tail -60 "$ACCESS_LOG" | grep "$(date '+%d/%b/%Y:%H:%M')" | wc -l)
    
    echo "$TIMESTAMP,$RPS" >> "$LOG_FILE"
    sleep 60
done
```

### Performance Analysis Tools

#### Log Analysis
```bash
#!/bin/bash
# analyze-performance.sh

LOG_FILE="/var/log/nginx/access.log"
OUTPUT_DIR="/tmp/performance-analysis"
DATE=$(date +%Y%m%d)

mkdir -p "$OUTPUT_DIR"

echo "Analyzing performance for $DATE"

# Top requested URLs
echo "=== Top Requested URLs ===" > "$OUTPUT_DIR/top-urls-$DATE.txt"
awk '{print $7}' "$LOG_FILE" | sort | uniq -c | sort -rn | head -20 >> "$OUTPUT_DIR/top-urls-$DATE.txt"

# Response codes distribution
echo "=== Response Codes ===" > "$OUTPUT_DIR/response-codes-$DATE.txt"
awk '{print $9}' "$LOG_FILE" | sort | uniq -c | sort -rn >> "$OUTPUT_DIR/response-codes-$DATE.txt"

# Response time analysis (if logged)
echo "=== Response Time Analysis ===" > "$OUTPUT_DIR/response-times-$DATE.txt"
awk '{print $NF}' "$LOG_FILE" | grep -E '^[0-9]+\.[0-9]+$' | \
awk '{
    sum += $1
    count++
    if ($1 > max) max = $1
    if (min == 0 || $1 < min) min = $1
}
END {
    print "Average:", sum/count
    print "Min:", min
    print "Max:", max
}' >> "$OUTPUT_DIR/response-times-$DATE.txt"

# Hourly request distribution
echo "=== Hourly Request Distribution ===" > "$OUTPUT_DIR/hourly-requests-$DATE.txt"
awk '{print substr($4, 14, 2)}' "$LOG_FILE" | sort | uniq -c >> "$OUTPUT_DIR/hourly-requests-$DATE.txt"

echo "Analysis complete. Results in $OUTPUT_DIR"
```

#### Resource Usage Trends
```bash
#!/bin/bash
# resource-trends.sh

DURATION=${1:-3600}  # Default 1 hour
INTERVAL=10
SAMPLES=$((DURATION / INTERVAL))

echo "Collecting resource usage data for $DURATION seconds..."
echo "timestamp,cpu_percent,memory_mb,connections,load_avg" > resource-usage.csv

for i in $(seq 1 $SAMPLES); do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    CPU_PERCENT=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    MEMORY_MB=$(ps -o pid,rss -p $(pgrep bws) | tail -1 | awk '{print $2/1024}')
    CONNECTIONS=$(ss -tuln | grep :8080 | wc -l)
    LOAD_AVG=$(uptime | awk -F'load average:' '{print $2}' | cut -d',' -f1 | xargs)
    
    echo "$TIMESTAMP,$CPU_PERCENT,$MEMORY_MB,$CONNECTIONS,$LOAD_AVG" >> resource-usage.csv
    
    sleep $INTERVAL
done

echo "Resource usage data collected in resource-usage.csv"
```

## Optimization Recommendations

### Hardware Recommendations

#### CPU Optimization
- Use modern CPUs with high single-thread performance
- Consider NUMA topology for multi-socket systems
- Enable CPU turbo boost for peak performance
- Pin BWS processes to specific CPU cores for consistency

#### Memory Optimization
- Use ECC RAM for data integrity
- Ensure sufficient RAM for file caching
- Consider NUMA-aware memory allocation
- Monitor for memory leaks and fragmentation

#### Storage Optimization
- Use NVMe SSDs for static file storage
- Implement RAID for redundancy without performance penalty
- Consider separate storage for logs and static files
- Use tmpfs for temporary files and cache

#### Network Optimization
- Use 10Gbps+ network interfaces for high-traffic scenarios
- Implement network bonding for redundancy
- Optimize network driver settings
- Consider SR-IOV for virtualized environments

### Application-Level Optimization

#### Code Optimization
```rust
// Example optimizations in BWS configuration
use pingora::prelude::*;
use tokio::runtime::Builder;

// Custom runtime configuration
let runtime = Builder::new_multi_thread()
    .worker_threads(num_cpus::get() * 2)
    .max_blocking_threads(512)
    .thread_keep_alive(Duration::from_secs(60))
    .enable_all()
    .build()
    .unwrap();

// Connection pooling optimization
let pool_config = ConnectionPoolConfig {
    max_connections_per_host: 100,
    idle_timeout: Duration::from_secs(300),
    connect_timeout: Duration::from_secs(10),
};
```

#### Memory Pool Configuration
```toml
[memory_pool]
small_buffer_size = "4KB"
medium_buffer_size = "64KB"
large_buffer_size = "1MB"
pool_size = 1000
```

### Scaling Strategies

#### Horizontal Scaling
```yaml
# Kubernetes deployment example
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bws-deployment
spec:
  replicas: 6
  selector:
    matchLabels:
      app: bws
  template:
    metadata:
      labels:
        app: bws
    spec:
      containers:
      - name: bws
        image: ghcr.io/yourusername/bws:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: bws-service
spec:
  selector:
    app: bws
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

#### Vertical Scaling
```bash
# Automatic resource scaling script
#!/bin/bash
# auto-scale.sh

CPU_THRESHOLD=80
MEMORY_THRESHOLD=80
CHECK_INTERVAL=60

while true; do
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    MEMORY_USAGE=$(free | grep Mem | awk '{printf "%.0f", $3/$2 * 100.0}')
    
    if (( $(echo "$CPU_USAGE > $CPU_THRESHOLD" | bc -l) )); then
        echo "High CPU usage detected: $CPU_USAGE%"
        # Scale up logic here
    fi
    
    if (( MEMORY_USAGE > MEMORY_THRESHOLD )); then
        echo "High memory usage detected: $MEMORY_USAGE%"
        # Scale up logic here
    fi
    
    sleep $CHECK_INTERVAL
done
```

## Performance Best Practices

### Configuration Best Practices
1. **Right-size worker threads**: 2x CPU cores for I/O-bound workloads
2. **Optimize buffer sizes**: Match your typical request/response sizes
3. **Enable compression**: For text-based content
4. **Use connection pooling**: Reuse connections for better performance
5. **Configure appropriate timeouts**: Balance responsiveness and resource usage

### Monitoring Best Practices
1. **Monitor key metrics**: Response time, throughput, error rate, resource usage
2. **Set up alerting**: For performance degradation
3. **Regular performance testing**: Catch regressions early
4. **Capacity planning**: Monitor trends and plan for growth
5. **Document baselines**: Know your normal performance characteristics

### Deployment Best Practices
1. **Load testing**: Test under realistic load before production
2. **Gradual rollouts**: Use blue-green or canary deployments
3. **Performance regression testing**: Automated checks for performance changes
4. **Resource monitoring**: Continuous monitoring of system resources
5. **Regular optimization**: Periodic performance reviews and tuning

## Next Steps

- Review [Configuration Schema](./config-schema.md) for advanced tuning options
- Check [Troubleshooting](./troubleshooting.md) for performance issues
- Learn about [Production Setup](./production.md) for deployment strategies
