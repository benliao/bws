# Introduction

**BWS** is a high-performance, multi-site web server built with [Pingora](https://github.com/cloudflare/pingora), Cloudflare's battle-tested proxy framework.

## What is BWS?

BWS hosts multiple websites on different ports with individual configurations. It combines Pingora's reliability and performance with comprehensive error handling, automatic SSL management, and production-grade features.

## Key Features

- **Multi-Site Support**: Host multiple websites with individual configurations
- **Reverse Proxy & Load Balancing**: Full proxy functionality with multiple algorithms
- **SSL/TLS Support**: Automatic and manual certificate management
- **High Performance**: Built on Pingora for enterprise-grade reliability
- **Security Focused**: Zero-panic policy with comprehensive error handling
- **Container Ready**: Docker support with multi-architecture images
- **Hot Reload**: Zero-downtime configuration updates

## Use Cases

- **Development environments**: Quick multi-site testing
- **Static site hosting**: Efficient multi-website serving
- **Reverse proxy**: Load balancing to backend services
- **Microservices**: Service routing and load balancing
- **API gateways**: Centralized API entry point

## Architecture

Built on Cloudflare's Pingora framework:
- Battle-tested reliability (handles millions of requests)
- High performance with low latency
- Memory safety with Rust
- Modern networking (HTTP/2, HTTP/3 ready)

## Getting Started

Start with the [Installation](./installation.md) guide or jump to [Quick Start](./quick-start.md) for immediate setup.
