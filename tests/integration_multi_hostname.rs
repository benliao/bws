use bws_web_server::config::{ServerConfig, SiteConfig, SiteSslConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_multi_hostname_integration() {
    // Create a site configuration with multiple hostnames
    let site = SiteConfig {
        name: "test-multi-hostname".to_string(),
        hostname: "example.com".to_string(),
        hostnames: vec![
            "www.example.com".to_string(),
            "example.org".to_string(),
            "www.example.org".to_string(),
        ],
        port: 8080,
        static_dir: "./test-static".to_string(),
        default: true,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    // Test hostname matching
    assert!(site.handles_hostname("example.com"));
    assert!(site.handles_hostname("www.example.com"));
    assert!(site.handles_hostname("example.org"));
    assert!(site.handles_hostname("www.example.org"));
    assert!(!site.handles_hostname("different.com"));

    // Test hostname:port matching
    assert!(site.handles_hostname_port("example.com", 8080));
    assert!(site.handles_hostname_port("www.example.com", 8080));
    assert!(site.handles_hostname_port("example.org", 8080));
    assert!(!site.handles_hostname_port("example.com", 8081));

    // Test getting all hostnames
    let all_hostnames = site.get_all_hostnames();
    assert_eq!(all_hostnames.len(), 4);
    assert!(all_hostnames.contains(&"example.com"));
    assert!(all_hostnames.contains(&"www.example.com"));
    assert!(all_hostnames.contains(&"example.org"));
    assert!(all_hostnames.contains(&"www.example.org"));

    // Test configuration validation
    assert!(site.validate().is_ok());

    // Create server configuration with the multi-hostname site
    let server_config = ServerConfig {
        server: bws_web_server::config::ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: "Multi-hostname test server".to_string(),
        },
        sites: vec![site],
        logging: Default::default(),
        performance: Default::default(),
        security: Default::default(),
    };

    // Test server configuration validation
    assert!(server_config.validate().is_ok());

    // Test site lookup by hostname:port
    assert!(server_config
        .find_site_by_host_port("example.com", 8080)
        .is_some());
    assert!(server_config
        .find_site_by_host_port("www.example.com", 8080)
        .is_some());
    assert!(server_config
        .find_site_by_host_port("example.org", 8080)
        .is_some());
    assert!(server_config
        .find_site_by_host_port("www.example.org", 8080)
        .is_some());

    // Test site lookup by domain
    assert!(server_config.get_site_by_domain("example.com").is_some());
    assert!(server_config
        .get_site_by_domain("www.example.com")
        .is_some());
    assert!(server_config.get_site_by_domain("example.org").is_some());
    assert!(server_config
        .get_site_by_domain("www.example.org")
        .is_some());

    println!("✅ Multi-hostname integration test passed!");
}

#[tokio::test]
async fn test_multi_hostname_ssl_integration() {
    // Create a site configuration with SSL and multiple hostnames
    let mut site = SiteConfig {
        name: "test-ssl-multi-hostname".to_string(),
        hostname: "secure.example.com".to_string(),
        hostnames: vec![
            "ssl.example.com".to_string(),
            "https.example.com".to_string(),
        ],
        port: 443,
        static_dir: "./test-static".to_string(),
        default: true,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    // Enable SSL with additional domains
    site.ssl.enabled = true;
    site.ssl.domains = vec!["extra.example.com".to_string()];

    // Test SSL domain collection includes all hostnames
    let ssl_domains = site.get_all_ssl_domains();
    assert_eq!(ssl_domains.len(), 4);
    assert!(ssl_domains.contains(&"secure.example.com")); // Primary hostname
    assert!(ssl_domains.contains(&"ssl.example.com")); // Additional hostname
    assert!(ssl_domains.contains(&"https.example.com")); // Additional hostname
    assert!(ssl_domains.contains(&"extra.example.com")); // Additional SSL domain

    // Test hostname matching still works with SSL enabled
    assert!(site.handles_hostname("secure.example.com"));
    assert!(site.handles_hostname("ssl.example.com"));
    assert!(site.handles_hostname("https.example.com"));
    assert!(!site.handles_hostname("extra.example.com")); // SSL domain, not hostname

    println!("✅ Multi-hostname SSL integration test passed!");
}

#[tokio::test]
async fn test_multi_hostname_conflict_detection() {
    // Test that duplicate hostname:port combinations are detected
    let site1 = SiteConfig {
        name: "site1".to_string(),
        hostname: "example.com".to_string(),
        hostnames: vec!["www.example.com".to_string()],
        port: 8080,
        static_dir: "./test-static1".to_string(),
        default: true,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let site2 = SiteConfig {
        name: "site2".to_string(),
        hostname: "different.com".to_string(),
        hostnames: vec!["www.example.com".to_string()], // Conflict with site1
        port: 8080,
        static_dir: "./test-static2".to_string(),
        default: false,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let server_config = ServerConfig {
        server: bws_web_server::config::ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: "Conflict test server".to_string(),
        },
        sites: vec![site1, site2],
        logging: Default::default(),
        performance: Default::default(),
        security: Default::default(),
    };

    // Should fail validation due to hostname conflict
    let result = server_config.validate();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Duplicate hostname:port combination"));
    assert!(error_msg.contains("www.example.com:8080"));

    println!("✅ Multi-hostname conflict detection test passed!");
}
