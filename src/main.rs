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
use plugins::{KeyboardAction, KeyboardEvent, PluginManager};
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
    let mut plugin_manager =
        PluginManager::new(entries.clone(), Some(usage_tracker.clone()), &config);

    // Load dynamic plugins
    info!("Loading dynamic plugins...");
    let (dynamic_plugins, plugin_metrics) = plugins::load_plugins();
    for plugin in dynamic_plugins {
        plugin_manager.register_plugin(plugin);
    }

    let plugin_manager = Rc::new(RefCell::new(plugin_manager));
    info!(
        "Enabled plugins: {:?}",
        plugin_manager.borrow().enabled_plugins()
    );

    // Store plugin metrics for UI display
    let plugin_metrics_rc = Rc::new(plugin_metrics);

    // Wrap usage tracker for shared access
    let usage_tracker_rc = Rc::new(RefCell::new(usage_tracker));

    // Create GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Store plugin manager and config for use in activate
    let plugin_manager_clone = plugin_manager.clone();
    let usage_tracker_clone = usage_tracker_rc.clone();
    let config_clone = config.clone();
    let metrics_clone = plugin_metrics_rc.clone();

    app.connect_activate(move |app| {
        if let Err(e) = build_ui(
            app,
            plugin_manager_clone.clone(),
            usage_tracker_clone.clone(),
            &config_clone,
            metrics_clone.clone(),
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
    plugin_metrics: Rc<Vec<plugins::PluginMetrics>>,
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

    // Check for slow plugins and show warning if needed
    let slow_plugins: Vec<_> = plugin_metrics
        .iter()
        .filter(|m| m.success && m.is_very_slow())
        .collect();

    let plugin_warning = if !slow_plugins.is_empty() {
        let warning_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .css_classes(vec!["plugin-warning"])
            .build();

        let warning_icon = gtk4::Image::from_icon_name("dialog-warning");
        warning_icon.set_pixel_size(16);

        let warning_text = gtk4::Label::builder()
            .label(format!(
                "⚠️ {} slow plugin{} detected (>50ms load time)",
                slow_plugins.len(),
                if slow_plugins.len() > 1 { "s" } else { "" }
            ))
            .css_classes(vec!["plugin-warning-text"])
            .halign(gtk4::Align::Start)
            .build();

        warning_box.append(&warning_icon);
        warning_box.append(&warning_text);

        // Log plugin details
        for metric in &slow_plugins {
            info!(
                "Slow plugin: {} - {:?}, {}",
                metric
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                metric.load_time,
                metric.memory_size_string()
            );
        }

        Some(warning_box)
    } else {
        None
    };

    // Create main container
    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    // Add plugin warning at the top if there are slow plugins
    if let Some(warning) = plugin_warning {
        main_box.append(&warning);
    }

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
        let plugin_manager_clone = plugin_manager.clone();

        search_widget.entry.connect_activate(move |entry| {
            // Get current modifiers
            let display = entry.display();
            let seat = display.default_seat();
            let modifiers = seat
                .and_then(|s| s.keyboard())
                .map(|k| k.modifier_state())
                .unwrap_or(gtk4::gdk::ModifierType::empty());

            // Create keyboard event and dispatch to plugins
            let query = search_entry_clone.text().to_string();
            let has_selection = results_list.get_selected_command().is_some();

            let keyboard_event = KeyboardEvent::new(Key::Return, modifiers, query, has_selection);

            // Dispatch to plugins
            let action = plugin_manager_clone
                .borrow()
                .dispatch_keyboard_event(&keyboard_event);

            match action {
                KeyboardAction::None => {
                    // No plugin handled it, launch selected item
                    if let Some((exec, terminal)) = results_list.get_selected_command() {
                        info!("Launching: {}", exec);

                        // Track usage
                        if let Some(path) = results_list.get_selected_path() {
                            usage_tracker_clone.borrow_mut().record_launch(&path);
                            info!("Recorded launch for {}", path);
                        }

                        // IMPORTANT: Hide window BEFORE launching app
                        // This ensures the new app gets focus and appears in foreground
                        window_clone.close();

                        if let Err(e) = execute_command(&exec, terminal) {
                            error!("Failed to launch {}: {}", exec, e);
                        }
                    }
                }
                KeyboardAction::OpenUrl(url) => {
                    info!("Opening URL from plugin: {}", url);
                    
                    // IMPORTANT: Hide window BEFORE opening URL
                    window_clone.close();
                    
                    if let Err(e) = execute_command(&format!("xdg-open '{}'", url), false) {
                        error!("Failed to open URL: {}", e);
                    }
                }
                KeyboardAction::Execute { command, terminal } => {
                    info!("Executing command from plugin: {}", command);
                    
                    // IMPORTANT: Hide window BEFORE executing command
                    window_clone.close();
                    
                    if let Err(e) = execute_command(&command, terminal) {
                        error!("Failed to execute command: {}", e);
                    }
                }
                KeyboardAction::Handled => {
                    // Plugin handled it but don't close window
                    debug!("Keyboard event handled by plugin");
                }
            }
        });
    }

    // Handle keyboard events
    {
        let results_list_clone = results_list.clone();
        let window_clone = launcher_window.window.clone();
        let usage_tracker_clone = usage_tracker.clone();
        let search_entry_clone = search_widget.entry.clone();
        let plugin_manager_clone = plugin_manager.clone();

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
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
                    // Create keyboard event and dispatch to plugins
                    let query = search_entry_clone.text().to_string();
                    let has_selection = results_list_clone.get_selected_command().is_some();

                    let keyboard_event = KeyboardEvent::new(key, modifiers, query, has_selection);

                    // Dispatch to plugins - they handle Ctrl+Enter for web search, etc.
                    let action = plugin_manager_clone
                        .borrow()
                        .dispatch_keyboard_event(&keyboard_event);

                    match action {
                        KeyboardAction::None => {
                            // No plugin handled it, use default behavior (launch selected item)
                            if let Some((exec, terminal)) =
                                results_list_clone.get_selected_command()
                            {
                                info!("Launching: {}", exec);

                                // Track usage
                                if let Some(path) = results_list_clone.get_selected_path() {
                                    usage_tracker_clone.borrow_mut().record_launch(&path);
                                    info!("Recorded launch for {}", path);
                                }

                                // IMPORTANT: Hide window BEFORE launching app
                                // This ensures the new app gets focus and appears in foreground
                                window_clone.close();

                                if let Err(e) = execute_command(&exec, terminal) {
                                    error!("Failed to launch {}: {}", exec, e);
                                }
                            }
                        }
                        KeyboardAction::OpenUrl(url) => {
                            info!("Opening URL from plugin: {}", url);
                            
                            // IMPORTANT: Hide window BEFORE opening URL
                            window_clone.close();
                            
                            if let Err(e) = execute_command(&format!("xdg-open '{}'", url), false) {
                                error!("Failed to open URL: {}", e);
                            }
                        }
                        KeyboardAction::Execute { command, terminal } => {
                            info!("Executing command from plugin: {}", command);
                            
                            // IMPORTANT: Hide window BEFORE executing command
                            window_clone.close();
                            
                            if let Err(e) = execute_command(&command, terminal) {
                                error!("Failed to execute command: {}", e);
                            }
                        }
                        KeyboardAction::Handled => {
                            // Plugin handled it but don't close window
                            debug!("Keyboard event handled by plugin");
                        }
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
