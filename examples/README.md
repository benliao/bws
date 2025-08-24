# BWS Configuration Examples

This directory contains various configuration examples for different use cases.

## Basic Configurations

### Single Site (`basic-single-site.toml`)
Simple single-site configuration for serving a static website.

### Multi-Site (`basic-multi-site.toml`)
Multiple sites on different ports with individual configurations.

## Advanced Configurations

### Load Balancer (`load-balancer.toml`)
BWS configured as a load balancer with multiple upstreams.

### SSL with ACME (`ssl-acme.toml`)
Automatic SSL certificate management with Let's Encrypt.

### Development Setup (`development.toml`)
Development-friendly configuration with debugging enabled.

## Production Configurations

### Production Multi-Site (`production-multi-site.toml`)
Production-ready configuration with security hardening.

### Microservices Gateway (`microservices-gateway.toml`)
BWS as an API gateway for microservices architecture.

## Usage

Copy any example configuration and modify it according to your needs:

```bash
cp examples/basic-single-site.toml config.toml
# Edit config.toml
./target/release/bws --config config.toml
```
