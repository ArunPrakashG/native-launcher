mod config;
mod daemon;
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

    // Check for daemon mode flag
    let args: Vec<String> = std::env::args().collect();
    let daemon_mode = args.contains(&"--daemon".to_string());

    if daemon_mode {
        info!("Starting in daemon mode");
        return run_daemon_mode();
    }

    // Check if daemon is already running
    if daemon::is_daemon_running() {
        info!("Daemon is already running, sending show signal");
        daemon::send_show_signal()?;
        return Ok(());
    }

    // Run in normal mode (single-shot)
    info!("Starting in normal mode");
    run_normal_mode()
}

fn run_normal_mode() -> Result<()> {
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

    // Initial results - check config for empty state behavior
    let max_results = config.search.max_results;
    let empty_state_on_launch = config.ui.empty_state_on_launch;

    if !empty_state_on_launch {
        // Traditional behavior: show all apps on launch
        match plugin_manager.borrow().search("", max_results) {
            Ok(initial_results) => results_list.update_plugin_results(initial_results),
            Err(e) => error!("Failed to get initial results: {}", e),
        }
    } else {
        // Spotlight-style: start with empty list
        results_list.update_plugin_results(Vec::new());
        // Hide results container initially
        results_list.container.set_visible(false);
    }

    // Handle search text changes with debouncing to prevent lag
    {
        let results_list = results_list.clone();
        let search_footer_clone = search_footer.clone();
        let plugin_manager = plugin_manager.clone();
        let max_results = config.search.max_results;
        let empty_state_on_launch = config.ui.empty_state_on_launch;

        // Debounce timeout holder and cancellation flag
        // We use a counter instead of removing sources to avoid GTK panics
        let debounce_counter: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

        search_widget.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();

            // Show/hide results container based on empty state config
            if empty_state_on_launch {
                if query.is_empty() {
                    results_list.container.set_visible(false);
                } else {
                    results_list.container.set_visible(true);
                }
            }

            // IMMEDIATE: Update web search footer (no delay)
            // This gives instant visual feedback even with debouncing
            if let Some((engine, search_term, _url)) = detect_web_search(&query) {
                let browser = get_default_browser();
                search_footer_clone.update(&engine, &search_term, &browser);
                search_footer_clone.show();
            } else {
                search_footer_clone.hide();
            }

            // Increment counter to cancel any pending searches
            // Previous timeout will check counter and skip search if stale
            let current_count = {
                let mut counter = debounce_counter.borrow_mut();
                *counter += 1;
                *counter
            };

            // Clone refs for closure
            let plugin_manager_clone = plugin_manager.clone();
            let plugin_manager_for_metrics = plugin_manager.clone();
            let results_list_clone = results_list.clone();
            let search_footer_for_loading = search_footer_clone.clone();
            let debounce_counter_clone = debounce_counter.clone();
            let query_clone = query.clone();

            // DEBOUNCED: Wait 150ms after last keystroke before searching
            // This prevents lag when typing quickly (e.g., "config" triggers 1 search, not 6)
            gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
                // Check if this timeout is still valid (not superseded by newer typing)
                if *debounce_counter_clone.borrow() != current_count {
                    debug!("Skipping stale search (user still typing)");
                    return;
                }

                // Use incremental search for better perceived performance
                let manager = plugin_manager_clone.borrow();
                let results_list_for_fast = results_list_clone.clone();
                let results_list_for_slow = results_list_clone.clone();
                let footer_for_fast = search_footer_for_loading.clone();
                let footer_for_slow = search_footer_for_loading.clone();
                let query_for_check = query_clone.clone();

                let result = manager.search_incremental(
                    &query_clone,
                    max_results,
                    // Fast results callback - apps, calculator (instant)
                    move |fast_results| {
                        debug!("Displaying {} fast results", fast_results.len());
                        results_list_for_fast.update_plugin_results(fast_results);

                        // Show loading indicator if query might trigger file search
                        if !query_for_check.starts_with('@') && query_for_check.len() >= 3 {
                            footer_for_fast.show_loading();
                        }
                    },
                    // Slow results callback - files, SSH (may take longer)
                    move |slow_results| {
                        debug!("Appending {} slow results", slow_results.len());
                        if !slow_results.is_empty() {
                            results_list_for_slow.append_plugin_results(slow_results);
                        }
                        footer_for_slow.hide_loading();

                        // Log performance metrics (every 10th search)
                        let manager_ref = plugin_manager_for_metrics.borrow();
                        let metrics = manager_ref.get_performance_metrics();
                        if !metrics.is_empty() {
                            let total_calls: u32 = metrics.iter().map(|(_, _, count)| count).sum();
                            if total_calls.is_multiple_of(10) {
                                debug!("Plugin performance (avg ms, calls):");
                                for (name, avg_ms, count) in metrics.iter().take(5) {
                                    debug!("  {}: {:.2}ms ({} calls)", name, avg_ms, count);
                                }
                            }
                        }
                    },
                );

                if let Err(e) = result {
                    error!("Incremental search failed: {}", e);
                }
            });
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

fn run_daemon_mode() -> Result<()> {
    info!("Initializing daemon mode");

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

    // Start Unix socket listener
    info!("Starting daemon socket listener...");
    let socket_receiver = daemon::start_socket_listener()?;

    // Register cleanup handler
    let cleanup_guard = scopeguard::guard((), |_| {
        daemon::cleanup_socket();
    });

    // Create GTK application for daemon
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::IS_SERVICE)
        .build();

    // Store state for activate callback
    let plugin_manager_clone = plugin_manager.clone();
    let usage_tracker_clone = usage_tracker_rc.clone();
    let config_clone = config.clone();
    let metrics_clone = plugin_metrics_rc.clone();

    // Track window state
    let window_ref: Rc<RefCell<Option<gtk4::ApplicationWindow>>> = Rc::new(RefCell::new(None));
    let window_ref_for_socket = window_ref.clone();

    // Handle socket messages in GTK main loop
    gtk4::glib::spawn_future_local(async move {
        loop {
            // Check for messages from socket listener
            if let Ok(command) = socket_receiver.try_recv() {
                info!("Daemon received command: {}", command);

                if command == "show" {
                    let window_opt = window_ref_for_socket.borrow_mut();

                    if let Some(window) = window_opt.as_ref() {
                        // Window exists, just show it
                        info!("Showing existing window");
                        window.present();
                    } else {
                        // Window doesn't exist, create it
                        info!("Window not found, this shouldn't happen in daemon mode");
                    }
                }
            }

            // Sleep a bit to avoid busy loop
            gtk4::glib::timeout_future(std::time::Duration::from_millis(50)).await;
        }
    });

    // Build UI on activation (first time only)
    app.connect_activate(move |app| {
        let mut window_opt = window_ref.borrow_mut();

        if window_opt.is_some() {
            // Window already exists, just show it
            info!("Window already exists, showing it");
            if let Some(window) = window_opt.as_ref() {
                window.present();
            }
            return;
        }

        // Build UI for the first time
        info!("Building UI for daemon mode");
        match build_ui(
            app,
            plugin_manager_clone.clone(),
            usage_tracker_clone.clone(),
            &config_clone,
            metrics_clone.clone(),
        ) {
            Ok(()) => {
                // Store window reference
                if let Some(window) = app.active_window() {
                    if let Ok(app_window) = window.downcast::<gtk4::ApplicationWindow>() {
                        *window_opt = Some(app_window);
                        info!("Window created and stored for daemon mode");
                    }
                }
            }
            Err(e) => {
                error!("Failed to build UI: {}", e);
                app.quit();
            }
        }
    });

    // Don't show window immediately in daemon mode - wait for signal
    info!("Daemon ready, waiting for show signals");

    // Run the application
    let exit_code = app.run();
    info!("Daemon exited with code: {:?}", exit_code);

    // Cleanup is handled by scopeguard
    drop(cleanup_guard);

    Ok(())
}
