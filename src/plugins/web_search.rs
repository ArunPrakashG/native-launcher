use super::traits::{KeyboardAction, KeyboardEvent, Plugin, PluginContext, PluginResult};
use anyhow::Result;
use gtk4::gdk::Key;
use std::collections::HashMap;

/// Plugin for quick web searches
#[derive(Debug)]
pub struct WebSearchPlugin {
    enabled: bool,
    engines: HashMap<String, String>,
}

impl WebSearchPlugin {
    pub fn new() -> Self {
        let mut engines = HashMap::new();

        // Default search engines
        engines.insert(
            "google".to_string(),
            "https://www.google.com/search?q={}".to_string(),
        );
        engines.insert(
            "ddg".to_string(),
            "https://duckduckgo.com/?q={}".to_string(),
        );
        engines.insert(
            "wiki".to_string(),
            "https://en.wikipedia.org/wiki/Special:Search?search={}".to_string(),
        );
        engines.insert(
            "github".to_string(),
            "https://github.com/search?q={}".to_string(),
        );
        engines.insert(
            "youtube".to_string(),
            "https://www.youtube.com/results?search_query={}".to_string(),
        );

        Self {
            enabled: true,
            engines,
        }
    }

    /// Parse query like "google rust wayland" or "@google rust wayland" into ("google", "rust wayland")
    pub fn parse_query<'a>(&self, query: &'a str) -> Option<(&'a str, String)> {
        // Support both "google query" and "@google query" formats
        let query_trimmed = query.trim_start_matches('@');

        let parts: Vec<&str> = query_trimmed.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }

        let keyword = parts[0];
        let search_term = parts[1];

        if self.engines.contains_key(keyword) {
            Some((keyword, search_term.to_string()))
        } else {
            None
        }
    }

    /// Build search URL
    pub fn build_url(&self, engine: &str, query: &str) -> Option<String> {
        self.engines
            .get(engine)
            .map(|template| template.replace("{}", &urlencoding::encode(query)))
    }

    /// Build web search URL from query (handles both explicit engine and fallback)
    pub fn build_search_url(&self, query: &str) -> Option<(String, String, String)> {
        // Try explicit engine first (e.g., "google rust")
        if let Some((engine, search_term)) = self.parse_query(query) {
            if let Some(url) = self.build_url(engine, &search_term) {
                return Some((engine.to_string(), search_term, url));
            }
        }

        // Fallback to Google for any query
        if !query.trim().is_empty() && query.len() >= 2 {
            let url = format!(
                "https://www.google.com/search?q={}",
                urlencoding::encode(query)
            );
            return Some(("google".to_string(), query.to_string(), url));
        }

        None
    }
}

impl Default for WebSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WebSearchPlugin {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Quick web searches (e.g., '@web rust', 'google linux')"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@web"]
    }

    fn should_handle(&self, query: &str) -> bool {
        // Handle explicit web search queries or offer fallback for any query
        self.enabled && !query.is_empty() && query.len() >= 2
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        // Strip @web prefix if present
        let clean_query = query.strip_prefix("@web").unwrap_or(query).trim();

        // Try to parse as explicit web search (e.g., "google query")
        if let Some((engine, search_term)) = self.parse_query(clean_query) {
            let url = match self.build_url(engine, &search_term) {
                Some(u) => u,
                None => return Ok(vec![]),
            };

            // Use xdg-open to open URL in default browser
            let command = format!("xdg-open '{}'", url);

            return Ok(vec![PluginResult::new(
                format!("Search {} for '{}'", engine, search_term),
                command,
                self.name().to_string(),
            )
            .with_subtitle(url.clone())
            .with_icon("web-browser".to_string())
            .with_score(9000)]); // High score for explicit web searches
        }

        // Fallback: Offer Google search for any query (lower priority)
        // This ensures there's always a web search option even if no results match
        let url = self.build_url("google", clean_query).unwrap_or_else(|| {
            format!(
                "https://www.google.com/search?q={}",
                urlencoding::encode(clean_query)
            )
        });
        let command = format!("xdg-open '{}'", url);

        Ok(vec![PluginResult::new(
            format!("Search Google for '{}'", clean_query),
            command,
            self.name().to_string(),
        )
        .with_subtitle(url.clone())
        .with_icon("web-browser".to_string())
        .with_score(100)]) // Low score so it appears at the bottom
    }

    fn priority(&self) -> i32 {
        600 // Medium-high priority
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle_keyboard_event(&self, event: &KeyboardEvent) -> KeyboardAction {
        // Handle Ctrl+Enter for web search
        if event.key == Key::Return && event.has_ctrl() {
            // Build web search URL from current query
            if let Some((_engine, _search_term, url)) = self.build_search_url(&event.query) {
                return KeyboardAction::OpenUrl(url);
            }
        }

        KeyboardAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let web = WebSearchPlugin::new();

        let result = web.parse_query("google rust wayland");
        assert!(result.is_some());
        let (engine, term) = result.unwrap();
        assert_eq!(engine, "google");
        assert_eq!(term, "rust wayland");

        assert!(web.parse_query("firefox").is_none());
        assert!(web.parse_query("unknown search term").is_none());
    }

    #[test]
    fn test_build_url() {
        let web = WebSearchPlugin::new();

        let url = web.build_url("google", "rust wayland").unwrap();
        assert!(url.contains("google.com"));
        assert!(url.contains("rust"));
    }

    #[test]
    fn test_search() {
        let web = WebSearchPlugin::new();
        let ctx = PluginContext::new(10);

        let results = web.search("google rust", &ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("rust"));
    }

    #[test]
    fn test_keyboard_event_ctrl_enter() {
        use super::super::traits::{KeyboardAction, KeyboardEvent, Plugin};
        use gtk4::gdk::{Key, ModifierType};

        let web = WebSearchPlugin::new();

        // Test Ctrl+Enter with a query
        let event = KeyboardEvent::new(
            Key::Return,
            ModifierType::CONTROL_MASK,
            "google rust programming".to_string(),
            false,
        );

        match web.handle_keyboard_event(&event) {
            KeyboardAction::OpenUrl(url) => {
                assert!(url.contains("google.com"));
                assert!(url.contains("rust"));
            }
            _ => panic!("Expected OpenUrl action"),
        }
    }

    #[test]
    fn test_keyboard_event_regular_enter() {
        use super::super::traits::{KeyboardAction, KeyboardEvent, Plugin};
        use gtk4::gdk::{Key, ModifierType};

        let web = WebSearchPlugin::new();

        // Test regular Enter (no Ctrl)
        let event = KeyboardEvent::new(
            Key::Return,
            ModifierType::empty(),
            "firefox".to_string(),
            true,
        );

        // Should return None, let other handlers deal with it
        match web.handle_keyboard_event(&event) {
            KeyboardAction::None => (),
            _ => panic!("Expected None action"),
        }
    }
}
