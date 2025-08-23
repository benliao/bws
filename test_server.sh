#!/bin/bash

echo "Testing BWS Web Server..."
echo

echo "1. Testing health endpoint:"
curl -s http://localhost:8080/api/health | jq .
echo

echo "2. Testing file endpoint with Cargo.toml:"
curl -s "http://localhost:8080/api/file?path=Cargo.toml" | jq .
echo

echo "3. Testing 404 endpoint:"
curl -s http://localhost:8080/nonexistent | jq .
echo

echo "4. Testing home page (first 200 chars):"
curl -s http://localhost:8080 | head -c 200
echo "..."
