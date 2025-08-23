use crate::config::site::{ProxyConfig, SiteConfig};
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use serde_json;

pub struct ProxyHandler {
    proxy_config: ProxyConfig,
}

impl ProxyHandler {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        Self { proxy_config }
    }

    /// Handle a proxy request for a specific site and path
    pub async fn handle_proxy_request(
        &self,
        session: &mut Session,
        _site: &SiteConfig,
        path: &str,
    ) -> Result<bool> {
        // Check if we should proxy this request
        if self.should_proxy(path) {
            // For now, just return a message indicating proxy is configured but not yet implemented
            self.send_proxy_placeholder_response(session).await?;
            Ok(true)
        } else {
            // No matching proxy route
            Ok(false)
        }
    }

    /// Check if a path should be proxied
    fn should_proxy(&self, path: &str) -> bool {
        if !self.proxy_config.enabled {
            return false;
        }

        // Check if path matches any proxy routes
        self.proxy_config
            .routes
            .iter()
            .any(|route| path.starts_with(&route.path))
    }

    /// Send a placeholder response indicating proxy is detected but not yet implemented
    async fn send_proxy_placeholder_response(&self, session: &mut Session) -> Result<()> {
        let response = serde_json::json!({
            "message": "Reverse proxy route detected",
            "status": "This request would be proxied to upstream servers",
            "note": "Proxy functionality is configured but full implementation is pending"
        });

        let response_body = response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(200, Some(3))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        session.write_response_header(Box::new(header), false).await?;
        session.write_response_body(Some(response_bytes.into()), true).await?;

        Ok(())
    }
}
