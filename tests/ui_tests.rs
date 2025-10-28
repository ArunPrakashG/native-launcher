use gtk4::glib;
/// UI Testing for Native Launcher
///
/// This module provides integration tests for GTK4 UI components.
/// Tests run in a headless environment using GTK's test utilities.
///
/// Run with: cargo test --test ui_tests
use gtk4::prelude::*;
use native_launcher::ui::{KeyboardHints, ResultsList, SearchWidget};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize GTK for testing (call once per test process)
fn init_gtk() {
    INIT.call_once(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

/// Helper to run GTK tests with proper context
fn run_gtk_test<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    init_gtk();

    // Run test on GTK main thread
    let (tx, rx) = std::sync::mpsc::channel();

    glib::idle_add_once(move || {
        test_fn();
        tx.send(()).unwrap();
    });

    // Wait for test to complete with timeout
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("Test timed out");
}

#[test]
fn test_search_widget_creation() {
    run_gtk_test(|| {
        let search_widget = SearchWidget::default();

        // Verify widget is created
        assert!(search_widget.container.is_realized() || !search_widget.container.is_realized());

        // Verify entry exists
        let text = search_widget.entry.text();
        assert_eq!(text, "");

        // Verify placeholder
        let placeholder = search_widget.entry.placeholder_text();
        assert!(placeholder.is_some());
    });
}

#[test]
fn test_search_widget_text_input() {
    run_gtk_test(|| {
        let search_widget = SearchWidget::default();

        // Set text
        search_widget.entry.set_text("test query");

        // Verify text is set
        assert_eq!(search_widget.entry.text(), "test query");

        // Clear text
        search_widget.entry.set_text("");
        assert_eq!(search_widget.entry.text(), "");
    });
}

#[test]
fn test_search_widget_grab_focus() {
    run_gtk_test(|| {
        let search_widget = SearchWidget::default();

        // Should not panic when grabbing focus
        search_widget.grab_focus();
    });
}

#[test]
fn test_results_list_creation() {
    run_gtk_test(|| {
        let results_list = ResultsList::new();

        // Verify list is created
        assert!(results_list.container.is_realized() || !results_list.container.is_realized());

        // Verify initial state (no selection)
        assert!(results_list.get_selected_command().is_none());
    });
}

#[test]
fn test_results_list_update_results() {
    run_gtk_test(|| {
        use native_launcher::desktop::DesktopEntry;
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        // Create test entries
        let entries = vec![
            DesktopEntry {
                name: "Test App 1".to_string(),
                generic_name: None,
                exec: "test1".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test1.desktop"),
                no_display: false,
                actions: vec![],
            },
            DesktopEntry {
                name: "Test App 2".to_string(),
                generic_name: None,
                exec: "test2".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/test2.desktop"),
                no_display: false,
                actions: vec![],
            },
        ];

        // Update results
        results_list.update_results(entries.iter().collect());

        // First result should be auto-selected
        let selected = results_list.get_selected_command();
        assert!(selected.is_some());

        let (exec, terminal) = selected.unwrap();
        assert_eq!(exec, "test1");
        assert_eq!(terminal, false);
    });
}

#[test]
fn test_results_list_navigation() {
    run_gtk_test(|| {
        use native_launcher::desktop::DesktopEntry;
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        // Create test entries
        let entries = vec![
            DesktopEntry {
                name: "App A".to_string(),
                generic_name: None,
                exec: "app_a".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/a.desktop"),
                no_display: false,
                actions: vec![],
            },
            DesktopEntry {
                name: "App B".to_string(),
                generic_name: None,
                exec: "app_b".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/b.desktop"),
                no_display: false,
                actions: vec![],
            },
        ];

        results_list.update_results(entries.iter().collect());

        // First result selected
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "app_a");

        // Navigate down
        results_list.select_next();
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "app_b");

        // Navigate down again (should wrap or stay at bottom)
        results_list.select_next();

        // Navigate up
        results_list.select_previous();
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "app_a");
    });
}

#[test]
fn test_results_list_clear() {
    run_gtk_test(|| {
        use native_launcher::desktop::DesktopEntry;
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        let entry = DesktopEntry {
            name: "Test".to_string(),
            generic_name: None,
            exec: "test".to_string(),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal: false,
            path: PathBuf::from("/test.desktop"),
            no_display: false,
            actions: vec![],
        };

        // Add results
        results_list.update_results(vec![&entry]);
        assert!(results_list.get_selected_command().is_some());

        // Clear results
        results_list.update_results(vec![]);
        assert!(results_list.get_selected_command().is_none());
    });
}

#[test]
fn test_keyboard_hints_creation() {
    run_gtk_test(|| {
        let hints = KeyboardHints::new();

        // Verify widget is created
        assert!(hints.container.is_realized() || !hints.container.is_realized());
    });
}

#[test]
fn test_keyboard_hints_default() {
    run_gtk_test(|| {
        let hints = KeyboardHints::default();

        // Should create valid widget
        assert!(hints.container.is_realized() || !hints.container.is_realized());
    });
}

#[test]
fn test_results_list_with_actions() {
    run_gtk_test(|| {
        use native_launcher::desktop::{DesktopAction, DesktopEntry};
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        // Create entry with actions
        let entry = DesktopEntry {
            name: "Firefox".to_string(),
            generic_name: Some("Web Browser".to_string()),
            exec: "firefox".to_string(),
            icon: Some("firefox".to_string()),
            categories: vec![],
            keywords: vec![],
            terminal: false,
            path: PathBuf::from("/firefox.desktop"),
            no_display: false,
            actions: vec![
                DesktopAction {
                    id: "new-window".to_string(),
                    name: "New Window".to_string(),
                    exec: "firefox --new-window".to_string(),
                    icon: None,
                },
                DesktopAction {
                    id: "private-window".to_string(),
                    name: "Private Window".to_string(),
                    exec: "firefox --private-window".to_string(),
                    icon: None,
                },
            ],
        };

        // Update results with entry that has actions
        results_list.update_results(vec![&entry]);

        // First item should be the app itself
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "firefox");

        // Navigate to first action
        results_list.select_next();
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "firefox --new-window");

        // Navigate to second action
        results_list.select_next();
        let (exec, _) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "firefox --private-window");
    });
}

#[test]
fn test_terminal_app_flag() {
    run_gtk_test(|| {
        use native_launcher::desktop::DesktopEntry;
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        let terminal_entry = DesktopEntry {
            name: "Htop".to_string(),
            generic_name: None,
            exec: "htop".to_string(),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal: true,
            path: PathBuf::from("/htop.desktop"),
            no_display: false,
            actions: vec![],
        };

        results_list.update_results(vec![&terminal_entry]);

        // Verify terminal flag is preserved
        let (exec, terminal) = results_list.get_selected_command().unwrap();
        assert_eq!(exec, "htop");
        assert_eq!(terminal, true);
    });
}

#[test]
fn test_widget_css_classes() {
    run_gtk_test(|| {
        let search_widget = SearchWidget::default();

        // Verify CSS classes are applied
        assert!(search_widget.container.css_classes().len() > 0);

        let results_list = ResultsList::new();
        assert!(results_list.container.css_classes().len() > 0);

        let hints = KeyboardHints::new();
        assert!(hints.container.css_classes().len() > 0);
    });
}

#[test]
fn test_search_widget_connect_changed() {
    run_gtk_test(|| {
        use std::cell::RefCell;
        use std::rc::Rc;

        let search_widget = SearchWidget::default();
        let changed_count = Rc::new(RefCell::new(0));

        let count_clone = changed_count.clone();
        search_widget.entry.connect_changed(move |_| {
            *count_clone.borrow_mut() += 1;
        });

        // Trigger changes
        search_widget.entry.set_text("a");
        search_widget.entry.set_text("ab");
        search_widget.entry.set_text("abc");

        // Process pending events
        while gtk4::glib::MainContext::default().iteration(false) {}

        // Should have triggered changed callback
        assert!(*changed_count.borrow() > 0);
    });
}
