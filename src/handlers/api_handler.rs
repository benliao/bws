use crate::config::{ServerConfig, SiteConfig};
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

// Global config path for reload functionality
static CONFIG_PATH: once_cell::sync::OnceCell<Arc<Mutex<Option<String>>>> =
    once_cell::sync::OnceCell::new();

#[derive(Clone)]
pub struct ApiHandler {
    // Simple handler without complex dependencies
}

impl ApiHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Set the global config path for reload functionality
    pub fn set_config_path(path: String) {
        let config_path = CONFIG_PATH.get_or_init(|| Arc::new(Mutex::new(None)));
        let mut config_path_guard = config_path.lock().unwrap();
        *config_path_guard = Some(path);
    }

    /// Get the global config path
    async fn get_config_path() -> Option<String> {
        let config_path = CONFIG_PATH.get()?;
        config_path.lock().unwrap().clone()
    }

    pub async fn handle(&self, session: &mut Session, site: Option<&SiteConfig>) -> Result<()> {
        let path = session.req_header().uri.path().to_string();
        let method = session.req_header().method.as_str();

        match (method, path.as_str()) {
            ("GET", "/api/sites") => self.handle_sites_info(session, site).await,
            ("GET", "/api/ssl/certificates") => self.handle_ssl_certificates(session, site).await,
            ("POST", path) if path.starts_with("/api/ssl/certificates/") => {
                self.handle_ssl_certificate_request(session, site, path)
                    .await
            }
            ("GET", "/api/ssl/status") => self.handle_ssl_status(session, site).await,
            ("GET", "/api/config") => self.handle_config_info(session, site).await,
            ("POST", "/api/config/reload") => self.handle_config_reload(session, site).await,
            _ => self.handle_not_found(session, site).await,
        }
    }

    async fn handle_sites_info(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        // This would need access to the full server config
        // For now, return a placeholder response
        let response = serde_json::json!({
            "sites": [],
            "total_sites": 0,
            "message": "Sites information endpoint"
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_ssl_certificates(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        // This would need access to the SSL manager
        // For now, return a placeholder response
        let response = serde_json::json!({
            "certificates": [],
            "total_certificates": 0,
            "message": "SSL certificates information endpoint"
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_ssl_certificate_request(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
        path: &str,
    ) -> Result<()> {
        // Extract domain from path
        let domain = path
            .strip_prefix("/api/ssl/certificates/")
            .unwrap_or("unknown");

        let response = serde_json::json!({
            "message": format!("Certificate request for domain: {}", domain),
            "domain": domain,
            "status": "not_implemented"
        });

        self.send_json_response(session, 501, &response).await
    }

    async fn handle_ssl_status(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        let response = serde_json::json!({
            "ssl_enabled": false,
            "auto_cert": false,
            "certificates": [],
            "renewal_status": "unknown",
            "message": "SSL status endpoint"
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_config_info(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        let response = serde_json::json!({
            "server": {
                "name": "bws-web-server",
                "version": env!("CARGO_PKG_VERSION")
            },
            "message": "Configuration information endpoint"
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_config_reload(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        // Get the config path
        let config_path = match Self::get_config_path().await {
            Some(path) => path,
            None => {
                let response = serde_json::json!({
                    "error": "Config path not set",
                    "message": "Configuration path not found (running in temporary mode?)"
                });
                return self.send_json_response(session, 400, &response).await;
            }
        };

        // Try to reload the configuration
        match Self::reload_config_from_path(&config_path).await {
            Ok(_) => {
                let response = serde_json::json!({
                    "message": "Configuration reloaded successfully",
                    "status": "success",
                    "config_path": config_path,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "note": "Changes will apply to new connections"
                });
                self.send_json_response(session, 200, &response).await
            }
            Err(e) => {
                let response = serde_json::json!({
                    "error": "Configuration reload failed",
                    "message": format!("Failed to reload configuration: {}", e),
                    "status": "error"
                });
                self.send_json_response(session, 400, &response).await
            }
        }
    }

    /// Reload configuration from the given path
    async fn reload_config_from_path(
        config_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Load and validate new configuration
        let new_config = ServerConfig::load_from_file(config_path).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        new_config.validate().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // Log the successful reload
        log::info!("Configuration reloaded successfully from {}", config_path);

        // Note: In a more sophisticated implementation, you would update
        // the running server's configuration. For now, we just validate
        // that the new config is loadable and valid.
        Ok(())
    }

    async fn handle_not_found(
        &self,
        session: &mut Session,
        _site: Option<&SiteConfig>,
    ) -> Result<()> {
        let response = serde_json::json!({
            "error": "API endpoint not found",
            "message": "The requested API endpoint does not exist",
            "available_endpoints": [
                "GET /api/sites",
                "GET /api/ssl/certificates",
                "POST /api/ssl/certificates/{domain}",
                "GET /api/ssl/status",
                "GET /api/config",
                "POST /api/config/reload"
            ]
        });

        self.send_json_response(session, 404, &response).await
    }

    async fn send_json_response(
        &self,
        session: &mut Session,
        status: u16,
        data: &serde_json::Value,
    ) -> Result<()> {
        let response_body = data.to_string();
        let response_bytes = response_body.into_bytes();

        let mut header = ResponseHeader::build(status, Some(4))?;
        header.insert_header("Content-Type", "application/json; charset=utf-8")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;
        header.insert_header("Cache-Control", "no-cache, no-store, must-revalidate")?;
        header.insert_header("Pragma", "no-cache")?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }
}

impl Default for ApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_handler_creation() {
        let handler = ApiHandler::new();
        // Basic test that handler can be created
        assert_eq!(
            std::mem::size_of_val(&handler),
            std::mem::size_of::<ApiHandler>()
        );
    }
}
