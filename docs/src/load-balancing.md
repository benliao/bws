# Load Balancing in BWS

BWS reverse proxy includes comprehensive load balancing functionality similar to Caddy, with three different algorithms available.

## Load Balancing Algorithms

### 1. Round Robin (`round_robin`)
Distributes requests evenly across all available backend servers in a circular manner.

**How it works:**
- Maintains a counter for each upstream group
- Increments counter for each request
- Selects server based on `counter % number_of_servers`
- Provides fair distribution when all servers have equal capacity

**Best for:**
- Servers with similar performance characteristics
- Simple, predictable load distribution
- Development and testing environments

### 2. Weighted (`weighted`)
Distributes requests based on assigned weights, allowing some servers to handle more traffic.

**How it works:**
- Each server has a weight value (default: 1)
- Random selection weighted by server capacity
- Higher weight = more requests
- Uses fast random number generation for selection

**Best for:**
- Servers with different capacities
- Gradual traffic migration
- Performance-based load distribution

### 3. Least Connections (`least_connections`)
Routes requests to the server with the fewest active connections.

**How it works:**
- Tracks active connections per server using atomic counters
- Increments counter when request starts
- Decrements counter when request completes
- Always routes to server with minimum connections

**Best for:**
- Long-running requests
- Servers with varying response times
- Optimal connection distribution

## Configuration

### Basic Setup
```toml
# Site configuration
[[sites]]
name = "example.com"
hostname = "localhost"
port = 8080

[sites.proxy]
enabled = true

# Multiple servers with same upstream name
[[sites.proxy.upstreams]]
name = "backend-servers"
url = "http://127.0.0.1:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "backend-servers"
url = "http://127.0.0.1:3002"
weight = 2  # This server gets 2x traffic in weighted mode

[[sites.proxy.upstreams]]
name = "backend-servers"
url = "http://127.0.0.1:3003"
weight = 1

# Route configuration
[[sites.proxy.routes]]
path = "/api/"
upstream = "backend-servers"

# Load balancing method
[sites.proxy.load_balancing]
method = "round_robin"  # or "weighted" or "least_connections"
```

### Advanced Configuration
```toml
[sites.proxy]
enabled = true

# Request timeout
timeout = { read = 30, write = 30 }

# Header management
[sites.proxy.headers]
add_x_forwarded = true
add_forwarded = true

[sites.proxy.headers.add]
"X-Custom-Header" = "BWS-Proxy"

[sites.proxy.headers.remove]
"X-Internal-Header" = true
```

## Testing Load Balancing

1. **Set up backend servers** on different ports (3001, 3002, 3003)
2. **Configure BWS** with the load balancing configuration
3. **Make multiple requests** to see distribution
4. **Monitor logs** to verify load balancing behavior

Use the included test script:
```bash
./test_load_balance.sh
```

## Implementation Details

### Thread Safety
- Round-robin counters use `AtomicUsize` for thread-safe access
- Connection tracking uses atomic operations
- No locks needed for load balancing decisions

### Performance
- O(1) complexity for round-robin selection
- O(n) complexity for weighted selection (where n = number of servers)
- O(n) complexity for least connections (where n = number of servers)
- Minimal overhead with atomic operations

### Reliability
- Automatic failover if upstream server is unavailable
- Connection tracking prevents memory leaks
- Graceful handling of server addition/removal

## Monitoring

BWS provides detailed logging for load balancing decisions:

```
INFO: Proxying request /api/users to upstream 'backend-servers'
INFO: Selected server http://127.0.0.1:3002 using round_robin
INFO: Successfully proxied request /api/users to http://127.0.0.1:3002
```

Connection counts and load balancing decisions are logged at debug level for troubleshooting.

## Comparison with Caddy

BWS load balancing is designed to be compatible with Caddy's approach:

| Feature | BWS | Caddy |
|---------|-----|-------|
| Round Robin | ✅ | ✅ |
| Weighted | ✅ | ✅ |
| Least Connections | ✅ | ✅ |
| Health Checks | 🔄 Planned | ✅ |
| Sticky Sessions | 🔄 Planned | ✅ |
| Circuit Breaker | 🔄 Planned | ✅ |

## Future Enhancements

- **Health Checks**: Automatic detection of unhealthy servers
- **Sticky Sessions**: Route requests from same client to same server
- **Circuit Breaker**: Temporary removal of failing servers
- **Metrics**: Detailed statistics on load balancing performance
- **Dynamic Configuration**: Runtime modification of upstream servers
