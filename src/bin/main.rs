use bws_web_server::config::{
    LoggingConfig, PerformanceConfig, SecurityConfig, ServerConfig, ServerInfo, SiteConfig,
};
use bws_web_server::core::{HotReloadManager, SignalHandler};
use bws_web_server::server::WebServerService;
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

/// Clean Windows extended path format for display
fn clean_path_for_display(path: &str) -> String {
    if path.starts_with("\\\\?\\") {
        path.strip_prefix("\\\\?\\").unwrap_or(path).to_string()
    } else {
        path.to_string()
    }
}

#[derive(Parser)]
#[command(name = "bws-web-server")]
#[command(
    about = "BWS (Blazing Web Server) - A high-performance multi-site web server built with Pingora"
)]
#[command(version)]
#[command(
    long_about = "BWS is a high-performance, multi-site web server that can host multiple websites \
on different ports with individual configurations. It supports configurable headers, static file serving, \
and health monitoring endpoints.

Quick start: bws [path]        - Serve directory on port 80
With config: bws -c config.toml - Use configuration file"
)]
struct Cli {
    /// Directory path to serve (creates temporary server on port 80)
    #[arg(help = "Directory to serve as static files")]
    directory: Option<String>,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Port to use when serving directory (default: 80)
    #[arg(short, long, default_value = "80")]
    port: u16,

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
}

/// Create a temporary configuration for serving a single directory
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

    println!("馃殌 Creating temporary web server:");
    println!("   馃搧 Directory: {}", absolute_dir);
    println!("   馃寪 Port: {}", port);
    println!("   馃敆 URL: http://localhost:{}", port);
    println!();

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
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Print help or version and exit
            err.print().expect("Failed to print help");
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
        create_temporary_config(directory, cli.port)
    } else {
        // Load configuration from file
        ServerConfig::load_from_file(&cli.config).unwrap_or_else(|e| {
            eprintln!("Failed to load configuration from '{}': {e}", cli.config);
            std::process::exit(1);
        })
    };

    if cli.directory.is_some() {
        println!("馃寪 Temporary web server ready!");
    } else {
        println!(
            "Loaded configuration from '{}' for {} sites:",
            cli.config,
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

    log::info!("Starting BWS multi-site server...");
    my_server.bootstrap();

    // Display clickable URLs for each site after server starts (only in foreground mode)
    #[cfg(unix)]
    let is_daemon = cli.daemon;
    #[cfg(not(unix))]
    let is_daemon = false;

    if !is_daemon {
        if cli.directory.is_some() {
            println!("\n馃搧 BWS Temporary Directory Server");
            println!("馃搵 Quick Start Server:");
        } else {
            println!("\n馃殌 BWS Multi-Site Server is running!");
            println!("馃搵 Available websites:");
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
            println!("  鈥?{} - {}", site.name, url);

            // Show certificate status for SSL sites
            if site.ssl.enabled {
                let cert_path = format!("./certs/{}.crt", site.hostname);
                if std::path::Path::new(&cert_path).exists() {
                    println!("    鈹斺攢 鉁?HTTPS enabled (certificates found)");
                } else {
                    println!("    鈹斺攢 鈿狅笍  HTTP only (certificates not found)");
                    if site.ssl.auto_cert {
                        println!("    鈹斺攢 馃攧 ACME auto-renewal enabled");
                    }
                }
            }

            // Show common endpoints for each site
            if cli.verbose {
                println!("    鈹斺攢 Health: {}/api/health", url);
                println!("    鈹斺攢 Sites: {}/api/sites", url);
            }
        }

        if cli.directory.is_some() {
            println!("\n馃殌 TEMPORARY SERVER MODE:");
            println!("   鈥?Press `Ctrl+C` to stop the server");
            println!(
                "   鈥?Files are served directly from: {}",
                cli.directory.as_ref().unwrap()
            );
            println!("   鈥?Simple static file server (no configuration file)");
        } else {
            println!("\n馃挕 Tip: Use Ctrl+C to stop the server");
            if !cli.verbose {
                println!("馃挕 Use --verbose to see health check URLs");
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
    if cli.directory.is_none() {
        log::info!("🔄 Initializing hot reload manager...");
        let config_path = cli.config;
        let signal_handler = Arc::new(SignalHandler::new());

        // Start hot reload manager
        let reload_manager = HotReloadManager::new(
            config_path.clone(),
            config.clone(),
            Arc::clone(&signal_handler),
        );

        // Start as master process (this will handle the hot reload monitoring)
        tokio::spawn(async move {
            if let Err(e) = reload_manager.start_as_master().await {
                log::error!("Hot reload manager failed: {}", e);
            }
        });

        log::info!("✅ Hot reload manager started - send SIGUSR1 or modify config to reload");
    } else {
        log::info!("🔄 Hot reload disabled (temporary directory mode)");
    }

    my_server.run_forever();
}
