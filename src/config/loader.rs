use super::Config;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Configuration file loader with hot-reload support
pub struct ConfigLoader {
    config_path: PathBuf,
    config: Config,
}

impl ConfigLoader {
    /// Create a new config loader with default path
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        let config = Config::default();

        Self {
            config_path,
            config,
        }
    }

    /// Load configuration from disk, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path();

        let config = if config_path.exists() {
            info!("Loading config from {:?}", config_path);
            let contents = fs::read_to_string(&config_path)?;

            match toml::from_str::<Config>(&contents) {
                Ok(cfg) => {
                    info!("Config loaded successfully");
                    cfg
                }
                Err(e) => {
                    warn!("Failed to parse config: {}, using defaults", e);
                    let default = Config::default();
                    // Try to save corrected config
                    if let Err(save_err) = Self::save_config(&config_path, &default) {
                        warn!("Failed to save default config: {}", save_err);
                    }
                    default
                }
            }
        } else {
            info!(
                "No config file found, creating default at {:?}",
                config_path
            );
            let default = Config::default();

            // Create default config file
            if let Err(e) = Self::save_config(&config_path, &default) {
                warn!("Failed to create default config: {}", e);
            }

            default
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Reload configuration from disk
    #[allow(dead_code)]
    pub fn reload(&mut self) -> Result<()> {
        debug!("Reloading config from {:?}", self.config_path);

        if !self.config_path.exists() {
            warn!("Config file not found, keeping current config");
            return Ok(());
        }

        let contents = fs::read_to_string(&self.config_path)?;
        let new_config: Config = toml::from_str(&contents)?;

        self.config = new_config;
        info!("Config reloaded successfully");

        Ok(())
    }

    /// Save current configuration to disk
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        Self::save_config(&self.config_path, &self.config)
    }

    /// Update configuration and save to disk
    #[allow(dead_code)]
    pub fn update(&mut self, config: Config) -> Result<()> {
        self.config = config;
        self.save()
    }

    /// Default configuration file path
    fn default_config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        config_dir.join("native-launcher").join("config.toml")
    }

    /// Save configuration to specified path
    fn save_config(path: &PathBuf, config: &Config) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(config)?;
        fs::write(path, toml)?;

        debug!("Config saved to {:?}", path);
        Ok(())
    }

    /// Get config file path
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_new() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.config().window.width, 700);
    }

    #[test]
    fn test_default_path() {
        let path = ConfigLoader::default_config_path();
        assert!(path.to_string_lossy().contains("native-launcher"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
