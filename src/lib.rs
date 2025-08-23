// a function read file content
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub server: ServerInfo,
    pub sites: Vec<SiteConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerInfo {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteConfig {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub static_dir: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub api_only: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl ServerConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn find_site_by_host_port(&self, host: &str, port: u16) -> Option<&SiteConfig> {
        // First try to match both hostname and port
        for site in &self.sites {
            if site.hostname == host && site.port == port {
                return Some(site);
            }
        }

        // Then try to match just the port (for cases where hostname might not match exactly)
        for site in &self.sites {
            if site.port == port {
                return Some(site);
            }
        }

        // Finally, return the default site if no match
        self.sites.iter().find(|site| site.default)
    }
}

pub fn read_file_content(file_path: &str) -> std::io::Result<String> {
    fs::read_to_string(file_path)
}

pub fn read_file_bytes(file_path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(file_path)
}

pub fn get_mime_type(file_path: &str) -> &'static str {
    let path = std::path::Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("woff") | Some("woff2") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("xml") => "application/xml",
        _ => "application/octet-stream",
    }
}

pub struct WebServerService {
    config: ServerConfig,
}

impl WebServerService {
    pub fn new(config: ServerConfig) -> Self {
        WebServerService { config }
    }

    pub fn get_config(&self) -> &ServerConfig {
        &self.config
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
        // This should not be called since we handle everything in request_filter
        Err(Error::new(ErrorType::InternalError).into_down())
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
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

        // Find the matching site configuration and store it in context
        let site_config = self.config.find_site_by_host_port(hostname, port);
        *ctx = site_config.cloned();

        let path = session.req_header().uri.path().to_string();

        // Log the incoming request with site info
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
                "No site configuration found for {}:{}, using default",
                hostname,
                port
            );
        }

        match path.as_str() {
            "/api/health" => {
                self.handle_health(session, ctx.as_ref()).await?;
                Ok(true)
            }
            "/api/file" => {
                self.handle_file_content(session, ctx.as_ref()).await?;
                Ok(true)
            }
            "/api/sites" => {
                self.handle_sites_info(session, ctx.as_ref()).await?;
                Ok(true)
            }
            "/" => {
                // Serve static index.html from the site's static directory
                if let Some(site) = ctx.as_ref() {
                    let file_path = format!("{}/index.html", site.static_dir);
                    self.handle_static_file(session, &file_path, ctx.as_ref())
                        .await?;
                } else {
                    self.handle_404(session, ctx.as_ref()).await?;
                }
                Ok(true)
            }
            path if path.starts_with("/static/") => {
                // Serve static files from the site's static directory
                if let Some(site) = ctx.as_ref() {
                    let file_path = format!("{}{}", site.static_dir, &path[7..]); // Remove "/static" prefix
                    self.handle_static_file(session, &file_path, ctx.as_ref())
                        .await?;
                } else {
                    self.handle_404(session, ctx.as_ref()).await?;
                }
                Ok(true)
            }
            path if path.ends_with(".html") => {
                // Serve HTML files from the site's static directory
                if let Some(site) = ctx.as_ref() {
                    let file_path = format!("{}{}", site.static_dir, path);
                    self.handle_static_file(session, &file_path, ctx.as_ref())
                        .await?;
                } else {
                    self.handle_404(session, ctx.as_ref()).await?;
                }
                Ok(true)
            }
            _ => {
                self.handle_404(session, ctx.as_ref()).await?;
                Ok(true)
            }
        }
    }
}

impl WebServerService {
    /// Apply site-specific headers to the response header
    fn apply_site_headers(
        &self,
        header: &mut ResponseHeader,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        if let Some(site) = site_config {
            for (key, value) in &site.headers {
                header.insert_header(key.clone(), value.clone())?;
            }
        }
        Ok(())
    }

    async fn handle_static_file(
        &self,
        session: &mut Session,
        file_path: &str,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        match read_file_bytes(file_path) {
            Ok(content) => {
                let mime_type = get_mime_type(file_path);
                let mut header = ResponseHeader::build(200, Some(4))?;
                header.insert_header("Content-Type", mime_type)?;
                header.insert_header("Content-Length", content.len().to_string())?;

                // Add cache headers for static files
                if file_path.starts_with("static/") {
                    header.insert_header("Cache-Control", "public, max-age=3600")?;
                }

                // Apply site-specific headers
                self.apply_site_headers(&mut header, site_config)?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(content.into()), true)
                    .await?;
            }
            Err(e) => {
                log::warn!("Failed to read static file {}: {}", file_path, e);

                // Return 404 for missing static files
                let error_response = serde_json::json!({
                    "error": "File Not Found",
                    "message": format!("The requested file '{}' was not found", file_path),
                    "status": 404
                });

                let response_body = error_response.to_string();
                let response_bytes = response_body.into_bytes();
                let mut header = ResponseHeader::build(404, Some(4))?;
                header.insert_header("Content-Type", "application/json")?;
                header.insert_header("Content-Length", response_bytes.len().to_string())?;

                // Apply site-specific headers even for error responses
                self.apply_site_headers(&mut header, site_config)?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(response_bytes.into()), true)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_sites_info(
        &self,
        session: &mut Session,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        let response = serde_json::json!({
            "server": self.config.server.name,
            "sites": self.config.sites.iter().map(|site| serde_json::json!({
                "name": site.name,
                "hostname": site.hostname,
                "port": site.port,
                "static_dir": site.static_dir,
                "default": site.default,
                "api_only": site.api_only,
                "headers": site.headers,
                "url": format!("http://{}:{}", site.hostname, site.port)
            })).collect::<Vec<_>>(),
            "total_sites": self.config.sites.len()
        });

        let response_body = response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(200, Some(4))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        // Apply site-specific headers
        self.apply_site_headers(&mut header, site_config)?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }

    async fn handle_health(
        &self,
        session: &mut Session,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        let response = serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "bws-web-server"
        });

        let response_body = response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(200, Some(4))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        // Apply site-specific headers
        self.apply_site_headers(&mut header, site_config)?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }

    async fn handle_file_content(
        &self,
        session: &mut Session,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        // Extract file path from query parameters
        let query = session.req_header().uri.query().unwrap_or("");
        let file_path = query
            .split('&')
            .find(|param| param.starts_with("path="))
            .and_then(|param| param.split('=').nth(1))
            .unwrap_or("");

        if file_path.is_empty() {
            let error_response = serde_json::json!({
                "error": "Missing 'path' query parameter",
                "example": "/api/file?path=Cargo.toml"
            });

            let response_body = error_response.to_string();
            let response_bytes = response_body.into_bytes();
            let mut header = ResponseHeader::build(400, Some(4))?;
            header.insert_header("Content-Type", "application/json")?;
            header.insert_header("Content-Length", response_bytes.len().to_string())?;

            // Apply site-specific headers
            self.apply_site_headers(&mut header, site_config)?;

            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(response_bytes.into()), true)
                .await?;

            return Ok(());
        }

        match read_file_content(file_path) {
            Ok(content) => {
                let response = serde_json::json!({
                    "file_path": file_path,
                    "content": content,
                    "size": content.len()
                });

                let response_body = response.to_string();
                let response_bytes = response_body.into_bytes();
                let mut header = ResponseHeader::build(200, Some(4))?;
                header.insert_header("Content-Type", "application/json")?;
                header.insert_header("Content-Length", response_bytes.len().to_string())?;

                // Apply site-specific headers
                self.apply_site_headers(&mut header, site_config)?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(response_bytes.into()), true)
                    .await?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to read file: {}", e),
                    "file_path": file_path
                });

                let response_body = error_response.to_string();
                let response_bytes = response_body.into_bytes();
                let mut header = ResponseHeader::build(404, Some(4))?;
                header.insert_header("Content-Type", "application/json")?;
                header.insert_header("Content-Length", response_bytes.len().to_string())?;

                // Apply site-specific headers
                self.apply_site_headers(&mut header, site_config)?;

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(response_bytes.into()), true)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_404(
        &self,
        session: &mut Session,
        site_config: Option<&SiteConfig>,
    ) -> Result<()> {
        let error_response = serde_json::json!({
            "error": "Not Found",
            "message": "The requested endpoint does not exist",
            "available_endpoints": ["/", "/api/health", "/api/file"]
        });

        let response_body = error_response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(404, Some(4))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        // Apply site-specific headers
        self.apply_site_headers(&mut header, site_config)?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }
}
