use instant_acme::{Account, AuthorizationStatus, ChallengeType, NewAccount, NewOrder, OrderStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeConfig {
    pub directory_url: String,
    pub email: String,
    pub terms_agreed: bool,
    pub challenge_type: AcmeChallengeType,
    pub challenge_dir: String,
    pub key_type: AcmeKeyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcmeChallengeType {
    Http01,
    Dns01,
    TlsAlpn01,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcmeKeyType {
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
}

#[derive(Clone)]  // Remove Debug for now since Account doesn't implement it
pub struct AcmeClient {
    account: Account,
    config: AcmeConfig,
    challenges: HashMap<String, String>, // domain -> challenge_content
}

impl Default for AcmeConfig {
    fn default() -> Self {
        Self {
            directory_url: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
            email: String::new(),
            terms_agreed: false,
            challenge_type: AcmeChallengeType::Http01,
            challenge_dir: "/var/www/html/.well-known/acme-challenge".to_string(),
            key_type: AcmeKeyType::EcdsaP256,
        }
    }
}

impl AcmeClient {
    pub async fn new(config: AcmeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create account key
        let (account, _account_credentials) = Account::create(
            &NewAccount {
                contact: &[&format!("mailto:{}", config.email)],
                terms_of_service_agreed: config.terms_agreed,
                only_return_existing: false,
            },
            &config.directory_url,
            None,
        )
        .await?;

        Ok(Self {
            account,
            config,
            challenges: HashMap::new(),
        })
    }

    pub async fn obtain_certificate(
        &mut self,
        domains: &[String],
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        log::info!("Obtaining certificate for domains: {:?}", domains);

        // Create a new order
        let mut order = self
            .account
            .new_order(&NewOrder {
                identifiers: &domains
                    .iter()
                    .map(|d| instant_acme::Identifier::Dns(d.clone()))
                    .collect::<Vec<_>>(),
            })
            .await?;

        log::info!("Created order, URL: {}", order.url());

        // Process authorizations
        let order_state = order.state();
        if order_state.status == OrderStatus::Pending {
            for auth_url in &order_state.authorizations {
                let auth = self.account.authorization(auth_url).await?;
                log::info!("Authorization for {}: {:?}", auth.identifier, auth.status);

                if auth.status == AuthorizationStatus::Pending {
                    let challenge = auth
                        .challenges
                        .iter()
                        .find(|c| match self.config.challenge_type {
                            AcmeChallengeType::Http01 => c.r#type == ChallengeType::Http01,
                            AcmeChallengeType::Dns01 => c.r#type == ChallengeType::Dns01,
                            AcmeChallengeType::TlsAlpn01 => c.r#type == ChallengeType::TlsAlpn01,
                        })
                        .ok_or("No suitable challenge found")?;

                    match self.config.challenge_type {
                        AcmeChallengeType::Http01 => {
                            self.setup_http01_challenge(&auth.identifier, challenge).await?;
                        }
                        AcmeChallengeType::Dns01 => {
                            return Err("DNS-01 challenge not implemented yet".into());
                        }
                        AcmeChallengeType::TlsAlpn01 => {
                            return Err("TLS-ALPN-01 challenge not implemented yet".into());
                        }
                    }

                    // Notify ACME server that challenge is ready
                    self.account.challenge(&challenge.url).await?;
                    log::info!("Challenge ready for {}", auth.identifier);
                }
            }

            // Wait for authorization completion
            log::info!("Waiting for authorization completion...");
            for _ in 0..30 {
                // Max 30 attempts (5 minutes)
                sleep(Duration::from_secs(10)).await;
                let order_state = order.refresh().await?;

                if order_state.status == OrderStatus::Ready {
                    break;
                } else if order_state.status == OrderStatus::Invalid {
                    return Err("Order became invalid".into());
                }
            }
        }

        // Generate certificate
        log::info!("Generating certificate...");
        let mut names = Vec::new();
        names.push(domains[0].clone());
        for domain in &domains[1..] {
            names.push(domain.clone());
        }

        let cert_key = match self.config.key_type {
            AcmeKeyType::Rsa2048 => rcgen::KeyPair::generate(&rcgen::PKCS_RSA_SHA256)?,
            AcmeKeyType::Rsa4096 => {
                // rcgen doesn't directly support RSA 4096, use 2048 as fallback
                rcgen::KeyPair::generate(&rcgen::PKCS_RSA_SHA256)?
            }
            AcmeKeyType::EcdsaP256 => rcgen::KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?,
            AcmeKeyType::EcdsaP384 => rcgen::KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384)?,
        };

        let mut cert_params = rcgen::CertificateParams::new(names);
        cert_params.key_pair = Some(cert_key);

        let cert_csr = rcgen::Certificate::from_params(cert_params)?;
        let csr_der = cert_csr.serialize_request_der()?;

        // Finalize order
        order.finalize(&csr_der).await?;

        // Wait for certificate
        log::info!("Waiting for certificate...");
        for _ in 0..30 {
            // Max 30 attempts
            sleep(Duration::from_secs(5)).await;
            let order_state = order.refresh().await?;

            if order_state.status == OrderStatus::Valid {
                if let Some(cert_url) = &order_state.certificate {
                    let cert_chain_pem = self.account.certificate(cert_url).await?;
                    let private_key_pem = cert_csr.serialize_private_key_pem();

                    log::info!("Certificate obtained successfully");
                    self.cleanup_challenges().await?;
                    return Ok((cert_chain_pem, private_key_pem));
                }
            } else if order_state.status == OrderStatus::Invalid {
                return Err("Order became invalid during certificate generation".into());
            }
        }

        Err("Timeout waiting for certificate".into())
    }

    async fn setup_http01_challenge(
        &mut self,
        domain: &str,
        challenge: &instant_acme::Challenge,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key_auth = self.account.key_authorization(challenge)?;
        let challenge_path = format!("{}/{}", self.config.challenge_dir, challenge.token);

        // Ensure challenge directory exists
        let challenge_dir = PathBuf::from(&self.config.challenge_dir);
        if !challenge_dir.exists() {
            fs::create_dir_all(&challenge_dir).await?;
        }

        // Write challenge file
        fs::write(&challenge_path, &key_auth).await?;

        // Store challenge for cleanup
        self.challenges.insert(domain.to_string(), challenge_path);

        log::info!(
            "HTTP-01 challenge set up for {} at {}",
            domain,
            challenge_path
        );
        Ok(())
    }

    async fn cleanup_challenges(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (domain, challenge_path) in &self.challenges {
            if let Err(e) = fs::remove_file(challenge_path).await {
                log::warn!("Failed to remove challenge file for {}: {}", domain, e);
            } else {
                log::info!("Cleaned up challenge file for {}", domain);
            }
        }
        self.challenges.clear();
        Ok(())
    }

    pub fn get_challenge_content(&self, token: &str) -> Option<String> {
        // This method can be used by the HTTP server to serve challenge responses
        for (_, challenge_path) in &self.challenges {
            if challenge_path.ends_with(token) {
                // In a real implementation, you might want to read the file
                // For now, we'll implement this when integrating with the HTTP server
                return None;
            }
        }
        None
    }
}

// Helper function to validate domain names
pub fn is_valid_domain(domain: &str) -> bool {
    // Basic domain validation
    !domain.is_empty()
        && domain.len() <= 253
        && domain
            .split('.')
            .all(|label| !label.is_empty() && label.len() <= 63)
        && !domain.starts_with('-')
        && !domain.ends_with('-')
        && !domain.contains("..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_validation() {
        assert!(is_valid_domain("example.com"));
        assert!(is_valid_domain("subdomain.example.com"));
        assert!(is_valid_domain("test-domain.com"));

        assert!(!is_valid_domain(""));
        assert!(!is_valid_domain("example..com"));
        assert!(!is_valid_domain("-example.com"));
        assert!(!is_valid_domain("example.com-"));
    }

    #[test]
    fn test_acme_config_default() {
        let config = AcmeConfig::default();
        assert_eq!(
            config.directory_url,
            "https://acme-v02.api.letsencrypt.org/directory"
        );
        assert!(config.email.is_empty());
        assert!(!config.terms_agreed);
    }
}
