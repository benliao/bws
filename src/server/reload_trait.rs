use async_trait::async_trait;

/// Trait for services that can reload their configuration
#[async_trait]
pub trait ConfigReloadable {
    async fn reload_config_from_path(&self)
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_config_path(&self) -> Option<String>;
}
