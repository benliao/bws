# BWS --dry-run Feature Implementation

## Overview
Successfully added a comprehensive `--dry-run` feature to BWS that validates configuration files and setups without starting the server.

## Implementation Details

### 1. Command Line Interface
Added `--dry-run` flag to the BWS CLI:
```bash
bws --dry-run                    # Validate config.toml (default)
bws --config custom.toml --dry-run   # Validate specific config
bws --dry-run /path/to/website   # Validate temporary directory setup
```

### 2. Validation Features
The dry-run mode performs comprehensive validation:

#### Configuration Structure
- ✅ TOML syntax validation
- ✅ Required sections ([server], [[sites]])
- ✅ Configuration schema compliance

#### Site Configuration
- ✅ Static directory existence and accessibility
- ✅ Index file availability
- ⚠️  Missing index files (warning)
- ✅ Custom headers validation

#### SSL/TLS Configuration
- ✅ SSL certificate file existence (manual certs)
- ✅ ACME configuration completeness
- ✅ Auto-certificate email validation
- ⚠️  Missing certificates with SSL enabled (warning)

#### Proxy Configuration
- ✅ Upstream server configuration
- ✅ Route configuration
- ❌ Proxy enabled without upstreams (error)
- ⚠️  Proxy enabled without routes (warning)

#### Port and Virtual Hosting
- ✅ Port conflict detection
- ✅ Virtual hosting validation
- ✅ Default site designation
- ❌ Multiple default sites on same port (error)
- ⚠️  Overlapping hostnames (warning)

### 3. Output Format
The dry-run feature provides clear, structured output:

```bash
🔍 BWS Configuration Validation (Dry Run Mode)
==========================================
✅ Configuration file 'config.toml' loaded successfully

📊 Configuration Summary:
   Server: BWS Multi-Site Server v0.3.4
   Sites: 4

🌐 Site 1: main
   Hostname: localhost
   Port: 8080
   Static directory: examples/sites/static
   ✅ Static directory exists
   📋 Custom headers: 4

==========================================
           VALIDATION RESULTS
==========================================
✅ Configuration validation passed!
🚀 Configuration is ready for deployment
```

### 4. Error Handling
Comprehensive error detection and reporting:
- **Exit Code 0**: Configuration valid, ready for deployment
- **Exit Code 1**: Configuration errors found, fix required
- **Clear Error Messages**: Specific issues with actionable solutions

### 5. Integration with Test Infrastructure
Updated configuration validation scripts to use the new dry-run feature:

```bash
# Test configuration validity
cargo run --bin bws -- --config tests/configs/basic.toml --dry-run

# Validate all test configurations
./tests/scripts/validate-configs-simple.sh
```

## Usage Examples

### Valid Configuration
```bash
$ bws --config config.toml --dry-run
🔍 BWS Configuration Validation (Dry Run Mode)
==========================================
✅ Configuration validation passed!
🚀 Configuration is ready for deployment
```

### Invalid Configuration
```bash
$ bws --config invalid.toml --dry-run
🔍 BWS Configuration Validation (Dry Run Mode)
==========================================
❌ Configuration validation failed (2 errors):
   ❌ Site 'test': Static directory '/nonexistent' does not exist
   ❌ Site 'test': ACME email is required when auto_cert is enabled

💡 Fix the errors above and try again
```

### Temporary Directory Mode
```bash
$ bws --dry-run /path/to/website --port 8080
🚀 Creating temporary web server:
   📁 Directory: /path/to/website
   🌐 Port: 8080
   🌐 URL: http://localhost:8080

🔍 BWS Configuration Validation (Dry Run Mode)
==========================================
✅ Configuration validation passed!
🚀 Configuration is ready for deployment
```

## Benefits

### For Development
- **Fast Feedback**: Validate configurations without starting the server
- **Clear Diagnostics**: Detailed error messages with specific fixes
- **Test Integration**: Automated validation in test pipelines

### For Operations
- **Deployment Safety**: Validate configurations before applying
- **Troubleshooting**: Identify configuration issues quickly
- **Documentation**: Self-documenting validation output

### For CI/CD
- **Pipeline Integration**: Easy integration into build/deployment pipelines
- **Configuration Testing**: Automated testing of configuration changes
- **Quality Assurance**: Prevent invalid configurations from reaching production

## Implementation Files Modified

### Core Implementation
- `src/bin/main.rs`: Added CLI flag and validation logic
  - Added `--dry-run` CLI argument
  - Implemented `handle_dry_run()` function
  - Comprehensive validation logic

### Test Infrastructure
- `tests/scripts/validate-configs-simple.sh`: Simple validator using dry-run
- `tests/run-tests.sh`: Updated prerequisites to use dry-run validation
- `tests/configs/daemon-test.toml`: Fixed invalid path for testing

### Documentation
- `README.md`: Updated with dry-run feature documentation
- Added configuration validation section
- Updated CLI options documentation

## Technical Implementation

### Validation Logic
The dry-run feature implements a multi-stage validation process:

1. **Configuration Loading**: Parse and load TOML configuration
2. **Structure Validation**: Verify required sections and schema
3. **Site-by-Site Validation**: Check each site configuration
4. **Cross-Site Validation**: Check for conflicts and issues
5. **Resource Validation**: Verify file paths and dependencies
6. **Output Generation**: Provide clear results and recommendations

### Error Categories
- **Errors**: Critical issues that prevent server startup
- **Warnings**: Issues that may cause problems but don't prevent startup
- **Info**: Configuration details and successful validations

### Exit Codes
- `0`: All validations passed, configuration ready
- `1`: Validation errors found, configuration needs fixes

## Future Enhancements

### Potential Improvements
1. **JSON Output**: Machine-readable validation results
2. **Configuration Templates**: Generate sample configurations
3. **Performance Validation**: Check resource requirements
4. **Security Validation**: Detect security misconfigurations
5. **Integration Testing**: Validate upstream connectivity

### Advanced Features
1. **Fix Suggestions**: Automated fix recommendations
2. **Configuration Diff**: Compare configurations
3. **Deployment Simulation**: Predict deployment behavior
4. **Resource Planning**: Estimate resource requirements

## Conclusion

The `--dry-run` feature significantly improves BWS usability by:
- **Reducing Deployment Risks**: Catch configuration errors before deployment
- **Improving Developer Experience**: Fast feedback during development
- **Enhancing Operations**: Better troubleshooting and validation tools
- **Supporting Automation**: Easy integration into CI/CD pipelines

This feature represents a major step toward making BWS more production-ready and developer-friendly.
