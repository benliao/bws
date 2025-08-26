use bws_web_server::core::BwsCtl;
use clap::Parser;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let ctl = BwsCtl::parse();

    // Run the command
    if let Err(e) = ctl.run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
