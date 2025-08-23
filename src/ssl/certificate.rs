use chrono::{DateTime, Utc};
use rustls_pemfile::{certs, private_key};
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::path::PathBuf;
use tokio::fs;
use x509_parser::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub domain: String,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issuer: String,
    pub san_domains: Vec<String>,
    pub auto_renew: bool,
    pub last_renewal_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStore {
    pub certificates: Vec<Certificate>,
    pub storage_path: PathBuf,
}

impl Certificate {
    pub async fn from_files(
        domain: String,
        cert_path: PathBuf,
        key_path: PathBuf,
        auto_renew: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Read certificate file
        let cert_data = fs::read(&cert_path).await?;
        let cert_info = Self::parse_certificate(&cert_data)?;

        Ok(Certificate {
            domain,
            cert_path,
            key_path,
            issued_at: cert_info.issued_at,
            expires_at: cert_info.expires_at,
            issuer: cert_info.issuer,
            san_domains: cert_info.san_domains,
            auto_renew,
            last_renewal_check: None,
        })
    }

    pub async fn save_certificate(
        &self,
        cert_pem: &str,
        key_pem: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directories exist
        if let Some(parent) = self.cert_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        if let Some(parent) = self.key_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write certificate and key files
        fs::write(&self.cert_path, cert_pem).await?;
        fs::write(&self.key_path, key_pem).await?;

        // Set appropriate permissions (600 for key file)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let key_perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.key_path, key_perms)?;

            let cert_perms = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&self.cert_path, cert_perms)?;
        }

        log::info!(
            "Certificate saved for {} at {} and {}",
            self.domain,
            self.cert_path.display(),
            self.key_path.display()
        );

        Ok(())
    }

    pub fn days_until_expiry(&self) -> i64 {
        let now = Utc::now();
        (self.expires_at - now).num_days()
    }

    pub fn needs_renewal(&self, days_before_expiry: i64) -> bool {
        self.auto_renew && self.days_until_expiry() <= days_before_expiry
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn covers_domain(&self, domain: &str) -> bool {
        self.domain == domain || self.san_domains.contains(&domain.to_string())
    }

    fn parse_certificate(cert_data: &[u8]) -> Result<CertificateInfo, Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(cert_data);
        let certs = certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse certificate: {}", e))?;

        if certs.is_empty() {
            return Err("No certificates found in file".into());
        }

        let cert = &certs[0];
        let (_, parsed_cert) = X509Certificate::from_der(cert.as_ref())
            .map_err(|e| format!("Failed to parse X509 certificate: {}", e))?;

        let issued_at = DateTime::from_timestamp(parsed_cert.validity().not_before.timestamp(), 0)
            .unwrap_or_else(Utc::now);

        let expires_at = DateTime::from_timestamp(parsed_cert.validity().not_after.timestamp(), 0)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::days(90));

        let issuer = parsed_cert
            .issuer()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .unwrap_or("Unknown")
            .to_string();

        // Extract SAN domains - simplified for now
        let san_domains = Vec::new();
        // TODO: Implement proper SAN parsing
        log::debug!("Certificate SAN parsing not implemented - using subject CN only");

        Ok(CertificateInfo {
            issued_at,
            expires_at,
            issuer,
            san_domains,
        })
    }

    pub async fn validate_certificate_files(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if files exist
        if !self.cert_path.exists() || !self.key_path.exists() {
            return Ok(false);
        }

        // Try to load certificate
        let cert_data = fs::read(&self.cert_path).await?;
        let mut cert_reader = BufReader::new(cert_data.as_slice());
        let certs_result = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>();
        if certs_result.is_err() {
            return Ok(false);
        }

        // Try to load private key
        let key_data = fs::read(&self.key_path).await?;
        let mut key_reader = BufReader::new(key_data.as_slice());
        let key_result = private_key(&mut key_reader);
        if key_result.is_err() {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn get_rustls_config(
        &self,
    ) -> Result<rustls::ServerConfig, Box<dyn std::error::Error>> {
        // Load certificate chain
        let cert_data = fs::read(&self.cert_path).await?;
        let mut cert_reader = BufReader::new(cert_data.as_slice());
        let cert_chain = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to load certificate: {}", e))?;

        // Load private key
        let key_data = fs::read(&self.key_path).await?;
        let mut key_reader = BufReader::new(key_data.as_slice());
        let private_key = private_key(&mut key_reader)
            .map_err(|e| format!("Failed to load private key: {}", e))?
            .ok_or("No private key found")?;

        // Create rustls config
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| format!("Invalid certificate/key: {}", e))?;

        Ok(config)
    }
}

impl CertificateStore {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            certificates: Vec::new(),
            storage_path,
        }
    }

    pub async fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.storage_path.exists() {
            log::info!("Certificate store file not found, starting with empty store");
            return Ok(());
        }

        let data = fs::read_to_string(&self.storage_path).await?;
        let store: CertificateStore = toml::from_str(&data)?;
        self.certificates = store.certificates;

        log::info!("Loaded {} certificates from store", self.certificates.len());
        Ok(())
    }

    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = toml::to_string_pretty(self)?;
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&self.storage_path, data).await?;
        log::info!(
            "Saved certificate store with {} certificates",
            self.certificates.len()
        );
        Ok(())
    }

    pub fn add_certificate(&mut self, certificate: Certificate) {
        // Remove any existing certificate for the same domain
        self.certificates
            .retain(|cert| cert.domain != certificate.domain);
        self.certificates.push(certificate);
    }

    pub fn get_certificate(&self, domain: &str) -> Option<&Certificate> {
        self.certificates
            .iter()
            .find(|cert| cert.covers_domain(domain))
    }

    pub fn get_certificates_needing_renewal(&self, days_before_expiry: i64) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|cert| cert.needs_renewal(days_before_expiry))
            .collect()
    }

    pub fn get_expired_certificates(&self) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|cert| cert.is_expired())
            .collect()
    }

    pub fn update_renewal_check(&mut self, domain: &str) {
        if let Some(cert) = self.certificates.iter_mut().find(|c| c.domain == domain) {
            cert.last_renewal_check = Some(Utc::now());
        }
    }

    pub fn remove_certificate(&mut self, domain: &str) -> bool {
        let original_len = self.certificates.len();
        self.certificates.retain(|cert| cert.domain != domain);
        self.certificates.len() != original_len
    }

    pub fn list_certificates(&self) -> &[Certificate] {
        &self.certificates
    }
}

#[derive(Debug)]
struct CertificateInfo {
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    issuer: String,
    san_domains: Vec<String>,
}

// Helper functions for certificate management
pub fn get_certificate_path(domain: &str, cert_dir: &str) -> PathBuf {
    PathBuf::from(cert_dir).join(format!("{}.crt", domain))
}

pub fn get_key_path(domain: &str, cert_dir: &str) -> PathBuf {
    PathBuf::from(cert_dir).join(format!("{}.key", domain))
}

pub async fn ensure_certificate_directory(
    cert_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(cert_dir);
    if !path.exists() {
        fs::create_dir_all(&path).await?;
        log::info!("Created certificate directory: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_certificate_store() {
        let temp_dir = tempdir().unwrap();
        let store_path = temp_dir.path().join("certificates.toml");

        let store = CertificateStore::new(store_path.clone());

        // Test saving empty store
        store.save().await.unwrap();
        assert!(store_path.exists());

        // Test loading empty store
        let mut store2 = CertificateStore::new(store_path);
        store2.load().await.unwrap();
        assert_eq!(store2.certificates.len(), 0);
    }

    #[test]
    fn test_certificate_expiry() {
        let now = Utc::now();
        let cert = Certificate {
            domain: "example.com".to_string(),
            cert_path: PathBuf::from("test.crt"),
            key_path: PathBuf::from("test.key"),
            issued_at: now - chrono::Duration::days(60),
            expires_at: now + chrono::Duration::days(30),
            issuer: "Test CA".to_string(),
            san_domains: vec!["www.example.com".to_string()],
            auto_renew: true,
            last_renewal_check: None,
        };

        // Allow for small timing differences (29-30 days)
        let days_until_expiry = cert.days_until_expiry();
        assert!((29..=30).contains(&days_until_expiry));
        assert!(cert.needs_renewal(45)); // Should renew if 45 days or less
        assert!(!cert.needs_renewal(25)); // Should not renew if more than 30 days left
        assert!(!cert.is_expired());
        assert!(cert.covers_domain("example.com"));
        assert!(cert.covers_domain("www.example.com"));
        assert!(!cert.covers_domain("other.com"));
    }
}
