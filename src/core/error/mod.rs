//! Core error types and handling for BWS Web Server
//!
//! This module provides centralized error handling with proper error types,
//! context, and formatting for the entire application.

use std::fmt;
use std::io;

/// The main error type for BWS operations
#[derive(Debug)]
pub enum BwsError {
    /// IO related errors (file operations, network, etc.)
    Io(io::Error),

    /// Configuration related errors
    Config(String),

    /// SSL/TLS certificate errors
    Certificate(String),

    /// HTTP request/response errors
    Http(String),

    /// Proxy operation errors
    Proxy(String),

    /// Authentication/authorization errors
    Auth(String),

    /// Internal server errors
    Internal(String),

    /// Validation errors for input data
    Validation(String),

    /// Resource not found errors
    NotFound(String),

    /// Rate limiting errors
    RateLimit(String),
}

impl fmt::Display for BwsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwsError::Io(err) => write!(f, "IO error: {}", err),
            BwsError::Config(msg) => write!(f, "Configuration error: {}", msg),
            BwsError::Certificate(msg) => write!(f, "Certificate error: {}", msg),
            BwsError::Http(msg) => write!(f, "HTTP error: {}", msg),
            BwsError::Proxy(msg) => write!(f, "Proxy error: {}", msg),
            BwsError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            BwsError::Internal(msg) => write!(f, "Internal error: {}", msg),
            BwsError::Validation(msg) => write!(f, "Validation error: {}", msg),
            BwsError::NotFound(msg) => write!(f, "Not found: {}", msg),
            BwsError::RateLimit(msg) => write!(f, "Rate limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for BwsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BwsError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for BwsError {
    fn from(err: io::Error) -> Self {
        BwsError::Io(err)
    }
}

impl From<pingora::Error> for BwsError {
    fn from(err: pingora::Error) -> Self {
        BwsError::Http(format!("Pingora error: {}", err))
    }
}

/// Result type alias for BWS operations
pub type BwsResult<T> = Result<T, BwsError>;

/// Error context extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> BwsResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<BwsError>,
{
    fn with_context<F>(self, f: F) -> BwsResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();

            match base_error {
                BwsError::Internal(msg) => BwsError::Internal(format!("{}: {}", context, msg)),
                other => BwsError::Internal(format!("{}: {}", context, other)),
            }
        })
    }
}

/// Macro for creating configuration errors
#[macro_export]
macro_rules! config_error {
    ($($arg:tt)*) => {
        BwsError::Config(format!($($arg)*))
    };
}

/// Macro for creating certificate errors
#[macro_export]
macro_rules! cert_error {
    ($($arg:tt)*) => {
        BwsError::Certificate(format!($($arg)*))
    };
}

/// Macro for creating HTTP errors
#[macro_export]
macro_rules! http_error {
    ($($arg:tt)*) => {
        BwsError::Http(format!($($arg)*))
    };
}

/// Macro for creating validation errors
#[macro_export]
macro_rules! validation_error {
    ($($arg:tt)*) => {
        BwsError::Validation(format!($($arg)*))
    };
}
