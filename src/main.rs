mod config;
mod daemon;
mod desktop;
mod pins;
mod plugins;
mod search;
mod ui;
mod updater;
mod usage;
mod utils;

use crate::pins::PinsStore;
use anyhow::Result;
use config::ConfigLoader;
use desktop::DesktopScanner;
use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{Application, Box as GtkBox, Orientation};
use plugins::{KeyboardAction, KeyboardEvent, PluginManager};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use ui::{load_theme_with_name, KeyboardHints, LauncherWindow, ResultsList, SearchWidget};
use usage::UsageTracker;
use utils::{build_open_command, execute_command};

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

    let usage_enabled = config.search.usage_ranking;
    if usage_enabled {
        info!("Usage-based ranking enabled (config.search.usage_ranking = true)");
    } else {
        info!("Usage-based ranking disabled via config");
    }

    let usage_tracker = if usage_enabled {
        info!("Loading usage tracking data...");
        let tracker = UsageTracker::load().unwrap_or_else(|e| {
            error!("Failed to load usage data: {}, starting fresh", e);
            UsageTracker::new()
        });
        info!("Loaded usage data for {} apps", tracker.app_count());
        tracker
    } else {
        info!("Skipping usage tracking initialization");
        UsageTracker::new()
    };

    // Check for updates in background (non-blocking)
    if config.updater.check_on_startup {
        info!("Checking for updates in background...");
        let _ = updater::check_for_updates_async();
    }

    // Scan for desktop applications
    info!("Scanning for desktop applications...");
    let scanner = DesktopScanner::new();
    let raw_entries = scanner.scan_cached()?;
    info!("Found {} applications", raw_entries.len());

    let entry_arena = desktop::DesktopEntryArena::from_vec(raw_entries);

    // OPTIMIZATION: Icon cache uses lazy loading on-demand (no preloading)
    // Icons are cached as they're requested during search results rendering
    // This reduces startup time (~10-20ms) and memory usage for rarely-used apps
    // The icon cache itself uses LRU eviction to stay within memory limits

    // Create plugin manager with all plugins
    info!("Initializing plugin system...");
    let usage_tracker_for_plugins = if usage_enabled {
        Some(usage_tracker.clone())
    } else {
        None
    };

    // Load pins store once and share
    let pins_store = if config.search.enable_pins {
        info!("Loading pins store...");
        PinsStore::load().unwrap_or_else(|e| {
            warn!("Failed to load pins: {} - starting empty", e);
            PinsStore::new()
        })
    } else {
        PinsStore::new()
    };
    let pins_store = Arc::new(pins_store);

    let mut plugin_manager = PluginManager::new(
        entry_arena.clone(),
        usage_tracker_for_plugins,
        if config.search.enable_pins {
            Some(pins_store.clone())
        } else {
            None
        },
        &config,
    );

    // Load dynamic plugins
    info!("Loading dynamic plugins...");
    let (dynamic_plugins, plugin_metrics) = plugins::load_plugins();
    for plugin in dynamic_plugins {
        plugin_manager.register_plugin(plugin);
    }

    // Populate browser index if enabled and stale (normal mode - dev only)
    // In production, users should run in daemon mode for background indexing
    if cfg!(debug_assertions) && config.plugins.browser_history {
        let browser_plugin = plugins::BrowserHistoryPlugin::new();
        if let Some(index) = browser_plugin.get_index() {
            if index.needs_rebuild() {
                info!("Browser index needs refresh, populating in background (dev mode)...");
                let browser_arc = std::sync::Arc::new(browser_plugin.clone());
                std::thread::spawn(move || {
                    info!("Fetching browser history for index...");
                    let entries = browser_arc.fetch_all_history();
                    info!("Indexing {} browser entries...", entries.len());
                    if let Err(e) = index.rebuild_index(entries) {
                        error!("Failed to build browser index: {}", e);
                    } else {
                        if let Ok(count) = index.entry_count() {
                            info!("Browser index built with {} entries", count);
                        }
                    }
                });
            } else {
                if let Ok(count) = index.entry_count() {
                    info!("Browser index already up-to-date with {} entries", count);
                }
            }
        }
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
            usage_enabled,
            &config_clone,
            metrics_clone.clone(),
            Some(pins_store.clone()), // Ensure new pins_store parameter is passed
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
    usage_enabled: bool,
    config: &config::Config,
    plugin_metrics: Rc<Vec<plugins::PluginMetrics>>,
    pins_store: Option<Arc<PinsStore>>,
) -> Result<()> {
    info!("Building UI");

    // Load CSS theme from config
    info!("Loading theme: {}", config.ui.theme);
    load_theme_with_name(&config.ui.theme);

    let merge_login_env = config.environment.merge_login_env;

    // Create main window with config
    let launcher_window = LauncherWindow::new(app);

    // Apply window config - use FIXED size to prevent expansion
    launcher_window
        .window
        .set_default_width(config.window.width);
    launcher_window
        .window
        .set_default_height(config.window.height);

    // CRITICAL: Prevent window from resizing beyond default size
    launcher_window.window.set_resizable(false);

    // Create search widget
    let search_widget = SearchWidget::new();

    // Create results list
    let results_list = ResultsList::new();
    if let Some(pins) = &pins_store {
        results_list.set_pins_store(pins.clone());
    }

    // Search footer removed (no longer used)

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

    // CRITICAL: Prevent container from expanding/shrinking
    main_box.set_vexpand(false);
    main_box.set_hexpand(false);

    // Apply density class from config
    let density_class = match config.ui.density.as_str() {
        "compact" => "density-compact",
        _ => "density-comfortable", // Default to comfortable
    };
    main_box.add_css_class(density_class);
    info!("Applied density mode: {}", config.ui.density);

    // Apply accent color class from config
    let accent_class = format!("accent-{}", config.ui.accent);
    main_box.add_css_class(&accent_class);
    info!("Applied accent color: {}", config.ui.accent);

    // Add plugin warning at the top if there are slow plugins
    if let Some(warning) = plugin_warning {
        main_box.append(&warning);
    }

    main_box.append(&search_widget.container);
    main_box.append(&results_list.container);
    // Footer removed from layout per design
    main_box.append(&keyboard_hints.container);

    launcher_window.window.set_child(Some(&main_box));

    // Initial results - show recently used apps and top applications (20 items)
    info!("Loading default results (recent + top apps)...");
    match plugin_manager.borrow().search("", 20) {
        Ok(default_results) => {
            info!("Showing {} default results", default_results.len());
            results_list.update_plugin_results(default_results);
        }
        Err(e) => {
            error!("Failed to get default results: {}", e);
            results_list.update_plugin_results(Vec::new());
        }
    }

    // Handle search text changes with debouncing to prevent lag
    {
        let results_list = results_list.clone();
        // Footer removed; no footer updates
        let plugin_manager = plugin_manager.clone();
        let max_results = config.search.max_results;

        // Debounce timeout holder and cancellation flag
        // We use a counter instead of removing sources to avoid GTK panics
        let debounce_counter: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

        search_widget.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();

            // Footer removed: no per-keystroke footer hints

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
            // Footer removed: no loading indicator
            let debounce_counter_clone = debounce_counter.clone();
            let query_clone = query.clone();

            // DEBOUNCED: Wait 30ms after last keystroke before searching (optimized for fast typing)
            // Shorter delay provides better responsiveness without excessive searches
            gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
                // Check if this timeout is still valid (not superseded by newer typing)
                if *debounce_counter_clone.borrow() != current_count {
                    debug!("Skipping stale search (user still typing)");
                    return;
                }

                // Use incremental search for better perceived performance
                let manager = plugin_manager_clone.borrow();
                let results_list_for_fast = results_list_clone.clone();
                let results_list_for_slow = results_list_clone.clone();

                // Keep current query for highlighting
                results_list_clone.set_query(&query_clone);

                let result = manager.search_incremental(
                    &query_clone,
                    max_results,
                    // Fast results callback - apps, calculator (instant)
                    move |fast_results| {
                        debug!("Displaying {} fast results", fast_results.len());
                        results_list_for_fast.update_plugin_results(fast_results);
                    },
                    // Slow results callback - files, SSH (may take longer)
                    move |slow_results| {
                        debug!("Appending {} slow results", slow_results.len());
                        if !slow_results.is_empty() {
                            results_list_for_slow.append_plugin_results(slow_results);
                        }

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

            // Shift+Enter on clipboard results: copy without closing window
            if modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK) {
                if let Some(plugin_name) = results_list.get_selected_plugin_name() {
                    if plugin_name == "clipboard" {
                        if let Some((command, terminal)) = results_list.get_selected_command() {
                            if let Err(e) = execute_command(&command, terminal, merge_login_env) {
                                error!("Failed to execute copy command: {}", e);
                            }
                            // Do not close window; stop further handling
                            return;
                        }
                    }
                }
            }

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

                        // Track usage when enabled
                        if usage_enabled {
                            if let Some(path) = results_list.get_selected_path() {
                                usage_tracker_clone.borrow_mut().record_launch(&path);
                                info!("Recorded launch for {}", path);
                            }
                        }

                        // IMPORTANT: Hide window BEFORE launching app
                        // This ensures the new app gets focus and appears in foreground
                        window_clone.close();

                        if let Err(e) = execute_command(&exec, terminal, merge_login_env) {
                            error!("Failed to launch {}: {}", exec, e);
                        }
                    }
                }
                KeyboardAction::OpenUrl(url) => {
                    info!("Opening URL from plugin: {}", url);

                    // IMPORTANT: Hide window BEFORE opening URL
                    window_clone.close();

                    let open_command = build_open_command(&url);

                    if let Err(e) = execute_command(&open_command, false, merge_login_env) {
                        error!("Failed to open URL: {}", e);
                    }
                }
                KeyboardAction::Execute { command, terminal } => {
                    info!("Executing command from plugin: {}", command);

                    // IMPORTANT: Hide window BEFORE executing command
                    window_clone.close();

                    if let Err(e) = execute_command(&command, terminal, merge_login_env) {
                        error!("Failed to execute command: {}", e);
                    }
                }
                KeyboardAction::Handled => {
                    // Plugin handled it but don't close window
                    debug!("Keyboard event handled by plugin");
                }
                KeyboardAction::OpenFolder(path) => {
                    info!("Opening folder: {}", path);
                    let folder = if std::path::Path::new(&path).is_dir() {
                        path
                    } else {
                        std::path::Path::new(&path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".to_string())
                    };

                    window_clone.close();

                    let open_command = build_open_command(&folder);
                    if let Err(e) = execute_command(&open_command, false, merge_login_env) {
                        error!("Failed to open folder: {}", e);
                    }
                }
                KeyboardAction::CopyPath(path) => {
                    info!("Copying path to clipboard: {}", path);
                    let copy_cmd = if std::process::Command::new("which")
                        .arg("wl-copy")
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                    {
                        format!("echo -n '{}' | wl-copy", path.replace('\'', r"'\''"))
                    } else if std::process::Command::new("which")
                        .arg("xclip")
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                    {
                        format!(
                            "echo -n '{}' | xclip -selection clipboard",
                            path.replace('\'', r"'\''")
                        )
                    } else {
                        error!("No clipboard tool found (need wl-copy or xclip)");
                        return;
                    };

                    if let Err(e) = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&copy_cmd)
                        .spawn()
                    {
                        error!("Failed to copy path: {}", e);
                    }
                }
            }
        });
    }

    // Handle mouse activation (double-click) on results list
    {
        let results_list_clone = results_list.clone();
        let window_clone = launcher_window.window.clone();
        let usage_tracker_clone = usage_tracker.clone();

        results_list.list.connect_row_activated(move |_, _| {
            handle_selected_result(
                &results_list_clone,
                &window_clone,
                &usage_tracker_clone,
                usage_enabled,
                merge_login_env,
            );
        });
    }

    // Handle keyboard events
    {
        let results_list_clone = results_list.clone();
        let window_clone = launcher_window.window.clone();
        let usage_tracker_clone = usage_tracker.clone();
        let search_entry_clone = search_widget.entry.clone();
        // Footer removed
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

                    // Preview theme if a theme item is selected
                    if let Some((command, _)) = results_list_clone.get_selected_command() {
                        if let Some(theme_name) = command.strip_prefix("@theme:") {
                            info!("Previewing theme: {}", theme_name);
                            load_theme_with_name(theme_name);
                        }
                    }

                    gtk4::glib::Propagation::Stop
                }
                Key::Up => {
                    // Move selection up
                    results_list_clone.select_previous();

                    // Preview theme if a theme item is selected
                    if let Some((command, _)) = results_list_clone.get_selected_command() {
                        if let Some(theme_name) = command.strip_prefix("@theme:") {
                            info!("Previewing theme: {}", theme_name);
                            load_theme_with_name(theme_name);
                        }
                    }

                    gtk4::glib::Propagation::Stop
                }
                Key::Return => {
                    // Shift+Enter on clipboard results: copy without closing window
                    if modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK) {
                        if let Some(plugin_name) = results_list_clone.get_selected_plugin_name() {
                            if plugin_name == "clipboard" {
                                if let Some((command, terminal)) =
                                    results_list_clone.get_selected_command()
                                {
                                    if let Err(e) =
                                        execute_command(&command, terminal, merge_login_env)
                                    {
                                        error!("Failed to execute copy command: {}", e);
                                    }
                                    // Keep window open
                                    return gtk4::glib::Propagation::Stop;
                                }
                            }
                        }
                    }
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
                            handle_selected_result(
                                &results_list_clone,
                                &window_clone,
                                &usage_tracker_clone,
                                usage_enabled,
                                merge_login_env,
                            );
                        }
                        KeyboardAction::OpenUrl(url) => {
                            info!("Opening URL from plugin: {}", url);
                            // IMPORTANT: Hide window BEFORE opening URL
                            window_clone.close();

                            let open_command = build_open_command(&url);

                            if let Err(e) = execute_command(&open_command, false, merge_login_env) {
                                error!("Failed to open URL: {}", e);
                            }
                        }
                        KeyboardAction::Execute { command, terminal } => {
                            info!("Executing command from plugin: {}", command);

                            // IMPORTANT: Hide window BEFORE executing command
                            window_clone.close();

                            if let Err(e) = execute_command(&command, terminal, merge_login_env) {
                                error!("Failed to execute command: {}", e);
                            }
                        }
                        KeyboardAction::Handled => {
                            // Plugin handled it but don't close window
                            debug!("Keyboard event handled by plugin");
                        }
                        KeyboardAction::OpenFolder(path) => {
                            info!("Opening folder: {}", path);
                            // Open containing folder (extract parent directory from path)
                            let folder = if std::path::Path::new(&path).is_dir() {
                                path
                            } else {
                                std::path::Path::new(&path)
                                    .parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| ".".to_string())
                            };

                            window_clone.close();

                            let open_command = build_open_command(&folder);
                            if let Err(e) = execute_command(&open_command, false, merge_login_env) {
                                error!("Failed to open folder: {}", e);
                            }
                        }
                        KeyboardAction::CopyPath(path) => {
                            info!("Copying path to clipboard: {}", path);
                            // Copy to clipboard using wl-copy or xclip
                            let copy_cmd = if std::process::Command::new("which")
                                .arg("wl-copy")
                                .output()
                                .map(|o| o.status.success())
                                .unwrap_or(false)
                            {
                                format!("echo -n '{}' | wl-copy", path.replace('\'', r"'\''"))
                            } else if std::process::Command::new("which")
                                .arg("xclip")
                                .output()
                                .map(|o| o.status.success())
                                .unwrap_or(false)
                            {
                                format!(
                                    "echo -n '{}' | xclip -selection clipboard",
                                    path.replace('\'', r"'\''")
                                )
                            } else {
                                error!("No clipboard tool found (need wl-copy or xclip)");
                                return gtk4::glib::Propagation::Stop;
                            };

                            if let Err(e) = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&copy_cmd)
                                .spawn()
                            {
                                error!("Failed to copy path: {}", e);
                            }

                            // Don't close window - user might want to copy multiple paths
                        }
                    }

                    gtk4::glib::Propagation::Stop
                }
                _ => {
                    // Ctrl+P: Toggle pin on selected app (if supported)
                    if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                        let maybe_char = key.to_unicode();
                        if maybe_char == Some('p') || maybe_char == Some('P') {
                            if let Some(path) = results_list_clone.get_selected_path() {
                                if let Some(pins) = &pins_store {
                                    match pins.toggle(&path) {
                                        Ok(_pinned) => {
                                            // Refresh only visuals (stars)
                                            results_list_clone.rerender();
                                        }
                                        Err(e) => warn!("Failed to toggle pin: {}", e),
                                    }
                                }
                            }
                            return gtk4::glib::Propagation::Stop;
                        }
                        // Ctrl+1: Execute first result (fast keyboard workflow)
                        else if maybe_char == Some('1') {
                            info!("Ctrl+1: Executing first result");

                            // Select first result if none selected
                            if results_list_clone.get_selected_command().is_none() {
                                results_list_clone.select_first();
                            }

                            // Execute the (now) selected result
                            handle_selected_result(
                                &results_list_clone,
                                &window_clone,
                                &usage_tracker_clone,
                                usage_enabled,
                                merge_login_env,
                            );

                            return gtk4::glib::Propagation::Stop;
                        }
                    }
                    gtk4::glib::Propagation::Proceed
                }
            }
        });

        launcher_window.window.add_controller(key_controller);
    }

    // Add key handler to search entry to prevent it from consuming Up/Down arrows
    // This ensures arrow keys always navigate results, not cursor position
    {
        let entry_key_controller = gtk4::EventControllerKey::new();

        entry_key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                Key::Up | Key::Down => {
                    // Let these keys propagate to the window controller
                    // which will handle result navigation
                    gtk4::glib::Propagation::Proceed
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });

        search_widget.entry.add_controller(entry_key_controller);
    }

    // Show window
    launcher_window.show();
    search_widget.grab_focus();

    info!("UI built successfully");
    Ok(())
}

// Footer hints removed – bottom bar now handles all shortcut hints

fn handle_selected_result(
    results_list: &ResultsList,
    window: &gtk4::ApplicationWindow,
    usage_tracker: &Rc<RefCell<UsageTracker>>,
    usage_enabled: bool,
    merge_login_env: bool,
) -> bool {
    if let Some((exec, terminal)) = results_list.get_selected_command() {
        if let Some(theme_name) = exec.strip_prefix("@theme:") {
            info!("Switching to theme: {}", theme_name);
            load_theme_with_name(theme_name);

            match ConfigLoader::load() {
                Ok(mut loader) => {
                    let mut updated_config = loader.config().clone();
                    updated_config.ui.theme = theme_name.to_string();

                    if let Err(e) = loader.update(updated_config) {
                        warn!("Failed to persist theme to config: {}", e);
                    } else {
                        info!("Theme '{}' saved to config", theme_name);
                    }
                }
                Err(e) => warn!("Failed to load config to persist theme: {}", e),
            }

            return true;
        }

        info!("Launching: {}", exec);

        if usage_enabled {
            if let Some(path) = results_list.get_selected_path() {
                usage_tracker.borrow_mut().record_launch(&path);
                info!("Recorded launch for {}", path);
            }
        }

        window.close();

        if let Err(e) = execute_command(&exec, terminal, merge_login_env) {
            error!("Failed to launch {}: {}", exec, e);
        }

        return true;
    }

    false
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

    let usage_enabled = config.search.usage_ranking;
    if usage_enabled {
        info!("Usage-based ranking enabled (config.search.usage_ranking = true)");
    } else {
        info!("Usage-based ranking disabled via config");
    }

    let usage_tracker = if usage_enabled {
        info!("Loading usage tracking data...");
        let tracker = UsageTracker::load().unwrap_or_else(|e| {
            error!("Failed to load usage data: {}, starting fresh", e);
            UsageTracker::new()
        });
        info!("Loaded usage data for {} apps", tracker.app_count());
        tracker
    } else {
        info!("Skipping usage tracking initialization");
        UsageTracker::new()
    };

    // Scan for desktop applications
    info!("Scanning for desktop applications...");
    let scanner = DesktopScanner::new();
    let raw_entries = scanner.scan_cached()?;
    info!("Found {} applications", raw_entries.len());

    let entry_arena = desktop::DesktopEntryArena::from_vec(raw_entries);

    // OPTIMIZATION: Icon cache uses lazy loading on-demand (no preloading)
    // Icons are cached as they're requested during search results rendering
    // This reduces startup time (~10-20ms) and memory usage for rarely-used apps
    // The icon cache itself uses LRU eviction to stay within memory limits

    // Create plugin manager with all plugins
    info!("Initializing plugin system...");
    let usage_tracker_for_plugins = if usage_enabled {
        Some(usage_tracker.clone())
    } else {
        None
    };

    // Load pins store once and share (daemon mode)
    // Pins store for daemon mode
    let pins_store = Arc::new(if config.search.enable_pins {
        info!("Loading pins store (daemon mode)...");
        PinsStore::load().unwrap_or_else(|e| {
            warn!("Failed to load pins: {} - starting empty", e);
            PinsStore::new()
        })
    } else {
        PinsStore::new()
    });

    // Create browser history plugin separately so we can start indexer
    let browser_plugin = if config.plugins.browser_history {
        Some(Arc::new(plugins::BrowserHistoryPlugin::new()))
    } else {
        None
    };

    let mut plugin_manager = PluginManager::new(
        entry_arena.clone(),
        usage_tracker_for_plugins,
        if config.search.enable_pins {
            Some(pins_store.clone())
        } else {
            None
        },
        &config,
    );

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

    // Start browser history indexer in background
    if let Some(ref browser) = browser_plugin {
        info!("Starting browser history indexer...");
        daemon::start_browser_indexer(browser.clone());
    }

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
            usage_enabled,
            &config_clone,
            metrics_clone.clone(),
            Some(pins_store.clone()),
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
