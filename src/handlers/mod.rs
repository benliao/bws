//! Request handlers for BWS Web Server
//!
//! This module contains all the request handlers for different types
//! of content and functionality.

pub mod api_handler;
pub mod proxy_handler;
pub mod static_handler;
pub mod websocket_proxy;

// Re-export handler types
pub use api_handler::ApiHandler;
pub use proxy_handler::ProxyHandler;
pub use static_handler::StaticFileHandler;
pub use websocket_proxy::WebSocketProxyHandler;
