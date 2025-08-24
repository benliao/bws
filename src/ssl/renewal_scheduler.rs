use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use log::{info, error, warn};
use crate::ssl::manager::SslManager;

pub struct AutoRenewalScheduler {
    ssl_managers: Vec<Arc<SslManager>>,
    check_interval: Duration,
    renewal_threshold_days: u32,
}

impl AutoRenewalScheduler {
    pub fn new(ssl_managers: Vec<Arc<SslManager>>) -> Self {
        Self {
            ssl_managers,
            check_interval: Duration::from_secs(24 * 60 * 60), // Check daily
            renewal_threshold_days: 30, // Renew if expires within 30 days
        }
    }

    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub fn with_renewal_threshold(mut self, days: u32) -> Self {
        self.renewal_threshold_days = days;
        self
    }

    pub async fn start_background_renewal(&self) {
        let ssl_managers = self.ssl_managers.clone();
        let check_interval = self.check_interval;
        let renewal_threshold_days = self.renewal_threshold_days;

        tokio::spawn(async move {
            let mut interval_timer = interval(check_interval);
            info!("Started automatic certificate renewal scheduler (checking every {} hours)", 
                  check_interval.as_secs() / 3600);

            loop {
                interval_timer.tick().await;
                
                info!("Running certificate renewal check...");
                
                for ssl_manager in &ssl_managers {
                    if let Err(e) = Self::check_and_renew_certificates(
                        ssl_manager.clone(), 
                        renewal_threshold_days
                    ).await {
                        error!("Failed to check/renew certificates for SSL manager: {}", e);
                    }
                }
                
                info!("Certificate renewal check completed");
            }
        });
    }

    async fn check_and_renew_certificates(
        ssl_manager: Arc<SslManager>, 
        threshold_days: u32
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if ACME is enabled
        if !ssl_manager.is_auto_cert_enabled() {
            return Ok(());
        }

        let domains = ssl_manager.get_managed_domains().await;
        
        for domain in domains {
            match ssl_manager.get_certificate_expiry(&domain).await {
                Ok(Some(expiry)) => {
                    let days_until_expiry = (expiry - chrono::Utc::now()).num_days();
                    
                    if days_until_expiry <= threshold_days as i64 {
                        info!("Certificate for domain '{}' expires in {} days, initiating renewal", 
                              domain, days_until_expiry);
                        
                        match ssl_manager.renew_certificate_public(&domain).await {
                            Ok(_) => {
                                info!("Successfully renewed certificate for domain '{}'", domain);
                            }
                            Err(e) => {
                                error!("Failed to renew certificate for domain '{}': {}", domain, e);
                            }
                        }
                    } else {
                        info!("Certificate for domain '{}' is valid for {} more days", 
                              domain, days_until_expiry);
                    }
                }
                Ok(None) => {
                    warn!("No certificate found for domain '{}', attempting to obtain one", domain);
                    
                    if let Err(e) = ssl_manager.renew_certificate_public(&domain).await {
                        error!("Failed to obtain certificate for domain '{}': {}", domain, e);
                    }
                }
                Err(e) => {
                    error!("Failed to check certificate expiry for domain '{}': {}", domain, e);
                }
            }
        }

        Ok(())
    }
}
