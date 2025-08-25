use bws_web_server::config::{ServerConfig, ServerInfo, SiteConfig, SiteSslConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_virtual_hosting_multiple_sites_same_port() {
    // Create multiple sites that share the same port but have different hostnames
    let site1 = SiteConfig {
        name: "main-site".to_string(),
        hostname: "example.com".to_string(),
        hostnames: vec!["www.example.com".to_string()], // Additional hostname for same site
        port: 8080,
        static_dir: "./sites/main".to_string(),
        default: true,
        api_only: false,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("X-Site".to_string(), "Main".to_string());
            headers
        },
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
        name: "blog-site".to_string(),
        hostname: "blog.example.com".to_string(), // Different hostname
        hostnames: vec![],                        // No additional hostnames
        port: 8080,                               // Same port as site1
        static_dir: "./sites/blog".to_string(),   // Different content directory
        default: false,
        api_only: false,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("X-Site".to_string(), "Blog".to_string());
            headers
        },
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let site3 = SiteConfig {
        name: "api-site".to_string(),
        hostname: "api.example.com".to_string(), // Different hostname
        hostnames: vec![],                       // No additional hostnames
        port: 8080,                              // Same port as site1 and site2
        static_dir: "./sites/api".to_string(),   // Different content directory
        default: false,
        api_only: true,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("X-Site".to_string(), "API".to_string());
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers
        },
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let site4 = SiteConfig {
        name: "docs-site".to_string(),
        hostname: "docs.example.com".to_string(), // Different hostname
        hostnames: vec![],                        // No additional hostnames
        port: 8080,                               // Same port as all other sites
        static_dir: "./sites/docs".to_string(),   // Different content directory
        default: false,
        api_only: false,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("X-Site".to_string(), "Documentation".to_string());
            headers.insert(
                "Cache-Control".to_string(),
                "public, max-age=3600".to_string(),
            );
            headers
        },
        redirect_to_https: false,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    // Create server configuration with all sites
    let server_config = ServerConfig {
        server: ServerInfo {
            name: "Virtual Host Test Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Test server for virtual hosting".to_string(),
        },
        sites: vec![site1, site2, site3, site4],
        logging: Default::default(),
        performance: Default::default(),
        security: Default::default(),
    };

    // Validate configuration - should pass
    assert!(
        server_config.validate().is_ok(),
        "Virtual hosting configuration should be valid"
    );

    // Test hostname-based routing
    // Main site (primary hostname)
    let main_site = server_config.find_site_by_host_port("example.com", 8080);
    assert!(main_site.is_some());
    assert_eq!(main_site.unwrap().name, "main-site");

    // Main site (additional hostname)
    let main_site_www = server_config.find_site_by_host_port("www.example.com", 8080);
    assert!(main_site_www.is_some());
    assert_eq!(main_site_www.unwrap().name, "main-site");

    // Blog site
    let blog_site = server_config.find_site_by_host_port("blog.example.com", 8080);
    assert!(blog_site.is_some());
    assert_eq!(blog_site.unwrap().name, "blog-site");

    // API site
    let api_site = server_config.find_site_by_host_port("api.example.com", 8080);
    assert!(api_site.is_some());
    assert_eq!(api_site.unwrap().name, "api-site");

    // Docs site
    let docs_site = server_config.find_site_by_host_port("docs.example.com", 8080);
    assert!(docs_site.is_some());
    assert_eq!(docs_site.unwrap().name, "docs-site");

    // Unknown hostname should fall back to default site
    let unknown_site = server_config.find_site_by_host_port("unknown.example.com", 8080);
    assert!(unknown_site.is_some());
    assert_eq!(unknown_site.unwrap().name, "main-site"); // Should be default site

    // Test different port - should not match
    let wrong_port = server_config.find_site_by_host_port("example.com", 8081);
    assert!(wrong_port.is_some()); // Should still return default site
    assert_eq!(wrong_port.unwrap().name, "main-site");

    println!("✅ Virtual hosting with multiple sites on same port test passed!");
}

#[tokio::test]
async fn test_virtual_hosting_mixed_ports() {
    // Test sites on different ports
    let http_site = SiteConfig {
        name: "http-site".to_string(),
        hostname: "example.com".to_string(),
        hostnames: vec![],
        port: 80,
        static_dir: "./sites/http".to_string(),
        default: true,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: true, // Redirect to HTTPS
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let mut https_site = SiteConfig {
        name: "https-site".to_string(),
        hostname: "example.com".to_string(), // Same hostname, different port
        hostnames: vec![],
        port: 443,
        static_dir: "./sites/https".to_string(),
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

    // Enable SSL for HTTPS site
    https_site.ssl.enabled = true;
    https_site.ssl.auto_cert = true;
    https_site.ssl.acme = Some(bws_web_server::config::SiteAcmeConfig {
        enabled: true,
        email: "test@example.com".to_string(),
        staging: true,
        challenge_dir: None,
    });

    let api_http_site = SiteConfig {
        name: "api-http".to_string(),
        hostname: "api.example.com".to_string(),
        hostnames: vec![],
        port: 80, // Same port as http_site, different hostname
        static_dir: "./sites/api-http".to_string(),
        default: false,
        api_only: true,
        headers: HashMap::new(),
        redirect_to_https: true,
        index_files: vec!["index.html".to_string()],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: SiteSslConfig::default(),
        proxy: Default::default(),
    };

    let mut api_https_site = SiteConfig {
        name: "api-https".to_string(),
        hostname: "api.example.com".to_string(), // Same hostname as api_http_site, different port
        hostnames: vec![],
        port: 443,
        static_dir: "./sites/api-https".to_string(),
        default: false,
        api_only: true,
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

    // Enable SSL for API HTTPS site
    api_https_site.ssl.enabled = true;
    api_https_site.ssl.auto_cert = true;
    api_https_site.ssl.acme = Some(bws_web_server::config::SiteAcmeConfig {
        enabled: true,
        email: "test@example.com".to_string(),
        staging: true,
        challenge_dir: None,
    });

    let server_config = ServerConfig {
        server: ServerInfo {
            name: "Mixed Port Test Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Test server for mixed HTTP/HTTPS hosting".to_string(),
        },
        sites: vec![http_site, https_site, api_http_site, api_https_site],
        logging: Default::default(),
        performance: Default::default(),
        security: Default::default(),
    };

    // Validate configuration
    let validation_result = server_config.validate();
    if let Err(e) = &validation_result {
        println!("Validation error: {}", e);
    }
    assert!(
        validation_result.is_ok(),
        "Mixed port configuration should be valid"
    );

    // Test routing by hostname and port
    let http_main = server_config.find_site_by_host_port("example.com", 80);
    assert!(http_main.is_some());
    assert_eq!(http_main.unwrap().name, "http-site");

    let https_main = server_config.find_site_by_host_port("example.com", 443);
    assert!(https_main.is_some());
    assert_eq!(https_main.unwrap().name, "https-site");

    let http_api = server_config.find_site_by_host_port("api.example.com", 80);
    assert!(http_api.is_some());
    assert_eq!(http_api.unwrap().name, "api-http");

    let https_api = server_config.find_site_by_host_port("api.example.com", 443);
    assert!(https_api.is_some());
    assert_eq!(https_api.unwrap().name, "api-https");

    println!("✅ Virtual hosting with mixed HTTP/HTTPS ports test passed!");
}

#[tokio::test]
async fn test_virtual_hosting_conflict_detection() {
    // Test that hostname conflicts are properly detected
    let site1 = SiteConfig {
        name: "site1".to_string(),
        hostname: "example.com".to_string(),
        hostnames: vec![],
        port: 8080,
        static_dir: "./sites/site1".to_string(),
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
        hostname: "example.com".to_string(), // Same hostname as site1 - should conflict
        hostnames: vec![],
        port: 8080, // Same port as site1
        static_dir: "./sites/site2".to_string(),
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
        server: ServerInfo {
            name: "Conflict Test Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Test server for conflict detection".to_string(),
        },
        sites: vec![site1, site2],
        logging: Default::default(),
        performance: Default::default(),
        security: Default::default(),
    };

    // Should fail validation due to hostname:port conflict
    let result = server_config.validate();
    assert!(
        result.is_err(),
        "Configuration with hostname conflicts should fail validation"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Duplicate hostname:port combination"));
    assert!(error_msg.contains("example.com:8080"));

    println!("✅ Virtual hosting conflict detection test passed!");
}
