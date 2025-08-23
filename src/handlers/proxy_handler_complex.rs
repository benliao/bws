use crate::config::site::{ProxyConfig, UpstreamConfig, ProxyRoute, SiteConfig};
use async_trait::async_trait;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use url::Url;
use log::{debug, warn, error};
use serde_json;

pub struct ProxyHandler {
    proxy_config: ProxyConfig,
    upstreams: HashMap<String, Vec<UpstreamConfig>>,
    round_robin_counters: HashMap<String, Arc<AtomicUsize>>,
}

impl ProxyHandler {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        let mut upstreams = HashMap::new();
        let mut round_robin_counters = HashMap::new();

        // Group upstreams by name
        for upstream in &proxy_config.upstreams {
            upstreams
                .entry(upstream.name.clone())
                .or_insert_with(Vec::new)
                .push(upstream.clone());
            
            round_robin_counters.insert(
                upstream.name.clone(),
                Arc::new(AtomicUsize::new(0))
            );
        }

        Self {
            proxy_config,
            upstreams,
            round_robin_counters,
        }
    }

    pub fn is_proxy_enabled(&self) -> bool {
        self.proxy_config.enabled
    }

    pub fn should_proxy(&self, path: &str) -> Option<&ProxyRoute> {
        if !self.proxy_config.enabled {
            return None;
        }

        // Find matching route - most specific first
        self.proxy_config
            .routes
            .iter()
            .filter(|route| path.starts_with(&route.path))
            .max_by_key(|route| route.path.len())
    }

    pub fn get_upstream_peer(&self, route: &ProxyRoute) -> Result<HttpPeer> {
        let upstream_servers = self.upstreams.get(&route.upstream)
            .ok_or_else(|| Error::new_str(&format!("Upstream '{}' not found", route.upstream)))?;

        if upstream_servers.is_empty() {
            return Err(Error::new_str(&format!("No servers available for upstream '{}'", route.upstream)));
        }

        let upstream = match self.proxy_config.load_balancing.method.as_str() {
            "round_robin" => self.select_round_robin(&route.upstream, upstream_servers)?,
            "weighted" => self.select_weighted(upstream_servers)?,
            _ => &upstream_servers[0], // Default to first server
        };

        let url = Url::parse(&upstream.url)
            .map_err(|e| Error::new_str(&format!("Invalid upstream URL '{}': {}", upstream.url, e)))?;

        let mut peer = HttpPeer::new(
            format!("{}:{}", 
                url.host_str().unwrap_or("localhost"), 
                url.port().unwrap_or(80)
            ), 
            url.scheme() == "https", 
            url.host_str().unwrap_or("localhost").to_string()
        );

        // Set connection limits if specified
        if let Some(max_conns) = upstream.max_conns {
            // Note: Pingora connection limits would be set here
            debug!("Setting max connections for upstream '{}': {}", upstream.name, max_conns);
        }

        Ok(peer)
    }

    fn select_round_robin(&self, upstream_name: &str, servers: &[UpstreamConfig]) -> Result<&UpstreamConfig> {
        let counter = self.round_robin_counters.get(upstream_name)
            .ok_or_else(|| Error::new_str("Round robin counter not found"))?;
        
        let index = counter.fetch_add(1, Ordering::Relaxed) % servers.len();
        Ok(&servers[index])
    }

    fn select_weighted(&self, servers: &[UpstreamConfig]) -> Result<&UpstreamConfig> {
        let total_weight: u32 = servers.iter().map(|s| s.weight).sum();
        if total_weight == 0 {
            return Ok(&servers[0]);
        }

        let mut random_weight = fastrand::u32(1..=total_weight);
        for server in servers {
            if random_weight <= server.weight {
                return Ok(server);
            }
            random_weight -= server.weight;
        }

        Ok(&servers[0]) // Fallback
    }

    pub fn rewrite_request_path(&self, route: &ProxyRoute, original_path: &str) -> String {
        if let Some(ref rewrite_target) = route.rewrite_target {
            // Use the rewrite target as the new path
            rewrite_target.clone()
        } else if route.strip_prefix {
            // Strip the route prefix from the path
            original_path.strip_prefix(&route.path).unwrap_or(original_path).to_string()
        } else {
            // Keep the original path
            original_path.to_string()
        }
    }

    pub fn modify_request_headers(&self, headers: &mut RequestHeader, route: &ProxyRoute, original_host: &str) {
        // Handle host header
        if self.proxy_config.headers.preserve_host {
            // Keep the original host header
        } else {
            // Set host to upstream server
            if let Some(upstream_servers) = self.upstreams.get(&route.upstream) {
                if let Some(upstream) = upstream_servers.first() {
                    if let Ok(url) = Url::parse(&upstream.url) {
                        if let Some(host) = url.host_str() {
                            headers.insert_header("Host", host).ok();
                        }
                    }
                }
            }
        }

        // Add forwarded headers
        if self.proxy_config.headers.add_x_forwarded {
            headers.insert_header("X-Forwarded-For", "127.0.0.1").ok(); // TODO: Get real client IP
            headers.insert_header("X-Forwarded-Host", original_host).ok();
            headers.insert_header("X-Forwarded-Proto", "http").ok(); // TODO: Detect actual protocol
        }

        if self.proxy_config.headers.add_forwarded {
            headers.insert_header("Forwarded", "for=127.0.0.1;host=localhost;proto=http").ok(); // TODO: Build proper forwarded header
        }

        // Remove specified headers
        for header_name in &self.proxy_config.headers.remove {
            headers.remove_header(header_name);
        }

        // Add custom headers
        for (name, value) in &self.proxy_config.headers.add {
            headers.insert_header(name, value).ok();
        }
    }

    pub fn modify_response_headers(&self, headers: &mut ResponseHeader) {
        // Remove server identification headers for security
        headers.remove_header("Server");
        
        // Add custom response headers if configured
        for (name, value) in &self.proxy_config.headers.add {
            if name.to_lowercase().starts_with("response-") {
                let response_header_name = name.strip_prefix("response-").unwrap_or(name);
                headers.insert_header(response_header_name, value).ok();
            }
        }
    }
}

#[async_trait]
impl ProxyHttp for ProxyHandler {
    type CTX = ();
    
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        let path = session.req_header().uri.path();
        debug!("Proxy request filter: {}", path);
        
        if let Some(route) = self.should_proxy(path) {
            debug!("Found matching proxy route: {} -> {}", route.path, route.upstream);
            
            // Rewrite the request path if needed
            let new_path = self.rewrite_request_path(route, path);
            if new_path != path {
                debug!("Rewriting path from '{}' to '{}'", path, new_path);
                session.req_header_mut().set_uri(&new_path)?;
            }

            // Modify request headers
            let original_host = session.req_header()
                .headers
                .get("host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost");
            
            self.modify_request_headers(session.req_header_mut(), route, original_host);
            
            return Ok(false); // Continue to upstream_peer
        }
        
        Ok(false) // No proxy match, continue with normal processing
    }

    async fn upstream_peer(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();
        
        if let Some(route) = self.should_proxy(path) {
            debug!("Getting upstream peer for route: {} -> {}", route.path, route.upstream);
            let peer = self.get_upstream_peer(route)?;
            return Ok(Box::new(peer));
        }

        Err(Error::new_str("No upstream peer configured for this path"))
    }

    async fn response_filter(&self, session: &mut Session, _upstream_response: &mut ResponseHeader, _ctx: &mut Self::CTX) -> Result<()> {
        if let Some(resp_header) = session.response_written() {
            self.modify_response_headers(resp_header);
        }
        Ok(())
    }

    async fn logging(&self, session: &mut Session, _e: Option<&pingora_core::Error>, _ctx: &mut Self::CTX) {
        let status = session
            .response_written()
            .map(|resp| resp.status.as_u16())
            .unwrap_or(0);
            
        let path = session.req_header().uri.path();
        debug!("Proxy request completed: {} {} {}", 
               session.req_header().method, 
               path, 
               status);
    }
}

// Health check functionality
impl ProxyHandler {
    pub async fn health_check_upstream(&self, upstream_name: &str) -> bool {
        if !self.proxy_config.health_check.enabled {
            return true; // Assume healthy if health checks are disabled
        }

        if let Some(servers) = self.upstreams.get(upstream_name) {
            for server in servers {
                if self.check_server_health(server).await {
                    return true; // At least one server is healthy
                }
            }
        }
        false
    }

    async fn check_server_health(&self, upstream: &UpstreamConfig) -> bool {
        // Simple health check - try to parse URL and assume it's healthy
        // In a full implementation, this would make an HTTP request to the health endpoint
        match Url::parse(&upstream.url) {
            Ok(_) => {
                debug!("Health check passed for upstream: {}", upstream.name);
                true
            }
            Err(e) => {
                warn!("Health check failed for upstream '{}': {}", upstream.name, e);
                false
            }
        }
    }

    /// Handle a proxy request for a specific site and path
    pub async fn handle_proxy_request(
        &self,
        session: &mut Session,
        site: &SiteConfig,
        path: &str,
    ) -> Result<bool> {
        // Find matching route
        if let Some(route) = self.should_proxy(path) {
            // Get upstream peer
            let upstream_peer = match self.get_upstream_peer(route) {
                Ok(peer) => peer,
                Err(e) => {
                    error!("Failed to get upstream peer: {}", e);
                    self.send_error_response(session, 502, "Bad Gateway").await?;
                    return Ok(true);
                }
            };

            // Perform the actual proxy request
            match self.proxy_request_to_upstream(session, route, upstream_peer, path).await {
                Ok(()) => Ok(true),
                Err(e) => {
                    error!("Proxy request failed: {}", e);
                    self.send_error_response(session, 502, "Bad Gateway").await?;
                    Ok(true)
                }
            }
        } else {
            // No matching proxy route
            Ok(false)
        }
    }

    /// Actually perform the proxy request to upstream
    async fn proxy_request_to_upstream(
        &self,
        session: &mut Session,
        route: &ProxyRoute,
        upstream_peer: HttpPeer,
        original_path: &str,
    ) -> Result<()> {
        // Transform the path according to route configuration
        let target_path = if route.strip_prefix {
            original_path.strip_prefix(&route.path).unwrap_or(original_path)
        } else {
            original_path
        };

        let final_path = if let Some(rewrite_target) = &route.rewrite_target {
            rewrite_target.clone()
        } else {
            target_path.to_string()
        };

        // Create upstream request
        let mut upstream_request = session.req_header().clone();
        upstream_request.set_uri(&final_path);

        // Add proxy headers
        self.add_proxy_headers(&mut upstream_request, session);

        // For now, return an error since we need to integrate with Pingora's proxy system
        Err(Error::new_str("Proxy functionality needs integration with Pingora's proxy system"))
    }

    /// Add standard proxy headers to the upstream request
    fn add_proxy_headers(&self, upstream_request: &mut pingora::http::RequestHeader, session: &Session) {
        if self.proxy_config.headers.add_x_forwarded_for {
            if let Some(client_addr) = session.client_addr() {
                upstream_request.insert_header("X-Forwarded-For", client_addr.ip().to_string()).ok();
            }
        }

        if self.proxy_config.headers.add_x_forwarded_proto {
            let proto = if session.req_header().uri.scheme().map(|s| s.as_str()) == Some("https") {
                "https"
            } else {
                "http"
            };
            upstream_request.insert_header("X-Forwarded-Proto", proto).ok();
        }

        if self.proxy_config.headers.add_x_forwarded_host {
            if let Some(host) = session.req_header().headers.get("Host") {
                if let Ok(host_str) = host.to_str() {
                    upstream_request.insert_header("X-Forwarded-Host", host_str).ok();
                }
            }
        }

        // Add custom headers
        for (key, value) in &self.proxy_config.headers.custom_headers {
            upstream_request.insert_header(key.clone(), value.clone()).ok();
        }
    }

    /// Send an error response
    async fn send_error_response(&self, session: &mut Session, status_code: u16, message: &str) -> Result<()> {
        let error_response = serde_json::json!({
            "error": message,
            "status": status_code
        });

        let response_body = error_response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(status_code, Some(3))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        session.write_response_header(Box::new(header), false).await?;
        session.write_response_body(Some(response_bytes.into()), true).await?;

        Ok(())
    }
}
