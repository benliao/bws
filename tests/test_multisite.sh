#!/bin/bash

echo "🚀 Testing BWS Multi-Site Configurable Server"
echo "=============================================="
echo

# Check if server is running
if ! pgrep -f "target/debug/main" > /dev/null; then
    echo "Starting BWS server in background..."
    RUST_LOG=info cargo run --manifest-path ../Cargo.toml &
    SERVER_PID=$!
    sleep 5
    echo "Server started with PID $SERVER_PID"
    echo
else
    echo "Server is already running"
    echo
fi

echo "Testing all configured sites:"
echo

echo "1. Main Site (localhost:8080) - Original BWS website:"
curl -s http://localhost:8080 | grep -E "(title|BWS)" | head -2
echo

echo "2. Blog Site (localhost:8081) - Blog content:"
curl -s http://localhost:8081 | grep -E "(title|Blog)" | head -2
echo

echo "3. API Documentation Site (localhost:8082) - API docs:"
curl -s http://localhost:8082 | grep -E "(title|API)" | head -2
echo

echo "4. Development Site (localhost:8083) - Dev environment:"
curl -s http://localhost:8083 | grep -E "(title|Development)" | head -2
echo

echo "5. Sites Configuration API:"
curl -s http://localhost:8080/api/sites | jq -r '.sites[] | "  - \(.name): \(.hostname):\(.port) -> \(.static_dir)"'
echo

echo "6. Health Check (works on all ports):"
echo "  Main site health:"
curl -s http://localhost:8080/api/health | jq -r '"  Status: " + .status + " (Service: " + .service + ")"'
echo "  Blog site health:"
curl -s http://localhost:8081/api/health | jq -r '"  Status: " + .status + " (Service: " + .service + ")"'
echo

echo "7. Virtual Host Testing (using Host header):"
echo "  Testing blog.localhost:"
curl -s -H "Host: blog.localhost:8081" http://localhost:8081 | grep -E "(title|Blog)" | head -1
echo "  Testing api.localhost:"
curl -s -H "Host: api.localhost:8082" http://localhost:8082 | grep -E "(title|API)" | head -1
echo

echo "8. Port-based routing test:"
echo "  Each port serves different content from different static directories"
echo "  Port 8080: static/"
echo "  Port 8081: static-blog/"
echo "  Port 8082: static-api/"
echo "  Port 8083: static-dev/"
echo

echo "✅ Multi-site configuration test completed!"
echo
echo "To test manually:"
echo "  Main site:    http://localhost:8080"
echo "  Blog site:    http://localhost:8081"
echo "  API docs:     http://localhost:8082"
echo "  Dev site:     http://localhost:8083"
echo "  Sites API:    http://localhost:8080/api/sites"
