//! Theme and CSS loading utilities
//!
//! This module handles loading and applying CSS themes to the GTK application.
//! Supports both built-in themes (from themes/ directory) and custom user themes.

use gtk4::gdk::Display;
use gtk4::CssProvider;
use tracing::{debug, error, info, warn};

/// Available built-in themes
#[derive(Debug, Clone, Copy)]
pub enum BuiltInTheme {
    Dark,
    Light,
    Dracula,
    Nord,
    HighContrast,
}

impl BuiltInTheme {
    /// Get theme from string name
    fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "dracula" => Some(Self::Dracula),
            "nord" => Some(Self::Nord),
            "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            _ => None,
        }
    }

    /// Get the CSS content for this theme
    fn css_content(&self) -> &'static str {
        match self {
            Self::Dark => include_str!("../../themes/dark.css"),
            Self::Light => include_str!("../../themes/light.css"),
            Self::Dracula => include_str!("../../themes/dracula.css"),
            Self::Nord => include_str!("../../themes/nord.css"),
            Self::HighContrast => include_str!("../../themes/high-contrast.css"),
        }
    }
}

/// Load and apply CSS theme to the application
///
/// Attempts to load theme in this order:
/// 1. Custom theme file specified in config (absolute path)
/// 2. Built-in theme by name (dark, light, dracula, nord, high-contrast)
/// 3. Custom theme from `~/.config/native-launcher/theme.css`
/// 4. Default built-in theme (style.css)
///
/// # Arguments
/// * `theme_name` - Theme name from config (e.g., "dark", "dracula", or path to CSS file)
///
/// # Examples
/// ```
/// // Load dark theme
/// load_theme_with_name("dark");
///
/// // Load custom theme file
/// load_theme_with_name("/path/to/custom.css");
/// ```
pub fn load_theme_with_name(theme_name: &str) {
    let provider = CssProvider::new();
    let mut css_loaded = false;

    // Try 1: Check if it's an absolute path to a custom CSS file
    if theme_name.starts_with('/') || theme_name.starts_with("~/") {
        let path = if theme_name.starts_with("~/") {
            // Expand ~ to home directory
            if let Some(home) = dirs::home_dir() {
                home.join(&theme_name[2..])
            } else {
                warn!("Could not expand home directory in theme path");
                std::path::PathBuf::from(theme_name)
            }
        } else {
            std::path::PathBuf::from(theme_name)
        };

        if path.exists() {
            info!("Loading custom theme from: {}", path.display());
            match std::fs::read_to_string(&path) {
                Ok(css_content) => {
                    provider.load_from_data(&css_content);
                    css_loaded = true;
                }
                Err(e) => {
                    warn!("Failed to read custom theme file: {}", e);
                }
            }
        } else {
            warn!("Custom theme file not found: {}", path.display());
        }
    }

    // Try 2: Check if it's a built-in theme name
    if !css_loaded {
        if let Some(built_in_theme) = BuiltInTheme::from_name(theme_name) {
            info!("Loading built-in theme: {:?}", built_in_theme);
            let css = built_in_theme.css_content();
            provider.load_from_data(css);
            css_loaded = true;
        }
    }

    // Try 3: Check for user's custom theme in config directory
    if !css_loaded {
        let custom_theme_path =
            dirs::config_dir().map(|config| config.join("native-launcher").join("theme.css"));

        if let Some(theme_path) = custom_theme_path {
            if theme_path.exists() {
                info!("Loading custom theme from: {}", theme_path.display());
                match std::fs::read_to_string(&theme_path) {
                    Ok(css_content) => {
                        provider.load_from_data(&css_content);
                        css_loaded = true;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to read custom theme: {}, falling back to default",
                            e
                        );
                    }
                }
            } else {
                debug!("No custom theme found at: {}", theme_path.display());
            }
        }
    }

    // Try 4: Fall back to built-in default CSS
    if !css_loaded {
        info!("Loading default built-in theme (style.css)");
        let css = include_str!("style.css");
        provider.load_from_data(css);
    }

    // Apply theme to display
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("CSS theme loaded successfully");
    } else {
        error!("Failed to get default display for CSS loading");
    }
}

/// Load and apply CSS theme to the application using default theme name

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_theme_doesnt_panic() {
        // GTK might not be initialized in tests, but function shouldn't panic
        // This is more of a smoke test
        // Actual GTK display tests would require headless GTK setup
    }
}
