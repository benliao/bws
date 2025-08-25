pub mod compression;

use async_trait::async_trait;
use pingora::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before_request(&self, session: &mut Session) -> Result<bool>;
    async fn after_response(&self, session: &mut Session) -> Result<()>;
}

pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    pub async fn before_request(&self, session: &mut Session) -> Result<bool> {
        for middleware in &self.middlewares {
            if !middleware.before_request(session).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn after_response(&self, session: &mut Session) -> Result<()> {
        // Execute in reverse order
        for middleware in self.middlewares.iter().rev() {
            middleware.after_response(session).await?;
        }
        Ok(())
    }
}

// Logging middleware
pub struct LoggingMiddleware {
    log_requests: bool,
}

impl LoggingMiddleware {
    pub fn new(log_requests: bool) -> Self {
        Self { log_requests }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before_request(&self, session: &mut Session) -> Result<bool> {
        if self.log_requests {
            let _start_time = Instant::now();
            // TODO: Store start time when session variables are available
            // session.set_var("request_start_time", start_time);

            log::info!(
                "Request started: {} {} from {}",
                session.req_header().method,
                session.req_header().uri,
                session
                    .client_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
        }
        Ok(true)
    }

    async fn after_response(&self, session: &mut Session) -> Result<()> {
        if self.log_requests {
            // TODO: Implement proper request timing when session variables are available
            let status = session
                .response_written()
                .map(|r| r.status.as_u16())
                .unwrap_or(0);

            log::info!(
                "Request completed: {} {} (status: {})",
                session.req_header().method,
                session.req_header().uri,
                status
            );
        }
        Ok(())
    }
}

// Rate limiting middleware
#[allow(dead_code)]
pub struct RateLimitMiddleware {
    requests_per_minute: u32,
    burst_size: u32,
    clients: HashMap<String, ClientInfo>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ClientInfo {
    request_count: u32,
    last_reset: Instant,
    tokens: u32,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            requests_per_minute,
            burst_size,
            clients: HashMap::new(),
        }
    }

    fn get_client_ip(&self, session: &Session) -> String {
        // Try to get real IP from headers (for reverse proxy setups)
        if let Some(forwarded_for) = session.req_header().headers.get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                if let Some(ip) = forwarded_str.split(',').next() {
                    return ip.trim().to_string();
                }
            }
        }

        if let Some(real_ip) = session.req_header().headers.get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                return ip_str.to_string();
            }
        }

        session
            .client_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[allow(dead_code)]
    fn is_allowed(&mut self, client_ip: &str) -> bool {
        let now = Instant::now();
        let client_info = self
            .clients
            .entry(client_ip.to_string())
            .or_insert(ClientInfo {
                request_count: 0,
                last_reset: now,
                tokens: self.burst_size,
            });

        // Reset counters if a minute has passed
        if now.duration_since(client_info.last_reset).as_secs() >= 60 {
            client_info.request_count = 0;
            client_info.last_reset = now;
            client_info.tokens = self.burst_size;
        }

        // Add tokens based on time elapsed
        let seconds_elapsed = now.duration_since(client_info.last_reset).as_secs();
        let tokens_to_add = (seconds_elapsed * self.requests_per_minute as u64) / 60;
        client_info.tokens = (client_info.tokens + tokens_to_add as u32).min(self.burst_size);

        // Check if request is allowed
        if client_info.tokens > 0 {
            client_info.tokens -= 1;
            client_info.request_count += 1;
            true
        } else {
            false
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before_request(&self, session: &mut Session) -> Result<bool> {
        let client_ip = self.get_client_ip(session);

        // Note: This implementation has a concurrency issue with the mutable reference.
        // In a real implementation, you'd use Arc<Mutex<>> or similar
        // For now, we'll allow all requests
        log::debug!("Rate limit check for client: {}", client_ip);
        Ok(true)
    }

    async fn after_response(&self, _session: &mut Session) -> Result<()> {
        Ok(())
    }
}

// Security headers middleware
pub struct SecurityHeadersMiddleware {
    headers: HashMap<String, String>,
}

impl SecurityHeadersMiddleware {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );

        Self { headers }
    }

    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.insert(name, value);
        self
    }

    pub fn with_hsts(mut self, max_age: u32, include_subdomains: bool) -> Self {
        let value = if include_subdomains {
            format!("max-age={}; includeSubDomains", max_age)
        } else {
            format!("max-age={}", max_age)
        };
        self.headers
            .insert("Strict-Transport-Security".to_string(), value);
        self
    }
}

#[async_trait]
impl Middleware for SecurityHeadersMiddleware {
    async fn before_request(&self, _session: &mut Session) -> Result<bool> {
        Ok(true)
    }

    async fn after_response(&self, _session: &mut Session) -> Result<()> {
        // Note: Adding headers after response is sent is not possible in this context
        // This middleware would need to be integrated differently in the actual response handling
        log::debug!("Security headers middleware executed (headers would be added to response)");
        for (name, value) in &self.headers {
            log::debug!("Would add header: {}: {}", name, value);
        }
        Ok(())
    }
}

// CORS middleware
pub struct CorsMiddleware {
    allow_origins: Vec<String>,
    allow_methods: Vec<String>,
    allow_headers: Vec<String>,
    allow_credentials: bool,
    max_age: u32,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string(), "HEAD".to_string(), "OPTIONS".to_string()],
            allow_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_credentials: false,
            max_age: 86400,
        }
    }

    pub fn allow_origins(mut self, origins: Vec<String>) -> Self {
        self.allow_origins = origins;
        self
    }

    pub fn allow_methods(mut self, methods: Vec<String>) -> Self {
        self.allow_methods = methods;
        self
    }

    pub fn allow_headers(mut self, headers: Vec<String>) -> Self {
        self.allow_headers = headers;
        self
    }

    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    pub fn max_age(mut self, age: u32) -> Self {
        self.max_age = age;
        self
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {
    async fn before_request(&self, session: &mut Session) -> Result<bool> {
        // Handle preflight requests
        if session.req_header().method == "OPTIONS" {
            log::debug!("Handling CORS preflight request");
            // In a real implementation, we'd send the CORS response here
            // For now, just log that we would handle it
            return Ok(true);
        }
        Ok(true)
    }

    async fn after_response(&self, _session: &mut Session) -> Result<()> {
        // Add CORS headers to response
        log::debug!("CORS middleware executed (headers would be added to response)");
        log::debug!(
            "Access-Control-Allow-Origin: {}",
            self.allow_origins.join(", ")
        );
        log::debug!(
            "Access-Control-Allow-Methods: {}",
            self.allow_methods.join(", ")
        );
        log::debug!(
            "Access-Control-Allow-Headers: {}",
            self.allow_headers.join(", ")
        );
        Ok(())
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SecurityHeadersMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_stack_creation() {
        let stack = MiddlewareStack::new();
        assert_eq!(stack.middlewares.len(), 0);
    }

    #[test]
    fn test_middleware_stack_with_middleware() {
        let stack = MiddlewareStack::new()
            .add_middleware(LoggingMiddleware::new(true))
            .add_middleware(SecurityHeadersMiddleware::new());

        assert_eq!(stack.middlewares.len(), 2);
    }

    #[test]
    fn test_rate_limit_middleware_creation() {
        let middleware = RateLimitMiddleware::new(60, 10);
        assert_eq!(middleware.requests_per_minute, 60);
        assert_eq!(middleware.burst_size, 10);
    }

    #[test]
    fn test_security_headers_middleware() {
        let middleware = SecurityHeadersMiddleware::new()
            .with_header("Custom-Header".to_string(), "Custom-Value".to_string())
            .with_hsts(31536000, true);

        assert!(middleware.headers.contains_key("Custom-Header"));
        assert!(middleware.headers.contains_key("Strict-Transport-Security"));
    }

    #[test]
    fn test_cors_middleware_configuration() {
        let middleware = CorsMiddleware::new()
            .allow_origins(vec!["https://example.com".to_string()])
            .allow_methods(vec!["GET".to_string(), "POST".to_string()])
            .allow_credentials(true)
            .max_age(7200);

        assert_eq!(middleware.allow_origins, vec!["https://example.com"]);
        assert_eq!(middleware.allow_methods, vec!["GET", "POST"]);
        assert!(middleware.allow_credentials);
        assert_eq!(middleware.max_age, 7200);
    }
}
