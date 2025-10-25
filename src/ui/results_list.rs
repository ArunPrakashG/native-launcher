use crate::desktop::{DesktopAction, DesktopEntry};
use crate::plugins::PluginResult;
use crate::utils::icons::resolve_icon;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, ListBox, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::info;

/// Represents an item in the results list
/// SIMPLIFIED: Each item maps directly to what you see and click
#[derive(Debug, Clone)]
enum ListItem {
    /// A main application entry
    #[allow(dead_code)]
    App { entry: DesktopEntry },
    /// An action belonging to an application
    #[allow(dead_code)]
    Action {
        action: DesktopAction,
        parent_entry: DesktopEntry,
    },
    /// A plugin result (from plugin system) - includes workspaces, files, etc.
    PluginResult { result: PluginResult },
}

/// Results list widget
#[derive(Clone)]
pub struct ResultsList {
    pub container: ScrolledWindow,
    pub list: ListBox,
    /// Single flat list: visual order = data order (wrapped in Rc so all clones share same data)
    items: Rc<RefCell<Vec<ListItem>>>,
}

impl ResultsList {
    pub fn new() -> Self {
        let list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        let container = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&list)
            .build();

        Self {
            container,
            list,
            items: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Update the list with search results (desktop apps)
    #[allow(dead_code)]

    pub fn update_results(&self, results: Vec<&DesktopEntry>) {
        // Build flat list: apps + their actions inline
        let mut items = Vec::new();
        for entry in results.iter() {
            let entry = (*entry).clone();
            items.push(ListItem::App {
                entry: entry.clone(),
            });

            // Add actions directly under the app
            for action in entry.actions.iter() {
                items.push(ListItem::Action {
                    action: action.clone(),
                    parent_entry: entry.clone(),
                });
            }
        }

        self.render_items(items);
    }

    /// Update the list with plugin results
    pub fn update_plugin_results(&self, results: Vec<PluginResult>) {
        let items: Vec<ListItem> = results
            .into_iter()
            .map(|result| ListItem::PluginResult { result })
            .collect();

        self.render_items(items);
    }

    /// Append plugin results without clearing existing items (for incremental search)
    pub fn append_plugin_results(&self, results: Vec<PluginResult>) {
        let new_items: Vec<ListItem> = results
            .into_iter()
            .map(|result| ListItem::PluginResult { result })
            .collect();

        // Add to existing items
        let mut items = self.items.borrow_mut();
        items.extend(new_items.clone());
        drop(items);

        // Render only the new items to the UI
        for item in new_items {
            self.render_single_item(item);
        }
    }

    /// Render items to the UI (common logic)
    fn render_items(&self, items: Vec<ListItem>) {
        tracing::debug!("Rendering {} items", items.len());

        *self.items.borrow_mut() = items.clone();

        // Clear existing items
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        // Create a row for each item
        for item in items {
            self.render_single_item(item);
        }

        // Select first row if available
        if let Some(first_row) = self.list.first_child() {
            if let Some(row) = first_row.downcast_ref::<gtk4::ListBoxRow>() {
                self.list.select_row(Some(row));
            }
        }
    }

    /// Render a single item to the UI
    fn render_single_item(&self, item: ListItem) {
        let content_box = match &item {
            ListItem::App { entry } => self.create_result_row(entry),
            ListItem::Action { action, .. } => self.create_action_row(action),
            ListItem::PluginResult { result } => self.create_plugin_result_row(result),
        };

        // Create ListBoxRow and set the child
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&content_box));
        self.list.append(&row);
    }

    /// Get the command to execute based on current selection
    pub fn get_selected_command(&self) -> Option<(String, bool)> {
        let selected_index = self.selected_index()? as usize;
        let items = self.items.borrow();

        // Validate index is within bounds
        if selected_index >= items.len() {
            tracing::error!(
                "Selected index {} is out of bounds (total items: {})",
                selected_index,
                items.len()
            );
            return None;
        }

        items.get(selected_index).map(|item| {
            let (cmd, term) = match item {
                ListItem::App { entry } => (entry.exec.clone(), entry.terminal),
                ListItem::Action {
                    action,
                    parent_entry,
                } => (action.exec.clone(), parent_entry.terminal),
                ListItem::PluginResult { result } => (result.command.clone(), result.terminal),
            };
            (cmd, term)
        })
    }

    /// Get the desktop file path of the currently selected item
    pub fn get_selected_path(&self) -> Option<String> {
        let selected_index = self.selected_index()? as usize;
        let items = self.items.borrow();

        items.get(selected_index).and_then(|item| match item {
            ListItem::App { entry } => Some(entry.path.to_string_lossy().to_string()),
            ListItem::Action { parent_entry, .. } => {
                Some(parent_entry.path.to_string_lossy().to_string())
            }
            // Plugin results don't have desktop file paths
            ListItem::PluginResult { .. } => None,
        })
    }

    /// Create an icon placeholder box for alignment
    fn create_icon_placeholder(&self, size: i32) -> GtkBox {
        GtkBox::builder()
            .width_request(size)
            .height_request(size)
            .build()
    }

    /// Create a row for a desktop action
    fn create_action_row(&self, action: &DesktopAction) -> GtkBox {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(0)
            .margin_bottom(0)
            .margin_start(24) // Indent actions
            .margin_end(0)
            .build();

        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        // Action name
        let name_label = Label::builder()
            .label(&action.name)
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .build();
        name_label.add_css_class("action-name");

        content_box.append(&name_label);
        row.append(&content_box);

        row
    }

    /// Create a row for a plugin result
    fn create_plugin_result_row(&self, result: &PluginResult) -> GtkBox {
        // Check if this is a linked entry (workspace, recent file, etc.)
        let is_linked_entry = result.parent_app.is_some();

        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(0)
            .margin_bottom(0)
            .margin_start(if is_linked_entry { 24 } else { 0 }) // Indent linked entries
            .margin_end(0)
            .build();

        // Add icon (with fallback to default icon)
        let icon_size = if is_linked_entry { 32 } else { 48 };
        let icon_path = result
            .icon
            .as_ref()
            .and_then(|name| resolve_icon(name))
            .or_else(|| {
                use crate::utils::icons::get_default_icon;
                Some(get_default_icon())
            });

        if let Some(icon_path) = icon_path {
            let image = Image::from_file(&icon_path);
            image.set_pixel_size(icon_size);
            image.add_css_class("app-icon");
            if is_linked_entry {
                image.add_css_class("workspace-icon");
            }
            row.append(&image);
        } else {
            row.append(&self.create_icon_placeholder(icon_size));
        }

        // Main content box (vertical layout for title and subtitle)
        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        // Title
        let name_label = Label::builder()
            .label(&result.title)
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .build();
        name_label.add_css_class("app-name");

        content_box.append(&name_label);

        // Subtitle (if available)
        if let Some(ref subtitle) = result.subtitle {
            let subtitle_label = Label::builder()
                .label(subtitle)
                .halign(gtk4::Align::Start)
                .xalign(0.0)
                .build();
            subtitle_label.add_css_class("app-generic");
            content_box.append(&subtitle_label);
        }

        row.append(&content_box);
        row
    }

    /// Create a row for a desktop entry
    fn create_result_row(&self, entry: &DesktopEntry) -> GtkBox {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(0)
            .margin_bottom(0)
            .margin_start(0)
            .margin_end(0)
            .build();

        // Add icon with fallback to default
        let icon_path = entry
            .icon
            .as_ref()
            .and_then(|name| resolve_icon(name))
            .or_else(|| {
                use crate::utils::icons::get_default_icon;
                Some(get_default_icon())
            });

        if let Some(icon_path) = icon_path {
            let image = Image::from_file(&icon_path);
            image.set_pixel_size(48);
            image.add_css_class("app-icon");
            row.append(&image);
        } else {
            row.append(&self.create_icon_placeholder(48));
        }

        // Main content box (vertical layout for name and generic name)
        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        // Application name
        let name_label = Label::builder()
            .label(&entry.name)
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .build();
        name_label.add_css_class("app-name");

        content_box.append(&name_label);

        // Generic name (if available)
        if let Some(ref generic) = entry.generic_name {
            let generic_label = Label::builder()
                .label(generic)
                .halign(gtk4::Align::Start)
                .xalign(0.0)
                .build();
            generic_label.add_css_class("dim-label");
            content_box.append(&generic_label);
        }

        row.append(&content_box);

        row
    }

    /// Get the currently selected entry index
    pub fn selected_index(&self) -> Option<i32> {
        self.list.selected_row().map(|row| row.index())
    }

    /// Select the next item
    pub fn select_next(&self) {
        if let Some(current) = self.list.selected_row() {
            let next_index = current.index() + 1;

            if let Some(next_row) = self.list.row_at_index(next_index) {
                self.list.select_row(Some(&next_row));
                // Auto-scroll to make the selected row visible
                self.scroll_to_selected();

                info!("Selected next row (index {})", next_index);
            }
        }
    }

    /// Select the previous item
    pub fn select_previous(&self) {
        if let Some(current) = self.list.selected_row() {
            let prev_index = current.index() - 1;

            if prev_index >= 0 {
                if let Some(prev_row) = self.list.row_at_index(prev_index) {
                    self.list.select_row(Some(&prev_row));
                    // Auto-scroll to make the selected row visible
                    self.scroll_to_selected();

                    info!("Selected previous row (index {})", prev_index);
                }
            }
        }
    }

    /// Scroll to the currently selected item
    fn scroll_to_selected(&self) {
        if let Some(selected_row) = self.list.selected_row() {
            // Get the adjustment from the scrolled window
            let vadj = self.container.vadjustment();
            let row_height = 60.0; // Approximate row height
            let selected_y = selected_row.index() as f64 * row_height;
            let viewport_height = vadj.page_size();
            let current_scroll = vadj.value();
            let max_scroll = vadj.upper() - viewport_height;

            // Add padding for better visibility
            let padding = 10.0;

            // Check if selected item is outside the visible area
            if selected_y < current_scroll + padding {
                // Scroll up - ensure some padding at the top
                let new_value = (selected_y - padding).max(0.0);
                vadj.set_value(new_value);
            } else if selected_y + row_height + padding > current_scroll + viewport_height {
                // Scroll down - ensure some padding at the bottom
                let new_value =
                    (selected_y + row_height + padding - viewport_height).min(max_scroll);
                vadj.set_value(new_value);
            }
        }
    }
}

impl Default for ResultsList {
    fn default() -> Self {
        Self::new()
    }
}
