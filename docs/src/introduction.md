# Introduction

Welcome to **BWS (Ben's Web Server)** - a high-performance, multi-site web server built with [Pingora](https://github.com/cloudflare/pingora), Cloudflare's battle-tested proxy framework.

## What is BWS?

BWS is designed to be a simple yet powerful web server that can host multiple websites on different ports with individual configurations. It combines the reliability and performance of Cloudflare's Pingora framework with an easy-to-use configuration system.

## Key Features

- **🌐 Multi-Site Support**: Host multiple websites on different ports with individual configurations
- **⚡ High Performance**: Built on Pingora for enterprise-grade performance and reliability  
- **🔧 Configurable Headers**: Set custom HTTP headers per site via TOML configuration
- **📊 Health Monitoring**: Built-in health check endpoints for monitoring
- **🔒 Security Focused**: Comprehensive security auditing and dependency management
- **🐳 Container Ready**: Docker images with multi-architecture support
- **📁 Static File Serving**: Efficient serving of static files with proper MIME types
- **🚀 Easy Deployment**: Simple configuration and deployment options

## Use Cases

BWS is perfect for:

- **Development environments** - Quickly spin up multiple sites for testing
- **Static site hosting** - Serve multiple static websites efficiently
- **Microservice frontends** - Host different frontend applications on different ports
- **Content delivery** - Serve static assets with proper caching and headers
- **Prototyping** - Rapid deployment of web applications

## Architecture

BWS is built on top of Cloudflare's Pingora framework, which provides:

- **Battle-tested reliability** - Used by Cloudflare to handle millions of requests
- **High performance** - Optimized for low latency and high throughput
- **Memory safety** - Built in Rust for security and stability
- **Modern networking** - HTTP/2, HTTP/3 ready

## Getting Started

Ready to get started? Head over to the [Installation](./installation.md) guide to begin using BWS!
