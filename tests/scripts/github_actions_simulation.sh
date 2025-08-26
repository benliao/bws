#!/bin/bash

# BWS GitHub Actions Simulation Test
# This mimics the exact GitHub Actions hot-reload-test.yml workflow

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

cleanup() {
    if [ ! -z "$BWS_PID" ] && kill -0 $BWS_PID 2>/dev/null; then
        echo "Cleaning up BWS process (PID: $BWS_PID)"
        kill $BWS_PID
        for i in {1..5}; do
            if ! kill -0 $BWS_PID 2>/dev/null; then
                break
            fi
            sleep 1
        done
        if kill -0 $BWS_PID 2>/dev/null; then
            kill -9 $BWS_PID 2>/dev/null || true
        fi
    fi
    
    # Clean up test files
    rm -rf test-site
    rm -f test-hot-reload.toml
}

trap cleanup EXIT

main() {
    echo "=================================================="
    echo "BWS GitHub Actions Hot Reload Test Simulation"
    echo "=================================================="
    
    # Build BWS (like GitHub Actions)
    echo "Building BWS..."
    cargo build --release --bin bws
    echo "✅ BWS built successfully"
    
    # Setup test environment (exactly like GitHub Actions)
    echo "Setting up test environment..."
    
    # Create test static files
    mkdir -p test-site/static
    echo "<html><body><h1>Original Content</h1></body></html>" > test-site/static/index.html
    
    # Create initial configuration
    cat > test-hot-reload.toml << 'EOF'
[server]
name = "Hot Reload Test Server"
host = "0.0.0.0"
worker_threads = 2

[[sites]]
name = "test-site"
hostname = "localhost"
port = 8080
static_dir = "test-site/static"
default_site = true

[sites.headers]
"X-Test-Header" = "original-value"
EOF
    
    echo "✅ Test environment set up"
    
    # Test hot reload functionality (exactly like GitHub Actions)
    echo "Starting BWS server..."
    
    # Start BWS in background
    ./target/release/bws --config test-hot-reload.toml &
    BWS_PID=$!
    
    # Wait for server to start
    echo "Waiting for server to start..."
    sleep 3
    
    # Test initial configuration
    echo "Testing initial configuration..."
    RESPONSE=$(curl -s -H "Host: localhost" http://localhost:8080/ | grep "Original Content")
    if [ -z "$RESPONSE" ]; then
        echo -e "${RED}❌ Initial content not found${NC}"
        exit 1
    fi
    
    HEADER=$(curl -s -I -H "Host: localhost" http://localhost:8080/ | grep "X-Test-Header: original-value")
    if [ -z "$HEADER" ]; then
        echo -e "${RED}❌ Initial header not found${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Initial configuration working${NC}"
    
    # Modify configuration
    echo "Modifying configuration..."
    cat > test-hot-reload.toml << 'EOF'
[server]
name = "Hot Reload Test Server"
host = "0.0.0.0"
worker_threads = 2

[[sites]]
name = "test-site"
hostname = "localhost"
port = 8080
static_dir = "test-site/static"
default_site = true

[sites.headers]
"X-Test-Header" = "updated-value"
"X-New-Header" = "new-value"
EOF
    
    # Update content
    echo "<html><body><h1>Updated Content</h1></body></html>" > test-site/static/index.html
    
    # Test content update (file-based changes should work immediately)
    echo "Testing content updates..."
    sleep 2
    
    RESPONSE=$(curl -s -H "Host: localhost" http://localhost:8080/ | grep "Updated Content")
    if [ -z "$RESPONSE" ]; then
        echo -e "${RED}❌ Updated content not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Content updates working${NC}"
    
    # Clean shutdown
    echo "Test completed successfully!"
    
    echo -e "${GREEN}🎉 GitHub Actions simulation completed successfully!${NC}"
}

main "$@"
