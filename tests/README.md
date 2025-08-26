# BWS Testing Suite

This directory contains the comprehensive testing suite for BWS (Blazing Web Server).

## 📁 Directory Structure

```
tests/
├── README.md                    # This file - testing overview
├── HOT_RELOAD_TESTING.md       # Detailed hot reload testing guide
├── configs/                    # Test configuration files
│   ├── config_test.toml        # Multi-site test configuration
│   ├── daemon-test.toml        # Daemon mode test configuration
│   ├── hot-reload.toml         # Hot reload test configuration
│   ├── test_load_balancing.toml # Load balancing test configuration
│   └── test_proxy_config.toml  # Proxy functionality test configuration
├── fixtures/                   # Test data and static files
│   └── test-site/              # Sample static site for testing
├── integration/                # Integration test files
│   ├── integration_multi_hostname.rs    # Multi-hostname testing
│   ├── integration_virtual_hosting.rs   # Virtual hosting testing
│   └── test_pingora_windows.rs         # Windows-specific tests
├── scripts/                    # Test execution scripts
│   ├── docker-test.bat         # Windows Docker test runner
│   ├── docker-test.sh          # Linux/macOS Docker test runner
│   ├── github_actions_simulation.sh    # CI/CD simulation
│   ├── run_hot_reload_tests.sh # Hot reload test runner
│   ├── simple_websocket_test.sh # WebSocket testing
│   ├── test_headers.sh         # HTTP headers testing
│   ├── test_hot_reload.sh      # Hot reload integration tests
│   ├── test_hot_reload_complete.sh     # Complete hot reload test suite
│   ├── test_load_balance.sh    # Load balancing tests
│   ├── test_local_ci.sh        # Local CI environment simulation
│   ├── test_multisite.sh       # Multi-site configuration tests
│   ├── test_server.sh          # Basic server functionality tests
│   ├── test_static_server.sh   # Static file serving tests
│   ├── test_websocket_client.py        # WebSocket client tests (Python)
│   ├── test_websocket_full.py          # Complete WebSocket tests (Python)
│   ├── test_websocket_proxy.sh         # WebSocket proxy tests
│   └── test_ws_upstream.py     # WebSocket upstream tests (Python)
└── unit/                       # Unit test helpers (future use)
```

## 🚀 Quick Start

### Prerequisites
- Rust and Cargo installed
- BWS compiled (`cargo build`)
- curl command available
- For Python scripts: Python 3.7+

### Running All Tests
```bash
# Build BWS first
cargo build

# Run unit tests
cargo test

# Run integration tests
cd tests/scripts
./test_server.sh
./test_multisite.sh
./run_hot_reload_tests.sh
```

### Running Specific Test Categories

#### Basic Server Tests
```bash
cd tests/scripts
./test_server.sh              # Basic API and file serving
./test_static_server.sh       # Static file serving
./test_headers.sh             # HTTP headers
```

#### Multi-Site and Configuration Tests
```bash
cd tests/scripts
./test_multisite.sh           # Multi-site configuration
./test_load_balance.sh        # Load balancing
```

#### Hot Reload Tests
```bash
cd tests/scripts
./run_hot_reload_tests.sh     # Complete hot reload suite
./test_hot_reload.sh          # Basic hot reload
./test_hot_reload_complete.sh # Extended hot reload tests
```

#### WebSocket Tests
```bash
cd tests/scripts
./simple_websocket_test.sh    # Basic WebSocket functionality
./test_websocket_proxy.sh     # WebSocket proxy
python3 test_websocket_client.py    # Python WebSocket client
python3 test_websocket_full.py      # Complete WebSocket test suite
```

#### CI/CD and Docker Tests
```bash
cd tests/scripts
./test_local_ci.sh            # Local CI simulation
./github_actions_simulation.sh       # GitHub Actions simulation
./docker-test.sh              # Docker environment tests
```

## 📋 Test Configurations

The `configs/` directory contains various test configurations:

- **`config_test.toml`**: Multi-site setup with different SSL configurations
- **`daemon-test.toml`**: Daemon mode testing configuration
- **`hot-reload.toml`**: Hot reload functionality testing
- **`test_load_balancing.toml`**: Load balancing configuration
- **`test_proxy_config.toml`**: Proxy and reverse proxy testing

### Using Test Configurations
```bash
# Start BWS with a specific test configuration
cargo run --bin bws -- --config tests/configs/config_test.toml

# Test hot reload functionality
cargo run --bin bws -- --config tests/configs/hot-reload.toml
```

## 🧪 Test Categories

### Unit Tests
- Located in source files (`src/**/*.rs`)
- Run with `cargo test`
- Focus on individual functions and modules

### Integration Tests
- Located in `tests/integration/`
- Test complete workflows and system interactions
- Written in Rust using the `#[test]` attribute

### End-to-End Tests
- Located in `tests/scripts/`
- Shell and Python scripts that test complete user scenarios
- Test actual HTTP requests and responses

### Performance Tests
- Load balancing and high-concurrency scenarios
- WebSocket connection handling
- Configuration reload performance

## 🔧 Development Workflow

### Adding New Tests

1. **Unit Tests**: Add to relevant source file in `src/`
2. **Integration Tests**: Add new `.rs` file to `tests/integration/`
3. **Script Tests**: Add new script to `tests/scripts/`
4. **Test Configs**: Add new config to `tests/configs/`

### Test Configuration Template
```toml
[server]
name = "BWS Test Server"

[[sites]]
name = "test_site"
hostname = "localhost"
port = 8080
static_dir = "./tests/fixtures/test-site"
default = true

# Add specific test configurations here
```

### Script Test Template
```bash
#!/bin/bash
set -e

echo "Running [Test Name] tests..."

# Setup
SERVER_PID=""
cleanup() {
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Start server
cargo run --bin bws -- --config tests/configs/your-config.toml &
SERVER_PID=$!
sleep 2

# Run tests
echo "✅ Test passed"

# Cleanup happens automatically via trap
```

## 📊 Test Coverage

Current test coverage includes:

- ✅ **Basic HTTP serving**
- ✅ **Multi-site configuration**
- ✅ **SSL/TLS functionality**
- ✅ **Hot configuration reload**
- ✅ **Load balancing**
- ✅ **WebSocket support**
- ✅ **Proxy functionality**
- ✅ **Docker deployment**
- ✅ **CI/CD integration**

## 🐛 Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Kill any existing BWS processes
   pkill -f bws
   # Or use a different port in test config
   ```

2. **Permission Denied**
   ```bash
   # Make scripts executable
   chmod +x tests/scripts/*.sh
   ```

3. **Dependencies Missing**
   ```bash
   # Install required tools
   sudo apt-get install curl jq python3  # Ubuntu/Debian
   brew install curl jq python3          # macOS
   ```

### Debug Mode
```bash
# Run BWS with verbose logging
RUST_LOG=debug cargo run --bin bws -- --config tests/configs/config_test.toml

# Run tests with verbose output
bash -x tests/scripts/test_server.sh
```

## 📝 Contributing

When adding new tests:

1. Follow the existing directory structure
2. Update this README with new test descriptions
3. Ensure tests clean up after themselves
4. Add error handling and proper exit codes
5. Include both positive and negative test cases

## 🎯 Test Philosophy

- **Fast Feedback**: Unit tests should run quickly
- **Isolated**: Each test should be independent
- **Comprehensive**: Cover both success and failure scenarios
- **Realistic**: Integration tests should simulate real usage
- **Maintainable**: Tests should be easy to understand and modify

---

**Last Updated**: August 26, 2025  
**BWS Version**: 0.3.4  
**Test Suite Version**: 2.0.0
