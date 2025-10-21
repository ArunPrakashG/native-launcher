//! Browser and web search utilities
//!
//! This module provides utilities for:
//! - Detecting the default web browser
//! - Building web search URLs via WebSearchPlugin
//! - Browser-related operations

use crate::plugins;

/// Detect if query is a web search and extract engine + search term + URL
///
/// Delegates all web search logic to WebSearchPlugin to maintain single source of truth.
///
/// # Returns
/// - `Some((engine_name, search_term, url))` if valid web search
/// - `None` if not a web search query
///
/// # Examples
/// ```
/// let result = detect_web_search("g rust lang");
/// assert_eq!(result, Some(("google".to_string(), "rust lang".to_string(), "https://...")));
/// ```
pub fn detect_web_search(query: &str) -> Option<(String, String, String)> {
    let web_search = plugins::WebSearchPlugin::new();
    web_search.build_search_url(query)
}

/// Get default browser name for display purposes
///
/// Attempts to detect the user's default web browser by querying `xdg-settings`.
/// Falls back to generic "Browser" if detection fails.
///
/// # Returns
/// - Capitalized browser name (e.g., "Firefox", "Chrome")
/// - "Browser" if detection fails
///
/// # Implementation
/// - Runs: `xdg-settings get default-web-browser`
/// - Parses: `firefox.desktop` → `Firefox`
/// - Handles: Multi-word names like `google-chrome.desktop` → `Google`
///
/// # Examples
/// ```
/// let browser = get_default_browser();
/// println!("Search in {}", browser); // "Search in Firefox"
/// ```
pub fn get_default_browser() -> String {
    // Try to get default browser from xdg-settings
    if let Ok(output) = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
    {
        if output.status.success() {
            if let Ok(browser_desktop) = String::from_utf8(output.stdout) {
                // Extract browser name from .desktop file (e.g., "firefox.desktop" -> "Firefox")
                let name = browser_desktop
                    .trim()
                    .trim_end_matches(".desktop")
                    .split('-')
                    .next()
                    .unwrap_or("Browser");
                return name[0..1].to_uppercase() + &name[1..];
            }
        }
    }

    "Browser".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_web_search_google() {
        // Test explicit "google" keyword (not "g")
        let result = detect_web_search("google rust programming");
        assert!(result.is_some());
        let (engine, term, url) = result.unwrap();
        assert_eq!(engine, "google");
        assert_eq!(term, "rust programming"); // Parsed: "google" prefix stripped
        assert!(url.contains("google.com"));
        assert!(url.contains("rust")); // Check query is encoded in URL
    }

    #[test]
    fn test_detect_web_search_fallback() {
        // Any query should get fallback Google search
        let result = detect_web_search("firefox");
        assert!(result.is_some());
        let (engine, term, _url) = result.unwrap();
        assert_eq!(engine, "google");
        assert_eq!(term, "firefox");
    }

    #[test]
    fn test_get_default_browser_fallback() {
        // Should at minimum return "Browser" fallback
        let browser = get_default_browser();
        assert!(!browser.is_empty());
    }
}
