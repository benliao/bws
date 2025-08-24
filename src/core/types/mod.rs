//! Core types and constants for BWS Web Server
//!
//! This module contains common types, constants, and data structures
//! used throughout the application.

use std::time::Duration;

/// HTTP methods supported by BWS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Trace,
    Connect,
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "PATCH" => Ok(HttpMethod::Patch),
            "TRACE" => Ok(HttpMethod::Trace),
            "CONNECT" => Ok(HttpMethod::Connect),
            _ => Err(format!("Unknown HTTP method: {}", method)),
        }
    }
}

impl HttpMethod {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
        }
    }
}

/// SSL/TLS configuration modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SslMode {
    /// No SSL/TLS encryption
    None,
    /// Manual certificate configuration
    Manual,
    /// Automatic certificate with ACME/Let's Encrypt
    Auto,
    /// Development mode with self-signed certificates
    Development,
}

/// Load balancing strategies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least connections
    LeastConnections,
    /// IP hash based routing
    IpHash,
    /// Random selection
    Random,
}

/// Request processing priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Content compression algorithms
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Deflate,
    Brotli,
    Zstd,
}

/// Cache control directives
#[derive(Debug, Clone)]
pub struct CacheControl {
    pub max_age: Option<Duration>,
    pub no_cache: bool,
    pub no_store: bool,
    pub must_revalidate: bool,
    pub public: bool,
    pub private: bool,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            max_age: Some(Duration::from_secs(3600)), // 1 hour default
            no_cache: false,
            no_store: false,
            must_revalidate: false,
            public: true,
            private: false,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
    /// Burst allowance
    pub burst: Option<u32>,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: Some(10),
        }
    }
}

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    pub content_security_policy: Option<String>,
    pub strict_transport_security: Option<String>,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: bool,
    pub x_xss_protection: bool,
    pub referrer_policy: Option<String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            content_security_policy: Some("default-src 'self'".to_string()),
            strict_transport_security: Some("max-age=31536000; includeSubDomains".to_string()),
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: true,
            x_xss_protection: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
        }
    }
}

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Application constants
pub mod constants {
    use std::time::Duration;

    /// Default HTTP port
    pub const DEFAULT_HTTP_PORT: u16 = 8080;

    /// Default HTTPS port
    pub const DEFAULT_HTTPS_PORT: u16 = 8443;

    /// Default buffer size for file operations
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;

    /// Maximum file size for static file serving (10MB)
    pub const MAX_STATIC_FILE_SIZE: u64 = 10 * 1024 * 1024;

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default read timeout
    pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default write timeout
    pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(60);

    /// Certificate renewal threshold (30 days)
    pub const CERT_RENEWAL_THRESHOLD: Duration = Duration::from_secs(30 * 24 * 60 * 60);

    /// Health check interval
    pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Maximum number of concurrent connections per upstream
    pub const DEFAULT_MAX_CONNECTIONS: usize = 100;

    /// BWS version for headers
    pub const BWS_VERSION: &str = env!("CARGO_PKG_VERSION");

    /// BWS user agent
    pub const BWS_USER_AGENT: &str = concat!("BWS/", env!("CARGO_PKG_VERSION"));
}
