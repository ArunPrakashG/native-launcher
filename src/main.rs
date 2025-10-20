mod desktop;
mod search;
mod ui;
mod utils;

use anyhow::Result;
use desktop::{DesktopEntry, DesktopScanner};
use gtk4::gdk::{Display, Key};
use gtk4::prelude::*;
use gtk4::{Application, Box as GtkBox, CssProvider, Orientation};
use search::SearchEngine;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use ui::{LauncherWindow, ResultsList, SearchWidget};
use utils::execute_command;

const APP_ID: &str = "com.github.native-launcher";

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Native Launcher");

    // Scan for desktop applications
    info!("Scanning for desktop applications...");
    let scanner = DesktopScanner::new();
    let entries = scanner.scan()?;
    info!("Found {} applications", entries.len());

    // Create search engine
    let search_engine = Rc::new(RefCell::new(SearchEngine::new(entries.clone())));

    // Create GTK application
    let app = Application::builder().application_id(APP_ID).build();

    // Store entries for use in activate
    let entries_clone = entries.clone();
    let search_engine_clone = search_engine.clone();

    app.connect_activate(move |app| {
        if let Err(e) = build_ui(app, &entries_clone, search_engine_clone.clone()) {
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
    entries: &[DesktopEntry],
    search_engine: Rc<RefCell<SearchEngine>>,
) -> Result<()> {
    info!("Building UI");

    // Load CSS
    load_css();

    // Create main window
    let launcher_window = LauncherWindow::new(app);

    // Create search widget
    let search_widget = SearchWidget::new();

    // Create results list
    let results_list = ResultsList::new();

    // Create main container
    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    main_box.append(&search_widget.container);
    main_box.append(&results_list.container);

    launcher_window.window.set_child(Some(&main_box));

    // Initial results (show all)
    let initial_results: Vec<&DesktopEntry> = entries.iter().collect();
    results_list.update_results(initial_results.into_iter().take(10).collect());

    // Handle search text changes
    {
        let results_list = results_list.clone();
        let search_engine = search_engine.clone();

        search_widget.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let engine = search_engine.borrow();
            let results = engine.search(&query, 10);
            results_list.update_results(results);
        });
    }

    // Handle keyboard events
    {
        let search_widget_clone = search_widget.clone();
        let results_list_clone = results_list.clone();
        let window_clone = launcher_window.window.clone();

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                Key::Escape => {
                    // Close window on Escape
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
                    // Launch selected application
                    if let Some(index) = results_list_clone.selected_index() {
                        let query = search_widget_clone.text();
                        let engine = search_engine.borrow();
                        let results = engine.search(&query, 10);

                        if let Some(entry) = results.get(index as usize) {
                            info!("Launching: {}", entry.name);
                            if let Err(e) = execute_command(&entry.exec, entry.terminal) {
                                error!("Failed to launch {}: {}", entry.name, e);
                            }
                            window_clone.close();
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

fn load_css() {
    let provider = CssProvider::new();
    let css = include_str!("ui/style.css");
    provider.load_from_data(css);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("CSS loaded successfully");
    } else {
        error!("Failed to get default display for CSS loading");
    }
}
