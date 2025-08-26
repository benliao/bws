//! Command-line interface for BWS operations
//!
//! This module provides CLI commands for hot reload, upgrade, and other
//! BWS server management operations.

use clap::{Parser, Subcommand};
use std::path::Path;
use std::process;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "bws-ctl")]
#[command(about = "BWS Server Control Utility")]
pub struct BwsCtl {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Hot reload server configuration
    Reload {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
        /// Process ID of the BWS server
        #[arg(short, long)]
        pid: Option<u32>,
        /// Validate configuration without applying
        #[arg(long)]
        validate_only: bool,
    },
    /// Upgrade server to new binary
    Upgrade {
        /// Path to new binary (defaults to current executable)
        #[arg(short, long)]
        binary: Option<String>,
        /// Process ID of the BWS server
        #[arg(short, long)]
        pid: Option<u32>,
        /// Force upgrade even if current process is not BWS
        #[arg(long)]
        force: bool,
    },
    /// Show server status
    Status {
        /// Process ID of the BWS server
        #[arg(short, long)]
        pid: Option<u32>,
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
    },
    /// Gracefully shutdown server
    Shutdown {
        /// Process ID of the BWS server
        #[arg(short, long)]
        pid: Option<u32>,
        /// Force shutdown (SIGKILL)
        #[arg(long)]
        force: bool,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
}

impl BwsCtl {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Reload {
                ref config,
                pid,
                validate_only,
            } => self.handle_reload(config.clone(), pid, validate_only).await,
            Commands::Upgrade {
                ref binary,
                pid,
                force,
            } => self.handle_upgrade(binary.clone(), pid, force).await,
            Commands::Status { pid, detailed } => self.handle_status(pid, detailed).await,
            Commands::Shutdown { pid, force } => self.handle_shutdown(pid, force).await,
            Commands::Validate { ref config } => self.handle_validate(config.clone()).await,
        }
    }

    async fn handle_reload(
        &self,
        config_path: String,
        pid: Option<u32>,
        validate_only: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Processing reload command for config: {}", config_path);

        // Check if config file exists
        if !Path::new(&config_path).exists() {
            error!("Configuration file not found: {}", config_path);
            process::exit(1);
        }

        if validate_only {
            // Just validate the configuration
            info!("Validating configuration file: {}", config_path);
            let config = crate::config::ServerConfig::load_from_file(&config_path)?;
            config.validate()?;
            info!("✅ Configuration validation successful");
            return Ok(());
        }

        if let Some(target_pid) = pid {
            // Send reload signal to specific process
            #[cfg(unix)]
            {
                crate::core::signals::utils::send_reload_signal(target_pid)?;
                info!("✅ Reload signal sent to process {}", target_pid);
            }
            #[cfg(windows)]
            {
                let _ = target_pid; // Suppress unused variable warning on Windows
                error!("Signal-based reload not supported on Windows");
                process::exit(1);
            }
        } else {
            // Try to find BWS process automatically
            let bws_pid = self.find_bws_process()?;
            #[cfg(unix)]
            {
                crate::core::signals::utils::send_reload_signal(bws_pid)?;
                info!("✅ Reload signal sent to BWS process {}", bws_pid);
            }
            #[cfg(windows)]
            {
                let _ = bws_pid; // Suppress unused variable warning on Windows
                error!("Signal-based reload not supported on Windows");
                process::exit(1);
            }
        }

        #[cfg(unix)]
        {
            Ok(())
        }
    }

    async fn handle_upgrade(
        &self,
        binary_path: Option<String>,
        pid: Option<u32>,
        _force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Processing upgrade command");

        let binary = binary_path.unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("bws"))
                .to_string_lossy()
                .to_string()
        });

        info!("Upgrade binary: {}", binary);

        if let Some(target_pid) = pid {
            // Send upgrade signal to specific process
            #[cfg(unix)]
            {
                crate::core::signals::utils::send_upgrade_signal(target_pid)?;
                info!("✅ Upgrade signal sent to process {}", target_pid);
            }
            #[cfg(windows)]
            {
                let _ = target_pid; // Suppress unused variable warning on Windows
                error!("Signal-based upgrade not supported on Windows");
                process::exit(1);
            }
        } else {
            // Try to find BWS process automatically
            let bws_pid = self.find_bws_process()?;
            #[cfg(unix)]
            {
                crate::core::signals::utils::send_upgrade_signal(bws_pid)?;
                info!("✅ Upgrade signal sent to BWS process {}", bws_pid);
            }
            #[cfg(windows)]
            {
                let _ = bws_pid; // Suppress unused variable warning on Windows
                error!("Signal-based upgrade not supported on Windows");
                process::exit(1);
            }
        }

        #[cfg(unix)]
        {
            Ok(())
        }
    }

    async fn handle_status(
        &self,
        pid: Option<u32>,
        detailed: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Processing status command");

        let target_pid = if let Some(pid) = pid {
            pid
        } else {
            self.find_bws_process()?
        };

        // Check if process is running
        if !self.is_process_running(target_pid) {
            error!("Process {} is not running", target_pid);
            process::exit(1);
        }

        println!("BWS Server Status");
        println!("================");
        println!("Process ID: {}", target_pid);
        println!("Status: Running");

        if detailed {
            // Try to get more detailed information
            // This would typically involve querying the server's API endpoints
            println!("\nDetailed Status:");
            println!("- Memory usage: [Would need process monitoring]");
            println!("- CPU usage: [Would need process monitoring]");
            println!("- Uptime: [Would need to query server API]");
            println!("- Active connections: [Would need to query server API]");
            println!("- Request rate: [Would need to query server API]");
        }

        Ok(())
    }

    async fn handle_shutdown(
        &self,
        pid: Option<u32>,
        #[allow(unused_variables)] force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Processing shutdown command");

        let target_pid = if let Some(pid) = pid {
            pid
        } else {
            self.find_bws_process()?
        };

        if !self.is_process_running(target_pid) {
            error!("Process {} is not running", target_pid);
            process::exit(1);
        }

        #[cfg(unix)]
        {
            if force {
                // Send SIGKILL
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                kill(Pid::from_raw(target_pid as i32), Signal::SIGKILL)?;
                info!("✅ Force shutdown signal sent to process {}", target_pid);
            } else {
                // Send SIGTERM for graceful shutdown
                crate::core::signals::utils::send_shutdown_signal(target_pid)?;
                info!("✅ Graceful shutdown signal sent to process {}", target_pid);
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we would use TerminateProcess or similar
            error!("Signal-based shutdown not supported on Windows");
            process::exit(1);
        }

        #[cfg(unix)]
        {
            Ok(())
        }
    }

    async fn handle_validate(&self, config_path: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("Validating configuration file: {}", config_path);

        if !Path::new(&config_path).exists() {
            error!("Configuration file not found: {}", config_path);
            process::exit(1);
        }

        let config = crate::config::ServerConfig::load_from_file(&config_path)?;
        config.validate()?;

        println!("✅ Configuration validation successful");
        println!("Server: {}", config.server.name);
        println!("Sites: {}", config.sites.len());
        for site in &config.sites {
            println!("  - {} ({}:{})", site.name, site.hostname, site.port);
            if site.ssl.enabled {
                println!("    SSL: Enabled (auto_cert: {})", site.ssl.auto_cert);
            }
            if site.proxy.enabled {
                println!("    Proxy: {} routes", site.proxy.routes.len());
            }
        }

        Ok(())
    }

    fn find_bws_process(&self) -> Result<u32, Box<dyn std::error::Error>> {
        // Try to find BWS process by reading PID file or process list
        // For now, we'll try to read from a standard PID file location
        let pid_file_paths = vec!["/var/run/bws.pid", "/tmp/bws.pid", "./bws.pid"];

        for path in pid_file_paths {
            if let Ok(pid_str) = std::fs::read_to_string(path) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if self.is_process_running(pid) {
                        return Ok(pid);
                    }
                }
            }
        }

        // If PID file approach fails, we could scan processes
        // For now, return an error
        Err("BWS process not found. Please specify --pid".into())
    }

    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), None) {
                Ok(()) => true,
                Err(_) => false,
            }
        }

        #[cfg(windows)]
        {
            use std::process::Command;

            // On Windows, use tasklist to check if process exists
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid)])
                .output();

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bws_ctl_parsing() {
        // Test reload command
        let args = vec!["bws-ctl", "reload", "--config", "test.toml"];
        let ctl = BwsCtl::try_parse_from(args).unwrap();

        match ctl.command {
            Commands::Reload { config, .. } => {
                assert_eq!(config, "test.toml");
            }
            _ => panic!("Expected reload command"),
        }
    }

    #[test]
    fn test_upgrade_command_parsing() {
        let args = vec!["bws-ctl", "upgrade", "--binary", "/usr/local/bin/bws"];
        let ctl = BwsCtl::try_parse_from(args).unwrap();

        match ctl.command {
            Commands::Upgrade { binary, .. } => {
                assert_eq!(binary, Some("/usr/local/bin/bws".to_string()));
            }
            _ => panic!("Expected upgrade command"),
        }
    }

    #[test]
    fn test_validate_command_parsing() {
        let args = vec!["bws-ctl", "validate"];
        let ctl = BwsCtl::try_parse_from(args).unwrap();

        match ctl.command {
            Commands::Validate { config } => {
                assert_eq!(config, "config.toml");
            }
            _ => panic!("Expected validate command"),
        }
    }
}
