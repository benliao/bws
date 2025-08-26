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

    /// Create SSL manager from site configuration
    pub async fn from_site_config(
        site: &crate::config::site::SiteConfig,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        if !site.ssl.enabled {
            return Ok(None);
        }

        // Determine cert_dir first
        let cert_dir = site
            .ssl
            .cert_file
            .as_ref()
            .and_then(|path| std::path::Path::new(path).parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if site.ssl.auto_cert {
                    "./certs".to_string() // Use local directory for ACME auto-certificates
                } else {
                    "/etc/bws/certs".to_string() // Use system directory for manual certificates
                }
            });

        let ssl_config = SslConfig {
            enabled: site.ssl.enabled,
            auto_cert: site.ssl.auto_cert,
            cert_dir: cert_dir.clone(),
            acme: site.ssl.acme.as_ref().map(|site_acme| AcmeConfig {
                directory_url: if site_acme.staging {
                    "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
                } else {
                    "https://acme-v02.api.letsencrypt.org/directory".to_string()
                },
                contact_email: site_acme.email.clone(),
                terms_agreed: !site_acme.email.is_empty(), // Auto-agree if email is provided
                challenge_dir: site_acme.challenge_dir.clone().unwrap_or_else(|| {
                    // Auto-generate challenge directory based on cert_dir
                    format!("{cert_dir}/challenges")
                }),
                account_key_file: format!("{cert_dir}/acme-account.key"),
                enabled: site_acme.enabled,
                staging: site_acme.staging,
            }),
            manual_certs: {
                let mut manual_certs = HashMap::new();
                if let (Some(cert_file), Some(key_file)) = (&site.ssl.cert_file, &site.ssl.key_file)
                {
                    manual_certs.insert(
                        site.hostname.clone(),
                        ManualCertConfig {
                            cert_file: cert_file.clone(),
                            key_file: key_file.clone(),
                            auto_renew: false,
                        },
                    );
                }
                manual_certs
            },
            renewal_check_interval_hours: 24,
            renewal_days_before_expiry: 30,
        };

        let manager = Self::new(ssl_config).await?;
        Ok(Some(manager))
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
                    log::info!("Valid certificate already exists for {domain}");
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
                log::warn!("No certificate configuration found for domain: {domain}");
                Ok(false)
            }
        }
    }

    async fn obtain_certificate_via_acme(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !is_valid_domain(domain) {
            return Err(format!("Invalid domain name: {domain}").into());
        }

        log::info!("Obtaining certificate via ACME for domain: {domain}");
        log::info!("Domain passed to ACME client: '{domain}'");

        if let Some(acme_client) = &self.acme_client {
            let client = acme_client.write().await;
            log::info!(
                "Calling ACME client.obtain_certificate with domains: {:?}",
                &[domain.to_string()]
            );
            let (cert_pem, key_pem): (String, String) = client
                .obtain_certificate(&[domain.to_string()])
                .await
                .map_err(|e| {
                    log::error!("ACME obtain_certificate failed: {e}");
                    Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
                })?;

            log::info!(
                "ACME certificate obtained successfully, cert size: {} bytes, key size: {} bytes",
                cert_pem.len(),
                key_pem.len()
            );

            // Create certificate paths
            let cert_path = get_certificate_path(domain, &self.config.cert_dir);
            let key_path = get_key_path(domain, &self.config.cert_dir);

            log::info!("Certificate will be saved to: {cert_path:?}");
            log::info!("Private key will be saved to: {key_path:?}");

            // Create certificate object
            log::info!("Creating certificate object for domain: {domain}");
            let certificate = Certificate::from_pem_data(
                domain.to_string(),
                cert_path.clone(),
                key_path.clone(),
                &cert_pem,
                true, // auto_renew enabled for ACME certificates
            )?;

            // Save certificate files
            log::info!("Saving certificate and private key files...");
            certificate.save_certificate(&cert_pem, &key_pem).await?;
            log::info!("Certificate files saved successfully");

            // Add to store
            log::info!("Adding certificate to store...");
            {
                let mut store = self.certificate_store.write().await;
                store.add_certificate(certificate);
                store.save().await?;
            }
            log::info!("Certificate added to store successfully");

            // Update TLS config
            log::info!("Updating TLS configuration...");
            self.update_tls_config(domain).await?;
            log::info!("TLS configuration updated successfully");

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
        log::info!("Loading manual certificate for domain: {domain}");

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
            return Err(format!("Invalid certificate files for domain: {domain}").into());
        }

        // Add to store
        {
            let mut store = self.certificate_store.write().await;
            store.add_certificate(certificate);
            store.save().await?;
        }

        // Update TLS config
        self.update_tls_config(domain).await?;

        log::info!("Successfully loaded manual certificate for {domain}");
        Ok(true)
    }

    async fn update_tls_config(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.certificate_store.read().await;
        if let Some(certificate) = store.get_certificate(domain) {
            let tls_config = certificate.get_rustls_config().await?;
            let mut configs = self.tls_configs.write().await;
            configs.insert(domain.to_string(), tls_config);
            log::info!("Updated TLS configuration for {domain}");
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

        log::info!("Loaded certificates for {cert_count} domains");
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
                log::error!("Error during certificate renewal check: {e}");
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
            log::info!("Renewing certificate for domain: {domain}");

            // Update renewal check timestamp
            {
                let mut store = self.certificate_store.write().await;
                store.update_renewal_check(&domain);
                store.save().await?;
            }

            match self.renew_certificate(&domain).await {
                Ok(()) => {
                    log::info!("Successfully renewed certificate for {domain}");
                }
                Err(e) => {
                    log::error!("Failed to renew certificate for {domain}: {e}");
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
                return Err(format!("No renewal method available for domain: {domain}").into());
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
            log::info!("Removed certificate for domain: {domain}");
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
            client.get_acme_challenge_response(token).await
        } else {
            None
        }
    }

    /// Check if auto-cert is enabled for this SSL manager
    pub fn is_auto_cert_enabled(&self) -> bool {
        self.config.auto_cert
    }

    /// Check if a certificate is available for a domain
    pub async fn has_certificate(&self, domain: &str) -> bool {
        let store = self.certificate_store.read().await;
        store.has_certificate(domain)
    }

    /// Get TLS configuration for a domain (Result version for dynamic TLS)
    pub async fn get_rustls_config(
        &self,
        domain: &str,
    ) -> Result<rustls::ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.certificate_store.read().await;

        if let Some(certificate) = store.get_certificate(domain) {
            match certificate.get_rustls_config().await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to create rustls config: {e}").into()),
            }
        } else {
            Err(format!("No certificate found for domain: {domain}").into())
        }
    }

    /// Get list of domains managed by this SSL manager
    pub async fn get_managed_domains(&self) -> Vec<String> {
        let store = self.certificate_store.read().await;
        store.get_all_domains()
    }

    /// Get certificate expiry date for a domain
    /// Get certificate expiry date for a domain
    ///
    /// # Errors
    ///
    /// Returns an error if the certificate cannot be parsed or accessed.
    pub async fn get_certificate_expiry(
        &self,
        domain: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let store = self.certificate_store.read().await;
        store.get_certificate_expiry(domain)
    }

    /// Renew certificate for a domain
    /// Renew certificate for a specific domain (public method)
    ///
    /// # Errors
    ///
    /// Returns an error if certificate renewal fails or if ACME client is not initialized.
    pub async fn renew_certificate_public(
        &self,
        domain: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(acme_client) = &self.acme_client {
            let (cert_pem, key_pem) = {
                let client = acme_client.write().await;
                client.obtain_certificate(&[domain.to_string()]).await?
            };

            // Save certificate files
            let cert_dir = std::path::Path::new(&self.config.cert_dir);
            tokio::fs::create_dir_all(cert_dir).await?;

            let cert_path = cert_dir.join(format!("{domain}.crt"));
            let key_path = cert_dir.join(format!("{domain}.key"));

            tokio::fs::write(&cert_path, &cert_pem).await?;
            tokio::fs::write(&key_path, &key_pem).await?;

            // Create certificate object and add to store
            match crate::ssl::certificate::Certificate::from_files(
                domain.to_string(),
                cert_path,
                key_path,
                true, // auto_renew = true
            )
            .await
            {
                Ok(certificate) => {
                    let mut store = self.certificate_store.write().await;
                    store.add_certificate(certificate);
                    // Note: Save functionality will be added later if needed
                }
                Err(e) => {
                    log::error!("Failed to create certificate object for {domain}: {e}");
                    return Err(format!("Failed to create certificate object: {e}").into());
                }
            }

            log::info!("Successfully renewed certificate for domain: {domain}");
            Ok(())
        } else {
            Err("ACME client not initialized".into())
        }
    }

    /// Check and renew certificate for a specific domain (public method)
    ///
    /// # Errors
    ///
    /// Returns an error if certificate renewal fails or if ACME client is not initialized.
    pub async fn check_and_renew_certificate(
        &self,
        domain: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Check if certificate needs renewal
        let needs_renewal = {
            let store = self.certificate_store.read().await;
            store
                .get_certificate(domain)
                .is_none_or(|cert| cert.needs_renewal(self.config.renewal_days_before_expiry))
        };

        if needs_renewal {
            log::info!("Certificate for {domain} needs renewal");
            match self.renew_certificate(domain).await {
                Ok(()) => {
                    log::info!("Successfully renewed certificate for {domain}");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("Failed to renew certificate for {domain}: {e}");
                    Err(format!("Certificate renewal failed: {e}").into())
                }
            }
        } else {
            log::debug!("Certificate for {domain} is still valid");
            Ok(false)
        }
    }
}

// Configuration validation
impl SslConfig {
    /// Validate SSL manager configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
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
                return Err(format!("Invalid domain name in manual_certs: {domain}").into());
            }

            if manual_config.cert_file.is_empty() {
                return Err(format!("Certificate file path required for domain: {domain}").into());
            }

            if manual_config.key_file.is_empty() {
                return Err(format!("Key file path required for domain: {domain}").into());
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

        config.acme.as_mut().unwrap().contact_email = "test@example.com".to_string();
        assert!(config.validate().is_err()); // Terms not agreed

        config.acme.as_mut().unwrap().terms_agreed = true;
        assert!(config.validate().is_ok()); // Should be valid now
    }

    #[test]
    fn test_manual_cert_validation() {
        let mut config = SslConfig {
            enabled: true,
            auto_cert: false,
            ..Default::default()
        };

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
