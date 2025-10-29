use super::traits::{Plugin, PluginContext, PluginResult};
use crate::config::Config;
use std::path::PathBuf;
use tracing::{info, warn};

/// Theme switcher plugin - allows switching themes in real-time
/// Activated with @theme prefix
#[derive(Debug)]
pub struct ThemeSwitcherPlugin {
    themes: Vec<String>,
}

impl ThemeSwitcherPlugin {
    pub fn new(_config: Config) -> Self {
        let themes = Self::scan_themes();
        Self { themes }
    }

    /// Scan the themes/ directory for available themes
    fn scan_themes() -> Vec<String> {
        let themes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("themes");
        let mut themes = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    // Only include .css files, exclude README
                    if file_name.ends_with(".css") {
                        // Strip .css extension to get theme name
                        if let Some(theme_name) = file_name.strip_suffix(".css") {
                            themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        } else {
            warn!("Could not read themes directory: {:?}", themes_dir);
            // Fallback to hardcoded themes
            themes = vec![
                "dark".to_string(),
                "dracula".to_string(),
                "nord".to_string(),
                "light".to_string(),
                "high-contrast".to_string(),
            ];
        }

        // Sort themes alphabetically for consistent ordering
        themes.sort();
        info!("Loaded {} themes: {:?}", themes.len(), themes);
        themes
    }

    /// Search for themes matching the query
    fn search_themes(&self, query: &str, max_results: usize) -> Vec<PluginResult> {
        let normalized_query = query
            .strip_prefix("@theme")
            .unwrap_or(query)
            .trim()
            .to_lowercase();

        self.themes
            .iter()
            .filter_map(|theme| {
                if normalized_query.is_empty() || theme.to_lowercase().contains(&normalized_query) {
                    let theme_lower = theme.to_lowercase();
                    let score = if theme_lower == normalized_query {
                        1000 // Exact match
                    } else if theme_lower.starts_with(&normalized_query) {
                        500 // Prefix match
                    } else {
                        100 // Contains match
                    };

                    Some(PluginResult {
                        title: format!("Theme: {}", theme),
                        subtitle: Some(format!("Switch to {} theme", theme)),
                        icon: Some("preferences-desktop-theme".to_string()),
                        command: format!("@theme:{}", theme),
                        terminal: false,
                        score,
                        plugin_name: "theme-switcher".to_string(),
                        sub_results: vec![],
                        parent_app: None,
                    })
                } else {
                    None
                }
            })
            .take(max_results)
            .collect()
    }

    /// Execute theme change command
    #[cfg(test)]
    pub fn execute_theme_change(&self, command: &str) -> anyhow::Result<()> {
        use crate::ui::load_theme_with_name;

        if let Some(theme_name) = command.strip_prefix("@theme:") {
            info!("Switching to theme: {}", theme_name);
            load_theme_with_name(theme_name);

            // Persist theme to config file
            if let Err(e) = self.persist_theme_to_config(theme_name) {
                warn!("Failed to persist theme to config: {}", e);
                // Don't fail the theme change if persistence fails
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid theme command: {}", command))
        }
    }

    /// Persist theme selection to config file
    #[cfg(test)]
    fn persist_theme_to_config(&self, theme_name: &str) -> anyhow::Result<()> {
        use crate::config::ConfigLoader;

        // Load current config
        let mut loader = ConfigLoader::load()?;

        // Update theme
        let mut updated_config = loader.config().clone();
        updated_config.ui.theme = theme_name.to_string();

        // Save back to disk
        loader.update(updated_config)?;

        info!("Theme '{}' persisted to config file", theme_name);
        Ok(())
    }
}

impl Plugin for ThemeSwitcherPlugin {
    fn name(&self) -> &str {
        "theme-switcher"
    }

    fn description(&self) -> &str {
        "Switch application themes in real-time"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@theme"]
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@theme")
    }

    fn search(&self, query: &str, context: &PluginContext) -> anyhow::Result<Vec<PluginResult>> {
        Ok(self.search_themes(query, context.max_results))
    }

    fn priority(&self) -> i32 {
        1500 // HIGHEST priority - should handle @theme exclusively before other plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_plugin() -> ThemeSwitcherPlugin {
        let config = Config::default();
        ThemeSwitcherPlugin::new(config)
    }

    #[test]
    fn test_theme_search_empty_query() {
        let plugin = create_test_plugin();
        let config = Config::default();
        let context = PluginContext::new(10, &config);
        let results = plugin.search("@theme", &context).unwrap();
        assert_eq!(results.len(), 5); // All 5 themes
    }

    #[test]
    fn test_theme_search_partial_match() {
        let plugin = create_test_plugin();
        let config = Config::default();
        let context = PluginContext::new(10, &config);
        let results = plugin.search("@theme drac", &context).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("dracula"));
    }

    #[test]
    fn test_theme_search_multiple_matches() {
        let plugin = create_test_plugin();
        let config = Config::default();
        let context = PluginContext::new(10, &config);
        let results = plugin.search("@theme dark", &context).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title.contains("dark")));
    }

    #[test]
    fn test_should_handle() {
        let plugin = create_test_plugin();
        assert!(plugin.should_handle("@theme"));
        assert!(plugin.should_handle("@theme dracula"));
        assert!(!plugin.should_handle("theme"));
        assert!(!plugin.should_handle("calculator"));
    }

    #[test]
    #[ignore] // Requires GTK to be initialized
    fn test_execute_theme_change() {
        let plugin = create_test_plugin();
        let result = plugin.execute_theme_change("@theme:dracula");
        assert!(result.is_ok());
    }
}
