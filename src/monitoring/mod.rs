//! Monitoring and observability for BWS Web Server
//!
//! This module provides health checks, metrics collection,
//! certificate monitoring, and logging functionality.

pub mod certificates;
pub mod health;
pub mod metrics;

// Re-export main types
pub use certificates::CertificateWatcher;
pub use health::HealthHandler;
