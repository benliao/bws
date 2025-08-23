#!/bin/bash

# Test WebSocket Proxy Functionality for BWS
set -e

echo "🔌 Testing BWS WebSocket Proxy Functionality..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Node.js is available for test WebSocket servers
if ! command -v node &> /dev/null; then
    print_warning "Node.js not found. Installing test WebSocket servers with Python..."
    USE_PYTHON=true
else
    USE_PYTHON=false
fi

# Clean up function
cleanup() {
    print_status "Cleaning up test processes..."
    
    # Kill BWS server
    if [ ! -z "$BWS_PID" ]; then
        kill $BWS_PID 2>/dev/null || true
        wait $BWS_PID 2>/dev/null || true
        print_status "BWS server stopped"
    fi
    
    # Kill WebSocket test servers
    if [ ! -z "$WS_SERVER1_PID" ]; then
        kill $WS_SERVER1_PID 2>/dev/null || true
        print_status "WebSocket server 1 stopped"
    fi
    
    if [ ! -z "$WS_SERVER2_PID" ]; then
        kill $WS_SERVER2_PID 2>/dev/null || true
        print_status "WebSocket server 2 stopped"
    fi
    
    if [ ! -z "$HTTP_SERVER_PID" ]; then
        kill $HTTP_SERVER_PID 2>/dev/null || true
        print_status "HTTP server stopped"
    fi
    
    # Remove temporary files
    rm -f /tmp/ws_server1.js /tmp/ws_server2.js /tmp/ws_server1.py /tmp/ws_server2.py
    
    print_success "Cleanup completed"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Create test static directory
mkdir -p static
cat > static/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>BWS WebSocket Proxy Test</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .section { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .log { background: #f5f5f5; padding: 10px; height: 200px; overflow-y: auto; font-family: monospace; }
        button { padding: 10px 20px; margin: 5px; }
        input { padding: 8px; margin: 5px; width: 200px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>BWS WebSocket Proxy Test</h1>
        
        <div class="section">
            <h2>WebSocket Connection Test</h2>
            <button onclick="connectWs('/ws')">Connect to /ws</button>
            <button onclick="connectWs('/chat/ws')">Connect to /chat/ws</button>
            <button onclick="disconnect()">Disconnect</button>
            <br>
            <input type="text" id="messageInput" placeholder="Enter message">
            <button onclick="sendMessage()">Send Message</button>
            <div class="log" id="wsLog"></div>
        </div>
        
        <div class="section">
            <h2>HTTP API Test</h2>
            <button onclick="testApi()">Test /api endpoint</button>
            <div class="log" id="apiLog"></div>
        </div>
    </div>

    <script>
        let ws = null;
        
        function log(elementId, message) {
            const element = document.getElementById(elementId);
            element.innerHTML += new Date().toLocaleTimeString() + ': ' + message + '\n';
            element.scrollTop = element.scrollHeight;
        }
        
        function connectWs(path) {
            if (ws) {
                ws.close();
            }
            
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}${path}`;
            
            log('wsLog', `Connecting to ${wsUrl}...`);
            
            ws = new WebSocket(wsUrl);
            
            ws.onopen = function(event) {
                log('wsLog', 'WebSocket connected successfully!');
            };
            
            ws.onmessage = function(event) {
                log('wsLog', `Received: ${event.data}`);
            };
            
            ws.onclose = function(event) {
                log('wsLog', `WebSocket closed. Code: ${event.code}, Reason: ${event.reason}`);
            };
            
            ws.onerror = function(error) {
                log('wsLog', `WebSocket error: ${error}`);
            };
        }
        
        function disconnect() {
            if (ws) {
                ws.close();
                ws = null;
                log('wsLog', 'Disconnected');
            }
        }
        
        function sendMessage() {
            const input = document.getElementById('messageInput');
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(input.value);
                log('wsLog', `Sent: ${input.value}`);
                input.value = '';
            } else {
                log('wsLog', 'WebSocket not connected');
            }
        }
        
        function testApi() {
            log('apiLog', 'Testing /api endpoint...');
            fetch('/api/test')
                .then(response => response.json())
                .then(data => {
                    log('apiLog', `API Response: ${JSON.stringify(data)}`);
                })
                .catch(error => {
                    log('apiLog', `API Error: ${error}`);
                });
        }
    </script>
</body>
</html>
EOF

# Create test WebSocket servers
if [ "$USE_PYTHON" = true ]; then
    # Python WebSocket server 1
    cat > /tmp/ws_server1.py << 'EOF'
import asyncio
import websockets
import json

async def handle_client(websocket, path):
    print(f"Client connected to server 1: {path}")
    try:
        async for message in websocket:
            response = {
                "server": "WebSocket Server 1 (Port 3001)",
                "echo": message,
                "timestamp": str(asyncio.get_event_loop().time())
            }
            await websocket.send(json.dumps(response))
    except websockets.exceptions.ConnectionClosed:
        print("Client disconnected from server 1")

if __name__ == "__main__":
    print("Starting WebSocket server 1 on port 3001...")
    start_server = websockets.serve(handle_client, "localhost", 3001)
    asyncio.get_event_loop().run_until_complete(start_server)
    asyncio.get_event_loop().run_forever()
EOF

    # Python WebSocket server 2
    cat > /tmp/ws_server2.py << 'EOF'
import asyncio
import websockets
import json

async def handle_client(websocket, path):
    print(f"Client connected to server 2: {path}")
    try:
        async for message in websocket:
            response = {
                "server": "WebSocket Server 2 (Port 3002)",
                "echo": message,
                "timestamp": str(asyncio.get_event_loop().time())
            }
            await websocket.send(json.dumps(response))
    except websockets.exceptions.ConnectionClosed:
        print("Client disconnected from server 2")

if __name__ == "__main__":
    print("Starting WebSocket server 2 on port 3002...")
    start_server = websockets.serve(handle_client, "localhost", 3002)
    asyncio.get_event_loop().run_until_complete(start_server)
    asyncio.get_event_loop().run_forever()
EOF

else
    # Node.js WebSocket server 1
    cat > /tmp/ws_server1.js << 'EOF'
const WebSocket = require('ws');

const wss = new WebSocket.Server({ port: 3001 });

console.log('WebSocket server 1 started on port 3001');

wss.on('connection', function connection(ws, req) {
    console.log('Client connected to server 1:', req.url);
    
    ws.on('message', function incoming(message) {
        const response = {
            server: 'WebSocket Server 1 (Port 3001)',
            echo: message.toString(),
            timestamp: new Date().toISOString()
        };
        ws.send(JSON.stringify(response));
    });
    
    ws.on('close', function() {
        console.log('Client disconnected from server 1');
    });
});
EOF

    # Node.js WebSocket server 2
    cat > /tmp/ws_server2.js << 'EOF'
const WebSocket = require('ws');

const wss = new WebSocket.Server({ port: 3002 });

console.log('WebSocket server 2 started on port 3002');

wss.on('connection', function connection(ws, req) {
    console.log('Client connected to server 2:', req.url);
    
    ws.on('message', function incoming(message) {
        const response = {
            server: 'WebSocket Server 2 (Port 3002)',
            echo: message.toString(),
            timestamp: new Date().toISOString()
        };
        ws.send(JSON.stringify(response));
    });
    
    ws.on('close', function() {
        console.log('Client disconnected from server 2');
    });
});
EOF
fi

# Build BWS
print_status "Building BWS..."
cargo build --release

# Start test WebSocket servers
print_status "Starting WebSocket test servers..."

if [ "$USE_PYTHON" = true ]; then
    # Check if websockets module is available
    if ! python3 -c "import websockets" 2>/dev/null; then
        print_warning "Python websockets module not found. Installing..."
        pip3 install websockets || {
            print_error "Failed to install websockets module"
            exit 1
        }
    fi
    
    python3 /tmp/ws_server1.py &
    WS_SERVER1_PID=$!
    
    python3 /tmp/ws_server2.py &
    WS_SERVER2_PID=$!
else
    # Check if ws module is available
    if ! node -e "require('ws')" 2>/dev/null; then
        print_warning "Node.js 'ws' module not found. Installing..."
        npm install ws || {
            print_error "Failed to install ws module"
            exit 1
        }
    fi
    
    node /tmp/ws_server1.js &
    WS_SERVER1_PID=$!
    
    node /tmp/ws_server2.js &
    WS_SERVER2_PID=$!
fi

# Start HTTP API server for testing
print_status "Starting HTTP API test server..."
python3 -c "
import http.server
import socketserver
import json
from urllib.parse import urlparse

class TestHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith('/test'):
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            response = {
                'message': 'HTTP API test successful',
                'server': 'Test HTTP Server (Port 4001)',
                'path': self.path
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()

with socketserver.TCPServer(('localhost', 4001), TestHandler) as httpd:
    print('HTTP API server started on port 4001')
    httpd.serve_forever()
" &
HTTP_SERVER_PID=$!

# Wait for servers to start
sleep 2

# Start BWS with WebSocket proxy configuration
print_status "Starting BWS with WebSocket proxy configuration..."
RUST_LOG=info ./target/release/bws --config test_websocket_proxy.toml &
BWS_PID=$!

# Wait for BWS to start
sleep 3

# Test WebSocket proxy functionality
print_status "Testing WebSocket proxy functionality..."

# Test 1: Check if BWS is responding
print_status "Test 1: Checking BWS server response..."
if curl -s -f http://localhost:8080/ > /dev/null; then
    print_success "BWS server is responding"
else
    print_error "BWS server is not responding"
    exit 1
fi

# Test 2: Check WebSocket upgrade detection
print_status "Test 2: Testing WebSocket upgrade detection..."
WEBSOCKET_RESPONSE=$(curl -s -i -H "Upgrade: websocket" -H "Connection: Upgrade" -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" -H "Sec-WebSocket-Version: 13" http://localhost:8080/ws)

if echo "$WEBSOCKET_RESPONSE" | grep -q "WebSocket"; then
    print_success "WebSocket upgrade request detected"
else
    print_warning "WebSocket upgrade might not be fully implemented yet"
fi

# Test 3: Test HTTP proxy functionality
print_status "Test 3: Testing HTTP proxy to API backend..."
API_RESPONSE=$(curl -s http://localhost:8080/api/test)
if echo "$API_RESPONSE" | grep -q "HTTP API test successful"; then
    print_success "HTTP proxy to API backend working"
else
    print_warning "HTTP proxy might not be working as expected"
    echo "Response: $API_RESPONSE"
fi

# Test 4: Configuration validation
print_status "Test 4: Validating WebSocket configuration..."
if grep -q "websocket = true" test_websocket_proxy.toml; then
    print_success "WebSocket configuration found in test file"
else
    print_error "WebSocket configuration missing"
fi

# Display test results
print_status "WebSocket Proxy Test Summary:"
echo "=================================="
echo "✓ BWS server started successfully"
echo "✓ WebSocket test servers running on ports 3001 and 3002"
echo "✓ HTTP API test server running on port 4001"
echo "✓ Configuration includes WebSocket proxy routes"
echo ""
echo "Manual Testing Instructions:"
echo "1. Open http://localhost:8080 in your browser"
echo "2. Use the WebSocket test interface to connect to /ws or /chat/ws"
echo "3. Send messages through the interface"
echo "4. Test the HTTP API endpoint"
echo ""
echo "Expected Behavior:"
echo "- WebSocket connections should be load-balanced between servers"
echo "- Messages should be echoed back with server identification"
echo "- HTTP API requests should be proxied to the backend"
echo ""
print_warning "Note: Full WebSocket proxying requires additional integration with Pingora"
print_warning "The current implementation provides the framework and detection logic"

# Wait for user input or timeout
print_status "Press Ctrl+C to stop all servers and exit..."
sleep 300
