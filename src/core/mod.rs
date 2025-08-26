//! Core functionality for BWS Web Server
//!
//! This module contains the foundational types, error handling,
//! and utilities used throughout the application.

pub mod cli;
pub mod error;
pub mod hot_reload;
pub mod signals;
pub mod types;
pub mod upgrade;
pub mod utils;

// Re-export commonly used items
pub use cli::BwsCtl;
pub use error::{BwsError, BwsResult, ErrorContext};
pub use hot_reload::{HotReloadManager, ReloadError};
pub use signals::SignalHandler;
pub use types::{
    constants, CacheControl, CompressionAlgorithm, HealthStatus, HttpMethod, LoadBalancingStrategy,
    Priority, RateLimit, SecurityHeaders, SslMode,
};
pub use upgrade::{ServerState, UpgradeError, UpgradeManager};
pub use utils::{fs, net, string, time};
