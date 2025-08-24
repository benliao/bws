//! Core functionality for BWS Web Server
//!
//! This module contains the foundational types, error handling,
//! and utilities used throughout the application.

pub mod error;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use error::{BwsError, BwsResult, ErrorContext};
pub use types::{
    constants, CacheControl, CompressionAlgorithm, HealthStatus, HttpMethod,
    LoadBalancingStrategy, Priority, RateLimit, SecurityHeaders, SslMode,
};
pub use utils::{fs, net, string, time};
