use std::path::PathBuf;
use tokio::fs;

use crate::domain::entities::config::AppConfig;

pub struct ConfigLoader;

impl ConfigLoader {
    pub async fn load(path: &std::path::Path) -> anyhow::Result<AppConfig> {
        if !path.exists() {
            // Si no existe, podemos crear uno por defecto y retornarlo
            let default_config = AppConfig::default();
            let toml_str = toml::to_string(&default_config)?;
            fs::write(path, toml_str).await?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(path).await?;
        let config: AppConfig = toml::from_str(&contents)?;
        
        Ok(config)
    }

    pub fn get_default_config_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push("config.toml");
        path
    }
}
