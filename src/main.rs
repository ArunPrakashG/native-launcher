mod config;
mod desktop;
mod plugins;
mod search;
mod ui;
mod usage;
mod utils;

use anyhow::Result;
use config::ConfigLoader;
use desktop::DesktopScanner;
use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{Application, Box as GtkBox, Orientation};
use plugins::PluginManager;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use ui::{load_theme, KeyboardHints, LauncherWindow, ResultsList, SearchWidget};
use usage::UsageTracker;
use utils::{detect_web_search, execute_command, get_default_browser};

const APP_ID: &str = "com.github.native-launcher";

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Native Launcher");

    // Load configuration
    info!("Loading configuration...");
    let config_loader = ConfigLoader::load().unwrap_or_else(|e| {
        error!("Failed to load config: {}, using defaults", e);
        ConfigLoader::new()
    });
    let config = config_loader.config().clone();
    info!("Config loaded from {:?}", config_loader.path());

    // Load usage tracking data
    info!("Loading usage tracking data...");
    let usage_tracker = UsageTracker::load().unwrap_or_else(|e| {
        error!("Failed to load usage data: {}, starting fresh", e);
        UsageTracker::new()
    });
    info!("Loaded usage data for {} apps", usage_tracker.app_count());

    // Scan for desktop applications
    info!("Scanning for desktop applications...");
    let scanner = DesktopScanner::new();
    let entries = scanner.scan_cached()?;
    info!("Found {} applications", entries.len());

    // Start background icon cache preloading
    info!("Starting icon cache preloading in background...");
    let entries_for_cache = entries.clone();
    std::thread::spawn(move || {
        utils::icons::preload_icon_cache(&entries_for_cache);
    });

    // Create plugin manager with all plugins
    info!("Initializing plugin system...");
    let plugin_manager = Rc::new(RefCell::new(PluginManager::new(
        entries.clone(),
        Some(usage_tracker.clone()),
        &config,
    )));
    info!(
        "Enabled plugins: {:?}",
        plugin_manager.borrow().enabled_plugins()
    );

    // Wrap usage tracker for shared access
    let usage_tracker_rc = Rc::new(RefCell::new(usage_tracker));

    // Create GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Store plugin manager and config for use in activate
    let plugin_manager_clone = plugin_manager.clone();
    let usage_tracker_clone = usage_tracker_rc.clone();
    let config_clone = config.clone();

    app.connect_activate(move |app| {
        if let Err(e) = build_ui(
            app,
            plugin_manager_clone.clone(),
            usage_tracker_clone.clone(),
            &config_clone,
        ) {
            error!("Failed to build UI: {}", e);
            app.quit();
        }
    });

    // Run the application
    let exit_code = app.run();
    info!("Application exited with code: {:?}", exit_code);

    Ok(())
}

fn build_ui(
    app: &Application,
    plugin_manager: Rc<RefCell<PluginManager>>,
    usage_tracker: Rc<RefCell<UsageTracker>>,
    config: &config::Config,
) -> Result<()> {
    info!("Building UI");

    // Load CSS theme
    load_theme();

    // Create main window with config
    let launcher_window = LauncherWindow::new(app);

    // Apply window config
    launcher_window
        .window
        .set_default_width(config.window.width);
    launcher_window
        .window
        .set_default_height(config.window.height);

    // Create search widget
    let search_widget = SearchWidget::new();

    // Create results list
    let results_list = ResultsList::new();

    // Create search footer
    let search_footer = ui::SearchFooter::new();

    // Create keyboard hints
    let keyboard_hints = KeyboardHints::new();

    // Create main container
    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    main_box.append(&search_widget.container);
    main_box.append(&results_list.container);
    main_box.append(&search_footer.container);
    main_box.append(&keyboard_hints.container);

    launcher_window.window.set_child(Some(&main_box));

    // Initial results (show all apps) - use max_results from config
    let max_results = config.search.max_results;
    match plugin_manager.borrow().search("", max_results) {
        Ok(initial_results) => results_list.update_plugin_results(initial_results),
        Err(e) => error!("Failed to get initial results: {}", e),
    }

    // Handle search text changes
    {
        let results_list = results_list.clone();
        let search_footer_clone = search_footer.clone();
        let plugin_manager = plugin_manager.clone();
        let max_results = config.search.max_results;

        search_widget.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let manager = plugin_manager.borrow();

            // Check if this is a web search query
            if let Some((engine, search_term, _url)) = detect_web_search(&query) {
                // Show footer with web search info
                let browser = get_default_browser();
                search_footer_clone.update(&engine, &search_term, &browser);
                search_footer_clone.show();
            } else {
                // Hide footer for non-web-search queries
                search_footer_clone.hide();
            }

            match manager.search(&query, max_results) {
                Ok(results) => results_list.update_plugin_results(results),
                Err(e) => error!("Search failed: {}", e),
            }
        });
    }

    // Handle Enter key in search entry
    {
        let results_list = results_list.clone();
        let window_clone = launcher_window.window.clone();
        let usage_tracker_clone = usage_tracker.clone();
        let search_entry_clone = search_widget.entry.clone();
        let search_footer_clone = search_footer.clone();

        search_widget.entry.connect_activate(move |entry| {
            // Get current event to check modifiers
            let display = entry.display();
            let seat = display.default_seat();

            if let Some(seat) = seat {
                if let Some(keyboard) = seat.keyboard() {
                    let modifiers = keyboard.modifier_state();

                    // Check if Ctrl is pressed for web search
                    if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                        // Ctrl+Enter: Execute web search directly
                        if search_footer_clone.is_visible() {
                            let query = search_entry_clone.text().to_string();
                            debug!("Ctrl+Enter pressed in entry, query: '{}'", query);
                            if let Some((engine, search_term, url)) = detect_web_search(&query) {
                                info!("Web search: {} for '{}'", engine, search_term);
                                // Open URL in default browser (URL built by WebSearchPlugin)
                                if let Err(e) =
                                    execute_command(&format!("xdg-open '{}'", url), false)
                                {
                                    error!("Failed to open URL: {}", e);
                                }
                                window_clone.close();
                                return;
                            }
                        }
                    }
                }
            }

            // Regular Enter: Launch selected application or action
            if let Some((exec, terminal)) = results_list.get_selected_command() {
                info!("Launching: {}", exec);

                // Track usage
                if let Some(path) = results_list.get_selected_path() {
                    usage_tracker_clone.borrow_mut().record_launch(&path);
                    info!("Recorded launch for {}", path);
                }

                if let Err(e) = execute_command(&exec, terminal) {
                    error!("Failed to launch {}: {}", exec, e);
                }
                window_clone.close();
            }
        });
    }

    // Handle keyboard events
    {
        let results_list_clone = results_list.clone();
        let window_clone = launcher_window.window.clone();
        let usage_tracker_clone = usage_tracker.clone();
        let search_entry_clone = search_widget.entry.clone();
        let search_footer_clone = search_footer.clone();

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            use gtk4::gdk::ModifierType;

            match key {
                Key::Escape => {
                    // Close window
                    window_clone.close();
                    gtk4::glib::Propagation::Stop
                }
                Key::Down => {
                    // Move selection down
                    results_list_clone.select_next();
                    gtk4::glib::Propagation::Stop
                }
                Key::Up => {
                    // Move selection up
                    results_list_clone.select_previous();
                    gtk4::glib::Propagation::Stop
                }
                Key::Return => {
                    // Check if Ctrl is pressed for web search
                    if modifiers.contains(ModifierType::CONTROL_MASK) {
                        // Ctrl+Enter: Execute web search directly
                        if search_footer_clone.is_visible() {
                            let query = search_entry_clone.text().to_string();
                            debug!("Ctrl+Enter pressed, query: '{}'", query);
                            if let Some((engine, search_term, url)) = detect_web_search(&query) {
                                info!("Web search: {} for '{}'", engine, search_term);
                                // Open URL in default browser (URL built by WebSearchPlugin)
                                if let Err(e) =
                                    execute_command(&format!("xdg-open '{}'", url), false)
                                {
                                    error!("Failed to open URL: {}", e);
                                }
                                window_clone.close();
                            }
                        }
                        return gtk4::glib::Propagation::Stop;
                    }

                    // Regular Enter: Launch selected application or action
                    if let Some((exec, terminal)) = results_list_clone.get_selected_command() {
                        info!("Launching: {}", exec);

                        // Track usage
                        if let Some(path) = results_list_clone.get_selected_path() {
                            usage_tracker_clone.borrow_mut().record_launch(&path);
                            info!("Recorded launch for {}", path);
                        }

                        if let Err(e) = execute_command(&exec, terminal) {
                            error!("Failed to launch {}: {}", exec, e);
                        }
                        window_clone.close();
                    }
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });

        launcher_window.window.add_controller(key_controller);
    }

    // Show window
    launcher_window.show();
    search_widget.grab_focus();

    info!("UI built successfully");
    Ok(())
}
