# BWS Test Scripts

This directory contains organized test scripts for the BWS web server.

## 📁 Script Categories

### Core Testing Scripts
- `validate-configs.sh` - Comprehensive configuration validation using --dry-run
- `test_websocket_proxy.sh` - Complete WebSocket proxy functionality testing
- `test_hot_reload.sh` - Hot reload functionality testing
- `run_hot_reload_tests.sh` - Hot reload test orchestration

### Integration Testing
- `test_local_ci.sh` - Local CI pipeline simulation
- `docker-test.sh` - Docker container testing
- `simple_websocket_test.sh` - WebSocket connection testing

### Feature Testing
- `test_headers.sh` - HTTP header handling tests
- `test_multisite.sh` - Multi-site configuration testing
- `test_load_balance.sh` - Load balancing functionality tests

### Development Utilities
- `validate-configs-simple.sh` - Simplified configuration validation
- `github_actions_simulation.sh` - GitHub Actions workflow simulation

## 🚀 Usage

All scripts are executable and can be run from the project root:

```bash
# Validate all configurations
./tests/scripts/validate-configs.sh

# Test WebSocket functionality
./tests/scripts/test_websocket_proxy.sh

# Run hot reload tests
./tests/scripts/run_hot_reload_tests.sh
```

## 📋 Maintenance Notes

- Scripts follow `test_*.sh` naming convention
- All scripts include proper error handling and cleanup
- Scripts are covered by `.gitignore` pattern to prevent accidental commits of temporary test files
- Permanent test scripts are intentionally committed to the repository
