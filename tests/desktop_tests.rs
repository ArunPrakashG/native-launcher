#[cfg(test)]
mod tests {
    use native_launcher::desktop::{DesktopEntry, DesktopScanner};
    use native_launcher::search::SearchEngine;
    use std::path::PathBuf;

    fn create_test_entry(name: &str, exec: &str) -> DesktopEntry {
        DesktopEntry {
            name: name.to_string(),
            generic_name: None,
            exec: exec.to_string(),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal: false,
            path: PathBuf::from("/test"),
            no_display: false,
        }
    }

    #[test]
    fn test_desktop_scanner() {
        let scanner = DesktopScanner::new();
        // Scanner should have at least system paths configured
        assert!(!scanner.paths().is_empty());
    }

    #[test]
    fn test_desktop_scanner_paths() {
        let scanner = DesktopScanner::new();
        let paths = scanner.paths();
        
        // Should include system path
        assert!(paths.iter().any(|p| p.to_str().unwrap().contains("/usr/share/applications")));
        
        // Should include user path if HOME is set
        if std::env::var("HOME").is_ok() {
            assert!(paths.iter().any(|p| p.to_str().unwrap().contains(".local/share/applications")));
        }
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
    fn test_entry_matching_case_insensitive() {
        let entry = create_test_entry("Firefox", "firefox");
        
        assert!(entry.matches("FIREFOX"));
        assert!(entry.matches("firefox"));
        assert!(entry.matches("FiReFoX"));
    }

    #[test]
    fn test_entry_matching_partial() {
        let entry = DesktopEntry {
            name: "Visual Studio Code".to_string(),
            generic_name: Some("Code Editor".to_string()),
            exec: "code".to_string(),
            icon: Some("vscode".to_string()),
            categories: vec!["Development".to_string()],
            keywords: vec!["editor".to_string(), "programming".to_string()],
            terminal: false,
            path: PathBuf::from("/test"),
            no_display: false,
        };

        assert!(entry.matches("visual"));
        assert!(entry.matches("studio"));
        assert!(entry.matches("code"));
        assert!(entry.matches("editor"));
        assert!(entry.matches("programming"));
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

    #[test]
    fn test_match_scoring_priority() {
        let entry = DesktopEntry {
            name: "Code".to_string(),
            generic_name: Some("Text Editor".to_string()),
            exec: "code".to_string(),
            icon: None,
            categories: vec![],
            keywords: vec!["programming".to_string()],
            terminal: false,
            path: PathBuf::from("/test"),
            no_display: false,
        };

        // Name match should score higher than generic name
        let name_score = entry.match_score("code");
        let generic_score = entry.match_score("editor");
        assert!(name_score > generic_score);
    }

    #[test]
    fn test_search_engine_empty_query() {
        let entries = vec![
            create_test_entry("App1", "app1"),
            create_test_entry("App2", "app2"),
        ];
        
        let engine = SearchEngine::new(entries);
        let results = engine.search("", 10);
        
        // Empty query should return all entries
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_engine_filtering() {
        let entries = vec![
            create_test_entry("Firefox", "firefox"),
            create_test_entry("Chrome", "chrome"),
            create_test_entry("Safari", "safari"),
        ];
        
        let engine = SearchEngine::new(entries);
        let results = engine.search("fire", 10);
        
        // Should only return Firefox
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_search_engine_limit() {
        let entries = vec![
            create_test_entry("App1", "app1"),
            create_test_entry("App2", "app2"),
            create_test_entry("App3", "app3"),
            create_test_entry("App4", "app4"),
        ];
        
        let engine = SearchEngine::new(entries);
        let results = engine.search("app", 2);
        
        // Should respect limit
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_engine_sorting() {
        let entries = vec![
            DesktopEntry {
                name: "Firefox Browser".to_string(),
                generic_name: None,
                exec: "firefox".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test1"),
                no_display: false,
            },
            DesktopEntry {
                name: "Firefox".to_string(),
                generic_name: None,
                exec: "firefox".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test2"),
                no_display: false,
            },
        ];
        
        let engine = SearchEngine::new(entries);
        let results = engine.search("firefox", 10);
        
        // Exact match should come first
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_no_display_entries_hidden() {
        let entries = vec![
            DesktopEntry {
                name: "Visible App".to_string(),
                generic_name: None,
                exec: "visible".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test1"),
                no_display: false,
            },
            DesktopEntry {
                name: "Hidden App".to_string(),
                generic_name: None,
                exec: "hidden".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test2"),
                no_display: true,
            },
        ];
        
        let engine = SearchEngine::new(entries);
        let results = engine.search("app", 10);
        
        // Should only show visible app
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Visible App");
    }

    #[test]
    fn test_terminal_flag() {
        let terminal_entry = DesktopEntry {
            name: "Htop".to_string(),
            generic_name: None,
            exec: "htop".to_string(),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal: true,
            path: PathBuf::from("/test"),
            no_display: false,
        };

        assert!(terminal_entry.terminal);
        
        let gui_entry = create_test_entry("Firefox", "firefox");
        assert!(!gui_entry.terminal);
    }
}
