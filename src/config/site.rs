use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteConfig {
    pub name: String,
    pub hostname: String,
    #[serde(default)]
    pub hostnames: Vec<String>, // Additional hostnames that share the same port and config
    pub port: u16,
    pub static_dir: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub api_only: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub redirect_to_https: bool,
    #[serde(default)]
    pub index_files: Vec<String>,
    #[serde(default)]
    pub error_pages: HashMap<u16, String>,
    #[serde(default)]
    pub compression: CompressionConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub access_control: AccessControlConfig,
    #[serde(default)]
    pub ssl: SiteSslConfig,
    #[serde(default)]
    pub proxy: ProxyConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct SiteSslConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_cert: bool,
    #[serde(default)]
    pub domains: Vec<String>, // Additional domains beyond hostname
    #[serde(default)]
    pub cert_file: Option<String>, // Manual certificate file
    #[serde(default)]
    pub key_file: Option<String>, // Manual key file
    #[serde(default)]
    pub acme: Option<SiteAcmeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SiteAcmeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub staging: bool,
    #[serde(default)]
    pub challenge_dir: Option<String>, // Make optional for automatic management
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct ProxyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub upstreams: Vec<UpstreamConfig>,
    #[serde(default)]
    pub routes: Vec<ProxyRoute>,
    #[serde(default)]
    pub health_check: HealthCheckConfig,
    #[serde(default)]
    pub load_balancing: LoadBalancingConfig,
    #[serde(default)]
    pub timeout: TimeoutConfig,
    #[serde(default)]
    pub headers: ProxyHeadersConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpstreamConfig {
    pub name: String,
    pub url: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub max_conns: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ProxyRoute {
    pub path: String,
    pub upstream: String, // References upstream name
    #[serde(default)]
    pub strip_prefix: bool,
    #[serde(default)]
    pub rewrite_target: Option<String>,
    #[serde(default)]
    pub websocket: bool, // Enable WebSocket proxying for this route
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HealthCheckConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_health_path")]
    pub path: String,
    #[serde(default = "default_health_interval")]
    pub interval: u64, // seconds
    #[serde(default = "default_health_timeout")]
    pub timeout: u64, // seconds
    #[serde(default = "default_health_retries")]
    pub retries: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LoadBalancingConfig {
    #[serde(default = "default_lb_method")]
    pub method: String, // "round_robin", "least_conn", "weighted"
    #[serde(default)]
    pub sticky_sessions: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TimeoutConfig {
    #[serde(default = "default_connect_timeout")]
    pub connect: u64, // seconds
    #[serde(default = "default_read_timeout")]
    pub read: u64, // seconds
    #[serde(default = "default_write_timeout")]
    pub write: u64, // seconds
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ProxyHeadersConfig {
    #[serde(default)]
    pub preserve_host: bool,
    #[serde(default)]
    pub add_forwarded: bool,
    #[serde(default)]
    pub add_x_forwarded: bool,
    #[serde(default)]
    pub remove: Vec<String>,
    #[serde(default)]
    pub add: HashMap<String, String>,
}

fn default_weight() -> u32 {
    1
}
fn default_health_path() -> String {
    "/health".to_string()
}
fn default_health_interval() -> u64 {
    30
}
fn default_health_timeout() -> u64 {
    5
}
fn default_health_retries() -> u32 {
    3
}
fn default_lb_method() -> String {
    "round_robin".to_string()
}
fn default_connect_timeout() -> u64 {
    10
}
fn default_read_timeout() -> u64 {
    30
}
fn default_write_timeout() -> u64 {
    30
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_health_path(),
            interval: default_health_interval(),
            timeout: default_health_timeout(),
            retries: default_health_retries(),
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            method: default_lb_method(),
            sticky_sessions: false,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: default_connect_timeout(),
            read: default_read_timeout(),
            write: default_write_timeout(),
        }
    }
}

impl Default for ProxyHeadersConfig {
    fn default() -> Self {
        Self {
            preserve_host: true,
            add_forwarded: true,
            add_x_forwarded: true,
            remove: Vec::new(),
            add: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompressionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_compression_types")]
    pub types: Vec<String>,
    #[serde(default = "default_compression_level")]
    pub level: u32,
    #[serde(default = "default_min_size")]
    pub min_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_cache_control")]
    pub cache_control: String,
    #[serde(default)]
    pub etag_enabled: bool,
    #[serde(default)]
    pub last_modified_enabled: bool,
    #[serde(default)]
    pub max_age_static: u32,
    #[serde(default)]
    pub max_age_dynamic: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessControlConfig {
    #[serde(default)]
    pub allow_methods: Vec<String>,
    #[serde(default)]
    pub allow_headers: Vec<String>,
    #[serde(default)]
    pub allow_origins: Vec<String>,
    #[serde(default)]
    pub allow_credentials: bool,
    #[serde(default)]
    pub max_age: u32,
}

// Default value functions
fn default_compression_types() -> Vec<String> {
    vec![
        "text/html".to_string(),
        "text/css".to_string(),
        "text/javascript".to_string(),
        "application/javascript".to_string(),
        "application/json".to_string(),
        "text/xml".to_string(),
        "application/xml".to_string(),
        "text/plain".to_string(),
    ]
}

fn default_compression_level() -> u32 {
    6
}

fn default_min_size() -> usize {
    1024 // 1KB
}

fn default_cache_control() -> String {
    "public, max-age=3600".to_string()
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            types: default_compression_types(),
            level: default_compression_level(),
            min_size: default_min_size(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_control: default_cache_control(),
            etag_enabled: true,
            last_modified_enabled: true,
            max_age_static: 3600, // 1 hour for static files
            max_age_dynamic: 300, // 5 minutes for dynamic content
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            allow_methods: vec!["GET".to_string(), "HEAD".to_string(), "OPTIONS".to_string()],
            allow_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_origins: vec!["*".to_string()],
            allow_credentials: false,
            max_age: 86400, // 24 hours
        }
    }
}

impl SiteConfig {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate required fields
        if self.name.is_empty() {
            return Err("Site name cannot be empty".into());
        }

        if self.hostname.is_empty() {
            return Err("Site hostname cannot be empty".into());
        }

        if self.port == 0 {
            return Err("Site port must be greater than 0".into());
        }

        if self.static_dir.is_empty() {
            return Err("Site static_dir cannot be empty".into());
        }

        // Validate static directory exists (or can be created)
        let static_path = Path::new(&self.static_dir);
        if !static_path.exists() {
            log::warn!(
                "Static directory does not exist for site '{}': {}",
                self.name,
                self.static_dir
            );
        }

        // Validate hostname format (basic check)
        if !self.is_valid_hostname() {
            return Err(format!("Invalid hostname format: {}", self.hostname).into());
        }

        // Validate additional hostnames
        for (i, hostname) in self.hostnames.iter().enumerate() {
            if hostname.is_empty() {
                return Err(format!("Additional hostname {} cannot be empty", i + 1).into());
            }
            if !self.is_hostname_valid(hostname) {
                return Err(format!("Invalid additional hostname format: {}", hostname).into());
            }
        }

        // Validate port range
        if self.port < 1 {
            return Err(format!("Invalid port number: {}", self.port).into());
        }

        // Validate SSL configuration
        if self.ssl.enabled {
            if self.ssl.auto_cert {
                if let Some(acme) = &self.ssl.acme {
                    if acme.email.is_empty() {
                        return Err("ACME email is required when auto_cert is enabled".into());
                    }
                } else {
                    return Err("ACME configuration is required when auto_cert is enabled".into());
                }
            } else if self.ssl.cert_file.is_none() || self.ssl.key_file.is_none() {
                return Err("Manual SSL requires both cert_file and key_file".into());
            }
        }

        // Validate index files
        for index_file in &self.index_files {
            if index_file.is_empty() {
                return Err("Index file name cannot be empty".into());
            }
        }

        // Validate error pages
        for (status_code, error_page) in &self.error_pages {
            if *status_code < 100 || *status_code > 999 {
                return Err(format!("Invalid HTTP status code: {}", status_code).into());
            }
            if error_page.is_empty() {
                return Err(
                    format!("Error page path cannot be empty for status {}", status_code).into(),
                );
            }
        }

        // Validate compression configuration
        self.compression.validate()?;

        // Validate cache configuration
        self.cache.validate()?;

        // Validate access control configuration
        self.access_control.validate()?;

        Ok(())
    }

    fn is_valid_hostname(&self) -> bool {
        self.is_hostname_valid(&self.hostname)
    }

    fn is_hostname_valid(&self, hostname: &str) -> bool {
        // Basic hostname validation
        if hostname.is_empty() || hostname.len() > 253 {
            return false;
        }

        // Allow localhost and IP addresses for development
        if hostname == "localhost"
            || hostname.starts_with("127.")
            || hostname.starts_with("0.0.0.0")
        {
            return true;
        }

        // Basic domain name validation
        hostname.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.chars().all(|c| c.is_alphanumeric() || c == '-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
    }

    pub fn get_ssl_domain(&self) -> Option<&str> {
        if self.ssl.enabled {
            // Return primary domain (hostname) plus any additional domains
            Some(&self.hostname)
        } else {
            None
        }
    }

    pub fn get_all_ssl_domains(&self) -> Vec<&str> {
        if self.ssl.enabled {
            let mut domains = vec![self.hostname.as_str()];
            // Add additional hostnames
            domains.extend(self.hostnames.iter().map(|h| h.as_str()));
            // Add explicit SSL domains from configuration
            domains.extend(self.ssl.domains.iter().map(|h| h.as_str()));
            domains
        } else {
            Vec::new()
        }
    }

    pub fn is_ssl_enabled(&self) -> bool {
        self.ssl.enabled
    }

    pub fn get_index_files(&self) -> Vec<&str> {
        if self.index_files.is_empty() {
            vec!["index.html", "index.htm"]
        } else {
            self.index_files.iter().map(|s| s.as_str()).collect()
        }
    }

    pub fn get_error_page(&self, status_code: u16) -> Option<&str> {
        self.error_pages.get(&status_code).map(|s| s.as_str())
    }

    pub fn should_compress(&self, content_type: &str, content_length: usize) -> bool {
        self.compression.enabled
            && content_length >= self.compression.min_size
            && self
                .compression
                .types
                .iter()
                .any(|t| content_type.starts_with(t))
    }

    pub fn get_cache_headers(&self, is_static: bool) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if self.cache.enabled {
            let max_age = if is_static {
                self.cache.max_age_static
            } else {
                self.cache.max_age_dynamic
            };

            headers.push((
                "Cache-Control".to_string(),
                format!("public, max-age={}", max_age),
            ));

            if self.cache.etag_enabled {
                // ETag would be calculated based on file content
                // This is a placeholder - actual implementation would calculate based on file
                headers.push(("ETag".to_string(), "\"placeholder\"".to_string()));
            }

            if self.cache.last_modified_enabled {
                // Last-Modified would be based on file modification time
                // This is a placeholder - actual implementation would use file mtime
                headers.push(("Last-Modified".to_string(), "placeholder".to_string()));
            }
        }

        headers
    }

    pub fn get_cors_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if !self.access_control.allow_origins.is_empty() {
            headers.push((
                "Access-Control-Allow-Origin".to_string(),
                self.access_control.allow_origins.join(", "),
            ));
        }

        if !self.access_control.allow_methods.is_empty() {
            headers.push((
                "Access-Control-Allow-Methods".to_string(),
                self.access_control.allow_methods.join(", "),
            ));
        }

        if !self.access_control.allow_headers.is_empty() {
            headers.push((
                "Access-Control-Allow-Headers".to_string(),
                self.access_control.allow_headers.join(", "),
            ));
        }

        if self.access_control.allow_credentials {
            headers.push((
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            ));
        }

        if self.access_control.max_age > 0 {
            headers.push((
                "Access-Control-Max-Age".to_string(),
                self.access_control.max_age.to_string(),
            ));
        }

        headers
    }

    pub fn url(&self) -> String {
        let protocol = if self.is_ssl_enabled() {
            "https"
        } else {
            "http"
        };
        let port_suffix = match (self.is_ssl_enabled(), self.port) {
            (true, 443) | (false, 80) => String::new(),
            _ => format!(":{}", self.port),
        };
        format!("{}://{}{}", protocol, self.hostname, port_suffix)
    }

    /// Check if this site handles the given hostname
    pub fn handles_hostname(&self, hostname: &str) -> bool {
        // Check primary hostname
        if self.hostname == hostname {
            return true;
        }

        // Check additional hostnames
        self.hostnames.iter().any(|h| h == hostname)
    }

    /// Get all hostnames handled by this site (primary + additional)
    pub fn get_all_hostnames(&self) -> Vec<&str> {
        let mut hostnames = vec![self.hostname.as_str()];
        hostnames.extend(self.hostnames.iter().map(|h| h.as_str()));
        hostnames
    }

    /// Check if this site handles the given hostname and port combination
    pub fn handles_hostname_port(&self, hostname: &str, port: u16) -> bool {
        self.port == port && self.handles_hostname(hostname)
    }
}

impl CompressionConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.level > 9 {
            return Err("Compression level must be between 0 and 9".into());
        }

        if self.types.is_empty() && self.enabled {
            return Err("Compression types cannot be empty when compression is enabled".into());
        }

        Ok(())
    }
}

impl CacheConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Cache configuration is generally permissive
        // Just ensure max_age values are reasonable
        if self.max_age_static > 365 * 24 * 3600 {
            log::warn!("Static cache max_age is very large (> 1 year)");
        }

        if self.max_age_dynamic > 24 * 3600 {
            log::warn!("Dynamic cache max_age is very large (> 1 day)");
        }

        Ok(())
    }
}

impl AccessControlConfig {
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate HTTP methods
        let valid_methods = [
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT",
        ];

        for method in &self.allow_methods {
            if !valid_methods.contains(&method.as_str()) {
                return Err(format!("Invalid HTTP method: {}", method).into());
            }
        }

        // Max age should be reasonable
        if self.max_age > 7 * 24 * 3600 {
            log::warn!("CORS max_age is very large (> 1 week)");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_config_validation() {
        let mut site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec![],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        assert!(site.validate().is_ok());

        // Test invalid hostname
        site.hostname = "".to_string();
        assert!(site.validate().is_err());

        // Test invalid port
        site.hostname = "example.com".to_string();
        site.port = 0;
        assert!(site.validate().is_err());

        // Test invalid port range (port 0 is invalid)
        site.port = 0;
        assert!(site.validate().is_err());
    }

    #[test]
    fn test_hostname_validation() {
        let site = SiteConfig {
            name: "test".to_string(),
            hostname: "localhost".to_string(),
            hostnames: vec![],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        assert!(site.is_valid_hostname());

        let mut invalid_site = site.clone();
        invalid_site.hostname = "invalid..hostname".to_string();
        assert!(!invalid_site.is_valid_hostname());

        invalid_site.hostname = "-invalid.hostname".to_string();
        assert!(!invalid_site.is_valid_hostname());
    }

    #[test]
    fn test_compression_config() {
        let site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec![],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        assert!(site.should_compress("text/html", 2048));
        assert!(!site.should_compress("text/html", 512)); // Below min_size
        assert!(!site.should_compress("image/png", 2048)); // Not in types list
    }

    #[test]
    fn test_site_url_generation() {
        let mut site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec![],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        assert_eq!(site.url(), "http://example.com:8080");

        site.ssl.enabled = true;
        site.port = 443;
        assert_eq!(site.url(), "https://example.com");

        site.port = 8443;
        assert_eq!(site.url(), "https://example.com:8443");
    }

    #[test]
    fn test_multi_hostname_functionality() {
        let site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec!["www.example.com".to_string(), "example.org".to_string()],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        // Test hostname handling
        assert!(site.handles_hostname("example.com"));
        assert!(site.handles_hostname("www.example.com"));
        assert!(site.handles_hostname("example.org"));
        assert!(!site.handles_hostname("different.com"));

        // Test hostname:port combination
        assert!(site.handles_hostname_port("example.com", 8080));
        assert!(site.handles_hostname_port("www.example.com", 8080));
        assert!(site.handles_hostname_port("example.org", 8080));
        assert!(!site.handles_hostname_port("example.com", 8081));
        assert!(!site.handles_hostname_port("different.com", 8080));

        // Test getting all hostnames
        let all_hostnames = site.get_all_hostnames();
        assert_eq!(all_hostnames.len(), 3);
        assert!(all_hostnames.contains(&"example.com"));
        assert!(all_hostnames.contains(&"www.example.com"));
        assert!(all_hostnames.contains(&"example.org"));
    }

    #[test]
    fn test_multi_hostname_ssl_domains() {
        let mut site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec!["www.example.com".to_string(), "api.example.com".to_string()],
            port: 443,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        // Enable SSL
        site.ssl.enabled = true;
        site.ssl.domains = vec!["cdn.example.com".to_string()];

        // Test SSL domain collection
        let ssl_domains = site.get_all_ssl_domains();
        assert_eq!(ssl_domains.len(), 4);
        assert!(ssl_domains.contains(&"example.com"));
        assert!(ssl_domains.contains(&"www.example.com"));
        assert!(ssl_domains.contains(&"api.example.com"));
        assert!(ssl_domains.contains(&"cdn.example.com"));
    }

    #[test]
    fn test_hostname_validation_with_additional_hostnames() {
        let mut site = SiteConfig {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            hostnames: vec![
                "www.example.com".to_string(),
                "valid.example.org".to_string(),
            ],
            port: 8080,
            static_dir: "/tmp".to_string(),
            default: false,
            api_only: false,
            headers: HashMap::new(),
            redirect_to_https: false,
            index_files: vec![],
            error_pages: HashMap::new(),
            compression: CompressionConfig::default(),
            cache: CacheConfig::default(),
            access_control: AccessControlConfig::default(),
            ssl: SiteSslConfig::default(),
            proxy: ProxyConfig::default(),
        };

        // Valid configuration should pass
        assert!(site.validate().is_ok());

        // Test invalid additional hostname
        site.hostnames = vec!["invalid..hostname".to_string()];
        assert!(site.validate().is_err());

        // Test empty additional hostname
        site.hostnames = vec!["".to_string()];
        assert!(site.validate().is_err());
    }
}
