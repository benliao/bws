//! Server infrastructure for BWS Web Server
//!
//! This module contains the core server components including
//! the main service, dynamic TLS handling, and server orchestration.

pub mod dynamic_tls;
pub mod service;

// Re-export main types
pub use dynamic_tls::DynamicTlsHandler;
pub use service::WebServerService;
