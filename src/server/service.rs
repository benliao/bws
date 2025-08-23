use crate::config::{ServerConfig, SiteConfig};
use crate::handlers::*;
use crate::ssl::SslManager;
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WebServerService {
    config: Arc<RwLock<ServerConfig>>,
    ssl_manager: Option<Arc<SslManager>>,
    static_handler: Arc<StaticFileHandler>,
    api_handler: Arc<ApiHandler>,
    health_handler: Arc<HealthHandler>,
    #[allow(dead_code)]
    proxy_handler: Arc<ProxyHandler>,
}

impl WebServerService {
    pub fn new(config: ServerConfig) -> Self {
        // For now, don't initialize SSL manager synchronously
        // TODO: Initialize SSL manager in background after service starts

        // Initialize handlers
        let static_handler = Arc::new(StaticFileHandler::new());
        let api_handler = Arc::new(ApiHandler::new());
        let health_handler = Arc::new(HealthHandler::new());

        WebServerService {
            config: Arc::new(RwLock::new(config)),
            ssl_manager: None, // Will be initialized later if needed
            static_handler,
            api_handler,
            health_handler,
            proxy_handler: Arc::new(ProxyHandler::new(crate::config::ProxyConfig::default())),
        }
    }

    pub async fn get_config(&self) -> ServerConfig {
        self.config.read().await.clone()
    }

    pub async fn reload_config(
        &self,
        new_config: ServerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate new configuration
        new_config.validate()?;

        // Update configuration
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }

        log::info!("Configuration reloaded successfully");
        Ok(())
    }

    pub async fn ensure_ssl_certificate(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(ssl_manager) = &self.ssl_manager {
            ssl_manager.ensure_certificate(domain).await
        } else {
            Ok(false)
        }
    }

    async fn find_site_by_request(&self, session: &Session) -> Option<SiteConfig> {
        let config = self.config.read().await;

        // Extract host and port information
        let host_header = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        // Parse host and port
        let (hostname, port) = if let Some(pos) = host_header.find(':') {
            let hostname = &host_header[..pos];
            let port_str = &host_header[pos + 1..];
            let port = port_str.parse::<u16>().unwrap_or(8080);
            (hostname, port)
        } else {
            // Default to 8080 if no port specified
            (host_header, 8080)
        };

        config.find_site_by_host_port(hostname, port).cloned()
    }

    async fn handle_ssl_redirect(&self, session: &mut Session, site: &SiteConfig) -> Result<bool> {
        if site.redirect_to_https && !self.is_https_request(session) {
            let https_url = format!(
                "https://{}{}",
                session
                    .req_header()
                    .headers
                    .get("Host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or(&site.hostname),
                session
                    .req_header()
                    .uri
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/")
            );

            let mut header = ResponseHeader::build(301, Some(2))?;
            header.insert_header("Location", https_url)?;
            header.insert_header("Content-Length", "0")?;

            session
                .write_response_header(Box::new(header), true)
                .await?;
            return Ok(true);
        }
        Ok(false)
    }

    fn is_https_request(&self, session: &Session) -> bool {
        // Check if the request is HTTPS
        // This is a simplified check - in production, you might need to check
        // X-Forwarded-Proto header if behind a reverse proxy
        session
            .req_header()
            .uri
            .scheme()
            .map(|s| s.as_str() == "https")
            .unwrap_or(false)
    }

    async fn handle_acme_challenge(&self, session: &mut Session, path: &str) -> Result<bool> {
        if let Some(ssl_manager) = &self.ssl_manager {
            if ssl_manager.handles_acme_challenge(path) {
                if let Some(token) = path.strip_prefix("/.well-known/acme-challenge/") {
                    if let Some(response) = ssl_manager.get_acme_challenge_response(token).await {
                        let mut header = ResponseHeader::build(200, Some(3))?;
                        header.insert_header("Content-Type", "text/plain")?;
                        header.insert_header("Content-Length", response.len().to_string())?;

                        session
                            .write_response_header(Box::new(header), false)
                            .await?;
                        session
                            .write_response_body(Some(response.into_bytes().into()), true)
                            .await?;
                        return Ok(true);
                    }
                }

                // Challenge not found, return 404
                self.handle_404(session, None).await?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn apply_site_headers(
        &self,
        header: &mut ResponseHeader,
        site: &SiteConfig,
    ) -> Result<()> {
        // Apply custom headers from site configuration
        for (key, value) in &site.headers {
            header.insert_header(key.clone(), value.clone())?;
        }

        // Apply security headers
        let config = self.config.read().await;
        for (key, value) in &config.security.security_headers {
            header.insert_header(key.clone(), value.clone())?;
        }

        // Hide server header if configured
        if config.security.hide_server_header {
            header.remove_header("Server");
        } else {
            header.insert_header(
                "Server",
                format!("{}/{}", config.server.name, config.server.version),
            )?;
        }

        Ok(())
    }

    async fn handle_404(&self, session: &mut Session, site: Option<&SiteConfig>) -> Result<()> {
        // Check if site has custom 404 page
        if let Some(site) = site {
            if let Some(error_page) = site.get_error_page(404) {
                let error_page_path = format!("{}/{}", site.static_dir, error_page);
                if let Ok(content) = tokio::fs::read(&error_page_path).await {
                    let mut header = ResponseHeader::build(404, Some(3))?;
                    header.insert_header("Content-Type", "text/html")?;
                    header.insert_header("Content-Length", content.len().to_string())?;
                    self.apply_site_headers(&mut header, site).await?;

                    session
                        .write_response_header(Box::new(header), false)
                        .await?;
                    session
                        .write_response_body(Some(content.into()), true)
                        .await?;
                    return Ok(());
                }
            }
        }

        // Default 404 response
        let error_response = serde_json::json!({
            "error": "Not Found",
            "message": "The requested resource was not found",
            "status": 404
        });

        let response_body = error_response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(404, Some(3))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        if let Some(site) = site {
            self.apply_site_headers(&mut header, site).await?;
        }

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for WebServerService {
    type CTX = Option<SiteConfig>;

    fn new_ctx(&self) -> Self::CTX {
        None
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Since we're handling requests locally, we don't need an upstream peer
        Err(Error::new(ErrorType::InternalError).into_down())
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // Find the matching site configuration
        let site_config = self.find_site_by_request(session).await;
        *ctx = site_config.clone();

        let path = session.req_header().uri.path().to_string();

        // Log the incoming request
        if let Some(site) = ctx.as_ref() {
            log::info!(
                "Incoming request: {} {} (site: {}, static_dir: {})",
                session.req_header().method,
                session.req_header().uri,
                site.name,
                site.static_dir
            );
        } else {
            log::warn!(
                "No site configuration found for request: {} {}",
                session.req_header().method,
                session.req_header().uri
            );
        }

        // Handle HTTPS redirect if configured
        if let Some(site) = ctx.as_ref() {
            if self.handle_ssl_redirect(session, site).await? {
                return Ok(true);
            }
        }

        // Handle ACME challenge requests
        if self.handle_acme_challenge(session, &path).await? {
            return Ok(true);
        }

        // Route request to appropriate handler
        match path.as_str() {
            path if path.starts_with("/api/health") => {
                self.health_handler.handle(session, ctx.as_ref()).await?;
                Ok(true)
            }
            path if path.starts_with("/api/") => {
                self.api_handler.handle(session, ctx.as_ref()).await?;
                Ok(true)
            }
            _ => {
                // Check if site has proxy enabled and route matches
                if let Some(site) = ctx.as_ref() {
                    if site.proxy.enabled {
                        // Check if request matches any proxy routes
                        for route in &site.proxy.routes {
                            if path.starts_with(&route.path) {
                                // Create a temporary proxy handler for this request
                                let proxy_handler = ProxyHandler::new(site.proxy.clone());
                                return proxy_handler
                                    .handle_proxy_request(session, site, &path)
                                    .await;
                            }
                        }
                    }

                    // No proxy route matched, handle as static files
                    self.static_handler.handle(session, site, &path).await?;
                } else {
                    self.handle_404(session, ctx.as_ref()).await?;
                }
                Ok(true)
            }
        }
    }

    async fn connected_to_upstream(
        &self,
        _session: &mut Session,
        _reused: bool,
        _peer: &HttpPeer,
        #[cfg(unix)] _fd: std::os::unix::io::RawFd,
        #[cfg(windows)] _fd: std::os::windows::io::RawSocket,
        _digest: Option<&pingora::protocols::Digest>,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Not used for local serving
        Ok(())
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut pingora::http::RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Not used for local serving
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut pingora::http::ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Not used for local serving
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let config = self.config.read().await;
        if config.logging.log_requests {
            let site_name = ctx.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown");
            let method = session.req_header().method.as_str();
            let uri = session.req_header().uri.to_string();
            let status = session
                .response_written()
                .map(|r| r.status.as_u16())
                .unwrap_or(0);

            log::info!(
                "Request completed: {} {} {} {} (site: {})",
                session
                    .client_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                method,
                uri,
                status,
                site_name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LoggingConfig, PerformanceConfig, SecurityConfig, ServerInfo};
    use std::collections::HashMap;

    fn create_test_config() -> ServerConfig {
        ServerConfig {
            server: ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test server".to_string(),
            },
            sites: vec![SiteConfig {
                name: "test-site".to_string(),
                hostname: "localhost".to_string(),
                port: 8080,
                static_dir: "/tmp/static".to_string(),
                default: true,
                api_only: false,
                headers: HashMap::new(),
                ssl: crate::config::SiteSslConfig::default(),
                redirect_to_https: false,
                index_files: vec!["index.html".to_string()],
                error_pages: HashMap::new(),
                compression: Default::default(),
                cache: Default::default(),
                access_control: Default::default(),
                proxy: crate::config::ProxyConfig::default(),
            }],
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_web_server_service_creation() {
        let config = create_test_config();
        let _service = WebServerService::new(config);
        // Service creation should succeed
    }

    #[tokio::test]
    async fn test_config_reload() {
        let config = create_test_config();
        let service = WebServerService::new(config.clone());

        let mut new_config = config.clone();
        new_config.server.name = "updated-server".to_string();

        let result = service.reload_config(new_config).await;
        assert!(result.is_ok());

        let updated_config = service.get_config().await;
        assert_eq!(updated_config.server.name, "updated-server");
    }
}
