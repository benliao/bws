#!/usr/bin/env bash
# BWS Development Setup Script
# This script sets up a development environment for BWS

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version)
    print_info "Found Rust: $rust_version"
}

# Check if Cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    local cargo_version=$(cargo --version)
    print_info "Found Cargo: $cargo_version"
}

# Create development directories
setup_directories() {
    print_info "Setting up development directories..."
    
    mkdir -p static-dev
    mkdir -p static-blog
    mkdir -p static-api
    mkdir -p logs
    mkdir -p certs/dev
    
    print_info "Development directories created"
}

# Create sample static files
create_sample_files() {
    print_info "Creating sample static files..."
    
    # Main site
    cat > static-dev/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BWS Development Server</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            max-width: 800px; 
            margin: 50px auto; 
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #333; }
        .status { color: #28a745; font-weight: bold; }
        .links { margin: 20px 0; }
        .links a { 
            display: inline-block; 
            margin: 5px 10px 5px 0; 
            padding: 8px 16px;
            background: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }
        .links a:hover { background: #0056b3; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 BWS Development Server</h1>
        <p class="status">✓ Server is running successfully!</p>
        
        <h2>Test Links</h2>
        <div class="links">
            <a href="/api/health">Health Check</a>
            <a href="/api/health/detailed">Detailed Health</a>
            <a href="http://blog.localhost:8081">Blog Site</a>
            <a href="http://api.localhost:8082">API Docs</a>
        </div>
        
        <h2>Features</h2>
        <ul>
            <li>Multi-site hosting</li>
            <li>SSL/TLS support</li>
            <li>Static file serving</li>
            <li>Health monitoring</li>
            <li>WebSocket proxy</li>
            <li>Load balancing</li>
        </ul>
    </div>
</body>
</html>
EOF

    # Blog site
    cat > static-blog/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BWS Blog</title>
    <style>
        body { font-family: Georgia, serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        h1 { color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }
        .post { margin: 30px 0; padding: 20px; background: #f8f9fa; border-left: 4px solid #3498db; }
    </style>
</head>
<body>
    <h1>BWS Blog Site</h1>
    <div class="post">
        <h2>Welcome to BWS!</h2>
        <p>This is a sample blog hosted on BWS (Blazing Web Server). BWS supports multiple sites with individual configurations.</p>
        <p><strong>Features:</strong> Multi-site hosting, SSL/TLS, load balancing, and more!</p>
    </div>
</body>
</html>
EOF

    # API docs
    cat > static-api/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BWS API Documentation</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
            max-width: 900px; 
            margin: 50px auto; 
            padding: 20px;
            background: #fafafa;
        }
        .container { background: white; padding: 30px; border-radius: 8px; }
        h1 { color: #1a73e8; }
        .endpoint { 
            margin: 20px 0; 
            padding: 15px; 
            background: #f1f3f4; 
            border-radius: 4px;
            border-left: 4px solid #1a73e8;
        }
        .method { 
            display: inline-block; 
            padding: 4px 8px; 
            border-radius: 3px; 
            font-weight: bold; 
            font-size: 12px;
        }
        .get { background: #4caf50; color: white; }
        code { background: #f5f5f5; padding: 2px 4px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>📚 BWS API Documentation</h1>
        
        <div class="endpoint">
            <div><span class="method get">GET</span> <code>/api/health</code></div>
            <p>Basic health check endpoint</p>
        </div>
        
        <div class="endpoint">
            <div><span class="method get">GET</span> <code>/api/health/detailed</code></div>
            <p>Detailed health information including uptime and system info</p>
        </div>
        
        <div class="endpoint">
            <div><span class="method get">GET</span> <code>/api/health/ready</code></div>
            <p>Readiness probe for load balancers</p>
        </div>
        
        <div class="endpoint">
            <div><span class="method get">GET</span> <code>/api/health/live</code></div>
            <p>Liveness probe for monitoring</p>
        </div>
    </div>
</body>
</html>
EOF

    print_info "Sample static files created"
}

# Create development config
create_dev_config() {
    print_info "Creating development configuration..."
    
    cat > config-dev.toml << 'EOF'
# BWS Development Configuration
[server]
name = "BWS Development Server"

# Main development site
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "examples/sites/static-dev"
default = true

[sites.headers]
"X-Site-Name" = "BWS Development"
"X-Environment" = "development"
"X-Debug-Mode" = "enabled"

# Blog site
[[sites]]
name = "blog"
hostname = "blog.localhost"
port = 8081
static_dir = "examples/sites/static-blog"

[sites.headers]
"X-Site-Name" = "BWS Dev Blog"
"X-Content-Type" = "blog"

# API documentation site
[[sites]]
name = "api"
hostname = "api.localhost"
port = 8082
static_dir = "examples/sites/static-api"
api_only = true

[sites.headers]
"X-Site-Name" = "BWS API Docs"
"Access-Control-Allow-Origin" = "*"
EOF

    print_info "Development configuration created: config-dev.toml"
}

# Build the project
build_project() {
    print_info "Building BWS in development mode..."
    
    if cargo build; then
        print_info "✓ Build successful"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Run development server
run_dev_server() {
    print_info "Starting BWS development server..."
    print_info "Main site: http://localhost:8080"
    print_info "Blog site: http://blog.localhost:8081"  
    print_info "API docs:  http://api.localhost:8082"
    print_info ""
    print_warn "Press Ctrl+C to stop the server"
    
    cargo run -- --config config-dev.toml --verbose
}

# Main function
main() {
    echo "🔧 BWS Development Setup"
    echo "======================="
    
    check_rust
    check_cargo
    setup_directories
    create_sample_files
    create_dev_config
    build_project
    
    print_info "Development environment setup complete!"
    print_info ""
    
    read -p "Start the development server now? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        run_dev_server
    else
        print_info "To start the server later, run:"
        print_info "  cargo run -- --config config-dev.toml --verbose"
    fi
}

# Run main function
main "$@"
