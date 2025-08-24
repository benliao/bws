use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder,
    OrderStatus,
};
use log::{debug, error, info, warn};
use rcgen::{Certificate as RcgenCertificate, CertificateParams, DnType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AcmeConfig {
    pub directory_url: String,
    pub contact_email: String,
    pub terms_agreed: bool,
    pub challenge_dir: String,
    pub account_key_file: String,
    pub enabled: bool,
    pub staging: bool,
}

impl Default for AcmeConfig {
    fn default() -> Self {
        Self {
            directory_url: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
            contact_email: "admin@example.com".to_string(),
            terms_agreed: false,
            challenge_dir: "./acme-challenges".to_string(),
            account_key_file: "./acme-account.key".to_string(),
            enabled: false,
            staging: false,
        }
    }
}

#[derive(Debug)]
pub struct AcmeClient {
    config: AcmeConfig,
}

impl AcmeClient {
    pub fn new(config: AcmeConfig) -> Self {
        Self { config }
    }

    /// Initialize the ACME client by creating or loading an account
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(());
        }

        // For now, just verify the config is valid
        Ok(())
    }

    /// Request a certificate for the given domains
    pub async fn obtain_certificate(
        &self,
        domains: &[String],
    ) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("ACME is disabled".into());
        }

        info!("Creating ACME account for domains: {:?}", domains);

        let le_url = if self.config.staging {
            LetsEncrypt::Staging.url()
        } else {
            LetsEncrypt::Production.url()
        };
        info!("Using Let's Encrypt URL: {}", le_url);

        // Create account
        let (mut account, _credentials) = Account::create(
            &NewAccount {
                contact: &[&format!("mailto:{}", self.config.contact_email)],
                terms_of_service_agreed: self.config.terms_agreed,
                only_return_existing: false,
            },
            le_url,
            None, // Let instant-acme generate the key
        )
        .await?;

        // Create identifiers for all domains
        let identifiers: Vec<Identifier> = domains
            .iter()
            .map(|domain| Identifier::Dns(domain.clone()))
            .collect();

        info!("Requesting certificate for domains: {:?}", domains);
        info!(
            "ACME identifiers being sent to Let's Encrypt: {:?}",
            identifiers
        );

        // Create a new order
        let mut order = account
            .new_order(&NewOrder {
                identifiers: &identifiers,
            })
            .await?;

        info!("Created ACME order, processing authorizations");

        // Process all authorizations
        let authorizations = order.authorizations().await?;
        for authz in authorizations {
            let challenge = authz
                .challenges
                .iter()
                .find(|c| matches!(c.r#type, ChallengeType::Http01))
                .ok_or("No HTTP-01 challenge found")?;

            let Identifier::Dns(domain) = &authz.identifier;

            info!("Processing HTTP-01 challenge for domain: {}", domain);

            // Get the key authorization
            let key_auth = order.key_authorization(challenge);
            let key_auth_str = key_auth.as_str();

            info!(
                "Challenge details for domain {}: token={}, key_auth={}",
                domain, challenge.token, key_auth_str
            );

            // Save challenge response to file system
            let challenge_dir = PathBuf::from(&self.config.challenge_dir)
                .join(".well-known")
                .join("acme-challenge");

            tokio::fs::create_dir_all(&challenge_dir).await?;
            let challenge_file = challenge_dir.join(&challenge.token);
            fs::write(&challenge_file, key_auth_str).await?;

            info!(
                "Saved challenge for domain {} to {} with content: {}",
                domain,
                challenge_file.display(),
                key_auth_str
            );

            // Verify the file was written correctly
            match fs::read_to_string(&challenge_file).await {
                Ok(content) => {
                    info!("Verified challenge file content: {}", content);
                }
                Err(e) => {
                    error!("Failed to verify challenge file: {}", e);
                }
            }

            // Give the challenge a moment to propagate and be available
            sleep(Duration::from_millis(500)).await;
            info!(
                "Challenge file ready, signaling to Let's Encrypt for domain: {}",
                domain
            );

            // Tell the server we're ready
            order.set_challenge_ready(&challenge.url).await?;

            info!("Challenge ready signal sent for domain: {}", domain);
        }

        // Wait for all challenges to be validated
        for identifier in &identifiers {
            let Identifier::Dns(domain) = identifier;
            self.wait_for_challenge_validation(&mut account, &mut order, domain)
                .await?;
        }

        // Wait for the order to be ready
        self.wait_for_order_ready(&mut account, &mut order).await?;

        // Generate a CSR with properly configured subject
        info!("Generating CSR for domains: {:?}", domains);
        for (i, domain) in domains.iter().enumerate() {
            info!(
                "Domain {}: '{}' (length: {}, chars: {:?})",
                i,
                domain,
                domain.len(),
                domain.chars().collect::<Vec<_>>()
            );
        }

        // Create certificate parameters with proper subject configuration
        let mut params = CertificateParams::new(domains.to_vec());

        // Set the subject to the first domain to avoid "rcgen self signed cert"
        if let Some(primary_domain) = domains.first() {
            params
                .distinguished_name
                .push(DnType::CommonName, primary_domain.clone());
            info!("Set CSR subject CN to: {}", primary_domain);
        }

        // Generate the certificate with proper subject
        let cert = RcgenCertificate::from_params(params)?;
        let csr = cert.serialize_request_der()?;

        // Finalize the order
        info!("Finalizing ACME order with CSR");
        order.finalize(&csr).await?;
        info!("Order finalized successfully");

        // Wait for certificate to be ready and download it
        info!("Waiting for certificate to be ready...");
        let cert_chain = self.wait_for_certificate(&mut account, &mut order).await?;
        info!(
            "Certificate downloaded successfully, length: {} bytes",
            cert_chain.len()
        );

        // Convert to the format expected by rustls
        let private_key = cert.serialize_private_key_pem();

        info!(
            "Successfully obtained certificate for domains: {:?}",
            domains
        );
        Ok((cert_chain, private_key))
    }

    async fn wait_for_challenge_validation(
        &self,
        account: &mut Account,
        order: &mut instant_acme::Order,
        domain: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Waiting for challenge validation for: {}", domain);

        for attempt in 0..30 {
            // Wait up to 5 minutes
            sleep(Duration::from_secs(10)).await;

            info!(
                "Challenge validation attempt {} for domain: {}",
                attempt + 1,
                domain
            );

            // Refresh order state
            *order = account.order(order.url().to_string()).await?;
            let authorizations = order.authorizations().await?;

            for authz in authorizations {
                let Identifier::Dns(auth_domain) = &authz.identifier;
                if auth_domain == domain {
                    info!("Authorization status for {}: {:?}", domain, authz.status);
                    match authz.status {
                        AuthorizationStatus::Valid => {
                            info!("Challenge validated for: {}", domain);
                            return Ok(());
                        }
                        AuthorizationStatus::Invalid => {
                            // Log more details about why it failed
                            error!(
                                "Challenge validation failed for: {}. Authorization details: {:?}",
                                domain, authz
                            );
                            for challenge in &authz.challenges {
                                if challenge.r#type == ChallengeType::Http01 {
                                    error!("HTTP-01 challenge details for {}: status={:?}, error={:?}, url={:?}", 
                                           domain, challenge.status, challenge.error, challenge.url);
                                }
                            }
                            return Err(
                                format!("Challenge validation failed for: {}", domain).into()
                            );
                        }
                        _ => {
                            info!(
                                "Authorization status for {}: {:?}, continuing to wait...",
                                domain, authz.status
                            );
                        }
                    }
                }
            }
        }

        Err(format!("Timeout waiting for challenge validation for: {}", domain).into())
    }

    async fn wait_for_order_ready(
        &self,
        account: &mut Account,
        order: &mut instant_acme::Order,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..30 {
            // Wait up to 5 minutes
            sleep(Duration::from_secs(10)).await;

            *order = account.order(order.url().to_string()).await?;
            match order.state().status {
                OrderStatus::Ready => {
                    info!("Order is ready for finalization");
                    return Ok(());
                }
                OrderStatus::Invalid => {
                    return Err("Order became invalid".into());
                }
                _ => {
                    debug!("Order status: {:?}", order.state().status);
                }
            }
        }

        Err("Timeout waiting for order to be ready".into())
    }

    async fn wait_for_certificate(
        &self,
        account: &mut Account,
        order: &mut instant_acme::Order,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..30 {
            // Wait up to 5 minutes
            sleep(Duration::from_secs(10)).await;

            *order = account.order(order.url().to_string()).await?;
            match order.state().status {
                OrderStatus::Valid => {
                    // Try to get the certificate directly from the order
                    if let Some(cert_pem) = order.certificate().await? {
                        info!("Certificate obtained successfully");
                        return Ok(cert_pem);
                    }
                }
                OrderStatus::Invalid => {
                    return Err("Order became invalid while waiting for certificate".into());
                }
                _ => {
                    debug!(
                        "Order status while waiting for certificate: {:?}",
                        order.state().status
                    );
                }
            }
        }

        Err("Timeout waiting for certificate".into())
    }

    pub fn handles_acme_challenge(&self, path: &str) -> bool {
        path.starts_with("/.well-known/acme-challenge/")
    }

    pub async fn get_acme_challenge_response(&self, token: &str) -> Option<String> {
        if !self.config.enabled {
            info!(
                "ACME not enabled, cannot retrieve challenge for token: {}",
                token
            );
            return None;
        }

        // Try to read from filesystem
        let challenge_path = PathBuf::from(&self.config.challenge_dir)
            .join(".well-known")
            .join("acme-challenge")
            .join(token);

        info!(
            "Looking for challenge token '{}' at path: {:?}",
            token, challenge_path
        );

        if let Ok(content) = fs::read_to_string(&challenge_path).await {
            info!("Found challenge content for token '{}': {}", token, content);
            Some(content)
        } else {
            warn!(
                "Challenge file not found for token '{}' at path: {:?}",
                token, challenge_path
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_challenge_file_saving_and_reading() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let challenge_dir = temp_dir.path().to_string_lossy().to_string();

        // Create ACME config
        let config = AcmeConfig {
            enabled: true,
            staging: true,
            directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            contact_email: "test@example.com".to_string(),
            terms_agreed: true,
            challenge_dir: challenge_dir.clone(),
            account_key_file: format!("{}/acme-account.key", challenge_dir),
        };

        // Create ACME client (won't actually connect to Let's Encrypt)
        let client = AcmeClient::new(config.clone());

        // Test token and key authorization
        let test_token = "test_challenge_token_12345";
        let test_key_auth = "test_key_authorization_67890.test_key_thumbprint";

        // Manually create the challenge file structure (simulating what save_challenge does)
        let full_challenge_dir = PathBuf::from(&challenge_dir)
            .join(".well-known")
            .join("acme-challenge");

        tokio::fs::create_dir_all(&full_challenge_dir)
            .await
            .expect("Failed to create challenge directory");

        let challenge_file = full_challenge_dir.join(test_token);
        tokio::fs::write(&challenge_file, test_key_auth)
            .await
            .expect("Failed to write challenge file");

        // Test reading the challenge response
        let response = client.get_acme_challenge_response(test_token).await;

        assert!(response.is_some(), "Challenge response should be found");
        assert_eq!(
            response.unwrap(),
            test_key_auth,
            "Challenge response content should match"
        );

        // Test with non-existent token
        let missing_response = client
            .get_acme_challenge_response("non_existent_token")
            .await;
        assert!(
            missing_response.is_none(),
            "Non-existent challenge should return None"
        );

        // Test handles_acme_challenge
        assert!(client.handles_acme_challenge("/.well-known/acme-challenge/some_token"));
        assert!(!client.handles_acme_challenge("/some/other/path"));
        assert!(!client.handles_acme_challenge("/.well-known/other-challenge/token"));
    }

    #[test]
    fn test_challenge_path_handling() {
        let config = AcmeConfig {
            enabled: true,
            staging: true,
            directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            contact_email: "test@example.com".to_string(),
            terms_agreed: true,
            challenge_dir: "./test-challenges".to_string(),
            account_key_file: "./test-acme-account.key".to_string(),
        };

        let client = AcmeClient::new(config);

        // Test various challenge paths
        assert!(client.handles_acme_challenge("/.well-known/acme-challenge/token123"));
        assert!(client.handles_acme_challenge("/.well-known/acme-challenge/"));
        assert!(!client.handles_acme_challenge("/.well-known/acme-challenge"));
        assert!(!client.handles_acme_challenge("/well-known/acme-challenge/token"));
        assert!(!client.handles_acme_challenge("/.well-known/other/token"));
        assert!(!client.handles_acme_challenge("/api/health"));
        assert!(!client.handles_acme_challenge("/"));
    }
}
