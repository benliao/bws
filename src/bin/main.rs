use bws_web_server::config::{
    LoggingConfig, PerformanceConfig, SecurityConfig, ServerConfig, ServerInfo, SiteConfig,
};
use bws_web_server::server::{ManagementApiService, WebServerService};
use clap::Parser;
#[cfg(unix)]
use daemonize::Daemonize;
use pingora::listeners::tls::TlsSettings;
use pingora::prelude::*;
use std::collections::HashMap;
#[cfg(unix)]
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Clean a Windows extended path format for display purposes.
/// On Windows, strips the \\?\ prefix; on other platforms, returns the path unchanged.
fn clean_path_for_display(path: &str) -> String {
    if path.starts_with("\\\\?\\") {
        path.strip_prefix("\\\\?\\").unwrap_or(path).to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn test_clean_path_for_display_windows() {
        let input = r"\\?\C:\Users\test";
        let cleaned = clean_path_for_display(input);
        assert_eq!(cleaned, "C:\\Users\\test");
    }

    #[test]
    fn test_clean_path_for_display_normal() {
        let input = "/usr/local/bin";
        let cleaned = clean_path_for_display(input);
        assert_eq!(cleaned, "/usr/local/bin");
    }

    #[test]
    fn test_generate_random_port_range() {
        for _ in 0..100 {
            let port = generate_random_port();
            assert!((7000..=9000).contains(&port));
        }
    }
}

/// Generate a random port between 7000 and 9000 (inclusive).
/// Used for temporary server instances when no port is specified.
fn generate_random_port() -> u16 {
    fastrand::u16(7000..=9000)
}

#[derive(Parser)]
#[command(name = "bws")]
#[command(about = "BWS - High-performance multi-site web server")]
#[command(version)]
#[command(
    long_about = "BWS is a high-performance multi-site web server with SSL/TLS, load balancing, and health monitoring.

Quick start: bws [path]        - Serve directory on random port (7000-9000)
With config: bws -c config.toml - Use configuration file"
)]
struct Cli {
    /// Directory path to serve (creates temporary server on port 80)
    #[arg(help = "Directory to serve as static files")]
    directory: Option<String>,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Port to use when serving directory (default: random between 7000-9000)
    #[arg(short, long)]
    port: Option<u16>,

    /// Run as daemon (background process) - Unix only
    #[cfg(unix)]
    #[arg(short, long)]
    daemon: bool,

    /// PID file path when running as daemon - Unix only
    #[cfg(unix)]
    #[arg(long, default_value = "/tmp/bws-web-server.pid")]
    pid_file: String,

    /// Log file path when running as daemon - Unix only
    #[cfg(unix)]
    #[arg(long, default_value = "/tmp/bws-web-server.log")]
    log_file: String,

    /// Validate configuration and exit (do not start server)
    #[arg(long)]
    dry_run: bool,
}

/// Create a temporary server configuration for serving a single directory.
/// Validates the directory and returns a ServerConfig with a single site.
fn create_temporary_config(directory: &str, port: u16) -> ServerConfig {
    // Validate that the directory exists
    if !Path::new(directory).exists() {
        eprintln!("Error: Directory '{}' does not exist", directory);
        std::process::exit(1);
    }

    if !Path::new(directory).is_dir() {
        eprintln!("Error: '{}' is not a directory", directory);
        std::process::exit(1);
    }

    // Convert to absolute path, but clean it for Windows compatibility
    let absolute_dir = match std::fs::canonicalize(directory) {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            clean_path_for_display(&path_str)
        }
        Err(_) => directory.to_string(),
    };

    println!("Creating temporary web server:");
    println!("  Directory: {}", absolute_dir);
    println!("  Port: {}", port);
    println!("  URL: http://localhost:{}", port);

    // Create a simple site configuration
    let site = SiteConfig {
        name: "main".to_string(),
        hostname: "localhost".to_string(),
        hostnames: vec![],
        port,
        static_dir: absolute_dir,
        default: true,
        api_only: false,
        headers: HashMap::new(),
        redirect_to_https: false,
        index_files: vec![
            "index.html".to_string(),
            "index.htm".to_string(),
            "default.html".to_string(),
        ],
        error_pages: HashMap::new(),
        compression: Default::default(),
        cache: Default::default(),
        access_control: Default::default(),
        ssl: Default::default(),
        proxy: Default::default(),
    };

    // Create server configuration
    ServerConfig {
        server: ServerInfo {
            name: "BWS Temporary Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: format!("Temporary server for directory: {}", directory),
        },
        sites: vec![site],
        logging: LoggingConfig::default(),
        performance: PerformanceConfig::default(),
        security: SecurityConfig::default(),
        management: Default::default(),
    }
}

/// Handle dry-run mode: validate configuration and exit
fn handle_dry_run(config: &ServerConfig, cli: &Cli) {
    println!("BWS Configuration Validation");
    println!("============================");

    if cli.directory.is_some() {
        println!(" Temporary directory configuration created successfully");
        println!("    Directory: {}", cli.directory.as_ref().unwrap());
        // Extract port from the first site in the config
        let port = config.sites.first().map(|s| s.port).unwrap_or(8080);
        println!("    Port: {}", port);
    } else {
        let config_path = cli.config.as_deref().unwrap_or("config.toml");
        println!(" Configuration file '{}' loaded successfully", config_path);
    }

    println!("\n Configuration Summary:");
    println!(
        "   Server: {} v{}",
        config.server.name, config.server.version
    );
    println!("   Sites: {}", config.sites.len());

    let mut validation_errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate each site configuration
    for (index, site) in config.sites.iter().enumerate() {
        println!("\n Site {}: {}", index + 1, site.name);
        println!("   Hostname: {}", site.hostname);
        if !site.hostnames.is_empty() {
            println!("   Additional hostnames: {}", site.hostnames.join(", "));
        }
        println!("   Port: {}", site.port);
        println!("   Static directory: {}", site.static_dir);

        // Validate static directory exists
        if !std::path::Path::new(&site.static_dir).exists() {
            validation_errors.push(format!(
                "Site '{}': Static directory '{}' does not exist",
                site.name, site.static_dir
            ));
        } else if !std::path::Path::new(&site.static_dir).is_dir() {
            validation_errors.push(format!(
                "Site '{}': Static path '{}' is not a directory",
                site.name, site.static_dir
            ));
        } else {
            println!("    Static directory exists");
        }

        // Validate index files exist
        let mut index_found = false;
        for index_file in &site.index_files {
            let index_path = std::path::Path::new(&site.static_dir).join(index_file);
            if index_path.exists() {
                println!("    Index file found: {}", index_file);
                index_found = true;
                break;
            }
        }
        if !index_found && !site.index_files.is_empty() {
            warnings.push(format!(
                "Site '{}': No index files found in static directory",
                site.name
            ));
        }

        // Validate SSL configuration
        if site.ssl.enabled {
            println!("    SSL enabled");
            if site.ssl.auto_cert {
                println!("    Auto-certificate (ACME) enabled");
                if let Some(acme) = &site.ssl.acme {
                    if acme.enabled {
                        if acme.email.is_empty() {
                            validation_errors.push(format!(
                                "Site '{}': ACME email is required when auto_cert is enabled",
                                site.name
                            ));
                        } else {
                            println!("    ACME email: {}", acme.email);
                        }
                    }
                }
            } else {
                // Check for manual certificates
                let cert_path = format!("./certs/{}.crt", site.hostname);
                let key_path = format!("./certs/{}.key", site.hostname);

                if std::path::Path::new(&cert_path).exists()
                    && std::path::Path::new(&key_path).exists()
                {
                    println!("    SSL certificates found");
                } else {
                    warnings.push(format!(
                        "Site '{}': SSL enabled but certificates not found at {} and {}",
                        site.name, cert_path, key_path
                    ));
                }
            }
        }

        // Validate proxy configuration
        if site.proxy.enabled {
            println!("    Proxy enabled");
            if site.proxy.upstreams.is_empty() {
                validation_errors.push(format!(
                    "Site '{}': Proxy enabled but no upstreams configured",
                    site.name
                ));
            } else {
                println!("    Upstreams: {}", site.proxy.upstreams.len());
                for upstream in &site.proxy.upstreams {
                    println!("     - {}: {}", upstream.name, upstream.url);
                }
            }

            if site.proxy.routes.is_empty() {
                warnings.push(format!(
                    "Site '{}': Proxy enabled but no routes configured",
                    site.name
                ));
            } else {
                println!("     Routes: {}", site.proxy.routes.len());
            }
        }

        // Check for custom headers
        if !site.headers.is_empty() {
            println!("    Custom headers: {}", site.headers.len());
        }
    }

    // Check for port conflicts
    let mut port_usage = std::collections::HashMap::new();
    for site in &config.sites {
        port_usage
            .entry(site.port)
            .or_insert_with(Vec::new)
            .push(&site.name);
    }

    for (port, sites) in &port_usage {
        if sites.len() > 1 {
            // Multiple sites on same port - check if they have different hostnames
            let mut hostnames = std::collections::HashSet::new();
            let mut has_default = false;

            for site_name in sites {
                if let Some(site) = config.sites.iter().find(|s| &s.name == *site_name) {
                    hostnames.insert(&site.hostname);
                    hostnames.extend(&site.hostnames);
                    if site.default {
                        if has_default {
                            validation_errors
                                .push(format!("Port {}: Multiple sites marked as default", port));
                        }
                        has_default = true;
                    }
                } else {
                    validation_errors.push(format!("Invalid site reference: {}", site_name));
                }
            }

            if hostnames.len() == sites.len() || has_default {
                println!(
                    "\n Port {} shared by {} sites with virtual hosting",
                    port,
                    sites.len()
                );
            } else {
                warnings.push(format!(
                    "Port {}: Multiple sites with overlapping hostnames",
                    port
                ));
            }
        }
    }

    // Print validation results
    println!("\n==========================================");
    println!("           VALIDATION RESULTS");
    println!("==========================================");

    if !warnings.is_empty() {
        println!("  Warnings ({}): ", warnings.len());
        for warning in &warnings {
            println!("     {}", warning);
        }
        println!();
    }

    if validation_errors.is_empty() {
        println!(" Configuration validation passed!");
        println!(" Configuration is ready for deployment");
        std::process::exit(0);
    } else {
        println!(
            " Configuration validation failed ({} errors):",
            validation_errors.len()
        );
        for error in &validation_errors {
            println!("    {}", error);
        }
        println!("\n Fix the errors above and try again");
        std::process::exit(1);
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Print help or version and exit
            if let Err(print_err) = err.print() {
                eprintln!("Failed to print help: {}", print_err);
            }
            std::process::exit(match err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 1,
            });
        }
    };

    // Initialize Rustls crypto provider
    if let Err(e) = rustls::crypto::aws_lc_rs::default_provider().install_default() {
        eprintln!("Failed to install default crypto provider: {e:?}");
        std::process::exit(1);
    }

    // Handle daemon mode (Unix only)
    #[cfg(unix)]
    if cli.daemon {
        println!("Starting BWS server as daemon...");
        println!("PID file: {}", cli.pid_file);
        println!("Log file: {}", cli.log_file);

        let stdout = File::create(&cli.log_file).unwrap_or_else(|e| {
            eprintln!("Failed to create log file '{}': {e}", cli.log_file);
            std::process::exit(1);
        });
        let stderr = stdout.try_clone().unwrap_or_else(|e| {
            eprintln!("Failed to clone log file handle: {e}");
            std::process::exit(1);
        });

        let daemonize = Daemonize::new()
            .pid_file(&cli.pid_file)
            .chown_pid_file(true)
            .working_directory(".") // Use current directory instead of /tmp
            .stdout(stdout)
            .stderr(stderr);

        match daemonize.start() {
            Ok(_) => {
                // We're now in the daemon process
                log::info!("BWS server daemonized successfully");
            }
            Err(e) => {
                eprintln!("Error starting daemon: {e}");
                std::process::exit(1);
            }
        }
    }

    // Initialize logging based on verbosity
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    // Load configuration from specified file or create temporary config
    let config = if let Some(directory) = &cli.directory {
        // Create temporary configuration for serving a directory
        let port = cli.port.unwrap_or_else(generate_random_port);
        create_temporary_config(directory, port)
    } else if let Some(config_path) = &cli.config {
        // Load configuration from explicitly specified file
        ServerConfig::load_from_file(config_path).unwrap_or_else(|e| {
            eprintln!("Failed to load configuration from '{}': {e}", config_path);
            std::process::exit(1);
        })
    } else {
        // No directory and no config file specified - check if default exists
        let default_config = "config.toml";
        if Path::new(default_config).exists() {
            ServerConfig::load_from_file(default_config).unwrap_or_else(|e| {
                eprintln!(
                    "Failed to load configuration from '{}': {e}",
                    default_config
                );
                std::process::exit(1);
            })
        } else {
            // No config file found - show usage information
            let default_port = generate_random_port();
            eprintln!("Usage:");
            eprintln!(
                "  bws [directory]                  - Serve directory on random port (7000-9000)"
            );
            eprintln!("  bws --config config.toml         - Use configuration file");
            eprintln!(
                "  bws . --port 8080               - Serve current directory on specific port"
            );
            eprintln!();
            eprintln!("Examples:");
            eprintln!(
                "  bws /path/to/website            - Serve website files on port {}",
                default_port
            );
            eprintln!("  bws . --port 3000               - Serve current dir on port 3000");
            eprintln!("  bws --config my-config.toml     - Use custom config");
            std::process::exit(1);
        }
    };

    // Handle dry-run mode: validate configuration and exit
    if cli.dry_run {
        return handle_dry_run(&config, &cli);
    }

    if cli.directory.is_some() {
        println!(" Temporary web server ready!");
    } else {
        let config_path = cli.config.as_deref().unwrap_or("config.toml");
        println!(
            "Loaded configuration from '{}' for {} sites:",
            config_path,
            config.sites.len()
        );
    }

    for site in &config.sites {
        if cli.directory.is_some() {
            println!(
                "   Serving: {} on http://{}:{}",
                clean_path_for_display(&site.static_dir),
                site.hostname,
                site.port
            );
        } else {
            println!(
                "  - {} ({}:{}) -> {}",
                site.name,
                site.hostname,
                site.port,
                clean_path_for_display(&site.static_dir)
            );
        }
        if cli.verbose && !site.headers.is_empty() {
            println!("    Headers: {:?}", site.headers);
        }
    }

    let mut my_server = Server::new(None).unwrap_or_else(|e| {
        eprintln!("Failed to create server: {e}");
        std::process::exit(1);
    });

    // Create the main web service instance
    let web_service = WebServerService::new(config.clone());

    // Set the config path for hot reload (only if not in temporary directory mode)
    if cli.directory.is_none() && cli.config.is_some() {
        let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            log::error!("Failed to create runtime for config path setup: {e}");
            std::process::exit(1);
        });
        rt.block_on(web_service.set_config_path(cli.config.clone().unwrap()));
        log::info!(" Config hot reload enabled via API at POST /api/config/reload");
    }

    // Check if any site has ACME enabled and create a dedicated HTTP challenge service on port 80
    let has_acme_enabled = config.sites.iter().any(|site| {
        site.ssl.enabled
            && site.ssl.auto_cert
            && site.ssl.acme.as_ref().is_some_and(|acme| acme.enabled)
    });

    if has_acme_enabled {
        // Check if we already have a service listening on port 80
        let has_port_80 = config.sites.iter().any(|site| site.port == 80);

        if has_port_80 {
            log::info!("Port 80 already configured for ACME challenges");
        } else {
            log::info!("Creating dedicated HTTP challenge service on port 80 for ACME validation");
            let mut acme_service =
                pingora::proxy::http_proxy_service(&my_server.configuration, web_service.clone());
            acme_service.add_tcp("0.0.0.0:80");
            my_server.add_service(acme_service);
        }
    }

    // Pre-initialize SSL certificates for ACME-enabled sites
    if has_acme_enabled {
        log::info!("Initializing ACME certificates before starting server...");
        let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            log::error!("Failed to create async runtime for SSL initialization: {e}");
            std::process::exit(1);
        });
        if let Err(e) = runtime.block_on(web_service.initialize_ssl_managers()) {
            log::error!("Failed to initialize SSL managers: {e}");
            log::warn!("Server will start without HTTPS support until certificates are obtained");
        } else {
            log::info!("SSL managers initialized successfully");
        }
    }

    // Create a service for each site configuration
    for site in &config.sites {
        let service_name = format!("BWS Site: {}", site.name);
        let mut proxy_service =
            pingora::proxy::http_proxy_service(&my_server.configuration, web_service.clone());

        let listen_addr = format!("0.0.0.0:{}", site.port);

        // Check if this is an HTTPS site and if we have certificates
        if site.ssl.enabled {
            // Check if certificates are available
            let cert_path = format!("./certs/{}.crt", site.hostname);
            let key_path = format!("./certs/{}.key", site.hostname);

            if std::path::Path::new(&cert_path).exists() && std::path::Path::new(&key_path).exists()
            {
                // Certificates available - configure TLS listener
                log::info!(
                    "Configuring HTTPS for site '{}' on {} (certificates found)",
                    site.name,
                    listen_addr
                );

                match TlsSettings::intermediate(&cert_path, &key_path) {
                    Ok(tls_settings) => {
                        proxy_service.add_tls_with_settings(&listen_addr, None, tls_settings);
                        log::info!(
                            "HTTPS listener configured successfully for site '{}'",
                            site.name
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to load TLS settings for {}: {}", site.name, e);
                        log::warn!("Falling back to HTTP for site '{}'", site.name);
                        proxy_service.add_tcp(&listen_addr);
                    }
                }
            } else {
                // No certificates - add HTTP listener only
                log::warn!(
                    "Certificates not found for site '{}' - serving HTTP only",
                    site.name
                );
                log::info!("Expected: {cert_path} and {key_path}");
                proxy_service.add_tcp(&listen_addr);
            }
        } else {
            // Regular HTTP site
            proxy_service.add_tcp(&listen_addr);
            log::info!("HTTP listener configured for site '{}'", site.name);
        }

        log::info!("Starting service '{service_name}' on {listen_addr}");
        my_server.add_service(proxy_service);
    }

    // Add management API service if enabled
    if config.management.enabled {
        log::info!(
            "Starting Management API service on {}:{}",
            config.management.host,
            config.management.port
        );
        let management_service =
            ManagementApiService::new(Arc::new(web_service.clone()), config.management.clone());
        let mut management_proxy_service =
            pingora::proxy::http_proxy_service(&my_server.configuration, management_service);
        let management_addr = format!("{}:{}", config.management.host, config.management.port);
        management_proxy_service.add_tcp(&management_addr);
        my_server.add_service(management_proxy_service);

        log::info!(" Management API enabled at http://{}", management_addr);
        if config.management.api_key.is_some() {
            log::info!(" API key authentication required for management endpoints");
        } else {
            log::warn!("  Management API has no API key - consider setting one for production");
        }
    } else {
        log::info!("Management API disabled");
    }

    log::info!("Starting BWS multi-site server...");
    my_server.bootstrap();

    // Display clickable URLs for each site after server starts (only in foreground mode)
    #[cfg(unix)]
    let is_daemon = cli.daemon;
    #[cfg(not(unix))]
    let is_daemon = false;

    if !is_daemon {
        if cli.directory.is_some() {
            println!("\n BWS Temporary Directory Server");
            println!("\n Quick Start Server:");
        } else {
            println!("\n BWS Multi-Site Server is running!");
            println!("Available websites:");
        }

        for site in &config.sites {
            let protocol = if site.ssl.enabled {
                // Check if certificates exist to determine actual protocol
                let cert_path = format!("./certs/{}.crt", site.hostname);
                if std::path::Path::new(&cert_path).exists() {
                    "https"
                } else {
                    "http"
                }
            } else {
                "http"
            };

            let url = format!(
                "{}://{}:{}",
                protocol,
                if site.hostname == "localhost" || site.hostname.ends_with(".localhost") {
                    site.hostname.clone()
                } else {
                    "localhost".to_string()
                },
                site.port
            );

            // Display clickable URL with site description
            println!("  {} - {}", site.name, url);

            // Show certificate status for SSL sites
            if site.ssl.enabled {
                let cert_path = format!("./certs/{}.crt", site.hostname);
                if std::path::Path::new(&cert_path).exists() {
                    println!("    HTTPS enabled (certificates found)");
                } else {
                    println!("    HTTP only (certificates not found)");
                    if site.ssl.auto_cert {
                        println!("    ACME auto-renewal enabled");
                    }
                }
            }

            // Show common endpoints for each site
            if cli.verbose {
                println!("Health: {}/api/health", url);
                println!("Sites: {}/api/sites", url);
            }
        }

        // Show management API information
        if config.management.enabled {
            println!("\nManagement API:");
            let mgmt_url = format!(
                "http://{}:{}",
                config.management.host, config.management.port
            );
            println!("  Config Reload: {}/api/config/reload", mgmt_url);
            if config.management.api_key.is_some() {
                println!("     API key required (use X-API-Key header)");
            } else {
                println!("     No authentication (localhost only)");
            }
        }

        if cli.directory.is_some() {
            println!("\n TEMPORARY SERVER MODE:");
            println!("  Press Ctrl+C to stop the server");
            if let Some(directory) = &cli.directory {
                println!("  Files served from: {}", directory);
            }
            println!("  Simple static file server (no configuration file)");
        } else {
            println!("\n Tip: Use Ctrl+C to stop the server");
            if !cli.verbose {
                println!(" Use --verbose to see health check URLs");
            }
        }
        println!();
    } else {
        log::info!("BWS daemon started successfully");
        log::info!(
            "Available sites: {}",
            config
                .sites
                .iter()
                .map(|s| format!("{}:{}", s.name, s.port))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Start certificate monitoring and renewal in background
    if has_acme_enabled {
        let web_service_for_monitoring = web_service;
        std::thread::spawn(move || {
            log::info!("Starting certificate monitoring and auto-renewal service...");
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("Failed to create async runtime for certificate monitoring: {e}");
                    log::error!("Certificate monitoring will be disabled");
                    return;
                }
            };

            loop {
                // Check and renew certificates every hour
                std::thread::sleep(std::time::Duration::from_secs(3600));

                if let Err(e) =
                    runtime.block_on(web_service_for_monitoring.check_and_renew_certificates())
                {
                    log::error!("Certificate renewal check failed: {e}");
                } else {
                    log::debug!("Certificate renewal check completed successfully");
                }
            }
        });
    }

    // Initialize hot reload functionality if not in temporary directory mode
    my_server.run_forever();
}
