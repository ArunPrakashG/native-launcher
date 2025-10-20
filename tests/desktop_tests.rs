#[cfg(test)]
mod tests {
    use native_launcher::desktop::{DesktopEntry, DesktopScanner};

    #[test]
    fn test_desktop_scanner() {
        let scanner = DesktopScanner::new();
        // Scanner should have at least system paths configured
        assert!(!scanner.search_paths.is_empty());
    }

    #[test]
    fn test_entry_matching() {
        let entry = DesktopEntry {
            name: "Firefox".to_string(),
            generic_name: Some("Web Browser".to_string()),
            exec: "firefox".to_string(),
            icon: Some("firefox".to_string()),
            categories: vec!["Network".to_string()],
            keywords: vec!["browser".to_string(), "web".to_string()],
            terminal: false,
            path: std::path::PathBuf::from("/test"),
            no_display: false,
        };

        // Should match on name
        assert!(entry.matches("fire"));
        assert!(entry.matches("Fox"));

        // Should match on generic name
        assert!(entry.matches("browser"));

        // Should match on keywords
        assert!(entry.matches("web"));

        // Should not match random text
        assert!(!entry.matches("xyz"));
    }

    #[test]
    fn test_match_scoring() {
        let entry = DesktopEntry {
            name: "Firefox".to_string(),
            generic_name: Some("Web Browser".to_string()),
            exec: "firefox".to_string(),
            icon: Some("firefox".to_string()),
            categories: vec!["Network".to_string()],
            keywords: vec!["browser".to_string()],
            terminal: false,
            path: std::path::PathBuf::from("/test"),
            no_display: false,
        };

        // Exact match should score highest
        assert_eq!(entry.match_score("firefox"), 100);

        // Prefix match should score high
        assert!(entry.match_score("fire") > 80);

        // Contains match should score lower
        assert!(entry.match_score("fox") < 80);

        // Generic name match should score medium
        let browser_score = entry.match_score("browser");
        assert!(browser_score > 30 && browser_score < 80);
    }
}
