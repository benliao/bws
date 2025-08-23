use bws::{ServerConfig, WebServerService};
use pingora::prelude::*;

fn main() {
    env_logger::init();

    // Load configuration
    let config = ServerConfig::load_from_file("config.toml").unwrap_or_else(|e| {
        eprintln!("Failed to load config.toml: {}", e);
        std::process::exit(1);
    });

    println!("Loaded configuration for {} sites:", config.sites.len());
    for site in &config.sites {
        println!(
            "  - {} ({}:{}) -> {}",
            site.name, site.hostname, site.port, site.static_dir
        );
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
    my_server.run_forever();
}
