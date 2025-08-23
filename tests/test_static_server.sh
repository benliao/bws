#!/bin/bash

echo "Testing BWS Static Website Server..."
echo

# Start server in background
echo "Starting server..."
RUST_LOG=info cargo run --manifest-path ../Cargo.toml &
SERVER_PID=$!
sleep 3

echo "Testing static website endpoints..."
echo

echo "1. Home page (HTML):"
curl -I http://localhost:8080
echo

echo "2. CSS file:"
curl -I http://localhost:8080/static/styles.css
echo

echo "3. JavaScript file:"
curl -I http://localhost:8080/static/script.js
echo

echo "4. About page:"
curl -I http://localhost:8080/about.html
echo

echo "5. Contact page:"
curl -I http://localhost:8080/contact.html
echo

echo "6. API Health check (still works):"
curl -s http://localhost:8080/api/health | jq .
echo

echo "7. File API (still works):"
curl -s "http://localhost:8080/api/file?path=Cargo.toml" | jq .file_path
echo

echo "8. 404 test:"
curl -I http://localhost:8080/nonexistent
echo

echo "Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo "✅ Static website server test completed!"
