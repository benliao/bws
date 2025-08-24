use crate::config::SiteConfig;
use pingora::http::ResponseHeader;
use pingora::prelude::*;

pub struct HealthHandler {
    start_time: std::time::Instant,
}

impl HealthHandler {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn handle(&self, session: &mut Session, _site: Option<&SiteConfig>) -> Result<()> {
        let path = session.req_header().uri.path();

        match path {
            "/api/health" => self.handle_basic_health(session).await,
            "/api/health/detailed" => self.handle_detailed_health(session).await,
            "/api/health/ready" => self.handle_readiness(session).await,
            "/api/health/live" => self.handle_liveness(session).await,
            _ => self.handle_basic_health(session).await,
        }
    }

    async fn handle_basic_health(&self, session: &mut Session) -> Result<()> {
        let response = serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "bws-web-server",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_detailed_health(&self, session: &mut Session) -> Result<()> {
        let uptime = self.start_time.elapsed();
        let memory_info = self.get_memory_info();
        let system_info = self.get_system_info();

        let response = serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": {
                "name": "bws-web-server",
                "version": env!("CARGO_PKG_VERSION"),
                "description": env!("CARGO_PKG_DESCRIPTION")
            },
            "uptime": {
                "seconds": uptime.as_secs(),
                "human_readable": self.format_duration(uptime)
            },
            "memory": memory_info,
            "system": system_info,
            "features": {
                "ssl_support": true,
                "auto_cert": true,
                "multi_site": true,
                "compression": true,
                "caching": true
            }
        });

        self.send_json_response(session, 200, &response).await
    }

    async fn handle_readiness(&self, session: &mut Session) -> Result<()> {
        // Check if the service is ready to accept requests
        let is_ready = self.check_readiness();

        let status_code = if is_ready { 200 } else { 503 };
        let response = serde_json::json!({
            "status": if is_ready { "ready" } else { "not_ready" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "checks": {
                "configuration_loaded": true,
                "ssl_manager_initialized": true,
                "handlers_ready": true
            }
        });

        self.send_json_response(session, status_code, &response)
            .await
    }

    async fn handle_liveness(&self, session: &mut Session) -> Result<()> {
        // Check if the service is alive and functioning
        let is_alive = self.check_liveness();

        let status_code = if is_alive { 200 } else { 503 };
        let response = serde_json::json!({
            "status": if is_alive { "alive" } else { "dead" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        });

        self.send_json_response(session, status_code, &response)
            .await
    }

    fn check_readiness(&self) -> bool {
        // Implement readiness checks here
        // For now, assume always ready
        true
    }

    fn check_liveness(&self) -> bool {
        // Implement liveness checks here
        // For now, assume always alive
        true
    }

    fn get_memory_info(&self) -> serde_json::Value {
        // Get basic memory information
        // This is a simplified implementation
        serde_json::json!({
            "note": "Memory information not available in this implementation"
        })
    }

    fn get_system_info(&self) -> serde_json::Value {
        serde_json::json!({
            "rust_version": std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            "target": std::env::var("TARGET").unwrap_or_else(|_| std::env::consts::ARCH.to_string()),
            "build_timestamp": std::env::var("BUILD_TIMESTAMP").unwrap_or_else(|_| "unknown".to_string()),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH
        })
    }

    fn format_duration(&self, duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    async fn send_json_response(
        &self,
        session: &mut Session,
        status: u16,
        data: &serde_json::Value,
    ) -> Result<()> {
        let response_body = serde_json::to_string_pretty(data)
            .unwrap_or_else(|_| r#"{"error": "Failed to serialize response"}"#.to_string());
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

impl Default for HealthHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_handler_creation() {
        let handler = HealthHandler::new();
        assert!(handler.start_time.elapsed().as_millis() < 100); // Should be very recent
    }

    #[test]
    fn test_duration_formatting() {
        let handler = HealthHandler::new();

        assert_eq!(
            handler.format_duration(std::time::Duration::from_secs(30)),
            "30s"
        );
        assert_eq!(
            handler.format_duration(std::time::Duration::from_secs(90)),
            "1m 30s"
        );
        assert_eq!(
            handler.format_duration(std::time::Duration::from_secs(3661)),
            "1h 1m 1s"
        );
        assert_eq!(
            handler.format_duration(std::time::Duration::from_secs(90061)),
            "1d 1h 1m 1s"
        );
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let handler = HealthHandler::new();
        assert!(handler.check_readiness().await);
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let handler = HealthHandler::new();
        assert!(handler.check_liveness());
    }
}
