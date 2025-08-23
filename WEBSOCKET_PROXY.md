# WebSocket Proxy Support in BWS

BWS now supports proxying WebSocket connections to upstream servers with full load balancing capabilities.

## Quick Start

1. Configure your WebSocket proxy routes in your TOML configuration:

```toml
[[sites.proxy.routes]]
path = "/ws"
upstream = "websocket_backend"
strip_prefix = true
websocket = true  # Enable WebSocket proxying
```

2. Start your upstream WebSocket servers
3. Launch BWS with your configuration
4. Connect WebSocket clients to BWS - connections will be automatically load-balanced

## Features

✅ **Automatic Detection**: BWS automatically detects WebSocket upgrade requests
✅ **Load Balancing**: WebSocket connections use the same algorithms as HTTP (round-robin, weighted, least-connections)
✅ **Protocol Upgrade**: Automatic HTTP to WebSocket protocol conversion
✅ **URL Transformation**: `http://` → `ws://`, `https://` → `wss://`
✅ **Header Forwarding**: Proper forwarding of WebSocket-specific headers
✅ **Configuration Integration**: Uses existing proxy configuration structure

## Testing

Run the included test script to see WebSocket proxy in action:

```bash
./tests/test_websocket_proxy.sh
```

This will:
- Start multiple WebSocket test servers
- Configure BWS with WebSocket proxy routes  
- Provide a web interface for testing connections
- Demonstrate load balancing between upstream servers

## Configuration Example

```toml
[[sites]]
name = "websocket-app"
hostname = "ws.example.com"
port = 8080
static_dir = "./static"

[sites.proxy]
enabled = true

# WebSocket upstream servers
[[sites.proxy.upstreams]]
name = "chat_servers"
url = "http://localhost:3001"
weight = 1

[[sites.proxy.upstreams]]
name = "chat_servers"
url = "http://localhost:3002"
weight = 1

# WebSocket routes
[[sites.proxy.routes]]
path = "/ws/chat"
upstream = "chat_servers"
strip_prefix = true
websocket = true

# Load balancing
[sites.proxy.load_balancing]
method = "round_robin"
```

## Implementation Status

🚀 **Framework Complete**: WebSocket detection, routing, and upstream selection
⚠️ **Streaming Pending**: Full bidirectional message proxying requires additional Pingora integration
📈 **Performance Ready**: Uses efficient load balancing and connection tracking

The current implementation provides the complete framework for WebSocket proxying and will be enhanced with full streaming capabilities in future releases.

## Use Cases

- **Chat Applications**: Load balance WebSocket chat connections
- **Real-time APIs**: Distribute WebSocket API connections
- **Live Updates**: Proxy live data feeds and notifications
- **Gaming**: Balance WebSocket game server connections
- **Monitoring**: Distribute real-time monitoring connections

For more details, see the [reverse proxy documentation](docs/src/reverse-proxy.md).
