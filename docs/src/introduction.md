# Introduction

Welcome to **BWS (Blazing Web Server)** - a production-ready, high-performance, multi-site web server built with [Pingora](https://github.com/cloudflare/pingora), Cloudflare's battle-tested proxy framework.

## What is BWS?

BWS is designed to be a robust yet easy-to-use web server that can host multiple websites on different ports with individual configurations. It combines the reliability and performance of Cloudflare's Pingora framework with comprehensive error handling, automatic SSL management, and production-grade reliability.

## Production-Ready Features

BWS has been extensively hardened for production deployment:

- **Zero Panic Policy**: No `.unwrap()` calls - all errors handled gracefully
- **Automatic SSL Renewal**: Background certificate monitoring with robust error handling
- **Code Quality**: Zero Clippy warnings for maximum code quality
- **Thread-Safe Operations**: All operations safe for concurrent access
- **Comprehensive Logging**: Structured logging with detailed error documentation
- **Resource Management**: Proper cleanup of connections and certificate operations

## Key Features

- **Multi-Site Support**: Host multiple websites on different ports with individual configurations
- **Reverse Proxy & Load Balancing**: Full reverse proxy functionality with multiple load balancing algorithms
- **High Performance**: Built on Pingora for enterprise-grade performance and reliability  
- **SSL/TLS Support**: Per-site SSL configuration with automatic and manual certificates
- **Configurable Headers**: Set custom HTTP headers per site via TOML configuration
- **Health Monitoring**: Built-in health check endpoints for monitoring
- **Security Focused**: Comprehensive security auditing and dependency management
- **Container Ready**: Docker images with multi-architecture support
- **Static File Serving**: Efficient serving of static files with proper MIME types
- **Easy Deployment**: Simple configuration and deployment options

## Use Cases

BWS is perfect for:

- **Development environments** - Quickly spin up multiple sites for testing
- **Static site hosting** - Serve multiple static websites efficiently
- **Reverse proxy setups** - Load balance and proxy requests to backend services
- **Microservice architectures** - Route and load balance between microservices
- **API gateways** - Centralized entry point for multiple API services
- **Content delivery** - Serve static assets with proper caching and headers
- **High availability setups** - Distribute load across multiple backend servers
- **Prototyping** - Rapid deployment of web applications with backend integration

## Architecture

BWS is built on top of Cloudflare's Pingora framework, which provides:

- **Battle-tested reliability** - Used by Cloudflare to handle millions of requests
- **High performance** - Optimized for low latency and high throughput
- **Memory safety** - Built in Rust for security and stability
- **Modern networking** - HTTP/2, HTTP/3 ready

## Getting Started

Ready to get started? Head over to the [Installation](./installation.md) guide to begin using BWS!

## Architecture

BWS is built on top of Cloudflare's Pingora framework, which provides:

- **Battle-tested reliability** - Used by Cloudflare to handle millions of requests
- **High performance** - Optimized for low latency and high throughput
- **Memory safety** - Built in Rust for security and stability
- **Modern networking** - HTTP/2, HTTP/3 ready

## Getting Started

Ready to get started? Head over to the [Installation](./installation.md) guide to begin using BWS!
