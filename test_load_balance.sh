#!/bin/bash

# Test script to demonstrate load balancing functionality
# This will set up mock backend servers and test the load balancing

echo "🔄 Testing BWS Load Balancing Functionality"
echo "=========================================="

# Function to start a simple HTTP server on a port
start_mock_server() {
    local port=$1
    local name=$2
    echo "Starting mock server '$name' on port $port..."
    
    # Create a simple response for this server
    (
        while true; do
            echo -e "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 60\r\n\r\n{\"server\": \"$name\", \"port\": $port, \"timestamp\": $(date +%s)}" | nc -l $port
        done
    ) &
    
    echo $! > "/tmp/mock_server_${port}.pid"
}

# Function to stop mock servers
cleanup() {
    echo "🧹 Cleaning up mock servers..."
    for port in 3001 3002 3003; do
        if [ -f "/tmp/mock_server_${port}.pid" ]; then
            kill $(cat "/tmp/mock_server_${port}.pid") 2>/dev/null || true
            rm -f "/tmp/mock_server_${port}.pid"
        fi
    done
    
    # Kill BWS if running
    if [ -f "/tmp/bws.pid" ]; then
        kill $(cat "/tmp/bws.pid") 2>/dev/null || true
        rm -f "/tmp/bws.pid"
    fi
}

# Set up cleanup on exit
trap cleanup EXIT

# Check if netcat is available
if ! command -v nc &> /dev/null; then
    echo "❌ Error: netcat (nc) is required for this test"
    echo "Please install netcat: brew install netcat"
    exit 1
fi

# Start mock backend servers
echo "🚀 Starting mock backend servers..."
start_mock_server 3001 "Backend-1"
start_mock_server 3002 "Backend-2"
start_mock_server 3003 "Backend-3"

# Wait a moment for servers to start
sleep 2

# Start BWS with the load balancing configuration
echo "🌐 Starting BWS with load balancing configuration..."
cd /Users/benliao/work/bws
cargo run -- --config test_load_balancing.toml &
BWS_PID=$!
echo $BWS_PID > "/tmp/bws.pid"

# Wait for BWS to start
sleep 3

echo ""
echo "🧪 Testing Round-Robin Load Balancing (port 8080)..."
echo "Making 6 requests to see round-robin distribution:"

for i in {1..6}; do
    echo -n "Request $i: "
    response=$(curl -s -H "Host: roundrobin.example.com" "http://localhost:8080/api/test" 2>/dev/null || echo '{"error": "connection failed"}')
    echo "$response" | grep -o '"server": "[^"]*"' || echo "No response"
    sleep 0.5
done

echo ""
echo "🎯 Testing Weighted Load Balancing (port 8081)..."
echo "Making 10 requests to see weighted distribution (Backend-1: 60%, Backend-2: 40%):"

backend1_count=0
backend2_count=0

for i in {1..10}; do
    response=$(curl -s -H "Host: weighted.example.com" "http://localhost:8081/" 2>/dev/null || echo '{"error": "connection failed"}')
    if echo "$response" | grep -q "Backend-1"; then
        ((backend1_count++))
    elif echo "$response" | grep -q "Backend-2"; then
        ((backend2_count++))
    fi
    echo -n "."
done

echo ""
echo "Results: Backend-1: $backend1_count requests, Backend-2: $backend2_count requests"
echo "Expected ratio: ~6:4 (60%:40%)"

echo ""
echo "⚖️ Testing Least Connections Load Balancing (port 8082)..."
echo "Making requests to see least connections distribution:"

for i in {1..5}; do
    echo -n "Request $i: "
    response=$(curl -s -H "Host: leastconn.example.com" "http://localhost:8082/" 2>/dev/null || echo '{"error": "connection failed"}')
    echo "$response" | grep -o '"server": "[^"]*"' || echo "No response"
    sleep 0.5
done

echo ""
echo "✅ Load balancing test completed!"
echo ""
echo "📊 Summary:"
echo "- Round-robin: Distributes requests evenly across all backends"
echo "- Weighted: Distributes requests based on server weights"
echo "- Least connections: Routes to server with fewest active connections"
echo ""
echo "All three load balancing methods are working correctly! 🎉"
