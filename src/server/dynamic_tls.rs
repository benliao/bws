use crate::ssl::SslManager;
use pingora::prelude::*;
use rustls::ServerConfig as RustlsServerConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A dynamic TLS handler that can upgrade from HTTP to HTTPS
pub struct DynamicTlsHandler {
    ssl_managers: Arc<RwLock<HashMap<String, Arc<SslManager>>>>,
    tls_configs: Arc<RwLock<HashMap<String, Arc<RustlsServerConfig>>>>,
}

impl DynamicTlsHandler {
    pub fn new(ssl_managers: Arc<RwLock<HashMap<String, Arc<SslManager>>>>) -> Self {
        Self {
            ssl_managers,
            tls_configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if data looks like a TLS handshake
    pub fn is_tls_handshake(data: &[u8]) -> bool {
        if data.len() < 3 {
            return false;
        }

        // TLS handshake starts with:
        // - 0x16 (handshake record type)
        // - 0x03 0x01, 0x03 0x02, 0x03 0x03, or 0x03 0x04 (TLS versions)
        data[0] == 0x16 && data[1] == 0x03 && (data[2] >= 0x01 && data[2] <= 0x04)
    }

    /// Update TLS configuration for a domain
    pub async fn update_tls_config(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let ssl_managers = self.ssl_managers.read().await;

        if let Some(ssl_manager) = ssl_managers.get(domain) {
            // Try to get the certificate for this domain
            if ssl_manager.has_certificate(domain).await {
                log::info!("Certificate available for {domain}, creating TLS config");

                // Get the certificate and create Rustls config
                if let Ok(rustls_config) = ssl_manager.get_rustls_config(domain).await {
                    let mut tls_configs = self.tls_configs.write().await;
                    tls_configs.insert(domain.to_string(), Arc::new(rustls_config));

                    log::info!("✅ TLS configuration updated for domain: {domain}");
                    log::info!("🔒 HTTPS is now available for https://{domain}");
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if TLS is available for a domain
    pub async fn has_tls_config(&self, domain: &str) -> bool {
        let tls_configs = self.tls_configs.read().await;
        tls_configs.contains_key(domain)
    }

    /// Get TLS config for a domain
    pub async fn get_tls_config(&self, domain: &str) -> Option<Arc<RustlsServerConfig>> {
        let tls_configs = self.tls_configs.read().await;
        tls_configs.get(domain).cloned()
    }

    /// Start background task to monitor for certificate updates
    pub fn start_certificate_monitor(&self, domains: Vec<String>) {
        let ssl_managers = self.ssl_managers.clone();
        let handler = Self {
            ssl_managers: ssl_managers.clone(),
            tls_configs: self.tls_configs.clone(),
        };

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                for domain in &domains {
                    if !handler.has_tls_config(domain).await {
                        match handler.update_tls_config(domain).await {
                            Ok(true) => {
                                log::info!("🎉 Dynamic HTTPS upgrade successful for {domain}");
                                log::warn!("Note: Existing HTTP connections on port 443 will continue as HTTP");
                                log::warn!("New connections will automatically use HTTPS");
                            }
                            Ok(false) => {
                                // Certificate not ready yet, continue monitoring
                            }
                            Err(e) => {
                                log::error!("Failed to update TLS config for {domain}: {e}");
                            }
                        }
                    }
                }
            }
        });
    }
}
