/// End-to-End Tests for Native Launcher
/// Tests complete workflows: search → select → launch, plugin integration, config loading
///
/// These tests verify the entire application stack works together correctly.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

// Import from main crate
use native_launcher::config::{Config, ConfigLoader};
use native_launcher::desktop::{DesktopEntry, DesktopEntryArena, DesktopScanner};
use native_launcher::plugins::{KeyboardAction, KeyboardEvent, PluginManager};
use native_launcher::search::SearchEngine;
use native_launcher::ui::{ResultsList, SearchWidget};
use native_launcher::usage::UsageTracker;

static GTK_INIT_RESULT: OnceLock<bool> = OnceLock::new();

/// Initialize GTK once for all tests. Returns true if initialization succeeded.
fn init_gtk() -> bool {
    *GTK_INIT_RESULT.get_or_init(|| gtk4::init().is_ok())
}

/// Helper to run tests that don't need GTK UI (just backend logic)
fn run_test<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    test_fn();
}

/// Helper to run UI tests on GTK main thread (for widget tests only)
fn run_gtk_ui_test<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    // Skip UI tests in headless environments or when GTK cannot initialize
    let has_display = std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("DISPLAY").is_ok();
    if !has_display || !init_gtk() {
        eprintln!("Skipping GTK UI test: GTK could not be initialized (no display?)");
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();

    gtk4::glib::idle_add_once(move || {
        test_fn();
        tx.send(()).unwrap();
    });

    // Pump the GLib main context while waiting, with a timeout
    let start = std::time::Instant::now();
    loop {
        if rx.try_recv().is_ok() {
            break;
        }
        while gtk4::glib::MainContext::default().iteration(false) {}
        if start.elapsed() > std::time::Duration::from_secs(10) {
            panic!("GTK UI test timed out");
        }
        // Avoid busy loop
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn test_e2e_desktop_scanner_to_search_engine() {
    // Test: Desktop scanning → Search engine creation → Basic search
    run_test(|| {
        // 1. Scan desktop applications
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().expect("Failed to scan desktop entries");

        assert!(
            !entries.is_empty(),
            "Should find at least some desktop applications"
        );

        // 2. Create search engine
        let entry_arena = DesktopEntryArena::from_vec(entries.clone());
        let search_engine = SearchEngine::new(entry_arena, false);

        // 3. Perform search for specific browsers if present
        let have_firefox = entries
            .iter()
            .any(|e| e.name.to_lowercase().contains("firefox"));
        let have_chrome = entries
            .iter()
            .any(|e| e.name.to_lowercase().contains("chrome"));

        if have_firefox {
            let firefox_results = search_engine.search("firefox", 10);
            assert!(
                !firefox_results.is_empty(),
                "Should find Firefox in search results"
            );
        }
        if have_chrome {
            let chrome_results = search_engine.search("chrome", 10);
            assert!(
                !chrome_results.is_empty(),
                "Should find Chrome in search results"
            );
        }

        // 4. Search for empty query (should return all apps, sorted)
        let all_results = search_engine.search("", 100);
        assert!(!all_results.is_empty(), "Empty search should return apps");
        assert!(all_results.len() <= 100, "Should respect max_results limit");
    });
}

#[test]
fn test_e2e_config_loading() {
    // Test: Config loading with defaults and merging
    run_test(|| {
        // Load config (will use defaults if file doesn't exist)
        let config_loader = ConfigLoader::load().unwrap_or_else(|_| ConfigLoader::new());
        let config = config_loader.config();

        // Verify essential config values
        assert!(config.window.width > 0, "Window width should be positive");
        assert!(config.window.height > 0, "Window height should be positive");
        assert!(
            config.search.max_results > 0,
            "Max results should be positive"
        );
        assert!(
            config.search.min_score_threshold >= 0,
            "Min score threshold should be non-negative"
        );

        // Verify defaults are reasonable
        assert!(
            config.window.width >= 600,
            "Window width should be at least 600px"
        );
        assert!(
            config.window.height >= 400,
            "Window height should be at least 400px"
        );
        assert!(
            config.search.max_results <= 100,
            "Max results should be reasonable"
        );
    });
}

#[test]
fn test_e2e_usage_tracking() {
    // Test: Usage tracking persistence and scoring

    run_test(|| {
        // Create a new usage tracker
        let mut tracker = UsageTracker::new();

        // Record some launches
        tracker.record_launch("/usr/share/applications/firefox.desktop");
        tracker.record_launch("/usr/share/applications/firefox.desktop");
        tracker.record_launch("/usr/share/applications/firefox.desktop");
        tracker.record_launch("/usr/share/applications/chrome.desktop");

        // Verify scoring
        let firefox_score = tracker.get_score("/usr/share/applications/firefox.desktop");
        let chrome_score = tracker.get_score("/usr/share/applications/chrome.desktop");
        let unknown_score = tracker.get_score("/usr/share/applications/unknown.desktop");

        assert!(
            firefox_score > chrome_score,
            "Firefox should have higher score (3 launches vs 1)"
        );
        assert!(
            chrome_score > unknown_score,
            "Chrome should have higher score than unknown app"
        );
        assert_eq!(unknown_score, 0.0, "Unknown app should have zero score");

        // Test app count
        assert_eq!(tracker.app_count(), 2, "Should track 2 unique apps");
    });
}

#[test]
fn test_e2e_plugin_manager_integration() {
    // Test: Plugin manager with real desktop entries and all plugins
    run_test(|| {
        // Scan desktop apps
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().expect("Failed to scan");
        let entry_arena = DesktopEntryArena::from_vec(entries.clone());

        // Create usage tracker
        let usage_tracker = UsageTracker::new();

        // Create config
        let config = Config::default();

        // Create plugin manager
        let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

        // Test search with no query (should show apps)
        let results = plugin_manager.search("", 10).expect("Search failed");
        assert!(!results.is_empty(), "Should return apps for empty query");

        // Test calculator plugin
        let calc_results = plugin_manager
            .search("2 + 2", 5)
            .expect("Calculator search failed");
        assert!(
            !calc_results.is_empty(),
            "Calculator should handle math expression"
        );
        assert!(
            calc_results[0].title.contains('4'),
            "Calculator should compute 2 + 2 = 4, got: {}",
            calc_results[0].title
        );

        // Test web search detection
        let web_results = plugin_manager
            .search("g rust programming", 5)
            .expect("Web search failed");
        assert!(!web_results.is_empty(), "Should detect web search query");
        let has_google = web_results.iter().any(|r| r.plugin_name == "web_search");
        assert!(has_google, "Should include web search result");

        // Verify enabled plugins
        let enabled = plugin_manager.enabled_plugins();
        assert!(
            enabled.contains(&"applications"),
            "Applications plugin should be enabled"
        );
        assert!(
            enabled.contains(&"calculator"),
            "Calculator plugin should be enabled"
        );
        assert!(
            enabled.contains(&"web_search"),
            "Web search plugin should be enabled"
        );
    });
}

#[test]
fn test_e2e_search_widget_to_results_list() {
    // Test: Search widget → Results list UI integration
    run_gtk_ui_test(|| {
        // Create test desktop entries
        let entries = vec![
            DesktopEntry {
                name: "Firefox".to_string(),
                generic_name: Some("Web Browser".to_string()),
                exec: "firefox".to_string(),
                icon: None,
                terminal: false,
                no_display: false,
                path: std::path::PathBuf::from("/usr/share/applications/firefox.desktop"),
                keywords: vec!["browser".to_string(), "web".to_string()],
                categories: vec!["Network".to_string()],
                actions: vec![],
            },
            DesktopEntry {
                name: "VS Code".to_string(),
                generic_name: Some("Code Editor".to_string()),
                exec: "code".to_string(),
                icon: None,
                terminal: false,
                no_display: false,
                path: std::path::PathBuf::from("/usr/share/applications/code.desktop"),
                keywords: vec!["editor".to_string(), "development".to_string()],
                categories: vec!["Development".to_string()],
                actions: vec![],
            },
        ];

        // Create widgets
        let search_widget = SearchWidget::new();
        let results_list = ResultsList::new();

        // Create plugin manager for results
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = Rc::new(RefCell::new(PluginManager::new(
            entry_arena,
            None,
            None,
            &config,
        )));

        // Connect search widget to results list
        let results_list_clone = results_list.clone();
        let plugin_manager_clone = plugin_manager.clone();

        search_widget.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            if let Ok(results) = plugin_manager_clone.borrow().search(&query, 10) {
                results_list_clone.update_plugin_results(results);
            }
        });

        // Test 1: Empty search should show apps
        // Manually trigger initial population for empty query (set_text("") doesn't emit 'changed')
        if let Ok(results) = plugin_manager.borrow().search("", 10) {
            results_list.update_plugin_results(results);
        }
        search_widget.entry.set_text("");
        while gtk4::glib::MainContext::default().iteration(false) {}

        // Try to get selected command; if nothing is selected yet (headless/unstyled widget),
        // explicitly select the first row and try again.
        let mut command = results_list.get_selected_command();
        if command.is_none() {
            if let Some(first_child) = results_list.list.first_child() {
                if let Some(row) = first_child.downcast_ref::<gtk4::ListBoxRow>() {
                    results_list.list.select_row(Some(row));
                    while gtk4::glib::MainContext::default().iteration(false) {}
                    command = results_list.get_selected_command();
                }
            }
        }
        assert!(command.is_some(), "Should have selected result");

        // Test 2: Search for "firefox"
        search_widget.entry.set_text("firefox");
        while gtk4::glib::MainContext::default().iteration(false) {}

        let firefox_command = results_list.get_selected_command();
        assert!(firefox_command.is_some(), "Should find Firefox");
        if let Some((exec, _)) = &firefox_command {
            assert!(exec.contains("firefox"), "Should select Firefox");
        }

        // Test 3: Navigation works
        results_list.select_next();
        let second_command = results_list.get_selected_command();
        assert_ne!(
            firefox_command, second_command,
            "Navigation should change selection"
        );
    });
}

#[test]
fn test_e2e_keyboard_event_handling() {
    // Test: Keyboard events through plugin system
    use gtk4::gdk::{Key, ModifierType};

    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        // Test 1: Ctrl+Enter with web search query
        let keyboard_event = KeyboardEvent::new(
            Key::Return,
            ModifierType::CONTROL_MASK,
            "g rust programming".to_string(),
            false,
        );

        let action = plugin_manager.dispatch_keyboard_event(&keyboard_event);

        match action {
            KeyboardAction::OpenUrl(url) => {
                assert!(
                    url.contains("google.com"),
                    "Should create Google search URL"
                );
                // Accept either + or %20 encoding for spaces
                let has_plus = url.contains("rust+programming");
                let has_percent = url.contains("rust%20programming");
                assert!(
                    has_plus || has_percent,
                    "Should include encoded search terms"
                );
            }
            _ => panic!(
                "Expected OpenUrl action for Ctrl+Enter web search, got: {:?}",
                action
            ),
        }

        // Test 2: Regular Enter without Ctrl should not trigger web search
        let normal_enter = KeyboardEvent::new(
            Key::Return,
            ModifierType::empty(),
            "g test".to_string(),
            true,
        );

        let action = plugin_manager.dispatch_keyboard_event(&normal_enter);

        match action {
            KeyboardAction::None => {
                // Expected - normal Enter is handled by main.rs, not plugins
            }
            _ => panic!("Expected None for normal Enter, got: {:?}", action),
        }
    });
}

#[test]
fn test_e2e_multi_plugin_search() {
    // Test: Multiple plugins responding to same query
    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        // Query that matches multiple plugins: "code"
        // - Applications: VS Code, VS Codium, etc.
        // - Editors: Recent workspaces
        // - Files: Potentially files with "code" in name
        let results = plugin_manager.search("code", 20).expect("Search failed");

        // Should have results from multiple plugins
        let unique_plugins: std::collections::HashSet<_> =
            results.iter().map(|r| r.plugin_name.as_str()).collect();

        // At minimum, should have applications
        assert!(
            unique_plugins.contains("applications") || unique_plugins.contains("editors"),
            "Should have results from applications or editors plugin"
        );

        // Results should be sorted by score
        let scores: Vec<i64> = results.iter().map(|r| r.score).collect();
        for i in 0..scores.len().saturating_sub(1) {
            assert!(
                scores[i] >= scores[i + 1],
                "Results should be sorted by score (descending)"
            );
        }
    });
}

#[test]
fn test_e2e_fuzzy_search_accuracy() {
    // Test: Fuzzy search finds results even with typos
    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries.clone());
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        // Test fuzzy matching with typos
        let test_cases = vec![
            ("firef", "firefox"),  // Missing letters
            ("frefox", "firefox"), // Transposed letters
            ("chrom", "chrome"),   // Missing letters
            ("vscod", "code"),     // Missing letters
        ];

        for (query, expected_app) in test_cases {
            let results = plugin_manager.search(query, 10).expect("Search failed");

            if !results.is_empty() {
                let found_match = results.iter().any(|r| {
                    r.title.to_lowercase().contains(expected_app)
                        || r.command.to_lowercase().contains(expected_app)
                });

                // Fuzzy search should find close matches
                // Note: This might fail if the app isn't installed, so we don't assert
                if entries
                    .iter()
                    .any(|e| e.name.to_lowercase().contains(expected_app))
                {
                    assert!(
                        found_match,
                        "Fuzzy search should find '{}' when querying '{}'",
                        expected_app, query
                    );
                }
            }
        }
    });
}

#[test]
fn test_e2e_plugin_performance() {
    // Test: Plugins respond within performance targets
    use std::time::Instant;

    run_test(|| {
        // Skip strict performance checks in debug builds unless explicitly enabled
        if cfg!(debug_assertions) && std::env::var("NL_STRICT_PERF").is_err() {
            eprintln!(
                "Skipping strict performance e2e test in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
            );
            return;
        }

        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);

        // Measure plugin manager creation time
        let start = Instant::now();
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);
        let creation_time = start.elapsed();

        println!("Plugin manager creation: {:?}", creation_time);
        assert!(
            creation_time.as_millis() < 100,
            "Plugin manager creation should be <100ms, got {:?}",
            creation_time
        );

        // Measure search performance
        let start = Instant::now();
        let _results = plugin_manager.search("test", 10).expect("Search failed");
        let search_time = start.elapsed();

        println!("Search time: {:?}", search_time);
        assert!(
            search_time.as_millis() < 50,
            "Search should be <50ms, got {:?}",
            search_time
        );

        // Measure multiple searches (cache effectiveness)
        let queries = vec!["firefox", "chrome", "code", "terminal", "2+2", "g test"];
        let start = Instant::now();

        for query in queries {
            plugin_manager.search(query, 10).expect("Search failed");
        }

        let total_time = start.elapsed();
        let avg_time = total_time / 6;

        println!("Average search time: {:?}", avg_time);
        assert!(
            avg_time.as_millis() < 10,
            "Average search should be <10ms, got {:?}",
            avg_time
        );
    });
}

#[test]
fn test_e2e_cache_integration() {
    // Test: Desktop cache saves and loads correctly
    use native_launcher::desktop::cache::DesktopCache;

    // Note: This test doesn't use GTK, but included in E2E suite
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan");

    // Create cache and add entries
    let mut cache = DesktopCache::new();
    for entry in &entries {
        let _ = cache.insert(entry.path.clone(), entry.clone());
    }

    // Save cache
    cache.save().expect("Failed to save cache");

    // Load cache
    let loaded_cache = DesktopCache::load().expect("Failed to load cache");
    let loaded_count = loaded_cache.get_all().len();

    // Verify entries were saved and loaded
    // Note: Some entries might be filtered out if they've been modified
    assert!(loaded_count > 0, "Loaded cache should have entries");
}

#[test]
fn test_e2e_advanced_calculator() {
    // Test: Advanced calculator with complex expressions
    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        let test_cases = vec![
            ("2 + 2", "4"),
            ("10 * 5", "50"),
            ("100 / 4", "25"),
            ("2 ^ 8", "256"),
            ("sqrt(16)", "4"),
            ("sin(0)", "0"),
        ];

        for (expression, expected_result) in test_cases {
            let results = plugin_manager
                .search(expression, 5)
                .expect("Calculator failed");

            if !results.is_empty() {
                let result_text = &results[0].title;
                assert!(
                    result_text.contains(expected_result),
                    "Calculator: '{}' should contain '{}', got '{}'",
                    expression,
                    expected_result,
                    result_text
                );
            }
        }
    });
}

#[test]
fn test_e2e_shell_commands() {
    // Test: Shell plugin handles commands
    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        // Test shell command detection
        let shell_queries = vec!["> ls -la", "> echo hello", "> pwd"];

        for query in shell_queries {
            let results = plugin_manager
                .search(query, 5)
                .expect("Shell search failed");

            if !results.is_empty() {
                let shell_result = results.iter().find(|r| r.plugin_name == "shell");
                assert!(
                    shell_result.is_some(),
                    "Shell plugin should handle '{}'",
                    query
                );
            }
        }
    });
}

#[test]
fn test_e2e_error_handling() {
    // Test: System handles errors gracefully
    run_test(|| {
        let scanner = DesktopScanner::new();
        let entries = scanner.scan().unwrap_or_default();
        let config = Config::default();
        let entry_arena = DesktopEntryArena::from_vec(entries);
        let plugin_manager = PluginManager::new(entry_arena, None, None, &config);

        // Test with very long query (shouldn't crash)
        let long_query = "a".repeat(10000);
        let result = plugin_manager.search(&long_query, 10);
        assert!(result.is_ok(), "Should handle very long queries");

        // Test with special characters
        let special_queries = vec![
            "🦀 rust",
            "test\nwith\nnewlines",
            "test\twith\ttabs",
            "test\"with'quotes",
        ];

        for query in special_queries {
            let result = plugin_manager.search(query, 10);
            assert!(
                result.is_ok(),
                "Should handle special characters: {}",
                query
            );
        }

        // Test with max_results = 0 (edge case)
        let result = plugin_manager.search("test", 0);
        assert!(result.is_ok(), "Should handle max_results = 0");
    });
}
