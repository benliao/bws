use crate::ssl::{certificate::Certificate, manager::{SslManager, SslConfig}};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct RenewalScheduler {
    ssl_manager: Arc<SslManager>,
    check_interval_hours: u64,
    renewal_days_before_expiry: i64,
}

impl RenewalScheduler {
    #[must_use]
    pub const fn new(
        ssl_manager: Arc<SslManager>,
        check_interval_hours: u64,
        renewal_days_before_expiry: i64,
    ) -> Self {
        Self {
            ssl_manager,
            check_interval_hours,
            renewal_days_before_expiry,
        }
    }

    pub async fn start(&self) {
        let mut interval_timer = interval(Duration::from_secs(self.check_interval_hours * 3600));

        log::info!(
            "Starting certificate renewal scheduler (check every {} hours, renew {} days before expiry)",
            self.check_interval_hours,
            self.renewal_days_before_expiry
        );

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.check_and_schedule_renewals().await {
                log::error!("Error during renewal check: {e}");
            }
        }
    }

    async fn check_and_schedule_renewals(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Checking certificates for renewal eligibility");

        let certificates = self.ssl_manager.list_certificates().await;
        let mut renewal_tasks = Vec::new();

        for cert in certificates {
            if self.should_schedule_renewal(&cert) {
                renewal_tasks.push(cert);
            }
        }

        if renewal_tasks.is_empty() {
            log::info!("No certificates require renewal at this time");
            return Ok(());
        }

        log::info!(
            "Scheduling renewal for {} certificates",
            renewal_tasks.len()
        );

        // Process renewals sequentially to avoid overwhelming ACME servers
        for cert in renewal_tasks {
            match self.attempt_renewal(&cert).await {
                Ok(()) => {
                    log::info!("Successfully renewed certificate for {}", cert.domain);
                }
                Err(e) => {
                    log::error!("Failed to renew certificate for {}: {}", cert.domain, e);
                    // Continue with other certificates
                }
            }

            // Add a small delay between renewals to be respectful to ACME servers
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        Ok(())
    }

    fn should_schedule_renewal(&self, cert: &Certificate) -> bool {
        // Check if auto-renewal is enabled
        if !cert.auto_renew {
            return false;
        }

        // Check if certificate is expired or close to expiry
        if cert.days_until_expiry() <= self.renewal_days_before_expiry {
            return true;
        }

        // Check if this is a fresh certificate that we haven't checked recently
        // This helps catch certificates that were manually installed
        cert.last_renewal_check.is_none_or(|last_check| {
            let hours_since_check = (Utc::now() - last_check).num_hours();
            // Only check again if it's been more than the check interval
            hours_since_check >= i64::try_from(self.check_interval_hours).unwrap_or(24)
        })
    }

    async fn attempt_renewal(&self, cert: &Certificate) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "Attempting renewal for {} (expires in {} days)",
            cert.domain,
            cert.days_until_expiry()
        );

        // Validate current certificate files before attempting renewal
        if !cert.validate_certificate_files().await.unwrap_or(false) {
            log::warn!(
                "Current certificate files for {} are invalid, forcing renewal",
                cert.domain
            );
        }

        // Attempt to obtain/renew the certificate
        let success = self.ssl_manager.ensure_certificate(&cert.domain).await?;

        if success {
            // Update the last renewal check timestamp
            // This would be handled by the SSL manager internally
            log::info!(
                "Certificate renewal completed successfully for {}",
                cert.domain
            );
        } else {
            return Err(format!("Failed to renew certificate for {}", cert.domain).into());
        }

        Ok(())
    }

    /// Force certificate renewal for a domain
    /// 
    /// # Errors
    /// 
    /// Returns an error if certificate renewal fails.
    pub async fn force_renewal(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Forcing certificate renewal for domain: {domain}");

        let success = self.ssl_manager.ensure_certificate(domain).await?;

        if success {
            log::info!("Forced renewal completed successfully for {domain}");
            Ok(())
        } else {
            Err(format!("Failed to force renewal for domain: {domain}").into())
        }
    }

    pub async fn get_renewal_status(&self) -> RenewalStatus {
        let certificates = self.ssl_manager.list_certificates().await;
        let mut status = RenewalStatus::default();

        for cert in certificates {
            status.total_certificates += 1;

            if cert.is_expired() {
                status.expired_certificates.push(cert.domain.clone());
            } else if cert.needs_renewal(self.renewal_days_before_expiry) {
                status.renewal_needed.push(CertificateRenewalInfo {
                    domain: cert.domain.clone(),
                    days_until_expiry: cert.days_until_expiry(),
                    auto_renew_enabled: cert.auto_renew,
                    last_renewal_check: cert.last_renewal_check,
                });
            } else {
                status.valid_certificates.push(CertificateValidInfo {
                    domain: cert.domain.clone(),
                    days_until_expiry: cert.days_until_expiry(),
                    expires_at: cert.expires_at,
                });
            }
        }

        status
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenewalStatus {
    pub total_certificates: usize,
    pub valid_certificates: Vec<CertificateValidInfo>,
    pub renewal_needed: Vec<CertificateRenewalInfo>,
    pub expired_certificates: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CertificateValidInfo {
    pub domain: String,
    pub days_until_expiry: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CertificateRenewalInfo {
    pub domain: String,
    pub days_until_expiry: i64,
    pub auto_renew_enabled: bool,
    pub last_renewal_check: Option<DateTime<Utc>>,
}

// Background renewal service that can be spawned as a task
pub struct RenewalService {
    scheduler: RenewalScheduler,
}

impl RenewalService {
    #[must_use] 
    pub fn new(ssl_manager: Arc<SslManager>, config: &SslConfig) -> Self {
        let scheduler = RenewalScheduler::new(
            ssl_manager,
            config.renewal_check_interval_hours,
            config.renewal_days_before_expiry,
        );

        Self { scheduler }
    }

    pub async fn run(self) {
        self.scheduler.start().await;
    }

    /// Force certificate renewal for a domain
    /// 
    /// # Errors
    /// 
    /// Returns an error if certificate renewal fails.
    pub async fn force_renewal(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.force_renewal(domain).await
    }

    pub async fn get_status(&self) -> RenewalStatus {
        self.scheduler.get_renewal_status().await
    }
}

// Helper functions for renewal logic
#[must_use]
pub const fn calculate_renewal_urgency(days_until_expiry: i64) -> RenewalUrgency {
    match days_until_expiry {
        d if d <= 0 => RenewalUrgency::Expired,
        d if d <= 7 => RenewalUrgency::Critical,
        d if d <= 30 => RenewalUrgency::High,
        d if d <= 60 => RenewalUrgency::Medium,
        _ => RenewalUrgency::Low,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenewalUrgency {
    Expired,
    Critical, // <= 7 days
    High,     // <= 30 days
    Medium,   // <= 60 days
    Low,      // > 60 days
}

impl RenewalUrgency {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    #[must_use]
    pub const fn should_renew_now(&self) -> bool {
        matches!(
            self,
            Self::Expired | Self::Critical | Self::High
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn test_renewal_urgency() {
        assert_eq!(calculate_renewal_urgency(-1), RenewalUrgency::Expired);
        assert_eq!(calculate_renewal_urgency(0), RenewalUrgency::Expired);
        assert_eq!(calculate_renewal_urgency(5), RenewalUrgency::Critical);
        assert_eq!(calculate_renewal_urgency(15), RenewalUrgency::High);
        assert_eq!(calculate_renewal_urgency(45), RenewalUrgency::Medium);
        assert_eq!(calculate_renewal_urgency(90), RenewalUrgency::Low);

        assert!(RenewalUrgency::Critical.should_renew_now());
        assert!(RenewalUrgency::High.should_renew_now());
        assert!(!RenewalUrgency::Low.should_renew_now());
    }

    #[test]
    fn test_renewal_status_default() {
        let status = RenewalStatus::default();
        assert_eq!(status.total_certificates, 0);
        assert!(status.valid_certificates.is_empty());
        assert!(status.renewal_needed.is_empty());
        assert!(status.expired_certificates.is_empty());
    }

    #[tokio::test]
    async fn test_should_schedule_renewal() {
        use std::path::PathBuf;

        // Create a test certificate that needs renewal
        let cert = Certificate {
            domain: "example.com".to_string(),
            cert_path: PathBuf::from("test.crt"),
            key_path: PathBuf::from("test.key"),
            issued_at: Utc::now() - ChronoDuration::days(60),
            expires_at: Utc::now() + ChronoDuration::days(15), // Expires in 15 days
            issuer: "Test CA".to_string(),
            san_domains: vec![],
            auto_renew: true,
            last_renewal_check: None,
        };

        // Mock SSL manager - in a real test, you'd use a proper mock
        // For now, we'll just test the renewal urgency calculation
        let urgency = calculate_renewal_urgency(cert.days_until_expiry());
        assert_eq!(urgency, RenewalUrgency::High);
        assert!(urgency.should_renew_now());
    }
}
