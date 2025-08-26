//! BWS (Blazing Web Server) - A high-performance multi-site web server
//!
//! BWS is built with the Pingora framework and provides enterprise-grade
//! features including SSL/TLS management, load balancing, and security.

pub mod config;
pub mod core;
pub mod handlers;
pub mod middleware;
pub mod monitoring;
pub mod server;
pub mod ssl;

// Re-export main types for convenience
pub use config::{ServerConfig, SiteConfig};
pub use core::{BwsError, BwsResult};
pub use monitoring::{CertificateWatcher, HealthHandler};
pub use server::WebServerService;
pub use ssl::{AcmeConfig, SslManager};
