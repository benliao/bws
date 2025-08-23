# Reverse Proxy Configuration

BWS now supports comprehensive Caddy-style reverse proxy functionality on a per-site basis. Each site can be configured to proxy specific routes to upstream servers with flexible routing, load balancing, and header management.

## Configuration Structure

### Basic Proxy Configuration

```toml
[sites.proxy]
enabled = true

# Multiple backend servers (same name for load balancing)
[[sites.proxy.upstreams]]
name = "backend-pool"
url = "http://127.0.0.1:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "backend-pool"
url = "http://127.0.0.1:3002"
weight = 2

# Route configuration
[[sites.proxy.routes]]
path = "/api/"
upstream = "backend-pool"
strip_prefix = false

# Load balancing configuration
[sites.proxy.load_balancing]
method = "round_robin"  # Options: round_robin, weighted, least_connections

# Timeout configuration
[sites.proxy.timeout]
read = 30
write = 30

# Header management
[sites.proxy.headers]
add_x_forwarded = true
add_forwarded = true

[sites.proxy.headers.add]
"X-Powered-By" = "BWS"

[sites.proxy.headers.remove]
"X-Internal-Header" = true
```

## Load Balancing Algorithms

### 1. Round Robin (`round_robin`)
Distributes requests evenly across all available backend servers.

```toml
[sites.proxy.load_balancing]
method = "round_robin"
```

- **How it works**: Cycles through servers in order
- **Best for**: Servers with similar performance characteristics
- **Provides**: Fair distribution when all servers have equal capacity

### 2. Weighted (`weighted`)
Distributes requests based on assigned weights.

```toml
[sites.proxy.load_balancing]
method = "weighted"

# Configure weights per server
[[sites.proxy.upstreams]]
name = "backend-pool"
url = "http://127.0.0.1:3001"
weight = 3  # Gets 3x traffic

[[sites.proxy.upstreams]]
name = "backend-pool"
url = "http://127.0.0.1:3002"
weight = 1  # Gets 1x traffic
```

- **How it works**: Random selection weighted by server capacity
- **Best for**: Servers with different capacities or performance
- **Provides**: Performance-based load distribution

### 3. Least Connections (`least_connections`)
Routes requests to the server with the fewest active connections.

```toml
[sites.proxy.load_balancing]
method = "least_connections"
```

- **How it works**: Tracks active connections per server using atomic counters
- **Best for**: Long-running requests or varying response times
- **Provides**: Optimal connection distribution

## Features

### **Multiple Upstream Servers**
Group multiple backend servers under the same upstream name for automatic load balancing.

### **Flexible Routing**
- **Path-based routing**: Route requests based on URL paths
- **Prefix stripping**: Remove path prefixes before forwarding to upstream
- **Pattern matching**: Match specific paths and route accordingly

### **Thread-Safe Load Balancing**
- **Atomic Counters**: Thread-safe request distribution
- **Connection Tracking**: Real-time monitoring of active connections
- **Zero Locks**: High-performance concurrent access

### **Header Management**
- **Standard Proxy Headers**: X-Forwarded-For, X-Forwarded-Proto, X-Forwarded-Host
- **RFC 7239 Forwarded Header**: Modern standard forwarded header support
- **Custom Headers**: Add or remove custom headers per site
- **Header Preservation**: Configure which headers to preserve or modify

### **Request/Response Handling**
- **Full HTTP Support**: All HTTP methods (GET, POST, PUT, DELETE, etc.)
- **Body Forwarding**: Complete request and response body forwarding
- **Status Code Preservation**: Maintains original response status codes
- **Error Handling**: Graceful fallback with proper error responses

### **Timeout Configuration**
- **Read Timeout**: Configurable upstream read timeouts
- **Write Timeout**: Configurable upstream write timeouts
- **Per-Site Settings**: Individual timeout settings per site

## Example Configurations

### Basic API Proxy
```toml
[[sites]]
name = "api.example.com"
hostname = "api.example.com"
port = 80

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "api-backend"
url = "http://127.0.0.1:3000"

[[sites.proxy.routes]]
path = "/v1/"
upstream = "api-backend"
```

### Load Balanced Application
```toml
[[sites]]
name = "app.example.com"
hostname = "app.example.com"
port = 80

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "app-server-1"
url = "http://10.0.1.10:8080"
weight = 2

[[sites.proxy.upstreams]]
name = "app-server-2"
url = "http://10.0.1.11:8080"
weight = 1

[[sites.proxy.routes]]
path = "/"
upstream = "app-servers"

[sites.proxy.load_balancing]
method = "weighted"

[sites.proxy.health_check]
enabled = true
interval_seconds = 15
path = "/health"
```

### Mixed Static and Proxy
```toml
[[sites]]
name = "example.com"
hostname = "example.com"
port = 80
static_dir = "/var/www/example.com"

[sites.proxy]
enabled = true

[[sites.proxy.upstreams]]
name = "api"
url = "http://127.0.0.1:3000"

[[sites.proxy.upstreams]]
name = "admin"
url = "http://127.0.0.1:4000"

# Proxy API requests
[[sites.proxy.routes]]
path = "/api/"
upstream = "api"
strip_prefix = true

# Proxy admin interface
[[sites.proxy.routes]]
path = "/admin/"
upstream = "admin"
strip_prefix = false

# Static files are served for all other paths
```

## Route Priority

When a request comes in, BWS checks routes in this order:

1. **API routes** (`/api/health`, `/api/config`, etc.) - BWS internal APIs
2. **Proxy routes** - Configured reverse proxy routes (most specific path first)
3. **Static files** - Files from the site's static directory

## Implementation Status

✅ **Fully Implemented:**
- ✅ **Per-site proxy configuration** - Complete TOML configuration parsing
- ✅ **Route detection and matching** - Intelligent path-based routing
- ✅ **Full HTTP proxy implementation** - Complete request/response forwarding using reqwest
- ✅ **Three load balancing algorithms** - Round-robin, weighted, and least-connections
- ✅ **Connection tracking** - Atomic counters for least-connections algorithm
- ✅ **Path transformation** - Prefix stripping and URL rewriting
- ✅ **Header management** - X-Forwarded-*, Forwarded, and custom headers
- ✅ **Error handling** - Graceful fallback with 502 Bad Gateway responses
- ✅ **Response forwarding** - Complete status code, header, and body forwarding
- ✅ **Timeout configuration** - Configurable read/write timeouts
- ✅ **Thread safety** - Concurrent request handling with atomic operations
- ✅ **Mixed mode operation** - Static files and proxy routes work together

� **Future Enhancements:**
- Health check system with automatic failover
- Connection pooling optimization
- Circuit breaker pattern for failing upstreams
- WebSocket proxy support
- Request body streaming for large payloads
- Detailed metrics and monitoring

## Testing

The reverse proxy functionality is fully operational with comprehensive load balancing! Here's how to test it:

### Quick Test

```bash
# 1. Start multiple backend servers
python3 -m http.server 3001 &
python3 -m http.server 3002 &
python3 -m http.server 3003 &

# 2. Start BWS with load balancing configuration
cargo run -- --config tests/test_load_balancing.toml

# 3. Test load balancing
curl http://localhost:8080/api/test  # Round-robin distribution
curl http://localhost:8081/         # Weighted distribution
curl http://localhost:8082/         # Least connections distribution
```

### Comprehensive Load Balancing Test

Use the included test script to verify all load balancing algorithms:

```bash
./tests/test_load_balance.sh
```

This script will:
- Start mock backend servers
- Test round-robin distribution
- Test weighted distribution (60%/40%)
- Test least connections balancing
- Verify proper request distribution

### Manual Testing

```bash
# Test specific load balancing methods
curl -H "Host: roundrobin.example.com" http://localhost:8080/api/test
curl -H "Host: weighted.example.com" http://localhost:8081/
curl -H "Host: leastconn.example.com" http://localhost:8082/

# Test header forwarding
curl -v -H "Host: proxy.localhost" http://localhost:8080/api/data
# Look for X-Forwarded-For, X-Forwarded-Proto headers

# Test static file serving still works
curl http://localhost:8080/
# Returns: static HTML content from BWS
```

### Verified Features:

- ✅ **Load Balancing**: All three algorithms (round-robin, weighted, least-connections) working
- ✅ **Connection Tracking**: Least-connections properly tracks active connections
- ✅ **Route Detection**: Correctly identifies proxy vs static routes
- ✅ **Path Transformation**: Prefix stripping and rewriting work as configured
- ✅ **HTTP Proxying**: Full request/response proxying with all HTTP methods
- ✅ **Header Forwarding**: Request and response headers properly forwarded
- ✅ **Error Handling**: Returns 502 Bad Gateway when upstream is unavailable
- ✅ **Thread Safety**: Concurrent requests handled safely with atomic operations
- ✅ **Timeouts**: Configurable request timeouts prevent hanging
- ✅ **Mixed Mode**: Static files and proxy routes work together seamlessly

### Performance Characteristics

- **Round-Robin**: O(1) selection complexity
- **Weighted**: O(n) selection complexity where n = number of servers
- **Least Connections**: O(n) selection complexity with atomic counters
- **Thread Safety**: Lock-free operations using atomic primitives
- **Memory Efficiency**: Minimal overhead with efficient data structures

This reverse proxy implementation makes BWS a complete web server solution, capable of serving static content, providing APIs, and proxying requests to backend services with enterprise-grade load balancing - just like Caddy!
