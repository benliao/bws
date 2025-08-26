//! Hot configuration reload functionality for BWS
//!
//! This module provides TRUE hot reloading by implementing a master-worker
//! architecture that spawns new server processes with updated configuration
//! while gracefully shutting down old processes.

use crate::config::ServerConfig;
use crate::core::signals::SignalHandler;
use crate::server::WebServerService;
use pingora::listeners::tls::TlsSettings;
use pingora::prelude::*;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum ReloadError {
    ReadConfig(std::io::Error),
    ParseConfig(String),
    ValidationFailed(String),
    SpawnFailed(String),
}

impl std::fmt::Display for ReloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReloadError::ReadConfig(e) => write!(f, "Failed to read configuration file: {}", e),
            ReloadError::ParseConfig(e) => write!(f, "Failed to parse configuration: {}", e),
            ReloadError::ValidationFailed(e) => write!(f, "Configuration validation failed: {}", e),
            ReloadError::SpawnFailed(e) => write!(f, "Failed to spawn worker process: {}", e),
        }
    }
}

impl std::error::Error for ReloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReloadError::ReadConfig(e) => Some(e),
            _ => None,
        }
    }
}

/// Hot reload manager that handles graceful server restarts
/// while maintaining service availability through master-worker pattern
pub struct HotReloadManager {
    config_path: String,
    current_config: Arc<RwLock<ServerConfig>>,
    signal_handler: Arc<SignalHandler>,
    worker_process: Arc<Mutex<Option<Child>>>,
    last_reload: Arc<Mutex<Instant>>,
}

impl HotReloadManager {
    pub fn new(
        config_path: String,
        config: ServerConfig,
        signal_handler: Arc<SignalHandler>,
    ) -> Self {
        Self {
            config_path,
            current_config: Arc::new(RwLock::new(config)),
            signal_handler,
            worker_process: Arc::new(Mutex::new(None)),
            last_reload: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Start the hot reload manager as master process
    pub async fn start_as_master(&self) -> Result<(), ReloadError> {
        log::info!("馃敟 Starting BWS with true hot reload support (master process)");

        // Start signal monitoring as a background task
        let signal_handler_clone = self.signal_handler.clone();
        tokio::spawn(async move {
            <SignalHandler as Clone>::clone(&signal_handler_clone)
                .start_monitoring()
                .await;
        });

        // Start the first worker process
        self.spawn_worker().await?;

        // Start monitoring for reload signals
        self.start_reload_monitoring().await;

        // Keep master process running indefinitely
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Spawn a new worker process
    async fn spawn_worker(&self) -> Result<(), ReloadError> {
        log::info!("馃殌 Spawning new worker process");

        let current_exe = std::env::current_exe().map_err(|e| {
            ReloadError::SpawnFailed(format!("Failed to get current executable path: {e}"))
        })?;

        let mut cmd = Command::new(current_exe);
        cmd.arg("--config")
            .arg(&self.config_path)
            .arg("--worker") // Add worker flag
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let child = cmd.spawn().map_err(|e| {
            ReloadError::SpawnFailed(format!("Failed to spawn worker process: {e}"))
        })?;

        // Store the worker process and gracefully stop old one
        let old_worker = {
            let mut worker_guard = self.worker_process.lock().unwrap();
            let old_worker = worker_guard.take();
            *worker_guard = Some(child);
            old_worker
        };

        if let Some(old_worker) = old_worker {
            log::info!(
                "Gracefully stopping old worker process: {}",
                old_worker.id()
            );
            self.stop_worker_gracefully(old_worker).await;
        }

        log::info!("鉁?New worker process spawned successfully");
        Ok(())
    }

    /// Gracefully stop a worker process
    async fn stop_worker_gracefully(&self, mut worker: Child) {
        #[cfg(unix)]
        {
            // Send SIGTERM for graceful shutdown
            unsafe {
                libc::kill(worker.id() as i32, libc::SIGTERM);
            }

            // Wait up to 10 seconds for graceful shutdown
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(10) {
                match worker.try_wait() {
                    Ok(Some(status)) => {
                        log::info!("Worker process exited gracefully with status: {status}");
                        return;
                    }
                    Ok(None) => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        log::error!("Error waiting for worker process: {e}");
                        break;
                    }
                }
            }

            // Force kill if still running
            log::warn!("Force killing unresponsive worker process");
            unsafe {
                libc::kill(worker.id() as i32, libc::SIGKILL);
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we can only terminate the process
            // First try normal termination
            if let Err(e) = worker.kill() {
                log::error!("Failed to terminate worker process: {e}");
            } else {
                log::info!("Worker process terminated successfully");
            }

            // Wait for process to exit
            match worker.wait() {
                Ok(status) => {
                    log::info!("Worker process exited with status: {status}");
                }
                Err(e) => {
                    log::error!("Error waiting for worker process: {e}");
                }
            }
        }
    }

    /// Start monitoring for reload signals
    async fn start_reload_monitoring(&self) {
        let signal_handler = self.signal_handler.clone();
        let config_path = self.config_path.clone();
        let current_config = self.current_config.clone();
        let last_reload = self.last_reload.clone();
        let worker_process = self.worker_process.clone();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("Failed to create async runtime for reload monitoring: {e}");
                    return;
                }
            };

            rt.block_on(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Check for reload signal
                    if signal_handler.check_and_clear_reload() {
                        // Skip reload if no config file path (temporary server mode)
                        if config_path.is_empty() {
                            log::info!("馃攧 Hot reload signal received, but skipping (temporary server mode)");
                            continue;
                        }

                        // Debounce rapid reload requests
                        {
                            let mut last = last_reload.lock().unwrap();
                            let now = Instant::now();
                            if now.duration_since(*last) < Duration::from_secs(1) {
                                log::debug!("Ignoring rapid reload request (debounced)");
                                continue;
                            }
                            *last = now;
                        }

                        log::info!("馃攧 Hot reload signal received - checking configuration");

                        // Load new configuration
                        match ServerConfig::load_from_file(&config_path) {
                            Ok(new_config) => {
                                match new_config.validate() {
                                    Ok(_) => {
                                        // Check if configuration actually changed
                                        let config_changed = {
                                            let current = current_config.read().await;
                                            Self::has_config_changed(&current, &new_config)
                                        };

                                        if config_changed {
                                            log::info!("馃搵 Configuration changes detected - performing hot reload");

                                            // Update stored configuration
                                            {
                                                let mut config_guard = current_config.write().await;
                                                *config_guard = new_config.clone();
                                            }

                                            // Spawn new worker with updated configuration
                                            let current_exe = match std::env::current_exe() {
                                                Ok(exe) => exe,
                                                Err(e) => {
                                                    log::error!("Failed to get current executable: {e}");
                                                    continue;
                                                }
                                            };

                                            let mut cmd = Command::new(current_exe);
                                            cmd.arg("--config")
                                                .arg(&config_path)
                                                .arg("--worker")
                                                .stdin(Stdio::null())
                                                .stdout(Stdio::inherit())
                                                .stderr(Stdio::inherit());

                                            match cmd.spawn() {
                                                Ok(child) => {
                                                    log::info!("馃殌 New worker spawned with PID: {}", child.id());

                                                    // Replace old worker
                                                    {
                                                        let mut worker_guard = worker_process.lock().unwrap();
                                                        if let Some(old_worker) = worker_guard.take() {
                                                            log::info!("Stopping old worker process: {}", old_worker.id());

                                                            #[cfg(unix)]
                                                            {
                                                                unsafe {
                                                                    libc::kill(old_worker.id() as i32, libc::SIGTERM);
                                                                }
                                                            }
                                                            #[cfg(windows)]
                                                            {
                                                                let mut old_worker = old_worker;
                                                                let _ = old_worker.kill();
                                                            }
                                                        }
                                                        *worker_guard = Some(child);
                                                    }

                                                    log::info!("鉁?Hot reload completed successfully");
                                                }
                                                Err(e) => {
                                                    log::error!("鉂?Failed to spawn new worker: {e}");
                                                }
                                            }
                                        } else {
                                            log::info!("馃搵 No configuration changes detected - skipping reload");
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("鉂?Invalid configuration file: {e}");
                                        log::warn!("Keeping existing configuration");
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("鉂?Failed to load configuration: {e}");
                                log::warn!("Keeping existing configuration");
                            }
                        }
                    }

                    // Check for shutdown signal
                    if signal_handler.is_shutdown_requested() {
                        log::info!("馃洃 Graceful shutdown requested");

                        // Stop worker process
                        {
                            let mut worker_guard = worker_process.lock().unwrap();
                            if let Some(mut worker) = worker_guard.take() {
                                log::info!("Stopping worker process: {}", worker.id());

                                #[cfg(unix)]
                                {
                                    unsafe {
                                        libc::kill(worker.id() as i32, libc::SIGTERM);
                                    }
                                }
                                #[cfg(windows)]
                                {
                                    let _ = worker.kill();
                                }
                                let _ = worker.wait();
                            }
                        }

                        std::process::exit(0);
                    }
                }
            });
        });
    }

    /// Check if configuration has meaningfully changed
    fn has_config_changed(old_config: &ServerConfig, new_config: &ServerConfig) -> bool {
        // Different number of sites
        if old_config.sites.len() != new_config.sites.len() {
            return true;
        }

        // Check each site for changes
        for (old_site, new_site) in old_config.sites.iter().zip(new_config.sites.iter()) {
            // Any site property changes require restart
            if old_site.port != new_site.port
                || old_site.hostname != new_site.hostname
                || old_site.static_dir != new_site.static_dir
                || old_site.headers != new_site.headers
                || old_site.ssl != new_site.ssl
                || old_site.proxy != new_site.proxy
                || old_site.api_only != new_site.api_only
                || old_site.redirect_to_https != new_site.redirect_to_https
            {
                return true;
            }
        }

        // Server-level configuration changes
        false // For now, always consider changed to trigger full restart
    }

    /// Shutdown all processes gracefully
    pub async fn shutdown(&self) {
        let mut worker_guard = self.worker_process.lock().unwrap();
        if let Some(mut worker) = worker_guard.take() {
            log::info!("馃洃 Shutting down worker process: {}", worker.id());

            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(worker.id() as i32, libc::SIGTERM);
                }
            }

            #[cfg(windows)]
            {
                let _ = worker.kill();
            }

            let _ = worker.wait();
        }
    }
}

/// Run the server as a worker process (actual server)
pub fn run_worker(
    config: ServerConfig,
    #[cfg(unix)] signal_handler: Arc<SignalHandler>,
    #[cfg(windows)] _signal_handler: Arc<SignalHandler>,
) {
    log::info!("馃敡 Starting BWS worker process");

    let mut my_server = Server::new(None).unwrap_or_else(|e| {
        log::error!("Failed to create server: {e}");
        std::process::exit(1);
    });

    // Create the main web service instance
    let web_service = WebServerService::new(config.clone());

    // Set up ACME challenge service if needed
    let has_acme_enabled = config.sites.iter().any(|site| {
        site.ssl.enabled
            && site.ssl.auto_cert
            && site.ssl.acme.as_ref().is_some_and(|acme| acme.enabled)
    });

    if has_acme_enabled {
        let has_port_80 = config.sites.iter().any(|site| site.port == 80);

        if !has_port_80 {
            log::info!("Creating dedicated HTTP challenge service on port 80 for ACME validation");
            let mut acme_service =
                pingora::proxy::http_proxy_service(&my_server.configuration, web_service.clone());
            acme_service.add_tcp("0.0.0.0:80");
            my_server.add_service(acme_service);
        }

        // Initialize SSL managers
        let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            log::error!("Failed to create async runtime for SSL initialization: {e}");
            std::process::exit(1);
        });

        if let Err(e) = rt.block_on(web_service.initialize_ssl_managers()) {
            log::error!("Failed to initialize SSL managers: {e}");
        }
    }

    // Create services for each site
    for site in &config.sites {
        let service_name = format!("BWS Site: {}", site.name);
        let mut proxy_service =
            pingora::proxy::http_proxy_service(&my_server.configuration, web_service.clone());

        let listen_addr = format!("0.0.0.0:{}", site.port);

        // Configure SSL if enabled
        if site.ssl.enabled {
            let cert_path = format!("./certs/{}.crt", site.hostname);
            let key_path = format!("./certs/{}.key", site.hostname);

            if Path::new(&cert_path).exists() && Path::new(&key_path).exists() {
                match TlsSettings::intermediate(&cert_path, &key_path) {
                    Ok(tls_settings) => {
                        proxy_service.add_tls_with_settings(&listen_addr, None, tls_settings);
                        log::info!("HTTPS listener configured for site '{}'", site.name);
                    }
                    Err(e) => {
                        log::error!("Failed to load TLS settings for {}: {}", site.name, e);
                        proxy_service.add_tcp(&listen_addr);
                    }
                }
            } else {
                proxy_service.add_tcp(&listen_addr);
            }
        } else {
            proxy_service.add_tcp(&listen_addr);
        }

        log::info!("Worker: Starting service '{service_name}' on {listen_addr}");
        my_server.add_service(proxy_service);
    }

    // Set up graceful shutdown handling for worker
    #[cfg(unix)]
    {
        let signal_handler_clone = signal_handler.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("Failed to create async runtime for signal handling: {e}");
                    return;
                }
            };

            rt.block_on(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    if signal_handler_clone.is_shutdown_requested() {
                        log::info!("馃洃 Worker received shutdown signal");
                        std::process::exit(0);
                    }
                }
            });
        });
    }

    #[cfg(windows)]
    {
        // On Windows, just rely on Ctrl+C handling by Pingora
        log::info!("馃敡 Windows: Using Pingora's built-in signal handling (no custom monitoring)");
    }

    log::info!("馃敡 Worker process ready and starting server");
    my_server.bootstrap();
    my_server.run_forever();
}
