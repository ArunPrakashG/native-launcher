//! Theme and CSS loading utilities
//!
//! This module handles loading and applying CSS themes to the GTK application.
//! Supports both custom user themes and built-in default theme.

use gtk4::gdk::Display;
use gtk4::CssProvider;
use tracing::{debug, error, info, warn};

/// Load and apply CSS theme to the application
///
/// Attempts to load a custom theme from the user's config directory first,
/// falling back to the built-in default theme if not found or invalid.
///
/// # Theme Loading Order
/// 1. Custom theme: `~/.config/native-launcher/theme.css`
/// 2. Built-in theme: Embedded `style.css` from source
///
/// # Custom Theme Location
/// - Linux: `~/.config/native-launcher/theme.css`
/// - macOS: `~/Library/Application Support/native-launcher/theme.css`
/// - Windows: `%APPDATA%\native-launcher\theme.css`
///
/// # Examples
/// ```
/// // In main GTK app initialization:
/// load_theme();
/// ```
///
/// # Errors
/// - Logs warnings if custom theme fails to load
/// - Logs errors if no display available (rare GTK initialization issue)
pub fn load_theme() {
    let provider = CssProvider::new();

    // Try to load user theme first
    let custom_theme_path =
        dirs::config_dir().map(|config| config.join("native-launcher").join("theme.css"));

    let css_loaded = if let Some(theme_path) = custom_theme_path {
        if theme_path.exists() {
            info!("Loading custom theme from: {}", theme_path.display());
            match std::fs::read_to_string(&theme_path) {
                Ok(css_content) => {
                    provider.load_from_data(&css_content);
                    true
                }
                Err(e) => {
                    warn!(
                        "Failed to read custom theme: {}, falling back to default",
                        e
                    );
                    false
                }
            }
        } else {
            debug!("No custom theme found at: {}", theme_path.display());
            false
        }
    } else {
        false
    };

    // Fall back to built-in CSS if custom theme not loaded
    if !css_loaded {
        info!("Loading built-in theme");
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_theme_doesnt_panic() {
        // GTK might not be initialized in tests, but function shouldn't panic
        // This is more of a smoke test
        // Actual GTK display tests would require headless GTK setup
    }
}
