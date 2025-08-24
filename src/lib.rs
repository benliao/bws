//! BWS (Ben's Web Server) - A high-performance multi-site web server
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

// Legacy compatibility exports for external crates
use std::fs;

/// Read file contents as bytes
///
/// # Errors
///
/// Returns an IO error if the file cannot be read.
pub fn read_file_bytes(file_path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(file_path)
}

/// Get MIME type for a file extension
///
/// # Arguments
///
/// * `file_path` - The file path to determine MIME type for
///
/// # Returns
///
/// Returns the MIME type string for the file extension
#[must_use]
pub fn get_mime_type(file_path: &str) -> &'static str {
    // Use the new utils module for MIME type detection
    let path = std::path::Path::new(file_path);
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    core::utils::fs::get_mime_type(extension)
}
