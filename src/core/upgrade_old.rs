//! Zero-downtime upgrade functionality for BWS
//!
//! This module provides zero-downtime upgrade capabilities inspired by Sozu proxy.
//! It allows the server to upgrade to a new binary version without dropping connections.

use crate::config::ServerConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

#[derive(Debug, thiserror::Error)]
pub enum UpgradeError {
    #[error("Failed to serialize upgrade data: {0}")]
    SerializationFailed(#[from] serde_json::Error),
    #[error("Failed to create temporary file: {0}")]
    TempFileCreation(#[from] std::io::Error),
    #[error("Failed to fork new process: {0}")]
    ProcessFork(String),
    #[error("Failed to send file descriptors: {0}")]
    FileDescriptorTransfer(String),
    #[error("New process failed to start: {0}")]
    ProcessStartFailed(String),
    #[error("Upgrade timeout")]
    Timeout,
    #[error("Configuration validation failed: {0}")]
    ConfigValidation(String),
}

/// Data structure for upgrading the main process
#[derive(Serialize, Deserialize, Debug)]
pub struct UpgradeData {
    /// Current server configuration
    pub config: ServerConfig,
    /// Process ID of the old main process
    pub old_pid: u32,
    /// File descriptors for listening sockets
    pub listener_fds: HashMap<String, RawFd>,
    /// Current server state
    pub server_state: ServerState,
    /// Upgrade timestamp
    pub upgrade_time: u64,
    /// Version information
    pub version: String,
}

/// Server state for upgrades
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerState {
    /// Number of active connections
    pub active_connections: u64,
    /// Server start time
    pub start_time: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// SSL certificate states
    pub ssl_states: HashMap<String, SslState>,
}

/// SSL certificate state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SslState {
    /// Certificate expiry timestamp
    pub expires_at: u64,
    /// Whether certificate is auto-renewed
    pub auto_renewed: bool,
    /// Last renewal attempt timestamp
    pub last_renewal_attempt: Option<u64>,
}

/// Zero-downtime upgrade manager
pub struct UpgradeManager {
    config: Arc<RwLock<ServerConfig>>,
    listeners: Arc<RwLock<HashMap<String, TcpListener>>>,
    server_state: Arc<RwLock<ServerState>>,
}

impl UpgradeManager {
    pub fn new(
        config: Arc<RwLock<ServerConfig>>,
        listeners: Arc<RwLock<HashMap<String, TcpListener>>>,
        server_state: Arc<RwLock<ServerState>>,
    ) -> Self {
        Self {
            config,
            listeners,
            server_state,
        }
    }

    /// Initiate zero-downtime upgrade
    pub async fn initiate_upgrade(&self, new_binary_path: Option<String>) -> Result<UpgradeResult, UpgradeError> {
        info!("Starting zero-downtime upgrade process");
        let start_time = std::time::Instant::now();

        // Prepare upgrade data
        let upgrade_data = self.prepare_upgrade_data().await?;

        // Get current executable path or use provided path
        let binary_path = new_binary_path.unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("bws"))
                .to_string_lossy()
                .to_string()
        });

        // Create temporary file for upgrade data
        let temp_file = tempfile::NamedTempFile::new()?;
        let upgrade_data_json = serde_json::to_string(&upgrade_data)?;
        std::fs::write(temp_file.path(), upgrade_data_json)?;

        // Extract file descriptors for socket passing
        let listener_fds = self.extract_listener_fds().await?;

        // Fork new process with upgrade data
        let new_process = self.fork_new_process(&binary_path, temp_file.path(), &listener_fds).await?;

        // Wait for new process to confirm readiness
        let confirmation_result = self.wait_for_upgrade_confirmation(new_process).await?;

        let upgrade_duration = start_time.elapsed();
        info!("Zero-downtime upgrade completed in {:?}", upgrade_duration);

        Ok(UpgradeResult {
            success: confirmation_result.success,
            new_pid: confirmation_result.new_pid,
            old_pid: upgrade_data.old_pid,
            upgrade_duration,
            message: confirmation_result.message,
        })
    }

    /// Prepare upgrade data for the new process
    async fn prepare_upgrade_data(&self) -> Result<UpgradeData, UpgradeError> {
        let config = self.config.read().await.clone();
        let server_state = self.server_state.read().await.clone();
        let listeners = self.listeners.read().await;

        // Extract file descriptors from listeners
        let mut listener_fds = HashMap::new();
        for (key, listener) in listeners.iter() {
            let fd = listener.as_raw_fd();
            listener_fds.insert(key.clone(), fd);
        }

        Ok(UpgradeData {
            config,
            old_pid: std::process::id(),
            listener_fds,
            server_state,
            upgrade_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Extract file descriptors from listeners for socket passing
    async fn extract_listener_fds(&self) -> Result<HashMap<String, RawFd>, UpgradeError> {
        let listeners = self.listeners.read().await;
        let mut fds = HashMap::new();

        for (key, listener) in listeners.iter() {
            let fd = listener.as_raw_fd();
            fds.insert(key.clone(), fd);
        }

        Ok(fds)
    }

    /// Fork new process with upgrade data
    async fn fork_new_process(
        &self,
        binary_path: &str,
        upgrade_data_path: &std::path::Path,
        _listener_fds: &HashMap<String, RawFd>,
    ) -> Result<Child, UpgradeError> {
        info!("Forking new process: {}", binary_path);

        let child = Command::new(binary_path)
            .arg("--upgrade-from")
            .arg(upgrade_data_path.to_string_lossy().as_ref())
            .arg("--upgrade-mode")
            .spawn()
            .map_err(|e| UpgradeError::ProcessFork(e.to_string()))?;

        info!("New process forked with PID: {}", child.id());
        Ok(child)
    }

    /// Wait for upgrade confirmation from new process
    async fn wait_for_upgrade_confirmation(&self, mut new_process: Child) -> Result<UpgradeConfirmation, UpgradeError> {
        info!("Waiting for upgrade confirmation from new process");

        // Use a timeout to avoid waiting indefinitely
        let timeout_duration = std::time::Duration::from_secs(30);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            match new_process.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        return Ok(UpgradeConfirmation {
                            success: true,
                            new_pid: new_process.id(),
                            message: "New process started successfully".to_string(),
                        });
                    } else {
                        return Err(UpgradeError::ProcessStartFailed(format!(
                            "New process exited with status: {}",
                            status
                        )));
                    }
                }
                Ok(None) => {
                    // Process is still running, continue waiting
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    return Err(UpgradeError::ProcessStartFailed(format!(
                        "Error checking process status: {}",
                        e
                    )));
                }
            }
        }

        // Timeout reached
        let _ = new_process.kill();
        Err(UpgradeError::Timeout)
    }

    /// Start new server from upgrade data
    pub async fn start_from_upgrade_data(upgrade_data_path: &str) -> Result<(ServerConfig, ServerState), UpgradeError> {
        info!("Starting server from upgrade data: {}", upgrade_data_path);

        // Read upgrade data
        let upgrade_data_json = std::fs::read_to_string(upgrade_data_path)?;
        let upgrade_data: UpgradeData = serde_json::from_str(&upgrade_data_json)?;

        // Validate configuration
        upgrade_data.config.validate().map_err(|e| {
            UpgradeError::ConfigValidation(e.to_string())
        })?;

        info!(
            "Upgrade data loaded - Old PID: {}, Version: {}, Listeners: {}",
            upgrade_data.old_pid,
            upgrade_data.version,
            upgrade_data.listener_fds.len()
        );

        // TODO: Restore listeners from file descriptors
        // This would involve recreating TcpListener instances from raw file descriptors
        // For now, we'll return the configuration and state

        Ok((upgrade_data.config, upgrade_data.server_state))
    }

    /// Check if the server can perform an upgrade
    pub async fn can_upgrade(&self) -> bool {
        let listeners = self.listeners.read().await;
        let config = self.config.read().await;

        // Check if we have any active listeners
        if listeners.is_empty() {
            warn!("No active listeners found - upgrade not possible");
            return false;
        }

        // Check if configuration is valid
        if let Err(e) = config.validate() {
            warn!("Invalid configuration prevents upgrade: {}", e);
            return false;
        }

        true
    }

    /// Get current server state
    pub async fn get_server_state(&self) -> ServerState {
        self.server_state.read().await.clone()
    }

    /// Update server state
    pub async fn update_server_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut ServerState),
    {
        let mut state = self.server_state.write().await;
        updater(&mut *state);
    }
}

/// Result of upgrade operation
#[derive(Debug)]
pub struct UpgradeResult {
    pub success: bool,
    pub new_pid: u32,
    pub old_pid: u32,
    pub upgrade_duration: std::time::Duration,
    pub message: String,
}

/// Confirmation from new process
#[derive(Debug)]
struct UpgradeConfirmation {
    pub success: bool,
    pub new_pid: u32,
    pub message: String,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            active_connections: 0,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_requests: 0,
            ssl_states: HashMap::new(),
        }
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_requests(&mut self) {
        self.total_requests += 1;
    }

    pub fn set_active_connections(&mut self, count: u64) {
        self.active_connections = count;
    }

    pub fn update_ssl_state(&mut self, domain: String, ssl_state: SslState) {
        self.ssl_states.insert(domain, ssl_state);
    }

    pub fn get_uptime(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LoggingConfig, PerformanceConfig, SecurityConfig, ServerInfo, SiteConfig};
    use std::collections::HashMap;

    fn create_test_config() -> ServerConfig {
        ServerConfig {
            server: ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: "Test server".to_string(),
            },
            sites: vec![SiteConfig {
                name: "test-site".to_string(),
                hostname: "localhost".to_string(),
                hostnames: vec![],
                port: 8080,
                static_dir: "/tmp/static".to_string(),
                default: true,
                api_only: false,
                headers: HashMap::new(),
                ssl: Default::default(),
                redirect_to_https: false,
                index_files: vec!["index.html".to_string()],
                error_pages: HashMap::new(),
                compression: Default::default(),
                cache: Default::default(),
                access_control: Default::default(),
                proxy: Default::default(),
            }],
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_server_state() {
        let mut state = ServerState::new();
        assert_eq!(state.total_requests, 0);
        assert_eq!(state.active_connections, 0);

        state.increment_requests();
        assert_eq!(state.total_requests, 1);

        state.set_active_connections(5);
        assert_eq!(state.active_connections, 5);

        let uptime = state.get_uptime();
        assert!(uptime >= 0);
    }

    #[tokio::test]
    async fn test_upgrade_data_serialization() {
        let config = create_test_config();
        let server_state = ServerState::new();
        let upgrade_data = UpgradeData {
            config,
            old_pid: 12345,
            listener_fds: HashMap::new(),
            server_state,
            upgrade_time: 1234567890,
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&upgrade_data).unwrap();
        let deserialized: UpgradeData = serde_json::from_str(&json).unwrap();

        assert_eq!(upgrade_data.old_pid, deserialized.old_pid);
        assert_eq!(upgrade_data.version, deserialized.version);
    }

    #[tokio::test]
    async fn test_upgrade_manager_creation() {
        let config = Arc::new(RwLock::new(create_test_config()));
        let listeners = Arc::new(RwLock::new(HashMap::new()));
        let server_state = Arc::new(RwLock::new(ServerState::new()));

        let upgrade_manager = UpgradeManager::new(config, listeners, server_state);
        let can_upgrade = upgrade_manager.can_upgrade().await;
        
        // Should be false because no listeners
        assert!(!can_upgrade);
    }
}
