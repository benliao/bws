use crate::config::site::{ProxyConfig, ProxyRoute, SiteConfig, UpstreamConfig};
use chrono;
use log::{debug, error, info};
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use url::Url;

pub struct ProxyHandler {
    proxy_config: ProxyConfig,
    upstreams: HashMap<String, Vec<UpstreamConfig>>,
    round_robin_counters: HashMap<String, Arc<AtomicUsize>>,
    connection_counts: HashMap<String, Arc<AtomicUsize>>,
}

impl ProxyHandler {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        let mut upstreams = HashMap::new();
        let mut round_robin_counters = HashMap::new();
        let mut connection_counts = HashMap::new();

        // Group upstreams by name
        for upstream in &proxy_config.upstreams {
            upstreams
                .entry(upstream.name.clone())
                .or_insert_with(Vec::new)
                .push(upstream.clone());

            round_robin_counters.insert(upstream.name.clone(), Arc::new(AtomicUsize::new(0)));

            connection_counts.insert(upstream.url.clone(), Arc::new(AtomicUsize::new(0)));
        }

        Self {
            proxy_config,
            upstreams,
            round_robin_counters,
            connection_counts,
        }
    }

    /// Find the appropriate proxy route for a given path
    pub fn find_proxy_route(&self, path: &str) -> Option<&ProxyRoute> {
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

    /// Select an upstream server for a given upstream name
    pub fn select_upstream(&self, upstream_name: &str) -> Result<&UpstreamConfig> {
        let upstream_servers = self
            .upstreams
            .get(upstream_name)
            .ok_or_else(|| Error::new_str("Upstream not found"))?;

        if upstream_servers.is_empty() {
            return Err(Error::new_str("No servers available for upstream"));
        }

        let upstream = match self.proxy_config.load_balancing.method.as_str() {
            "round_robin" => self.select_round_robin(upstream_name, upstream_servers)?,
            "weighted" => self.select_weighted(upstream_servers)?,
            "least_connections" => self.select_least_connections(upstream_servers)?,
            _ => &upstream_servers[0], // Default to first server
        };

        Ok(upstream)
    }

    /// Round-robin load balancing
    fn select_round_robin<'a>(
        &self,
        upstream_name: &str,
        servers: &'a [UpstreamConfig],
    ) -> Result<&'a UpstreamConfig> {
        let counter = self
            .round_robin_counters
            .get(upstream_name)
            .ok_or_else(|| Error::new_str("Round robin counter not found"))?;

        let index = counter.fetch_add(1, Ordering::Relaxed) % servers.len();
        Ok(&servers[index])
    }

    /// Weighted load balancing
    fn select_weighted<'a>(&self, servers: &'a [UpstreamConfig]) -> Result<&'a UpstreamConfig> {
        let total_weight: u32 = servers.iter().map(|s| s.weight).sum();
        if total_weight == 0 {
            return Ok(&servers[0]);
        }

        let random_weight = fastrand::u32(1..=total_weight);
        let mut current_weight = 0;

        for server in servers {
            current_weight += server.weight;
            if random_weight <= current_weight {
                return Ok(server);
            }
        }

        Ok(&servers[0])
    }

    /// Least connections load balancing (uses actual connection tracking)
    fn select_least_connections<'a>(
        &self,
        servers: &'a [UpstreamConfig],
    ) -> Result<&'a UpstreamConfig> {
        // Find the server with the least current connections
        let mut min_connections = usize::MAX;
        let mut selected_server = &servers[0];

        for server in servers {
            let connections = self
                .connection_counts
                .get(&server.url)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0);

            if connections < min_connections {
                min_connections = connections;
                selected_server = server;
            }
        }

        info!(
            "Selected server '{}' with {} connections",
            selected_server.url, min_connections
        );
        Ok(selected_server)
    }

    /// Increment connection count for a server
    fn increment_connections(&self, server_url: &str) {
        if let Some(counter) = self.connection_counts.get(server_url) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Decrement connection count for a server
    fn decrement_connections(&self, server_url: &str) {
        if let Some(counter) = self.connection_counts.get(server_url) {
            counter.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Create HTTP peer for upstream server (simplified)
    pub fn get_upstream_url(&self, upstream: &UpstreamConfig) -> Result<Url> {
        Url::parse(&upstream.url).map_err(|_| Error::new_str("Invalid upstream URL"))
    }

    /// Transform request path according to route configuration
    pub fn transform_path(&self, route: &ProxyRoute, original_path: &str) -> String {
        let target_path = if route.strip_prefix {
            original_path
                .strip_prefix(&route.path)
                .unwrap_or(original_path)
        } else {
            original_path
        };

        if let Some(rewrite_target) = &route.rewrite_target {
            rewrite_target.clone()
        } else {
            format!("/{}", target_path.trim_start_matches('/'))
        }
    }

    /// Add proxy headers to upstream request
    pub fn add_proxy_headers(
        &self,
        req: &mut RequestHeader,
        session: &Session,
        original_host: &str,
    ) {
        if self.proxy_config.headers.add_x_forwarded {
            if let Some(client_addr) = session.client_addr() {
                req.insert_header("X-Forwarded-For", client_addr.to_string())
                    .ok();
            }
        }

        if self.proxy_config.headers.add_x_forwarded {
            let proto = if session.req_header().uri.scheme().map(|s| s.as_str()) == Some("https") {
                "https"
            } else {
                "http"
            };
            req.insert_header("X-Forwarded-Proto", proto).ok();
        }

        if self.proxy_config.headers.add_x_forwarded {
            req.insert_header("X-Forwarded-Host", original_host).ok();
        }

        if self.proxy_config.headers.add_forwarded {
            if let Some(client_addr) = session.client_addr() {
                let proto =
                    if session.req_header().uri.scheme().map(|s| s.as_str()) == Some("https") {
                        "https"
                    } else {
                        "http"
                    };
                let forwarded =
                    format!("for={};proto={};host={}", client_addr, proto, original_host);
                req.insert_header("Forwarded", forwarded).ok();
            }
        }

        // Add custom headers
        for (key, value) in &self.proxy_config.headers.add {
            req.insert_header(key.clone(), value.clone()).ok();
        }
    }

    /// Handle a proxy request for a specific site and path
    pub async fn handle_proxy_request(
        &self,
        session: &mut Session,
        _site: &SiteConfig,
        path: &str,
    ) -> Result<bool> {
        // Find matching route
        if let Some(route) = self.find_proxy_route(path) {
            info!("Proxying request {} to upstream '{}'", path, route.upstream);

            // Select upstream server
            let upstream = match self.select_upstream(&route.upstream) {
                Ok(upstream) => upstream,
                Err(e) => {
                    error!("Failed to select upstream: {}", e);
                    self.send_error_response(session, 502, "Bad Gateway")
                        .await?;
                    return Ok(true);
                }
            };

            // Get upstream URL
            let upstream_url = match self.get_upstream_url(upstream) {
                Ok(url) => url,
                Err(e) => {
                    error!("Failed to parse upstream URL: {}", e);
                    self.send_error_response(session, 502, "Bad Gateway")
                        .await?;
                    return Ok(true);
                }
            };

            // Transform the request path
            let new_path = self.transform_path(route, path);

            // Track connection for load balancing
            self.increment_connections(&upstream.url);

            // Perform the proxy request
            let proxy_result = self
                .proxy_to_upstream(session, &upstream_url, &new_path, route)
                .await;

            // Always decrement connection count when done
            self.decrement_connections(&upstream.url);

            match proxy_result {
                Ok(()) => {
                    info!("Successfully proxied request {} to {}", path, upstream.url);
                    Ok(true)
                }
                Err(e) => {
                    error!("Proxy request failed: {}", e);
                    self.send_error_response(session, 502, "Bad Gateway")
                        .await?;
                    Ok(true)
                }
            }
        } else {
            // No matching proxy route
            Ok(false)
        }
    }

    /// Perform the actual proxy request to upstream
    async fn proxy_to_upstream(
        &self,
        session: &mut Session,
        upstream_url: &Url,
        new_path: &str,
        _route: &ProxyRoute,
    ) -> Result<()> {
        // Create a new HTTP client for the upstream request
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.proxy_config.timeout.read,
            ))
            .build()
            .map_err(|_| Error::new_str("Failed to create HTTP client"))?;

        // Get original host header
        let original_host = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        // Build upstream URL with new path
        let full_upstream_url = format!(
            "{}://{}{}{}",
            upstream_url.scheme(),
            upstream_url.host_str().unwrap_or("localhost"),
            upstream_url
                .port()
                .map(|p| format!(":{}", p))
                .unwrap_or_default(),
            new_path
        );

        debug!("Proxying to upstream URL: {}", full_upstream_url);

        // Create upstream request
        let method = session.req_header().method.clone();
        let mut req_builder = match method.as_str() {
            "GET" => client.get(&full_upstream_url),
            "POST" => client.post(&full_upstream_url),
            "PUT" => client.put(&full_upstream_url),
            "DELETE" => client.delete(&full_upstream_url),
            "PATCH" => client.patch(&full_upstream_url),
            "HEAD" => client.head(&full_upstream_url),
            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &full_upstream_url),
            _ => client.get(&full_upstream_url), // Default to GET
        };

        // Copy headers from original request
        for (name, value) in session.req_header().headers.iter() {
            if let Ok(value_str) = value.to_str() {
                let name_str = name.as_str();
                // Skip host header as we'll set it appropriately
                if name_str.to_lowercase() != "host" {
                    req_builder = req_builder.header(name_str, value_str);
                }
            }
        }

        // Add proxy headers
        let mut temp_header = session.req_header().clone();
        self.add_proxy_headers(&mut temp_header, session, original_host);

        // Copy proxy headers to request
        for (name, value) in temp_header.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                let name_str = name.as_str();
                if name_str.starts_with("X-Forwarded") || name_str == "Forwarded" {
                    req_builder = req_builder.header(name_str, value_str);
                }
            }
        }

        // Read request body if present
        let body = if method.as_str() == "POST"
            || method.as_str() == "PUT"
            || method.as_str() == "PATCH"
        {
            // For now, we'll handle requests without body.
            // Full body proxying would require reading from session.read_request_body()
            Vec::new()
        } else {
            Vec::new()
        };

        if !body.is_empty() {
            req_builder = req_builder.body(body);
        }

        // Send request to upstream
        let response = req_builder
            .send()
            .await
            .map_err(|_| Error::new_str("Upstream request failed"))?;

        // Get response status
        let status = response.status().as_u16();

        // Collect headers before consuming response
        let mut header_map = std::collections::HashMap::new();
        for (name, value) in response.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                header_map.insert(name.as_str().to_string(), value_str.to_string());
            }
        }

        // Get response body (this consumes the response)
        let body = response
            .bytes()
            .await
            .map_err(|_| Error::new_str("Failed to read upstream response"))?;

        // Build response header
        let mut resp_header = ResponseHeader::build(status, Some(4))?;

        // Add collected headers
        for (name, value) in header_map {
            resp_header.insert_header(name, value)?;
        }

        // Send response back to client
        session
            .write_response_header(Box::new(resp_header), false)
            .await?;
        session.write_response_body(Some(body), true).await?;

        Ok(())
    }

    /// Send an error response
    async fn send_error_response(
        &self,
        session: &mut Session,
        status_code: u16,
        message: &str,
    ) -> Result<()> {
        let error_response = serde_json::json!({
            "error": message,
            "status": status_code,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let response_body = error_response.to_string();
        let response_bytes = response_body.into_bytes();
        let mut header = ResponseHeader::build(status_code, Some(3))?;
        header.insert_header("Content-Type", "application/json")?;
        header.insert_header("Content-Length", response_bytes.len().to_string())?;

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(response_bytes.into()), true)
            .await?;

        Ok(())
    }
}
