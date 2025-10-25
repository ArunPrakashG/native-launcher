use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub window: WindowConfig,
    pub search: SearchConfig,
    pub ui: UIConfig,
    pub plugins: PluginsConfig,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    /// Window width in pixels
    pub width: i32,
    /// Window height in pixels
    pub height: i32,
    /// Window position: "top", "center", or "bottom"
    pub position: String,
    /// Enable semi-transparent background
    pub transparency: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 700,
            height: 550,
            position: "top".to_string(),
            transparency: true,
        }
    }
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Maximum number of results to show
    pub max_results: usize,
    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
    /// Enable usage-based ranking
    pub usage_ranking: bool,
    /// Minimum score threshold for results (0-100)
    pub min_score_threshold: i32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            fuzzy_matching: true,
            usage_ranking: true,
            min_score_threshold: 0,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UIConfig {
    /// Icon size in pixels
    pub icon_size: i32,
    /// Show keyboard hints at the bottom
    pub show_keyboard_hints: bool,
    /// Animation duration in milliseconds
    pub animation_duration: u32,
    /// Theme: "dark" or "light" (currently only dark is supported)
    pub theme: String,
    /// Show empty state on launch (Spotlight-style) - hides results until user types
    pub empty_state_on_launch: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            icon_size: 48,
            show_keyboard_hints: true,
            animation_duration: 150,
            theme: "dark".to_string(),
            empty_state_on_launch: true,
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    /// Enable calculator plugin
    pub calculator: bool,
    /// Enable shell command plugin
    pub shell: bool,
    /// Enable web search plugin
    pub web_search: bool,
    /// Enable SSH plugin
    pub ssh: bool,
    /// Enable editors plugin (workspaces)
    pub editors: bool,
    /// Enable file browser plugin
    pub files: bool,
    /// Shell command prefix (default: ">")
    pub shell_prefix: String,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            calculator: true,
            shell: true,
            web_search: true,
            ssh: true,
            editors: true,
            files: true,
            shell_prefix: ">".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.window.width, 700);
        assert_eq!(config.search.max_results, 10);
        assert_eq!(config.ui.icon_size, 48);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();

        assert_eq!(config.window.width, deserialized.window.width);
        assert_eq!(config.search.max_results, deserialized.search.max_results);
    }
}
