use bws_web_server::{ServerConfig, WebServerService};
use clap::Parser;
#[cfg(unix)]
use daemonize::Daemonize;
use pingora::prelude::*;
#[cfg(unix)]
use std::fs::File;

#[derive(Parser)]
#[command(name = "bws-web-server")]
#[command(
    about = "BWS (Ben's Web Server) - A high-performance multi-site web server built with Pingora"
)]
#[command(version)]
#[command(
    long_about = "BWS is a high-performance, multi-site web server that can host multiple websites \
on different ports with individual configurations. It supports configurable headers, static file serving, \
and health monitoring endpoints."
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

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

fn main() {
    let cli = Cli::parse();

    // Handle daemon mode (Unix only)
    #[cfg(unix)]
    if cli.daemon {
        println!("Starting BWS server as daemon...");
        println!("PID file: {}", cli.pid_file);
        println!("Log file: {}", cli.log_file);

        let stdout = File::create(&cli.log_file).unwrap_or_else(|e| {
            eprintln!("Failed to create log file '{}': {}", cli.log_file, e);
            std::process::exit(1);
        });
        let stderr = stdout.try_clone().unwrap();

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
                eprintln!("Error starting daemon: {}", e);
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

    // Load configuration from specified file
    let config = ServerConfig::load_from_file(&cli.config).unwrap_or_else(|e| {
        eprintln!("Failed to load config file '{}': {}", cli.config, e);
        eprintln!("Make sure the config file exists and is properly formatted.");
        eprintln!("Use --help for more information.");
        std::process::exit(1);
    });

    println!(
        "Loaded configuration from '{}' for {} sites:",
        cli.config,
        config.sites.len()
    );
    for site in &config.sites {
        println!(
            "  - {} ({}:{}) -> {}",
            site.name, site.hostname, site.port, site.static_dir
        );
        if cli.verbose && !site.headers.is_empty() {
            println!("    Headers: {:?}", site.headers);
        }
    }

    let mut my_server = Server::new(None).unwrap();

    // Create a service for each site configuration
    for site in &config.sites {
        let service_name = format!("BWS Site: {}", site.name);
        let mut proxy_service = pingora::proxy::http_proxy_service(
            &my_server.configuration,
            WebServerService::new(config.clone()),
        );

        // Listen on the configured port
        let listen_addr = format!("0.0.0.0:{}", site.port);
        proxy_service.add_tcp(&listen_addr);

        log::info!("Starting service '{}' on {}", service_name, listen_addr);
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
        println!("\n🚀 BWS Multi-Site Server is running!");
        println!("📋 Available websites:");

        for site in &config.sites {
            let url = format!(
                "http://{}:{}",
                if site.hostname == "localhost" || site.hostname.ends_with(".localhost") {
                    site.hostname.clone()
                } else {
                    "localhost".to_string()
                },
                site.port
            );

            // Display clickable URL with site description
            println!("  • {} - {}", site.name, url);

            // Show common endpoints for each site
            if cli.verbose {
                println!("    └─ Health: {}/api/health", url);
                println!("    └─ Sites: {}/api/sites", url);
            }
        }

        println!("\n💡 Tip: Use Ctrl+C to stop the server");
        if !cli.verbose {
            println!("💡 Use --verbose to see health check URLs");
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

    my_server.run_forever();
}
