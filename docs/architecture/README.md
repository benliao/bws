# BWS Architecture Overview

## Project Structure

BWS follows a modular, enterprise-grade architecture designed for maintainability, security, and performance.

```
src/
├── bin/                    # Binary executables
│   └── main.rs            # Main server binary
├── core/                  # Core business logic and utilities
│   ├── error/             # Centralized error handling
│   ├── types/             # Common types and constants
│   └── utils/             # Utility functions
├── config/                # Configuration management
│   ├── server.rs          # Server configuration
│   └── site.rs            # Site-specific configuration
├── handlers/              # Request handlers
│   ├── api_handler.rs     # API endpoint handler
│   ├── proxy_handler.rs   # Reverse proxy handler
│   ├── static_handler.rs  # Static file serving
│   └── websocket_proxy.rs # WebSocket proxy
├── middleware/            # HTTP middleware components
│   └── mod.rs             # CORS, security headers, rate limiting
├── monitoring/            # Observability and health checks
│   ├── certificates.rs    # Certificate monitoring
│   ├── health.rs          # Health check endpoints
│   └── metrics.rs         # Metrics collection
├── server/                # Server infrastructure
│   ├── dynamic_tls.rs     # Dynamic TLS handling
│   └── service.rs         # Main server service
├── ssl/                   # SSL/TLS and certificate management
│   ├── acme.rs            # ACME/Let's Encrypt client
│   ├── certificate.rs     # Certificate operations
│   ├── manager.rs         # SSL manager
│   └── renewal.rs         # Certificate renewal logic
└── lib.rs                 # Library root with exports
```

## Core Principles

### 1. **Separation of Concerns**
- **Core**: Foundation types, error handling, utilities
- **Config**: Configuration parsing and validation
- **Handlers**: Request processing logic
- **Middleware**: Cross-cutting concerns (security, logging, etc.)
- **Monitoring**: Observability and health checks
- **Server**: Infrastructure and service orchestration
- **SSL**: Certificate management and TLS handling

### 2. **Error Handling**
- Centralized error types in `core::error`
- Consistent error propagation with `BwsResult<T>`
- Context-aware error messages
- Structured error handling for different error categories

### 3. **Security First**
- Path traversal protection in static file serving
- Input validation and sanitization
- Security headers middleware
- Rate limiting and CORS protection
- Secure certificate management

### 4. **Performance**
- Async/await throughout with Tokio runtime
- Efficient load balancing algorithms
- Connection pooling and resource management
- Optimized static file serving
- Metrics collection with minimal overhead

### 5. **Observability**
- Comprehensive health check endpoints
- Metrics collection and reporting
- Certificate expiration monitoring
- Structured logging integration

## Module Responsibilities

### Core (`src/core/`)
**Purpose**: Foundation layer providing common types, error handling, and utilities.

**Key Components**:
- `error::BwsError`: Centralized error type with context
- `types`: Common enums, structs, and constants
- `utils`: File system, string, time, and network utilities

**Dependencies**: Standard library, minimal external deps

### Config (`src/config/`)
**Purpose**: Configuration parsing, validation, and management.

**Key Components**:
- `ServerConfig`: Global server configuration
- `SiteConfig`: Per-site configuration
- TOML parsing and validation

### Handlers (`src/handlers/`)
**Purpose**: HTTP request processing and response generation.

**Key Components**:
- `StaticFileHandler`: Secure static file serving
- `ApiHandler`: REST API endpoints
- `ProxyHandler`: Reverse proxy functionality
- `WebSocketProxyHandler`: WebSocket proxying

**Security Features**:
- Path traversal protection
- File size limits
- MIME type validation
- Input sanitization

### Middleware (`src/middleware/`)
**Purpose**: Cross-cutting HTTP concerns.

**Key Components**:
- CORS handling
- Security headers
- Rate limiting
- Request/response middleware stack

### Monitoring (`src/monitoring/`)
**Purpose**: Health checks, metrics, and observability.

**Key Components**:
- `HealthHandler`: Health check endpoints
- `MetricsCollector`: Performance metrics
- `CertificateWatcher`: Certificate monitoring

### Server (`src/server/`)
**Purpose**: Server infrastructure and service orchestration.

**Key Components**:
- `WebServerService`: Main Pingora service
- `DynamicTlsHandler`: TLS certificate switching

### SSL (`src/ssl/`)
**Purpose**: SSL/TLS certificate management and ACME integration.

**Key Components**:
- `SslManager`: Certificate lifecycle management
- `AcmeClient`: Let's Encrypt integration
- `CertificateStore`: Certificate storage and retrieval
- `RenewalScheduler`: Automatic certificate renewal

## Data Flow

### 1. **Request Processing**
```
Client Request
    ↓
Server Service (routing)
    ↓
Middleware Stack
    ↓
Handler (static/api/proxy/websocket)
    ↓
Response Generation
    ↓
Middleware Stack (response)
    ↓
Client Response
```

### 2. **Certificate Management**
```
Server Startup
    ↓
SSL Manager Initialization
    ↓
Certificate Loading/Generation
    ↓
Dynamic TLS Handler Setup
    ↓
Certificate Monitoring
    ↓
Automatic Renewal (background)
```

### 3. **Configuration Management**
```
Config File (TOML)
    ↓
Config Parser
    ↓
Validation
    ↓
Server/Site Configuration Objects
    ↓
Service Initialization
    ↓
Runtime Configuration Updates
```

## Security Architecture

### 1. **Input Validation**
- Path sanitization and normalization
- File extension validation
- Size limits and resource constraints
- Null byte injection protection

### 2. **Output Security**
- Security headers (CSP, HSTS, etc.)
- MIME type enforcement
- Content length validation
- Error message sanitization

### 3. **TLS Security**
- Modern TLS versions only
- Secure cipher suites
- Certificate validation
- HSTS enforcement

### 4. **Access Control**
- Rate limiting
- CORS policy enforcement
- Request origin validation
- Resource access controls

## Performance Characteristics

### 1. **Concurrency**
- Tokio async runtime
- Connection pooling
- Non-blocking I/O
- Efficient load balancing

### 2. **Memory Management**
- Streaming file responses
- Bounded resource usage
- Efficient data structures
- Garbage collection friendly

### 3. **Caching**
- Static file caching headers
- Certificate caching
- Configuration caching
- Metrics aggregation

## Extensibility

### 1. **Handler System**
- Pluggable request handlers
- Middleware composition
- Custom response generation
- Error handling integration

### 2. **Configuration**
- TOML-based configuration
- Environment variable support
- Runtime reconfiguration
- Site-specific settings

### 3. **Monitoring**
- Pluggable metrics collectors
- Custom health checks
- Alert integration points
- Logging customization
