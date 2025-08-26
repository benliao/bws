//! Management API Service for BWS Web Server
//!
//! This module provides a secure management API service that runs on localhost only.
//! It handles administrative operations like configuration reload with proper security checks.

use crate::config::ManagementConfig;
use crate::handlers::ApiHandler;
use crate::server::WebServerService;
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::sync::Arc;

/// Management API Service with localhost-only security
#[derive(Clone)]
pub struct ManagementApiService {
    api_handler: ApiHandler,
    config: ManagementConfig,
}

impl ManagementApiService {
    /// Create a new Management API service
    pub fn new(_web_service: Arc<WebServerService>, config: ManagementConfig) -> Self {
        Self {
            api_handler: ApiHandler::new(),
            config,
        }
    }

    /// Check if the request is from localhost
    fn is_localhost_request(&self, session: &Session) -> bool {
        if let Some(client_addr) = session.client_addr() {
            if let Some(socket_addr) = client_addr.as_inet() {
                let ip = socket_addr.ip();
                return ip.is_loopback();
            }
        }
        false
    }

    /// Check API key authentication
    fn check_api_key(&self, session: &Session) -> bool {
        if let Some(expected_key) = &self.config.api_key {
            if let Some(provided_key) = session.get_header("X-API-Key") {
                if let Ok(key_str) = provided_key.to_str() {
                    return key_str == expected_key;
                }
            }
            false
        } else {
            true // No API key configured, allow access
        }
    }

    /// Send JSON error response
    async fn send_error_response(
        &self,
        session: &mut Session,
        status: u16,
        message: &str,
    ) -> Result<()> {
        let error_body = format!(r#"{{"error": "{}"}}"#, message);
        let mut header = ResponseHeader::build(status, Some(4))?;
        header.insert_header("Content-Type", "application/json; charset=utf-8")?;
        header.insert_header("Content-Length", error_body.len().to_string())?;
        header.insert_header("Cache-Control", "no-cache, no-store, must-revalidate")?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(error_body.into_bytes().into()), true)
            .await?;

        Ok(())
    }

    /// Send JSON success response
    async fn send_success_response(&self, session: &mut Session, message: &str) -> Result<()> {
        let success_body = format!(r#"{{"message": "{}"}}"#, message);
        let mut header = ResponseHeader::build(200, Some(4))?;
        header.insert_header("Content-Type", "application/json; charset=utf-8")?;
        header.insert_header("Content-Length", success_body.len().to_string())?;
        header.insert_header("Cache-Control", "no-cache, no-store, must-revalidate")?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(success_body.into_bytes().into()), true)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for ManagementApiService {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Management API doesn't proxy to upstream
        Err(Error::new(ErrorType::InternalError).into_down())
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        // Security check: only allow localhost requests
        if !self.is_localhost_request(session) {
            log::warn!(
                "Management API access denied: request not from localhost ({})",
                session
                    .client_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
            self.send_error_response(session, 403, "Access denied: localhost only")
                .await?;
            return Ok(true);
        }

        // API key authentication check
        if !self.check_api_key(session) {
            log::warn!("Management API access denied: invalid or missing API key");
            self.send_error_response(session, 401, "Unauthorized: invalid API key")
                .await?;
            return Ok(true);
        }

        let path = session.req_header().uri.path();
        let method = session.req_header().method.as_str();

        match (method, path) {
            ("POST", "/api/config/reload") => {
                log::info!("Management API: Config reload requested");

                // Delegate to the main API handler for the actual reload logic
                match self.api_handler.handle(session, None).await {
                    Ok(_) => {
                        log::info!("Configuration reloaded successfully via management API");
                        self.send_success_response(session, "Configuration reloaded successfully")
                            .await?;
                    }
                    Err(e) => {
                        log::error!("Configuration reload failed via management API: {}", e);
                        self.send_error_response(session, 500, "Configuration reload failed")
                            .await?;
                        return Ok(true);
                    }
                }
                Ok(true)
            }
            _ => {
                // Unknown endpoint
                self.send_error_response(session, 404, "Endpoint not found")
                    .await?;
                Ok(true)
            }
        }
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut pingora::http::RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Not used for management API
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut pingora::http::ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Not used for management API
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) {
        if let Some(client_addr) = session.client_addr() {
            log::info!(
                "Management API request: {} {} from {}",
                session.req_header().method,
                session.req_header().uri.path(),
                client_addr
            );
        }
    }
}
