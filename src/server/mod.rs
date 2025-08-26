//! Server infrastructure for BWS Web Server
//!
//! This module contains the core server components including
//! the main service, dynamic TLS handling, server orchestration,
//! and the secure management API service.

pub mod config_reload;
pub mod dynamic_tls;
pub mod management_api;
pub mod reload_trait;
pub mod service;

// Re-export main types
pub use config_reload::ConfigReloadService;
pub use dynamic_tls::DynamicTlsHandler;
pub use management_api::ManagementApiService;
pub use reload_trait::ConfigReloadable;
pub use service::WebServerService;
