use crate::config::site::{ProxyConfig, ProxyRoute, UpstreamConfig};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use pingora::http::RequestHeader;
use pingora::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

pub struct WebSocketProxyHandler {
    proxy_config: ProxyConfig,
    upstreams: HashMap<String, Vec<UpstreamConfig>>,
    round_robin_counters: HashMap<String, Arc<AtomicUsize>>,
}

impl WebSocketProxyHandler {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        let mut upstreams = HashMap::new();
        let mut round_robin_counters = HashMap::new();

        // Group upstreams by name
        for upstream in &proxy_config.upstreams {
            upstreams
                .entry(upstream.name.clone())
                .or_insert_with(Vec::new)
                .push(upstream.clone());

            round_robin_counters.insert(upstream.name.clone(), Arc::new(AtomicUsize::new(0)));
        }

        Self {
            proxy_config,
            upstreams,
            round_robin_counters,
        }
    }

    /// Check if a request should be upgraded to WebSocket
    pub fn is_websocket_upgrade_request(req_header: &RequestHeader) -> bool {
        let has_upgrade = req_header
            .headers
            .get("Upgrade")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase() == "websocket")
            .unwrap_or(false);

        let has_connection = req_header
            .headers
            .get("Connection")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase().contains("upgrade"))
            .unwrap_or(false);

        let has_ws_key = req_header.headers.get("Sec-WebSocket-Key").is_some();

        has_upgrade && has_connection && has_ws_key
    }

    /// Find WebSocket proxy route for a given path
    pub fn find_websocket_route(&self, path: &str) -> Option<&ProxyRoute> {
        if !self.proxy_config.enabled {
            return None;
        }

        self.proxy_config
            .routes
            .iter()
            .filter(|route| route.websocket && path.starts_with(&route.path))
            .max_by_key(|route| route.path.len())
    }

    /// Select an upstream server using load balancing
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

    /// Handle WebSocket proxy connection
    pub async fn handle_websocket_proxy(
        &self,
        session: &mut Session,
        path: &str,
    ) -> Result<bool> {
        // Find matching WebSocket route
        if let Some(route) = self.find_websocket_route(path) {
            info!(
                "Proxying WebSocket request {} to upstream '{}'",
                path, route.upstream
            );

            // Select upstream server
            let upstream = match self.select_upstream(&route.upstream) {
                Ok(upstream) => upstream,
                Err(e) => {
                    error!("Failed to select upstream: {}", e);
                    return Ok(false);
                }
            };

            // Convert upstream URL to WebSocket URL
            let ws_url = match self.get_websocket_url(upstream, route, path) {
                Ok(url) => url,
                Err(e) => {
                    error!("Failed to construct WebSocket URL: {}", e);
                    return Ok(false);
                }
            };

            // Handle the WebSocket upgrade and proxy
            match self.proxy_websocket(session, &ws_url).await {
                Ok(()) => {
                    info!("WebSocket proxy completed successfully");
                    Ok(true)
                }
                Err(e) => {
                    error!("WebSocket proxy failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            // No matching WebSocket route
            Ok(false)
        }
    }

    /// Convert HTTP upstream URL to WebSocket URL
    fn get_websocket_url(
        &self,
        upstream: &UpstreamConfig,
        route: &ProxyRoute,
        path: &str,
    ) -> Result<String> {
        let upstream_url = Url::parse(&upstream.url)
            .map_err(|_| Error::new_str("Invalid upstream URL"))?;

        let scheme = match upstream_url.scheme() {
            "http" => "ws",
            "https" => "wss",
            "ws" | "wss" => upstream_url.scheme(),
            _ => return Err(Error::new_str("Unsupported upstream scheme")),
        };

        let target_path = if route.strip_prefix {
            path.strip_prefix(&route.path).unwrap_or(path)
        } else {
            path
        };

        let target_path = if let Some(rewrite_target) = &route.rewrite_target {
            rewrite_target.as_str()
        } else {
            target_path
        };

        let ws_url = format!(
            "{}://{}{}{}",
            scheme,
            upstream_url.host_str().unwrap_or("localhost"),
            upstream_url
                .port()
                .map(|p| format!(":{}", p))
                .unwrap_or_default(),
            target_path
        );

        Ok(ws_url)
    }

    /// Proxy WebSocket connection
    async fn proxy_websocket(&self, session: &mut Session, ws_url: &str) -> Result<()> {
        debug!("Connecting to upstream WebSocket: {}", ws_url);

        // Extract headers from the original request
        let req_header = session.req_header();
        let mut headers = Vec::new();

        // Copy relevant headers for WebSocket handshake
        for (name, value) in req_header.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                let name_str = name.as_str();
                match name_str.to_lowercase().as_str() {
                    "sec-websocket-key"
                    | "sec-websocket-version" 
                    | "sec-websocket-protocol"
                    | "sec-websocket-extensions"
                    | "origin"
                    | "user-agent" => {
                        headers.push((name_str, value_str));
                    }
                    _ => {}
                }
            }
        }

        // Add proxy headers
        let client_addr_string;
        if self.proxy_config.headers.add_x_forwarded {
            if let Some(client_addr) = session.client_addr() {
                client_addr_string = client_addr.to_string();
                headers.push(("X-Forwarded-For", client_addr_string.as_str()));
            }
        }

        // Connect to upstream WebSocket
        let (_upstream_ws, _response) = match self.connect_upstream_websocket(ws_url, headers).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to connect to upstream WebSocket: {}", e);
                return Err(Error::new_str("Upstream WebSocket connection failed"));
            }
        };

        // Get the raw stream from the session
        // Note: This is a simplified approach. In a real implementation,
        // you'd need to handle the WebSocket upgrade properly with Pingora
        warn!("WebSocket proxying requires manual stream handling - this is a placeholder implementation");
        
        // For now, we'll return an error as full implementation requires
        // more complex integration with Pingora's HTTP handling
        Err(Error::new_str("WebSocket proxying not yet fully implemented"))
    }

    /// Connect to upstream WebSocket server
    async fn connect_upstream_websocket(
        &self,
        ws_url: &str,
        _headers: Vec<(&str, &str)>,
    ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response)> {
        // For a proper implementation, we would use tokio_tungstenite::connect_async
        // and handle the headers properly. For now, this is a simplified version.
        
        let _url = Url::parse(ws_url)
            .map_err(|_| Error::new_str("Invalid WebSocket URL"))?;

        // Use connect_async with the URL string
        // In a production implementation, you would want to handle custom headers
        let (ws_stream, response) = tokio_tungstenite::connect_async(ws_url)
            .await
            .map_err(|_| Error::new_str("WebSocket connection failed"))?;

        Ok((ws_stream, response))
    }

    /// Relay messages between client and upstream WebSocket
    async fn relay_websocket_messages(
        client_ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        upstream_ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<()> {
        let (mut client_sink, mut client_stream) = client_ws.split();
        let (mut upstream_sink, mut upstream_stream) = upstream_ws.split();

        // Create two tasks for bidirectional message forwarding
        let client_to_upstream = async {
            while let Some(msg) = client_stream.next().await {
                match msg {
                    Ok(Message::Close(_)) => {
                        debug!("Client WebSocket closed");
                        let _ = upstream_sink.send(Message::Close(None)).await;
                        break;
                    }
                    Ok(msg) => {
                        if let Err(e) = upstream_sink.send(msg).await {
                            error!("Failed to forward message to upstream: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from client WebSocket: {}", e);
                        break;
                    }
                }
            }
        };

        let upstream_to_client = async {
            while let Some(msg) = upstream_stream.next().await {
                match msg {
                    Ok(Message::Close(_)) => {
                        debug!("Upstream WebSocket closed");
                        let _ = client_sink.send(Message::Close(None)).await;
                        break;
                    }
                    Ok(msg) => {
                        if let Err(e) = client_sink.send(msg).await {
                            error!("Failed to forward message to client: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from upstream WebSocket: {}", e);
                        break;
                    }
                }
            }
        };

        // Run both forwarding tasks concurrently
        tokio::select! {
            _ = client_to_upstream => {
                debug!("Client to upstream forwarding completed");
            }
            _ = upstream_to_client => {
                debug!("Upstream to client forwarding completed");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::site::{LoadBalancingConfig, ProxyHeadersConfig, TimeoutConfig};
    use pingora::http::{RequestHeader, Method};
    use std::collections::HashMap;

    fn create_test_config() -> ProxyConfig {
        ProxyConfig {
            enabled: true,
            upstreams: vec![
                UpstreamConfig {
                    name: "websocket_upstream".to_string(),
                    url: "http://localhost:3001".to_string(),
                    weight: 1,
                    max_conns: None,
                },
                UpstreamConfig {
                    name: "websocket_upstream".to_string(),
                    url: "http://localhost:3002".to_string(),
                    weight: 1,
                    max_conns: None,
                },
            ],
            routes: vec![
                ProxyRoute {
                    path: "/ws".to_string(),
                    upstream: "websocket_upstream".to_string(),
                    strip_prefix: true,
                    rewrite_target: None,
                    websocket: true,
                },
                ProxyRoute {
                    path: "/api".to_string(),
                    upstream: "websocket_upstream".to_string(),
                    strip_prefix: false,
                    rewrite_target: None,
                    websocket: false,
                },
            ],
            health_check: Default::default(),
            load_balancing: LoadBalancingConfig {
                method: "round_robin".to_string(),
                sticky_sessions: false,
            },
            timeout: TimeoutConfig {
                connect: 10,
                read: 30,
                write: 30,
            },
            headers: ProxyHeadersConfig {
                preserve_host: true,
                add_forwarded: true,
                add_x_forwarded: true,
                remove: vec![],
                add: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_websocket_upgrade_detection() {
        let mut req = RequestHeader::build(Method::GET, b"/ws", None).unwrap();
        
        // Missing headers - should not be WebSocket
        assert!(!WebSocketProxyHandler::is_websocket_upgrade_request(&req));

        // Add WebSocket headers
        req.insert_header("Upgrade", "websocket").unwrap();
        req.insert_header("Connection", "Upgrade").unwrap();
        req.insert_header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==").unwrap();

        // Now should be detected as WebSocket
        assert!(WebSocketProxyHandler::is_websocket_upgrade_request(&req));
    }

    #[test]
    fn test_websocket_route_detection() {
        let proxy_config = create_test_config();
        let handler = WebSocketProxyHandler::new(proxy_config);

        // Should find WebSocket route
        assert!(handler.find_websocket_route("/ws").is_some());
        assert!(handler.find_websocket_route("/ws/chat").is_some());

        // Should not find WebSocket route for HTTP-only route
        assert!(handler.find_websocket_route("/api").is_none());

        // Should not find route for non-matching path
        assert!(handler.find_websocket_route("/other").is_none());
    }

    #[test]
    fn test_websocket_url_construction() {
        let proxy_config = create_test_config();
        let handler = WebSocketProxyHandler::new(proxy_config);

        let upstream = &UpstreamConfig {
            name: "test".to_string(),
            url: "http://localhost:3001".to_string(),
            weight: 1,
            max_conns: None,
        };

        let route = &ProxyRoute {
            path: "/ws".to_string(),
            upstream: "test".to_string(),
            strip_prefix: true,
            rewrite_target: None,
            websocket: true,
        };

        let ws_url = handler.get_websocket_url(upstream, route, "/ws/chat").unwrap();
        assert_eq!(ws_url, "ws://localhost:3001/chat");

        // Test with HTTPS upstream
        let https_upstream = &UpstreamConfig {
            name: "test".to_string(),
            url: "https://localhost:3001".to_string(),
            weight: 1,
            max_conns: None,
        };

        let wss_url = handler.get_websocket_url(https_upstream, route, "/ws/chat").unwrap();
        assert_eq!(wss_url, "wss://localhost:3001/chat");
    }

    #[test]
    fn test_upstream_selection() {
        let proxy_config = create_test_config();
        let handler = WebSocketProxyHandler::new(proxy_config);

        // Test round-robin selection
        let upstream1 = handler.select_upstream("websocket_upstream").unwrap();
        let upstream2 = handler.select_upstream("websocket_upstream").unwrap();

        // Should alternate between upstreams
        assert_ne!(upstream1.url, upstream2.url);
    }
}
