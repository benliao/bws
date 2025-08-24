use crate::config::{ServerConfig, SiteConfig};
use crate::handlers::*;
use crate::monitoring::HealthHandler;
use crate::ssl::SslManager;
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct WebServerService {
    config: Arc<RwLock<ServerConfig>>,
    ssl_managers: Arc<RwLock<HashMap<String, Arc<SslManager>>>>, // hostname -> SslManager
    static_handler: Arc<StaticFileHandler>,
    api_handler: Arc<ApiHandler>,
    health_handler: Arc<HealthHandler>,
    #[allow(dead_code)]
    proxy_handler: Arc<ProxyHandler>,
}

impl WebServerService {
    pub fn new(config: ServerConfig) -> Self {
        // Initialize handlers
        let static_handler = Arc::new(StaticFileHandler::new());
        let api_handler = Arc::new(ApiHandler::new());
        let health_handler = Arc::new(HealthHandler::new());

        // Initialize SSL managers storage
        let ssl_managers = Arc::new(RwLock::new(HashMap::new()));

        WebServerService {
            config: Arc::new(RwLock::new(config)),
            ssl_managers,
            static_handler,
            api_handler,
            health_handler,
            proxy_handler: Arc::new(ProxyHandler::new(crate::config::ProxyConfig::default())),
        }
    }

    /// Initialize SSL managers for all sites with SSL enabled
    pub async fn initialize_ssl_managers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        let mut ssl_managers = self.ssl_managers.write().await;

        for site in &config.sites {
            if site.ssl.enabled {
                log::info!(
                    "Initializing SSL manager for site: {} ({})",
                    site.name,
                    site.hostname
                );

                match SslManager::from_site_config(site).await {
                    Ok(Some(ssl_manager)) => {
                        // Initialize certificate for this domain
                        match ssl_manager.ensure_certificate(&site.hostname).await {
                            Ok(success) => {
                                if success {
                                    log::info!("SSL certificate ready for {}", site.hostname);
                                } else {
                                    log::warn!(
                                        "Failed to obtain SSL certificate for {}",
                                        site.hostname
                                    );
                                }
                            }
                            Err(e) => {
                                log::error!("SSL certificate error for {}: {}", site.hostname, e);
                            }
                        }

                        ssl_managers.insert(site.hostname.clone(), Arc::new(ssl_manager));
                    }
                    Ok(None) => {
                        log::debug!("SSL not enabled for site: {}", site.name);
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to initialize SSL manager for {}: {}",
                            site.hostname,
                            e
                        );
                    }
                }
            }
        }

        log::info!("SSL managers initialized for {} sites", ssl_managers.len());

        // Start automatic certificate renewal scheduler
        let acme_managers_count = ssl_managers
            .values()
            .filter(|manager| manager.is_auto_cert_enabled())
            .count();

        if acme_managers_count > 0 {
            log::info!(
                "Starting certificate renewal monitoring for {} ACME-enabled domains",
                acme_managers_count
            );
            
            // Start background certificate renewal monitoring
            for (domain, manager) in ssl_managers.iter() {
                if manager.is_auto_cert_enabled() {
                    let manager_clone = manager.clone();
                    let domain_clone = domain.clone();
                    tokio::spawn(async move {
                        // Check certificates every hour
                        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
                        loop {
                            interval.tick().await;
                            if let Err(e) = manager_clone.check_and_renew_certificate(&domain_clone).await {
                                log::error!("Certificate renewal check failed for {domain_clone}: {e}");
                            }
                        }
                    });
                }
            }
        }

        Ok(())
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
        let ssl_managers = self.ssl_managers.read().await;
        if let Some(ssl_manager) = ssl_managers.get(domain) {
            ssl_manager.ensure_certificate(domain).await
        } else {
            Ok(false)
        }
    }

    /// Check and renew certificates for all SSL-enabled sites
    pub async fn check_and_renew_certificates(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ssl_managers = self.ssl_managers.read().await;
        let mut renewed_any = false;

        for (domain, ssl_manager) in ssl_managers.iter() {
            if ssl_manager.is_auto_cert_enabled() {
                log::debug!("Checking certificate renewal for domain: {}", domain);

                match ssl_manager.check_and_renew_certificate(domain).await {
                    Ok(renewed) => {
                        if renewed {
                            log::info!("Certificate renewed for domain: {}", domain);
                            renewed_any = true;
                        } else {
                            log::debug!(
                                "Certificate for {} is still valid, no renewal needed",
                                domain
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to check/renew certificate for {}: {}", domain, e);
                    }
                }
            }
        }

        if renewed_any {
            log::info!("🔄 Some certificates were renewed. Server restart may be required for HTTPS to use new certificates.");
        }

        Ok(())
    }

    async fn get_ssl_manager_for_domain(&self, domain: &str) -> Option<Arc<SslManager>> {
        let ssl_managers = self.ssl_managers.read().await;
        ssl_managers.get(domain).cloned()
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
            // Default to port 80 for HTTP or 443 for HTTPS based on scheme
            let default_port = if session.req_header().uri.scheme_str() == Some("https") {
                443
            } else {
                80
            };
            (host_header, default_port)
        };

        // First try exact hostname:port match
        if let Some(site) = config.find_site_by_host_port(hostname, port).cloned() {
            return Some(site);
        }

        // For ACME challenge requests on port 80, find any site with the same hostname that has ACME enabled
        let path = session.req_header().uri.path();
        if port == 80 && path.starts_with("/.well-known/acme-challenge/") {
            // Look for any site with this hostname that has ACME enabled
            for site in &config.sites {
                if site.hostname == hostname && site.ssl.enabled && site.ssl.auto_cert {
                    if let Some(acme_config) = &site.ssl.acme {
                        if acme_config.enabled {
                            log::debug!(
                                "Using site '{}' for ACME challenge on port 80 for hostname '{}'",
                                site.name,
                                hostname
                            );
                            return Some(site.clone());
                        }
                    }
                }
            }

            // If no exact hostname match, try to find any ACME-enabled site (for wildcard scenarios)
            log::debug!(
                "Looking for any ACME-enabled site for challenge request to '{}'",
                hostname
            );
            for site in &config.sites {
                if site.ssl.enabled && site.ssl.auto_cert {
                    if let Some(acme_config) = &site.ssl.acme {
                        if acme_config.enabled {
                            log::debug!(
                                "Using ACME-enabled site '{}' for challenge request",
                                site.name
                            );
                            return Some(site.clone());
                        }
                    }
                }
            }
        }

        None
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

    async fn handle_acme_challenge_for_site(
        &self,
        session: &mut Session,
        path: &str,
        site: &SiteConfig,
    ) -> Result<bool> {
        // Check if this is an ACME challenge request
        if !path.starts_with("/.well-known/acme-challenge/") {
            return Ok(false);
        }

        log::debug!(
            "ACME challenge handler started for site '{}', ssl.enabled={}, ssl.auto_cert={}",
            site.name,
            site.ssl.enabled,
            site.ssl.auto_cert
        );

        // Check if the site has ACME enabled
        if !site.ssl.enabled || !site.ssl.auto_cert {
            log::warn!("ACME challenge request for site '{}' but SSL auto_cert not enabled (ssl.enabled={}, ssl.auto_cert={})", 
                      site.name, site.ssl.enabled, site.ssl.auto_cert);

            let mut header = ResponseHeader::build(404, Some(3))?;
            header.insert_header("Content-Type", "application/json")?;
            header.insert_header("X-Site-Name", &site.hostname)?;
            header.insert_header("X-ACME-Enabled", "false")?;
            header.insert_header("X-ACME-Challenge-Status", "ssl-not-enabled")?;
            header.insert_header("X-ACME-Site", &site.name)?;

            let error_body = format!(
                r#"{{"error":"ACME Not Enabled","message":"Site {} does not have SSL auto_cert enabled","status":404,"site":"{}","hostname":"{}"}}"#,
                site.name, site.name, site.hostname
            );
            header.insert_header("Content-Length", error_body.len().to_string())?;

            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(error_body.into_bytes().into()), true)
                .await?;
            return Ok(true);
        }

        let acme_enabled = site
            .ssl
            .acme
            .as_ref()
            .map(|acme| acme.enabled)
            .unwrap_or(false);

        log::debug!(
            "ACME config check for site '{}': acme_config_exists={}, acme_enabled={}",
            site.name,
            site.ssl.acme.is_some(),
            acme_enabled
        );

        if !acme_enabled {
            log::warn!("ACME challenge request for site '{}' but ACME not enabled in config (acme_config_exists={}, acme_enabled={})", 
                      site.name, site.ssl.acme.is_some(), acme_enabled);

            let mut header = ResponseHeader::build(404, Some(3))?;
            header.insert_header("Content-Type", "application/json")?;
            header.insert_header("X-Site-Name", &site.hostname)?;
            header.insert_header("X-ACME-Enabled", "false")?;
            header.insert_header("X-ACME-Challenge-Status", "acme-disabled")?;
            header.insert_header("X-ACME-Site", &site.name)?;

            let error_body = format!(
                r#"{{"error":"ACME Disabled","message":"Site {} has ACME disabled in configuration","status":404,"site":"{}","hostname":"{}"}}"#,
                site.name, site.name, site.hostname
            );
            header.insert_header("Content-Length", error_body.len().to_string())?;

            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(error_body.into_bytes().into()), true)
                .await?;
            return Ok(true);
        }

        log::info!(
            "Handling ACME challenge request for site '{}': {}",
            site.name,
            path
        );
        let start_time = std::time::Instant::now();

        // For ACME challenges, prioritize filesystem access to avoid blocking on SSL manager initialization
        if let Some(token) = path.strip_prefix("/.well-known/acme-challenge/") {
            log::debug!(
                "Looking for ACME challenge token: {} (took {:?})",
                token,
                start_time.elapsed()
            );

            // First try reading challenge file directly from filesystem (fastest path)
            let challenge_path = std::path::PathBuf::from("./certs")
                .join("challenges")
                .join(".well-known")
                .join("acme-challenge")
                .join(token);

            log::debug!("Trying to read challenge file from: {:?}", challenge_path);

            if let Ok(content) = tokio::fs::read_to_string(&challenge_path).await {
                log::info!(
                    "Serving ACME challenge response from filesystem for token: {} (took {:?})",
                    token,
                    start_time.elapsed()
                );
                let mut header = ResponseHeader::build(200, Some(3))?;
                header.insert_header("Content-Type", "text/plain")?;
                header.insert_header("Content-Length", content.len().to_string())?;

                // Add ACME-specific headers for debugging
                header.insert_header("X-Site-Name", &site.hostname)?;
                header.insert_header("X-ACME-Enabled", "true")?;
                header.insert_header("X-ACME-Challenge-Source", "filesystem")?;
                header.insert_header("X-ACME-Site", &site.name)?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(content.into_bytes().into()), true)
                    .await?;
                log::info!(
                    "Successfully sent ACME challenge response for token: {} (took {:?})",
                    token,
                    start_time.elapsed()
                );
                return Ok(true);
            } else {
                log::debug!(
                    "Challenge file not found at {:?}, checking SSL manager (took {:?})",
                    challenge_path,
                    start_time.elapsed()
                );
            }
        }

        // Fallback to SSL manager (only if filesystem lookup failed)
        if let Some(ssl_manager) = self.get_ssl_manager_for_domain(&site.hostname).await {
            log::debug!(
                "Found SSL manager for domain '{}' (took {:?})",
                site.hostname,
                start_time.elapsed()
            );

            if ssl_manager.handles_acme_challenge(path) {
                if let Some(token) = path.strip_prefix("/.well-known/acme-challenge/") {
                    log::debug!(
                        "Looking for ACME challenge token in SSL manager: {} (took {:?})",
                        token,
                        start_time.elapsed()
                    );

                    // Try to get challenge response from SSL manager
                    if let Some(response) = ssl_manager.get_acme_challenge_response(token).await {
                        log::info!("Serving ACME challenge response from SSL manager for token: {} (took {:?})", token, start_time.elapsed());
                        let mut header = ResponseHeader::build(200, Some(3))?;
                        header.insert_header("Content-Type", "text/plain")?;
                        header.insert_header("Content-Length", response.len().to_string())?;

                        // Add ACME-specific headers for debugging
                        header.insert_header("X-Site-Name", &site.hostname)?;
                        header.insert_header("X-ACME-Enabled", "true")?;
                        header.insert_header("X-ACME-Challenge-Source", "ssl-manager")?;
                        header.insert_header("X-ACME-Site", &site.name)?;

                        session
                            .write_response_header(Box::new(header), false)
                            .await?;
                        session
                            .write_response_body(Some(response.into_bytes().into()), true)
                            .await?;
                        log::info!(
                            "Successfully sent ACME challenge response for token: {} (took {:?})",
                            token,
                            start_time.elapsed()
                        );
                        return Ok(true);
                    } else {
                        log::debug!(
                            "SSL manager has no challenge response for token: {} (took {:?})",
                            token,
                            start_time.elapsed()
                        );
                    }
                }
            } else {
                log::warn!(
                    "SSL manager does not handle ACME challenges for path: {} (took {:?})",
                    path,
                    start_time.elapsed()
                );
            }
        } else {
            log::debug!(
                "No SSL manager found for hostname: {} (took {:?})",
                site.hostname,
                start_time.elapsed()
            );
        }

        // Challenge not found, return 404 with ACME debugging headers
        log::warn!(
            "ACME challenge not found for path: {} (took {:?})",
            path,
            start_time.elapsed()
        );

        let mut header = ResponseHeader::build(404, Some(3))?;
        header.insert_header("Content-Type", "application/json")?;

        // Add ACME debugging headers
        header.insert_header("X-Site-Name", &site.hostname)?;
        header.insert_header("X-ACME-Enabled", "true")?;
        header.insert_header("X-ACME-Challenge-Status", "not-found")?;
        header.insert_header("X-ACME-Site", &site.name)?;

        let error_body = format!(
            r#"{{"error":"ACME Challenge Not Found","message":"Challenge token not found for site {}","status":404,"site":"{}","hostname":"{}"}}"#,
            site.name, site.name, site.hostname
        );
        header.insert_header("Content-Length", error_body.len().to_string())?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(error_body.into_bytes().into()), true)
            .await?;
        Ok(true)
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
        let host_header = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        // Log the incoming request
        if let Some(site) = ctx.as_ref() {
            log::info!(
                "Incoming request: {} {} (site: {}, static_dir: {}, host: {})",
                session.req_header().method,
                session.req_header().uri,
                site.name,
                site.static_dir,
                host_header
            );
        } else {
            log::warn!(
                "No site configuration found for request: {} {} (host: {})",
                session.req_header().method,
                session.req_header().uri,
                host_header
            );
        }

        // Handle HTTPS redirect if configured
        if let Some(site) = ctx.as_ref() {
            if self.handle_ssl_redirect(session, site).await? {
                return Ok(true);
            }
        }

        // Handle ACME challenge requests
        if path.starts_with("/.well-known/acme-challenge/") {
            log::debug!(
                "ACME challenge request detected: {} (site found: {})",
                path,
                ctx.is_some()
            );
            if let Some(site) = ctx.as_ref() {
                log::debug!(
                    "Calling handle_acme_challenge_for_site for site '{}'",
                    site.name
                );
                if self
                    .handle_acme_challenge_for_site(session, &path, site)
                    .await?
                {
                    return Ok(true);
                }
                log::debug!(
                    "handle_acme_challenge_for_site returned false for site '{}'",
                    site.name
                );
            } else {
                // ACME challenge request but no site found
                log::warn!(
                    "ACME challenge request but no site configuration found: {}",
                    path
                );
                let mut header = ResponseHeader::build(404, Some(3))?;
                header.insert_header("Content-Type", "application/json")?;
                header.insert_header("X-ACME-Challenge-Status", "no-site-found")?;
                header.insert_header("X-ACME-Enabled", "false")?;

                let error_body = r#"{"error":"No Site Found","message":"No site configuration found for ACME challenge","status":404}"#;
                header.insert_header("Content-Length", error_body.len().to_string())?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(error_body.as_bytes().into()), true)
                    .await?;
                return Ok(true);
            }
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
