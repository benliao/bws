use crate::config::SiteConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub server: ServerInfo,
    pub sites: Vec<SiteConfig>,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub management: ManagementConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerInfo {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub access_log: Option<String>,
    #[serde(default)]
    pub error_log: Option<String>,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default)]
    pub log_requests: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerformanceConfig {
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_keep_alive_timeout")]
    pub keep_alive_timeout: u64,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_buffer_size")]
    pub read_buffer_size: String,
    #[serde(default = "default_buffer_size")]
    pub write_buffer_size: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    #[serde(default)]
    pub hide_server_header: bool,
    #[serde(default = "default_max_request_size")]
    pub max_request_size: String,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub security_headers: HashMap<String, String>,
    #[serde(default)]
    pub rate_limiting: Option<RateLimitConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    #[serde(default)]
    pub whitelist: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ManagementConfig {
    #[serde(default = "default_management_enabled")]
    pub enabled: bool,
    #[serde(default = "default_management_host")]
    pub host: String,
    #[serde(default = "default_management_port")]
    pub port: u16,
    #[serde(default)]
    pub api_key: Option<String>,
}

// Default value functions
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "combined".to_string()
}

fn default_worker_threads() -> usize {
    num_cpus::get().max(1)
}

fn default_max_connections() -> usize {
    1000
}

fn default_keep_alive_timeout() -> u64 {
    60
}

fn default_request_timeout() -> u64 {
    30
}

fn default_buffer_size() -> String {
    "32KB".to_string()
}

fn default_max_request_size() -> String {
    "10MB".to_string()
}

fn default_management_enabled() -> bool {
    false
}

fn default_management_host() -> String {
    "127.0.0.1".to_string()
}

fn default_management_port() -> u16 {
    7654
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            access_log: None,
            error_log: None,
            format: default_log_format(),
            log_requests: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: default_worker_threads(),
            max_connections: default_max_connections(),
            keep_alive_timeout: default_keep_alive_timeout(),
            request_timeout: default_request_timeout(),
            read_buffer_size: default_buffer_size(),
            write_buffer_size: default_buffer_size(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut security_headers = HashMap::new();
        security_headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        security_headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        security_headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        security_headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );

        Self {
            hide_server_header: false,
            max_request_size: default_max_request_size(),
            allowed_origins: vec![],
            security_headers,
            rate_limiting: None,
        }
    }
}

impl Default for ManagementConfig {
    fn default() -> Self {
        Self {
            enabled: default_management_enabled(),
            host: default_management_host(),
            port: default_management_port(),
            api_key: None,
        }
    }
}

impl ServerConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: ServerConfig = toml::from_str(&content)?;

        // Post-process configuration first (to set automatic defaults)
        config.post_process()?;

        // Then validate the processed configuration
        config.validate()?;

        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        log::info!("Configuration saved to {}", path);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate server info
        if self.server.name.is_empty() {
            return Err("Server name cannot be empty".into());
        }

        // Validate sites
        if self.sites.is_empty() {
            return Err("At least one site must be configured".into());
        }

        let mut default_sites = 0;
        let mut used_hostname_ports = std::collections::HashSet::new();

        for (i, site) in self.sites.iter().enumerate() {
            site.validate().map_err(|e| format!("Site {}: {}", i, e))?;

            if site.default {
                default_sites += 1;
            }

            // Check for hostname:port conflicts across all hostnames for this site
            // Allow multiple sites on same port with different hostnames (virtual hosting)
            for hostname in site.get_all_hostnames() {
                let hostname_port_key = (hostname, site.port);
                if used_hostname_ports.contains(&hostname_port_key) {
                    return Err(format!(
                        "Duplicate hostname:port combination: {}:{}. Each hostname must be unique per port.",
                        hostname, site.port
                    )
                    .into());
                }
                used_hostname_ports.insert(hostname_port_key);
            }
        }

        if default_sites == 0 {
            // This should not happen after post_process, but let's be defensive
            if self.sites.len() == 1 {
                // Single site should have been auto-marked as default in post_process
                return Err(
                    "Internal error: single site was not marked as default during post-processing"
                        .into(),
                );
            } else {
                return Err("At least one site must be marked as default when multiple sites are configured".into());
            }
        }
        if default_sites > 1 {
            return Err("Only one site can be marked as default".into());
        }

        // Validate that each site has proper SSL configuration if enabled
        for site in &self.sites {
            if site.ssl.enabled {
                if site.ssl.auto_cert {
                    if let Some(acme) = &site.ssl.acme {
                        if acme.email.is_empty() {
                            return Err(format!(
                                "Site '{}': ACME email is required when auto_cert is enabled",
                                site.name
                            )
                            .into());
                        }
                    } else {
                        return Err(format!(
                            "Site '{}': ACME configuration is required when auto_cert is enabled",
                            site.name
                        )
                        .into());
                    }
                } else if site.ssl.cert_file.is_none() || site.ssl.key_file.is_none() {
                    return Err(format!(
                        "Site '{}': Manual SSL requires both cert_file and key_file",
                        site.name
                    )
                    .into());
                }
            }
        }

        // Validate performance configuration
        self.performance.validate()?;

        // Validate security configuration
        self.security.validate()?;

        Ok(())
    }

    fn post_process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // If there's only one site and no site is explicitly marked as default,
        // automatically make the single site the default
        if self.sites.len() == 1 && !self.sites[0].default {
            log::info!(
                "Automatically setting single site '{}' as default",
                self.sites[0].name
            );
            self.sites[0].default = true;
        }

        // Sort sites by priority (default site first, then by name)
        self.sites.sort_by(|a, b| match (a.default, b.default) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        // No additional post-processing needed for per-site SSL
        // Each site manages its own SSL configuration

        Ok(())
    }

    pub fn find_site_by_host_port(&self, host: &str, port: u16) -> Option<&SiteConfig> {
        // First try to match both hostname and port exactly using the new method
        for site in &self.sites {
            if site.handles_hostname_port(host, port) {
                return Some(site);
            }
        }

        // Then try to match just the port (for cases where hostname might not match exactly)
        for site in &self.sites {
            if site.port == port {
                return Some(site);
            }
        }

        // Finally, return the default site if no match
        self.sites.iter().find(|site| site.default)
    }

    pub fn get_ssl_domains(&self) -> Vec<String> {
        self.sites
            .iter()
            .filter_map(|site| {
                if site.ssl.enabled {
                    Some(
                        site.get_all_ssl_domains()
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    pub fn get_site_by_domain(&self, domain: &str) -> Option<&SiteConfig> {
        self.sites.iter().find(|site| {
            site.handles_hostname(domain) || site.ssl.domains.contains(&domain.to_string())
        })
    }

    pub fn reload_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let new_config = Self::load_from_file(path)?;
        *self = new_config;
        log::info!("Configuration reloaded from {}", path);
        Ok(())
    }
}

impl PerformanceConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.worker_threads == 0 {
            return Err("Worker threads must be greater than 0".into());
        }

        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".into());
        }

        if self.keep_alive_timeout == 0 {
            return Err("Keep alive timeout must be greater than 0".into());
        }

        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".into());
        }

        // Validate buffer sizes
        self.parse_buffer_size(&self.read_buffer_size)
            .map_err(|_| "Invalid read buffer size format")?;
        self.parse_buffer_size(&self.write_buffer_size)
            .map_err(|_| "Invalid write buffer size format")?;

        Ok(())
    }

    pub fn parse_buffer_size(&self, size_str: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let size_str = size_str.trim().to_uppercase();

        if let Some(value) = size_str.strip_suffix("KB") {
            Ok(value.parse::<usize>()? * 1024)
        } else if let Some(value) = size_str.strip_suffix("MB") {
            Ok(value.parse::<usize>()? * 1024 * 1024)
        } else if let Some(value) = size_str.strip_suffix("GB") {
            Ok(value.parse::<usize>()? * 1024 * 1024 * 1024)
        } else if let Some(value) = size_str.strip_suffix("B") {
            Ok(value.parse::<usize>()?)
        } else {
            // Assume bytes if no suffix
            Ok(size_str.parse::<usize>()?)
        }
    }
}

impl SecurityConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate max request size format
        self.parse_size(&self.max_request_size)
            .map_err(|_| "Invalid max request size format")?;

        // Validate rate limiting configuration
        if let Some(rate_limit) = &self.rate_limiting {
            if rate_limit.requests_per_minute == 0 {
                return Err("Rate limit requests per minute must be greater than 0".into());
            }
            if rate_limit.burst_size == 0 {
                return Err("Rate limit burst size must be greater than 0".into());
            }
        }

        Ok(())
    }

    pub fn parse_size(&self, size_str: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let size_str = size_str.trim().to_uppercase();

        if let Some(value) = size_str.strip_suffix("KB") {
            Ok(value.parse::<usize>()? * 1024)
        } else if let Some(value) = size_str.strip_suffix("MB") {
            Ok(value.parse::<usize>()? * 1024 * 1024)
        } else if let Some(value) = size_str.strip_suffix("GB") {
            Ok(value.parse::<usize>()? * 1024 * 1024 * 1024)
        } else if let Some(value) = size_str.strip_suffix("B") {
            Ok(value.parse::<usize>()?)
        } else {
            // Assume bytes if no suffix
            Ok(size_str.parse::<usize>()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_buffer_size_parsing() {
        let config = PerformanceConfig::default();

        assert_eq!(config.parse_buffer_size("1024").unwrap(), 1024);
        assert_eq!(config.parse_buffer_size("1KB").unwrap(), 1024);
        assert_eq!(config.parse_buffer_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(config.parse_buffer_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(config.parse_buffer_size("500B").unwrap(), 500);

        assert!(config.parse_buffer_size("invalid").is_err());
        assert!(config.parse_buffer_size("").is_err());
    }

    #[test]
    fn test_automatic_default_site() {
        use crate::config::SiteConfig;

        // Test that a single site without explicit default=true gets auto-marked as default
        let mut config = ServerConfig {
            server: ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test server".to_string(),
            },
            sites: vec![SiteConfig {
                name: "single-site".to_string(),
                hostname: "localhost".to_string(),
                hostnames: vec![],
                port: 8080,
                static_dir: "/tmp/static".to_string(),
                default: false, // Explicitly NOT marked as default
                api_only: false,
                headers: HashMap::new(),
                redirect_to_https: false,
                index_files: vec!["index.html".to_string()],
                error_pages: HashMap::new(),
                compression: Default::default(),
                cache: Default::default(),
                access_control: Default::default(),
                ssl: Default::default(),
                proxy: Default::default(),
            }],
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            management: ManagementConfig::default(),
        };

        // Before post_process, the site should not be marked as default
        assert!(!config.sites[0].default);

        // After post_process, the single site should be automatically marked as default
        config.post_process().unwrap();
        assert!(config.sites[0].default);

        // Validation should pass
        assert!(config.validate().is_ok());

        // Test with multiple sites - auto-default should not apply
        config.sites.push(SiteConfig {
            name: "second-site".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec![],
            port: 8081,
            static_dir: "/tmp/static2".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec!["index.html".to_string()],
            error_pages: HashMap::new(),
            compression: Default::default(),
            cache: Default::default(),
            access_control: Default::default(),
            ssl: Default::default(),
            proxy: Default::default(),
        });

        // Reset first site's default flag
        config.sites[0].default = false;

        // Post-process should not auto-mark any site as default when there are multiple
        config.post_process().unwrap();
        assert!(!config.sites[0].default);
        assert!(!config.sites[1].default);

        // Validation should fail because no site is marked as default
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_security_config_validation() {
        let mut config = SecurityConfig::default();
        assert!(config.validate().is_ok());

        config.max_request_size = "invalid".to_string();
        assert!(config.validate().is_err());

        config.max_request_size = "10MB".to_string();
        config.rate_limiting = Some(RateLimitConfig {
            requests_per_minute: 0,
            burst_size: 10,
            whitelist: vec![],
        });
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_config_save_load() {
        use crate::config::SiteConfig;

        let config = ServerConfig {
            server: ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test server".to_string(),
            },
            sites: vec![SiteConfig {
                name: "test-site".to_string(),
                hostname: "localhost".to_string(),
                hostnames: vec![],
                port: 8080,
                static_dir: "/tmp/static".to_string(),
                default: true,
                api_only: false,
                headers: HashMap::new(),
                redirect_to_https: false,
                index_files: vec!["index.html".to_string()],
                error_pages: HashMap::new(),
                compression: Default::default(),
                cache: Default::default(),
                access_control: Default::default(),
                ssl: Default::default(),
                proxy: Default::default(),
            }],
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            management: ManagementConfig::default(),
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Test save
        config.save_to_file(path).unwrap();

        // Test load
        let loaded_config = ServerConfig::load_from_file(path).unwrap();
        assert_eq!(config.server.name, loaded_config.server.name);
        assert_eq!(config.sites.len(), loaded_config.sites.len());
    }
}
