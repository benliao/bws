use crate::ssl::{acme::*, certificate::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

// Utility function to validate domain names
fn is_valid_domain(domain: &str) -> bool {
    // Basic domain validation
    !domain.is_empty()
        && domain.len() <= 253
        && domain
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
        && !domain.starts_with('-')
        && !domain.ends_with('-')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub auto_cert: bool,
    pub cert_dir: String,
    pub acme: Option<AcmeConfig>,
    pub manual_certs: HashMap<String, ManualCertConfig>,
    pub renewal_check_interval_hours: u64,
    pub renewal_days_before_expiry: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualCertConfig {
    pub cert_file: String,
    pub key_file: String,
    pub auto_renew: bool,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_cert: false,
            cert_dir: "/etc/bws/certs".to_string(),
            acme: None,
            manual_certs: HashMap::new(),
            renewal_check_interval_hours: 24, // Check daily
            renewal_days_before_expiry: 30,   // Renew 30 days before expiry
        }
    }
}

#[derive(Debug)]
pub struct SslManager {
    config: SslConfig,
    certificate_store: Arc<RwLock<CertificateStore>>,
    acme_client: Option<Arc<RwLock<AcmeClient>>>,
    tls_configs: Arc<RwLock<HashMap<String, rustls::ServerConfig>>>,
}

impl SslManager {
    pub async fn new(config: SslConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure certificate directory exists
        ensure_certificate_directory(&config.cert_dir).await?;

        // Initialize certificate store
        let store_path = PathBuf::from(&config.cert_dir).join("certificates.toml");
        let mut certificate_store = CertificateStore::new(store_path);
        certificate_store.load().await?;

        // Initialize ACME client if auto_cert is enabled
        let acme_client = if config.auto_cert {
            if let Some(acme_config) = &config.acme {
                let client = AcmeClient::new(acme_config.clone());
                Some(Arc::new(RwLock::new(client)))
            } else {
                return Err("ACME configuration required when auto_cert is enabled".into());
            }
        } else {
            None
        };

        let manager = Self {
            config,
            certificate_store: Arc::new(RwLock::new(certificate_store)),
            acme_client,
            tls_configs: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing certificates
        manager.load_certificates().await?;

        Ok(manager)
    }

    pub async fn get_tls_config(&self, domain: &str) -> Option<rustls::ServerConfig> {
        let configs = self.tls_configs.read().await;
        configs.get(domain).cloned()
    }

    pub async fn ensure_certificate(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if we already have a valid certificate
        {
            let store = self.certificate_store.read().await;
            if let Some(cert) = store.get_certificate(domain) {
                if !cert.is_expired() && cert.validate_certificate_files().await.unwrap_or(false) {
                    log::info!("Valid certificate already exists for {}", domain);
                    return Ok(true);
                }
            }
        }

        // Try to obtain certificate
        if self.config.auto_cert {
            self.obtain_certificate_via_acme(domain).await
        } else {
            // Check manual certificate configuration
            if let Some(manual_config) = self.config.manual_certs.get(domain) {
                self.load_manual_certificate(domain, manual_config).await
            } else {
                log::warn!("No certificate configuration found for domain: {}", domain);
                Ok(false)
            }
        }
    }

    async fn obtain_certificate_via_acme(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !is_valid_domain(domain) {
            return Err(format!("Invalid domain name: {}", domain).into());
        }

        log::info!("Obtaining certificate via ACME for domain: {}", domain);

        if let Some(acme_client) = &self.acme_client {
            let mut client = acme_client.write().await;
            let (cert_pem, key_pem): (String, String) = client
                .obtain_certificate(&[domain.to_string()])
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
                })?;

            // Create certificate paths
            let cert_path = get_certificate_path(domain, &self.config.cert_dir);
            let key_path = get_key_path(domain, &self.config.cert_dir);

            // Create certificate object
            let certificate = Certificate::from_files(
                domain.to_string(),
                cert_path.clone(),
                key_path.clone(),
                true, // auto_renew enabled for ACME certificates
            )
            .await?;

            // Save certificate files
            certificate.save_certificate(&cert_pem, &key_pem).await?;

            // Add to store
            {
                let mut store = self.certificate_store.write().await;
                store.add_certificate(certificate);
                store.save().await?;
            }

            // Update TLS config
            self.update_tls_config(domain).await?;

            log::info!(
                "Successfully obtained and configured certificate for {}",
                domain
            );
            Ok(true)
        } else {
            Err("ACME client not initialized".into())
        }
    }

    async fn load_manual_certificate(
        &self,
        domain: &str,
        manual_config: &ManualCertConfig,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        log::info!("Loading manual certificate for domain: {}", domain);

        let cert_path = PathBuf::from(&manual_config.cert_file);
        let key_path = PathBuf::from(&manual_config.key_file);

        // Validate certificate files exist and are readable
        if !cert_path.exists() {
            return Err(format!("Certificate file not found: {}", cert_path.display()).into());
        }
        if !key_path.exists() {
            return Err(format!("Key file not found: {}", key_path.display()).into());
        }

        // Create certificate object
        let certificate = Certificate::from_files(
            domain.to_string(),
            cert_path,
            key_path,
            manual_config.auto_renew,
        )
        .await?;

        // Validate certificate files
        if !certificate.validate_certificate_files().await? {
            return Err(format!("Invalid certificate files for domain: {}", domain).into());
        }

        // Add to store
        {
            let mut store = self.certificate_store.write().await;
            store.add_certificate(certificate);
            store.save().await?;
        }

        // Update TLS config
        self.update_tls_config(domain).await?;

        log::info!("Successfully loaded manual certificate for {}", domain);
        Ok(true)
    }

    async fn update_tls_config(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.certificate_store.read().await;
        if let Some(certificate) = store.get_certificate(domain) {
            let tls_config = certificate.get_rustls_config().await?;
            let mut configs = self.tls_configs.write().await;
            configs.insert(domain.to_string(), tls_config);
            log::info!("Updated TLS configuration for {}", domain);
        }
        Ok(())
    }

    async fn load_certificates(&self) -> Result<(), Box<dyn std::error::Error>> {
        let certificates = {
            let store = self.certificate_store.read().await;
            store.list_certificates().to_vec()
        };

        for certificate in certificates {
            // Validate certificate files
            if certificate
                .validate_certificate_files()
                .await
                .unwrap_or(false)
            {
                // Update TLS config for valid certificates
                self.update_tls_config(&certificate.domain).await?;
            } else {
                log::warn!(
                    "Certificate files invalid for domain: {}, will attempt renewal",
                    certificate.domain
                );
            }
        }

        let store = self.certificate_store.read().await;
        let cert_count = store.list_certificates().len();
        drop(store);

        log::info!("Loaded certificates for {} domains", cert_count);
        Ok(())
    }

    pub async fn start_renewal_monitor(self: Arc<Self>) {
        let renewal_interval = Duration::from_secs(self.config.renewal_check_interval_hours * 3600);
        let mut interval_timer = interval(renewal_interval);

        log::info!(
            "Starting certificate renewal monitor (check every {} hours)",
            self.config.renewal_check_interval_hours
        );

        loop {
            interval_timer.tick().await;
            if let Err(e) = self.check_and_renew_certificates().await {
                log::error!("Error during certificate renewal check: {}", e);
            }
        }
    }

    async fn check_and_renew_certificates(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Checking certificates for renewal");

        let certificates_to_renew = {
            let store = self.certificate_store.read().await;
            store
                .get_certificates_needing_renewal(self.config.renewal_days_before_expiry)
                .into_iter()
                .map(|cert| cert.domain.clone())
                .collect::<Vec<_>>()
        };

        if certificates_to_renew.is_empty() {
            log::info!("No certificates need renewal");
            return Ok(());
        }

        log::info!(
            "Found {} certificates that need renewal",
            certificates_to_renew.len()
        );

        for domain in certificates_to_renew {
            log::info!("Renewing certificate for domain: {}", domain);

            // Update renewal check timestamp
            {
                let mut store = self.certificate_store.write().await;
                store.update_renewal_check(&domain);
                store.save().await?;
            }

            match self.renew_certificate(&domain).await {
                Ok(()) => {
                    log::info!("Successfully renewed certificate for {}", domain);
                }
                Err(e) => {
                    log::error!("Failed to renew certificate for {}: {}", domain, e);
                    // Continue with other certificates even if one fails
                }
            }
        }

        Ok(())
    }

    async fn renew_certificate(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.auto_cert {
            // For ACME certificates, obtain a new certificate
            self.obtain_certificate_via_acme(domain).await?;
        } else {
            // For manual certificates, reload from files (in case they were updated)
            if let Some(manual_config) = self.config.manual_certs.get(domain) {
                self.load_manual_certificate(domain, manual_config).await?;
            } else {
                return Err(format!("No renewal method available for domain: {}", domain).into());
            }
        }

        Ok(())
    }

    pub async fn remove_certificate(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let removed = {
            let mut store = self.certificate_store.write().await;
            let removed = store.remove_certificate(domain);
            if removed {
                store.save().await?;
            }
            removed
        };

        if removed {
            // Remove from TLS configs
            let mut configs = self.tls_configs.write().await;
            configs.remove(domain);
            log::info!("Removed certificate for domain: {}", domain);
        }

        Ok(removed)
    }

    pub async fn list_certificates(&self) -> Vec<Certificate> {
        let store = self.certificate_store.read().await;
        store.list_certificates().to_vec()
    }

    pub async fn get_certificate_info(&self, domain: &str) -> Option<Certificate> {
        let store = self.certificate_store.read().await;
        store.get_certificate(domain).cloned()
    }

    pub fn is_ssl_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn handles_acme_challenge(&self, path: &str) -> bool {
        path.starts_with("/.well-known/acme-challenge/")
    }

    pub async fn get_acme_challenge_response(&self, token: &str) -> Option<String> {
        if let Some(acme_client) = &self.acme_client {
            let client = acme_client.read().await;
            client.get_challenge_content(token)
        } else {
            None
        }
    }
}

// Configuration validation
impl SslConfig {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        if self.auto_cert {
            if self.acme.is_none() {
                return Err("ACME configuration required when auto_cert is enabled".into());
            }

            if let Some(acme_config) = &self.acme {
                if acme_config.contact_email.is_empty() {
                    return Err("ACME email is required".into());
                }
                if !acme_config.terms_agreed {
                    return Err("ACME terms of service must be agreed to".into());
                }
            }
        }

        for (domain, manual_config) in &self.manual_certs {
            if !is_valid_domain(domain) {
                return Err(format!("Invalid domain name in manual_certs: {}", domain).into());
            }

            if manual_config.cert_file.is_empty() {
                return Err(
                    format!("Certificate file path required for domain: {}", domain).into(),
                );
            }

            if manual_config.key_file.is_empty() {
                return Err(format!("Key file path required for domain: {}", domain).into());
            }
        }

        if self.renewal_check_interval_hours == 0 {
            return Err("Renewal check interval must be greater than 0".into());
        }

        if self.renewal_days_before_expiry < 1 {
            return Err("Renewal days before expiry must be at least 1".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssl_config_validation() {
        let mut config = SslConfig::default();
        assert!(config.validate().is_ok()); // Disabled SSL should be valid

        config.enabled = true;
        config.auto_cert = true;
        assert!(config.validate().is_err()); // Missing ACME config

        config.acme = Some(AcmeConfig::default());
        assert!(config.validate().is_err()); // Empty email

        config.acme.as_mut().unwrap().email = "test@example.com".to_string();
        assert!(config.validate().is_err()); // Terms not agreed

        config.acme.as_mut().unwrap().terms_agreed = true;
        assert!(config.validate().is_ok()); // Should be valid now
    }

    #[test]
    fn test_manual_cert_validation() {
        let mut config = SslConfig::default();
        config.enabled = true;
        config.auto_cert = false;

        // Add invalid domain
        config.manual_certs.insert(
            "".to_string(),
            ManualCertConfig {
                cert_file: "cert.pem".to_string(),
                key_file: "key.pem".to_string(),
                auto_renew: false,
            },
        );
        assert!(config.validate().is_err());

        // Fix domain but empty cert file
        config.manual_certs.clear();
        config.manual_certs.insert(
            "example.com".to_string(),
            ManualCertConfig {
                cert_file: "".to_string(),
                key_file: "key.pem".to_string(),
                auto_renew: false,
            },
        );
        assert!(config.validate().is_err());

        // Fix cert file but empty key file
        config
            .manual_certs
            .get_mut("example.com")
            .unwrap()
            .cert_file = "cert.pem".to_string();
        config.manual_certs.get_mut("example.com").unwrap().key_file = "".to_string();
        assert!(config.validate().is_err());

        // Fix key file - should be valid
        config.manual_certs.get_mut("example.com").unwrap().key_file = "key.pem".to_string();
        assert!(config.validate().is_ok());
    }
}
