# Docker Clippy Scripts

These scripts allow you to run `cargo clippy` in a Linux Rust Docker container, ensuring consistent linting across different platforms.

## Scripts

### `docker-clippy.ps1` (PowerShell - Windows)
```powershell
# Basic usage
.\scripts\docker-clippy.ps1

# With verbose output
.\scripts\docker-clippy.ps1 -Verbose

# Run with --fix to automatically fix issues
.\scripts\docker-clippy.ps1 -Fix

# Use specific Rust version
.\scripts\docker-clippy.ps1 -RustVersion "latest"

# With additional clippy arguments
.\scripts\docker-clippy.ps1 -ClippyArgs '--allow','clippy::too_many_arguments'

# Show help
.\scripts\docker-clippy.ps1 -Help
```

### `docker-clippy.sh` (Bash - Linux/macOS/WSL)
```bash
# Basic usage
./scripts/docker-clippy.sh

# With verbose output
./scripts/docker-clippy.sh --verbose

# Run with --fix to automatically fix issues
./scripts/docker-clippy.sh --fix

# Use specific Rust version
./scripts/docker-clippy.sh --rust-version latest

# With additional clippy arguments
./scripts/docker-clippy.sh --args '--allow' --args 'clippy::too_many_arguments'

# Show help
./scripts/docker-clippy.sh --help
```

## Features

✅ **Cross-platform consistency** - Uses Linux Rust container regardless of host OS
✅ **Version flexibility** - Specify any Rust version (default: 1.89.0)
✅ **Auto-fix support** - Run clippy with `--fix` to automatically resolve issues
✅ **Custom arguments** - Pass additional clippy flags and options
✅ **Permission handling** - Automatically handles file permissions to avoid Docker issues
✅ **Verbose output** - Enable detailed logging for debugging
✅ **Progress tracking** - Shows execution time and status

## Requirements

- **Docker**: Must be installed and running
- **Project structure**: Run from BWS project root (where `Cargo.toml` is located)
- **Internet access**: For pulling Rust Docker images

## Default Clippy Configuration

Both scripts run clippy with these default arguments:
- `--all-targets` - Check all targets (lib, bins, tests, examples)
- `--all-features` - Enable all features
- `-D warnings` - Treat warnings as errors

## Examples

### Quick lint check
```bash
# PowerShell
.\scripts\docker-clippy.ps1

# Bash
./scripts/docker-clippy.sh
```

### Fix all auto-fixable issues
```bash
# PowerShell
.\scripts\docker-clippy.ps1 -Fix

# Bash
./scripts/docker-clippy.sh --fix
```

### Verbose debugging
```bash
# PowerShell
.\scripts\docker-clippy.ps1 -Verbose

# Bash
./scripts/docker-clippy.sh --verbose
```

### Use latest Rust nightly
```bash
# PowerShell
.\scripts\docker-clippy.ps1 -RustVersion "nightly"

# Bash
./scripts/docker-clippy.sh --rust-version nightly
```

## Output

The scripts provide colored output showing:
- ✅ **Success**: Green indicators for successful completion
- ❌ **Errors**: Red indicators for failures
- ⚠️ **Warnings**: Yellow indicators for warnings
- 📋 **Info**: Blue indicators for information
- 🔍 **Debug**: Cyan indicators for verbose information

## Troubleshooting

### Docker not found
```
❌ Docker is not installed or not running!
```
**Solution**: Install Docker Desktop and ensure it's running.

### Permission issues
```
Permission denied when accessing files
```
**Solution**: The scripts automatically handle permissions using current user ID in Docker.

### Image pull fails
```
⚠️ Warning: Failed to pull image, proceeding with local image
```
**Solution**: Check internet connection. The script will continue with locally cached image.

### Not in BWS project
```
❌ Cargo.toml not found!
```
**Solution**: Run the script from the BWS project root directory.

## Integration with CI/CD

These scripts can be integrated into your CI/CD pipeline:

```yaml
# GitHub Actions example
- name: Run Clippy in Docker
  run: |
    chmod +x scripts/docker-clippy.sh
    ./scripts/docker-clippy.sh --verbose
```

```yaml
# GitLab CI example
lint:
  script:
    - chmod +x scripts/docker-clippy.sh
    - ./scripts/docker-clippy.sh
```
