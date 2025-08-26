# BWS Examples

This directory contains example configurations and static files for demonstrating BWS features.

## Directory Structure

```
examples/
├── sites/                    # Static file examples for different sites
│   ├── static/              # Main site static files
│   ├── static-api/          # API documentation static files
│   ├── static-blog/         # Blog site static files
│   └── static-dev/          # Development site static files
├── basic-single-site.toml   # Basic configuration examples
├── basic-multi-site.toml
├── load-balancer.toml       # Advanced configurations
├── ssl-acme.toml
├── development.toml
├── production-multi-site.toml
├── microservices-gateway.toml
└── README.md               # This file
```

## Static Sites

### Main Site (`sites/static/`)
The default site example with basic HTML files demonstrating BWS static file serving.

### API Documentation (`sites/static-api/`)
Example API documentation site showing how to serve documentation with proper headers.

### Blog Site (`sites/static-blog/`)
Example blog site demonstrating multi-site configuration.

### Development Site (`sites/static-dev/`)
Development environment example with debug headers and development-specific content.

## Configuration Examples

### Basic Configurations

#### Single Site (`basic-single-site.toml`)
Simple single-site configuration for serving a static website.

#### Multi-Site (`basic-multi-site.toml`)
Multiple sites on different ports with individual configurations.

### Advanced Configurations

#### Load Balancer (`load-balancer.toml`)
BWS configured as a load balancer with multiple upstreams.

#### SSL with ACME (`ssl-acme.toml`)
Automatic SSL certificate management with Let's Encrypt.

#### Development Setup (`development.toml`)
Development-friendly configuration with debugging enabled.

### Production Configurations

#### Production Multi-Site (`production-multi-site.toml`)
Production-ready configuration with security hardening.

#### Microservices Gateway (`microservices-gateway.toml`)
BWS as an API gateway for microservices architecture.

## Usage

### Using Static Site Examples

The static site directories are referenced in the main `config.toml`:

```toml
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "examples/sites/static"
default = true

[[sites]]
name = "blog"
hostname = "blog.localhost"  
port = 8081
static_dir = "examples/sites/static-blog"

# ... more sites
```

### Using Configuration Examples

All example configurations can be validated before use:

```bash
# Validate any example configuration
bws --config examples/basic-single-site.toml --dry-run
bws --config examples/production-multi-site.toml --dry-run

# Copy and customize for your needs
cp examples/basic-single-site.toml config.toml
# Edit config.toml to match your requirements
bws --config config.toml --dry-run && bws --config config.toml
```

### Validation and Testing

All example configurations are automatically tested in CI/CD:

```bash
# Test all example configurations
for config in examples/*.toml; do
  echo "Validating $config..."
  bws --config "$config" --dry-run
done

# Run the validation script
./tests/scripts/validate-configs.sh --examples-only
```

These examples provide starting templates that you can customize for your specific requirements. The `--dry-run` flag ensures your modifications are valid before deployment.
