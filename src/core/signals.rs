//! Signal handling for BWS
//!
//! This module provides signal handling capabilities for hot reconfiguration
//! and graceful shutdown inspired by Sozu proxy architecture.

#[cfg(unix)]
use log::error;
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;

/// Shared signal state
#[derive(Clone)]
pub struct SignalHandler {
    /// Flag indicating if configuration reload was requested
    pub reload_requested: Arc<AtomicBool>,
    /// Flag indicating if graceful shutdown was requested
    pub shutdown_requested: Arc<AtomicBool>,
    /// Flag indicating if upgrade was requested
    pub upgrade_requested: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            reload_requested: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            upgrade_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if reload was requested and reset the flag
    pub fn check_and_clear_reload(&self) -> bool {
        self.reload_requested.swap(false, Ordering::Relaxed)
    }

    /// Check if shutdown was requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    /// Check if upgrade was requested and reset the flag
    pub fn check_and_clear_upgrade(&self) -> bool {
        self.upgrade_requested.swap(false, Ordering::Relaxed)
    }

    /// Start signal monitoring task
    pub async fn start_monitoring(self) {
        let handler = self.clone();
        tokio::spawn(async move {
            handler.monitor_signals().await;
        });
    }

    /// Monitor signals and set appropriate flags
    async fn monitor_signals(&self) {
        #[cfg(unix)]
        {
            use signal::unix::{signal, SignalKind};

            let sighup = match signal(SignalKind::hangup()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create SIGHUP handler: {e}");
                    return;
                }
            };
            let sigterm = match signal(SignalKind::terminate()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create SIGTERM handler: {e}");
                    return;
                }
            };
            let sigint = match signal(SignalKind::interrupt()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create SIGINT handler: {e}");
                    return;
                }
            };
            let sigusr1 = match signal(SignalKind::user_defined1()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create SIGUSR1 handler: {e}");
                    return;
                }
            };
            let sigusr2 = match signal(SignalKind::user_defined2()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create SIGUSR2 handler: {e}");
                    return;
                }
            };

            let mut sighup = sighup;
            let mut sigterm = sigterm;
            let mut sigint = sigint;
            let mut sigusr1 = sigusr1;
            let mut sigusr2 = sigusr2;

            loop {
                tokio::select! {
                    _ = sighup.recv() => {
                        info!("Received SIGHUP - requesting configuration reload");
                        self.reload_requested.store(true, Ordering::Relaxed);
                    }
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM - requesting graceful shutdown");
                        self.shutdown_requested.store(true, Ordering::Relaxed);
                        break;
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT - requesting graceful shutdown");
                        self.shutdown_requested.store(true, Ordering::Relaxed);
                        break;
                    }
                    _ = sigusr1.recv() => {
                        info!("Received SIGUSR1 - requesting hot upgrade");
                        self.upgrade_requested.store(true, Ordering::Relaxed);
                    }
                    _ = sigusr2.recv() => {
                        info!("Received SIGUSR2 - requesting configuration reload");
                        self.reload_requested.store(true, Ordering::Relaxed);
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows signal handling is more limited
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Received Ctrl+C - requesting graceful shutdown");
                    self.shutdown_requested.store(true, Ordering::Relaxed);
                }
            }
        }

        info!("Signal monitoring task exiting");
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal-related utility functions
pub mod utils {
    #[cfg(unix)]
    pub fn send_reload_signal(pid: u32) -> Result<(), std::io::Error> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGHUP)?;
        Ok(())
    }

    #[cfg(unix)]
    pub fn send_upgrade_signal(pid: u32) -> Result<(), std::io::Error> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGUSR1)?;
        Ok(())
    }

    #[cfg(unix)]
    pub fn send_shutdown_signal(pid: u32) -> Result<(), std::io::Error> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
        Ok(())
    }

    #[cfg(windows)]
    pub fn send_reload_signal(_pid: u32) -> Result<(), std::io::Error> {
        // Windows doesn't have the same signal system
        // Would need to implement via named pipes or other IPC
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Signal-based reloading not supported on Windows",
        ))
    }

    #[cfg(windows)]
    pub fn send_upgrade_signal(_pid: u32) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Signal-based upgrades not supported on Windows",
        ))
    }

    #[cfg(windows)]
    pub fn send_shutdown_signal(_pid: u32) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Signal-based shutdown not supported on Windows",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(!handler.is_shutdown_requested());
        assert!(!handler.check_and_clear_reload());
        assert!(!handler.check_and_clear_upgrade());
    }

    #[tokio::test]
    async fn test_signal_flags() {
        let handler = SignalHandler::new();

        // Test reload flag
        handler.reload_requested.store(true, Ordering::Relaxed);
        assert!(handler.check_and_clear_reload());
        assert!(!handler.check_and_clear_reload()); // Should be cleared

        // Test shutdown flag
        handler.shutdown_requested.store(true, Ordering::Relaxed);
        assert!(handler.is_shutdown_requested());

        // Test upgrade flag
        handler.upgrade_requested.store(true, Ordering::Relaxed);
        assert!(handler.check_and_clear_upgrade());
        assert!(!handler.check_and_clear_upgrade()); // Should be cleared
    }
}
