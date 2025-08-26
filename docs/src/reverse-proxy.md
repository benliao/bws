# Reverse Proxy

BWS provides comprehensive reverse proxy functionality with load balancing and WebSocket support.

## Basic Configuration

```toml
[[sites]]
name = "proxy"
hostname = "api.example.com"
port = 80
static_dir = "static"

[sites.proxy]
enabled = true

# Backend servers
[[sites.proxy.upstreams]]
name = "backend"
url = "http://127.0.0.1:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "backend" 
url = "http://127.0.0.1:3002"
weight = 2

# Routes
[[sites.proxy.routes]]
path = "/api/"
upstream = "backend"
strip_prefix = false

# Load balancing
[sites.proxy.load_balancing]
method = "round_robin"
```

## Load Balancing Methods

### Round Robin
```toml
[sites.proxy.load_balancing]
method = "round_robin"    # Distribute requests evenly
```

### Weighted Round Robin
```toml
[[sites.proxy.upstreams]]
name = "backend"
url = "http://server1:3001"
weight = 3               # Gets 3x more requests

[[sites.proxy.upstreams]]
name = "backend"
url = "http://server2:3001"
weight = 1               # Gets 1x requests

[sites.proxy.load_balancing]
method = "weighted"
```

### Least Connections
```toml
[sites.proxy.load_balancing]
method = "least_connections"  # Route to server with fewest active connections
```

## Route Configuration

### Path-Based Routing
```toml
# API routes to backend
[[sites.proxy.routes]]
path = "/api/"
upstream = "api-backend"
strip_prefix = false     # Keep /api/ in forwarded path

# Admin routes to different backend
[[sites.proxy.routes]]
path = "/admin/"
upstream = "admin-backend"
strip_prefix = true      # Remove /admin/ from forwarded path
```

### WebSocket Proxying
```toml
[[sites.proxy.routes]]
path = "/ws"
upstream = "websocket-backend"
websocket = true         # Enable WebSocket support
strip_prefix = false

[[sites.proxy.upstreams]]
name = "websocket-backend"
url = "http://127.0.0.1:3001"  # Will proxy WebSocket connections
```

## Header Management

### Automatic Headers
```toml
[sites.proxy.headers]
add_x_forwarded = true   # Add X-Forwarded-For, X-Forwarded-Proto
add_forwarded = true     # Add standard Forwarded header
```

### Custom Headers
```toml
[sites.proxy.headers.add]
"X-API-Gateway" = "BWS"
"X-Request-ID" = "auto"

[sites.proxy.headers.remove]
"Server" = true
"X-Powered-By" = true
```

## Timeout Configuration

```toml
[sites.proxy.timeout]
read = 30               # Read timeout in seconds
write = 30              # Write timeout in seconds
connect = 5             # Connection timeout in seconds
```

## Complete Example

```toml
[server]
name = "BWS Proxy Server"

# Main site with mixed static/proxy
[[sites]]
name = "main"
hostname = "example.com"
port = 80
static_dir = "static"

[sites.proxy]
enabled = true

# API backend cluster
[[sites.proxy.upstreams]]
name = "api-cluster"
url = "http://api1.internal:3001"
weight = 2

[[sites.proxy.upstreams]]
name = "api-cluster"
url = "http://api2.internal:3001"
weight = 1

# WebSocket backend
[[sites.proxy.upstreams]]
name = "websocket-backend"
url = "http://ws.internal:3002"

# Route API requests
[[sites.proxy.routes]]
path = "/api/"
upstream = "api-cluster"
strip_prefix = false

# Route WebSocket connections
[[sites.proxy.routes]]
path = "/ws"
upstream = "websocket-backend"
websocket = true

# Load balancing
[sites.proxy.load_balancing]
method = "weighted"

# Timeouts
[sites.proxy.timeout]
read = 30
write = 30
connect = 5

# Headers
[sites.proxy.headers]
add_x_forwarded = true
add_forwarded = true

[sites.proxy.headers.add]
"X-Gateway" = "BWS"
```

## Testing Proxy Setup

### Start Backend Services
```bash
# Start test backends
python3 -m http.server 3001 &
python3 -m http.server 3002 &

# Or use Node.js
npx http-server -p 3001 &
npx http-server -p 3002 &
```

### Test Load Balancing
```bash
# Test multiple requests to see load balancing
for i in {1..10}; do
  curl -H "X-Request-ID: $i" http://localhost/api/test
done

# Check which backend handled each request
curl -v http://localhost/api/test
```

### WebSocket Testing
```bash
# Test WebSocket connection
npx wscat -c ws://localhost/ws

# Test through proxy
curl --upgrade websocket http://localhost/ws
```

## Monitoring

### Health Checks
```bash
# Check proxy status
curl http://localhost/api/health

# Monitor backend connectivity
curl -v http://localhost/api/test
```

### Log Analysis
```bash
# Monitor proxy logs
tail -f /var/log/bws.log | grep proxy

# Check upstream connections
netstat -an | grep :3001
```

## Production Considerations

### High Availability
```toml
# Multiple backend servers
[[sites.proxy.upstreams]]
name = "production-api"
url = "http://api1.prod:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "production-api"
url = "http://api2.prod:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "production-api"
url = "http://api3.prod:3001"
weight = 1

# Conservative timeouts
[sites.proxy.timeout]
read = 60
write = 60
connect = 10
```

### Security Headers
```toml
[sites.proxy.headers.add]
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
"X-XSS-Protection" = "1; mode=block"

[sites.proxy.headers.remove]
"Server" = true
"X-Powered-By" = true
```

### SSL Termination
```toml
# Terminate SSL at BWS, HTTP to backends
[[sites]]
name = "ssl-proxy"
hostname = "api.example.com"
port = 443
static_dir = "static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["api.example.com"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "backend"
url = "http://internal-api:3001"  # HTTP to internal backend
```
