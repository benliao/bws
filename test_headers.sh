#!/bin/bash

echo "=== Testing Configurable Headers for BWS Multi-Site Server ==="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test headers for a specific site
test_site_headers() {
    local site_name=$1
    local port=$2
    local url="http://localhost:$port"
    
    echo -e "${BLUE}Testing $site_name site headers (Port $port)${NC}"
    echo "URL: $url"
    echo
    
    # Test main page headers
    echo -e "${YELLOW}Main page headers:${NC}"
    curl -I "$url/" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control)" | head -10
    echo
    
    # Test API health endpoint headers
    echo -e "${YELLOW}API health endpoint headers:${NC}"
    curl -I "$url/api/health" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control)" | head -10
    echo
    
    # Test sites info endpoint headers
    echo -e "${YELLOW}API sites endpoint headers:${NC}"
    curl -I "$url/api/sites" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control)" | head -10
    echo
    
    # Test static file headers
    echo -e "${YELLOW}Static file headers (styles.css):${NC}"
    curl -I "$url/static/styles.css" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control|Cache-Control)" | head -10
    echo
    
    echo "----------------------------------------"
    echo
}

# Function to check if server is running
check_server() {
    local port=$1
    if curl -s "http://localhost:$port/api/health" > /dev/null; then
        return 0
    else
        return 1
    fi
}

# Check if any server instance is running
echo -e "${YELLOW}Checking server status...${NC}"
if check_server 8080; then
    echo -e "${GREEN}✓ Server is running${NC}"
    echo
else
    echo -e "${RED}✗ Server is not running. Please start the server first with: cargo run${NC}"
    echo
    exit 1
fi

# Test all sites
test_site_headers "Main" 8080
test_site_headers "Blog" 8081  
test_site_headers "API" 8082
test_site_headers "Dev" 8083

# Test virtual host headers
echo -e "${BLUE}Testing Virtual Host Headers${NC}"
echo

echo -e "${YELLOW}Testing blog site with Host header:${NC}"
curl -H "Host: blog.localhost:8081" -I "http://localhost:8081/" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control)" | head -10
echo

echo -e "${YELLOW}Testing API site with Host header:${NC}"
curl -H "Host: api.localhost:8082" -I "http://localhost:8082/" 2>/dev/null | grep -E "^(HTTP|X-|Access-Control)" | head -10
echo

# Test sites configuration endpoint to see all configured headers
echo -e "${BLUE}Sites Configuration (showing configured headers)${NC}"
echo
curl -s "http://localhost:8080/api/sites" | python3 -m json.tool 2>/dev/null || curl -s "http://localhost:8080/api/sites"
echo

echo -e "${GREEN}=== Header Testing Complete ===${NC}"
