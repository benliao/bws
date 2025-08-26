use crate::config::ServerConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A service responsible for handling configuration reloads
#[derive(Clone)]
pub struct ConfigReloadService {
    config: Arc<RwLock<ServerConfig>>,
    config_path: Arc<RwLock<Option<String>>>,
}

impl ConfigReloadService {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_config_path(&self, path: String) {
        let mut config_path = self.config_path.write().await;
        *config_path = Some(path);
    }

    pub async fn get_config_path(&self) -> Option<String> {
        self.config_path.read().await.clone()
    }

    pub async fn get_config(&self) -> ServerConfig {
        self.config.read().await.clone()
    }

    pub async fn reload_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = match self.get_config_path().await {
            Some(path) => path,
            None => return Err("No configuration file path set".into()),
        };

        // Load new configuration from file
        let new_config = ServerConfig::load_from_file(&config_path)?;

        // Validate new configuration
        new_config.validate()?;

        // Update configuration
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }

        log::info!("Configuration reloaded successfully from {}", config_path);
        Ok(())
    }
}
