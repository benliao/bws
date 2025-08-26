# BWS Hot Reload Testing Guide

This document provides comprehensive testing guidelines for BWS hot reload functionality, including unit tests and integration test scripts.

## 📋 Table of Contents

1. [Unit Tests](#unit-tests)
2. [Integration Test Scripts](#integration-test-scripts)
3. [Test Coverage](#test-coverage)
4. [Running Tests](#running-tests)
5. [Test Scenarios](#test-scenarios)
6. [Troubleshooting](#troubleshooting)

## 🧪 Unit Tests

### Location
- **File**: `src/core/hot_reload.rs`
- **Test Module**: `tests` module at the end of the file
- **Test Count**: 20 comprehensive unit tests

### Test Categories

#### 1. **ReloadResult Tests**
- `test_reload_result()` - Basic reload result functionality
- `test_reload_result_all_changes()` - All change types detection

#### 2. **Configuration Loading Tests**
- `test_load_config_from_file()` - Valid configuration loading
- `test_load_config_from_nonexistent_file()` - Error handling for missing files
- `test_load_config_from_invalid_toml()` - Error handling for malformed TOML
- `test_validate_config_file()` - Configuration validation

#### 3. **Configuration Analysis Tests**
- `test_analyze_config_changes_no_changes()` - No changes detection
- `test_analyze_config_changes_server_info()` - Server info changes
- `test_analyze_config_changes_logging()` - Logging configuration changes
- `test_analyze_config_changes_performance()` - Performance settings changes
- `test_analyze_config_changes_security()` - Security configuration changes
- `test_analyze_config_changes_sites()` - Site configuration changes
- `test_analyze_config_changes_multiple()` - Multiple change types

#### 4. **Site Configuration Comparison Tests**
- `test_compare_sites_config()` - Basic site comparison
- `test_compare_sites_config_different_count()` - Different number of sites
- `test_compare_sites_config_ssl_changes()` - SSL configuration changes
- `test_compare_sites_config_proxy_changes()` - Proxy configuration changes

#### 5. **Integration Tests**
- `test_get_current_config()` - Current configuration retrieval
- `test_hot_reload_integration()` - End-to-end reload testing
- `test_reload_error_handling()` - Error handling during reload

### Test Configuration

Each test uses a helper function `create_test_config()` that creates a minimal valid configuration:

```rust
fn create_test_config() -> ServerConfig {
    ServerConfig {
        server: ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: "Test server".to_string(),
        },
        sites: vec![SiteConfig {
            name: "test-site".to_string(),
            hostname: "localhost".to_string(),
            port: 8080,
            static_dir: "/tmp/static".to_string(),
            default: true,
            // ... other default values
        }],
        logging: LoggingConfig::default(),
        performance: PerformanceConfig::default(),
        security: SecurityConfig::default(),
    }
}
```

## 🚀 Integration Test Scripts

### 1. Bash Script (Linux/macOS)
- **File**: `tests/test_hot_reload.sh`
- **Purpose**: Complete end-to-end testing of hot reload functionality
- **Requirements**: Bash shell, curl command

#### Features:
- ✅ Automated BWS server startup/shutdown
- ✅ Configuration file creation and modification
- ✅ Hot reload via SIGHUP signal
- ✅ Hot reload via bws-ctl command
- ✅ Configuration validation testing
- ✅ Error handling verification
- ✅ Server response validation
- ✅ Comprehensive test reporting

#### Usage:
```bash
cd tests
chmod +x test_hot_reload.sh
./test_hot_reload.sh
```

### 2. PowerShell Script (Windows)
- **File**: `tests/test_hot_reload.ps1`
- **Purpose**: Windows-compatible end-to-end testing
- **Requirements**: PowerShell 5.1+, Invoke-RestMethod cmdlet

#### Features:
- ✅ Cross-platform Windows testing
- ✅ Automated BWS server management
- ✅ Configuration file handling
- ⚠️ Hot reload testing (Windows limitation documented)
- ✅ Validation and error testing
- ✅ Server response verification
- ✅ Detailed test reporting

#### Usage:
```powershell
cd tests
.\test_hot_reload.ps1
# OR with verbose output
.\test_hot_reload.ps1 -Verbose
```

## � Platform Limitations

### Windows Hot Reload Status

**Current Status**: Hot reload is **not supported** on Windows platforms.

**Technical Reason**: BWS hot reload relies on Unix signals (SIGHUP) which are not available on Windows.

**Impact**:
- `bws-ctl reload` command fails on Windows with error: "Signal-based reload not supported on Windows"
- Configuration changes require manual server restart on Windows
- Hot reload tests report the limitation but don't fail (expected behavior)

**Workarounds**:
1. **Development**: Use WSL (Windows Subsystem for Linux) for hot reload testing
2. **Production**: Plan for brief downtime during configuration updates
3. **Testing**: Use Linux containers or VMs for comprehensive testing

**Future Enhancement**: Windows support could be implemented using:
- Named pipes for inter-process communication
- File system watching for configuration changes
- Windows-specific IPC mechanisms

## �📊 Test Coverage

### Configuration Areas Tested

| Configuration Section | Unit Tests | Integration Tests | Coverage |
|----------------------|------------|-------------------|----------|
| Server Info | ✅ | ✅ | 100% |
| Logging Config | ✅ | ✅ | 100% |
| Performance Config | ✅ | ✅ | 100% |
| Security Config | ✅ | ✅ | 100% |
| Sites Config | ✅ | ✅ | 100% |
| SSL Configuration | ✅ | ✅ | 100% |
| Proxy Configuration | ✅ | ✅ | 100% |

### Error Scenarios Tested

| Error Type | Description | Test Coverage |
|------------|-------------|---------------|
| Missing Config File | Configuration file doesn't exist | ✅ |
| Invalid TOML | Malformed configuration syntax | ✅ |
| Validation Errors | Configuration fails validation rules | ✅ |
| Parse Errors | TOML parsing failures | ✅ |
| Runtime Errors | Errors during config application | ✅ |

### Reload Methods Tested

| Method | Platform | Test Type | Status |
|--------|----------|-----------|--------|
| SIGHUP Signal | Linux/macOS | Integration | ✅ |
| SIGHUP Signal | Windows | Integration | ❌ Not Supported |
| bws-ctl reload | Linux/macOS | Integration | ✅ |
| bws-ctl reload | Windows | Integration | ❌ Not Supported |
| Direct API | All | Unit | ✅ |

> **⚠️ Windows Limitation**: Hot reload is currently only supported on Unix-like systems (Linux, macOS). Windows lacks the signal system required for hot reload functionality.

## 🏃‍♂️ Running Tests

### Unit Tests Only
```bash
# Run all hot reload unit tests
cargo test hot_reload

# Run specific hot reload test
cargo test test_hot_reload_integration

# Run with verbose output
cargo test hot_reload -- --nocapture
```

### Integration Tests

#### Linux/macOS:
```bash
# Build BWS first
cargo build

# Run integration tests
cd tests
./test_hot_reload.sh
```

#### Windows:
```powershell
# Build BWS first
cargo build

# Run integration tests
cd tests
.\test_hot_reload.ps1
```

### All Tests
```bash
# Run all tests (unit + integration)
cargo test
```

## 🎯 Test Scenarios

### Scenario 1: Server Info Changes
```toml
# Before
[server]
name = "bws-server"
description = "Original description"

# After
[server]
name = "bws-server-updated"
description = "Updated description"
```
**Expected**: `server_info_changed = true`

### Scenario 2: Performance Tuning
```toml
# Before
[performance]
worker_threads = 4
max_connections = 1000

# After
[performance]
worker_threads = 8
max_connections = 2000
```
**Expected**: `performance_changed = true`

### Scenario 3: Adding New Site
```toml
# Before
[[sites]]
name = "site1"
hostname = "localhost"
port = 8080

# After
[[sites]]
name = "site1"
hostname = "localhost"
port = 8080

[[sites]]
name = "site2"
hostname = "example.com"
port = 8081
```
**Expected**: `sites_changed = true`

### Scenario 4: SSL Configuration
```toml
# Before
[[sites]]
name = "site1"
hostname = "localhost"
port = 8080
[sites.ssl]
enabled = false

# After
[[sites]]
name = "site1"
hostname = "localhost"
port = 8080
[sites.ssl]
enabled = true
auto_cert = true
```
**Expected**: `sites_changed = true`

## 🔧 Troubleshooting

### Common Issues

#### 1. Tests Fail to Start Server
**Symptoms**: BWS server doesn't start during integration tests
**Solutions**:
- Ensure BWS binary is built: `cargo build`
- Check port availability (18080)
- Verify file permissions on test directory
- Check firewall settings

#### 2. Hot Reload Doesn't Work
**Symptoms**: Configuration changes aren't detected
**Solutions**:
- Verify PID file exists and is readable
- Check server is still running
- Ensure configuration syntax is valid
- Verify file permissions

#### 3. Signal Tests Fail on Windows
**Symptoms**: SIGHUP-related tests fail
**Solutions**:
- Use PowerShell script instead of Bash
- Windows doesn't support Unix signals
- Use bws-ctl for reloads on Windows

#### 4. Permission Errors
**Symptoms**: Cannot create/modify test files
**Solutions**:
- Run tests with appropriate permissions
- Check temp directory access
- Verify static file directory permissions

### Debug Mode

#### Enable Verbose Logging:
```bash
# Unit tests with output
cargo test hot_reload -- --nocapture

# Integration tests with verbose mode
./test_hot_reload.sh  # (automatic verbose)
.\test_hot_reload.ps1 -Verbose
```

#### Check BWS Logs:
```bash
# During integration tests, check log file
tail -f /tmp/bws_test_*/bws.log
```

### Test Data Cleanup

Tests automatically clean up temporary files, but if needed:

```bash
# Remove leftover test directories
rm -rf /tmp/bws_test_*

# On Windows
Remove-Item $env:TEMP\bws_test_* -Recurse -Force
```

## 📝 Adding New Tests

### Unit Test Template:
```rust
#[tokio::test]
async fn test_your_scenario() {
    let config = create_test_config();
    let (reload_manager, _temp_file) = create_hot_reload_manager(config.clone()).await;

    // Your test logic here
    let result = reload_manager.your_method().await.unwrap();
    assert!(result.expected_condition);
}
```

### Integration Test Addition:
1. Add new test function to appropriate script
2. Update main test runner to include new test
3. Add test case documentation
4. Update test counter logic

## 🎉 Success Criteria

All tests should pass with the following outcomes:

### Unit Tests:
- **20/20 tests passing**
- **Zero compilation warnings** (hot reload specific)
- **Complete code coverage** for hot reload module

### Integration Tests:
- **Server starts successfully**
- **All reload methods work**
- **Configuration validation works**
- **Error handling is robust**
- **Server remains stable during reloads**

### Performance Criteria:
- **Hot reload completes in <2 seconds**
- **No memory leaks during reload**
- **Zero downtime during configuration changes**
- **All existing connections remain active**

---

**Last Updated**: August 26, 2025  
**BWS Version**: 0.3.3  
**Test Suite Version**: 1.0.0
