use crate::desktop::{DesktopAction, DesktopEntry};
use crate::plugins::PluginResult;
use crate::utils::icons::resolve_icon;
use gtk4::prelude::*;
use gtk4::{
    pango::EllipsizeMode, Box as GtkBox, Image, Label, ListBox, Orientation, ScrolledWindow,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, info};

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
            .child(&list)
            .has_frame(false) // No frame for clean rounded corners
            .build();

        // Removed hexpand/vexpand - window is fixed size, should not expand

        // Ensure scrolling policies are set correctly
        container.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

        // CRITICAL: Set fixed size to prevent expansion/shrinking
        // This ensures the scrolled window always maintains 400px height
        container.set_size_request(-1, 400); // -1 for width (use available), 400px height
        container.set_vexpand(false); // Don't expand vertically
        container.set_hexpand(false); // Don't expand horizontally

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
        // Pre-allocate capacity (estimate: 1 app + average 1 action per app)
        let mut items = Vec::with_capacity(results.len() * 2);

        for entry in results.iter() {
            // Clone entry once for all uses
            let entry_owned = (*entry).clone();

            items.push(ListItem::App {
                entry: entry_owned.clone(),
            });

            // Add actions directly under the app
            for action in entry_owned.actions.iter() {
                items.push(ListItem::Action {
                    action: action.clone(),
                    parent_entry: entry_owned.clone(),
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

        if new_items.is_empty() {
            return;
        }

        // Add to existing items - no need to clone, we can move
        let mut items = self.items.borrow_mut();
        let was_empty = items.is_empty();
        items.extend(new_items.iter().cloned());
        drop(items);

        // Render only the new items to the UI
        for item in new_items {
            self.render_single_item(item);
        }

        if was_empty {
            if let Some(first_row) = self.list.first_child() {
                if let Some(row) = first_row.downcast_ref::<gtk4::ListBoxRow>() {
                    self.list.select_row(Some(row));
                }
            }
        }
    }

    /// Render items to the UI (common logic)
    fn render_items(&self, items: Vec<ListItem>) {
        tracing::debug!("Rendering {} items", items.len());

        // Store items for later use (e.g., getting selected command)
        *self.items.borrow_mut() = items;

        // Clear existing items from UI
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        // Render items from stored copy (borrow and clone individual items as needed)
        // This is more efficient than cloning the entire Vec upfront
        for item in self.items.borrow().iter() {
            // Clone individual items only when rendering (GTK requires ownership)
            self.render_single_item(item.clone());
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
        row.add_css_class("inline-action-row");

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
        name_label.set_hexpand(true);
        name_label.set_wrap(false);
        name_label.set_ellipsize(EllipsizeMode::End);
        name_label.set_max_width_chars(60);

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
            .margin_start(if is_linked_entry { 8 } else { 0 }) // Subtle indent for linked entries
            .margin_end(0)
            .build();

        if is_linked_entry {
            row.add_css_class("inline-action-row");
            row.add_css_class("inline-action-with-icon");
        }

        // Add icon (with fallback to default icon)
        let icon_size = if is_linked_entry { 32 } else { 48 };
        let icon_path = Self::resolve_plugin_icon(result).or_else(|| {
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
        name_label.set_hexpand(true);
        name_label.set_wrap(false);
        name_label.set_ellipsize(EllipsizeMode::End);
        name_label.set_max_width_chars(60);

        content_box.append(&name_label);

        // Subtitle (if available)
        if let Some(ref subtitle) = result.subtitle {
            let subtitle_label = Label::builder()
                .label(subtitle)
                .halign(gtk4::Align::Start)
                .xalign(0.0)
                .build();
            subtitle_label.add_css_class("app-generic");
            subtitle_label.set_hexpand(true);
            subtitle_label.set_wrap(false);
            subtitle_label.set_ellipsize(EllipsizeMode::End);
            subtitle_label.set_max_width_chars(60);
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
        name_label.set_hexpand(true);
        name_label.set_wrap(false);
        name_label.set_ellipsize(EllipsizeMode::End);
        name_label.set_max_width_chars(60);

        content_box.append(&name_label);

        // Generic name (if available)
        if let Some(ref generic) = entry.generic_name {
            let generic_label = Label::builder()
                .label(generic)
                .halign(gtk4::Align::Start)
                .xalign(0.0)
                .build();
            generic_label.add_css_class("dim-label");
            generic_label.set_hexpand(true);
            generic_label.set_wrap(false);
            generic_label.set_ellipsize(EllipsizeMode::End);
            generic_label.set_max_width_chars(60);
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

            // Check if there's actually a next row before doing anything
            if let Some(next_row) = self.list.row_at_index(next_index) {
                self.list.select_row(Some(&next_row));
                // Auto-scroll to make the selected row visible
                self.scroll_to_selected();

                info!("Selected next row (index {})", next_index);
            } else {
                // Already at the last item, do nothing to avoid unnecessary computation
                debug!("Already at last item, ignoring Down arrow");
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
                } else {
                    // Already at the first item, do nothing
                    debug!("Already at first item, ignoring Up arrow");
                }
            } else {
                // Already at the first item (index 0), do nothing
                debug!("Already at first item, ignoring Up arrow");
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

impl ResultsList {
    fn resolve_plugin_icon(result: &PluginResult) -> Option<PathBuf> {
        if let Some(icon_name) = result.icon.as_deref() {
            if let Some(path) = resolve_icon(icon_name) {
                return Some(path);
            }
        }

        if let Some(parent_app) = result.parent_app.as_deref() {
            if let Some(path) = Self::resolve_parent_app_icon(parent_app) {
                return Some(path);
            }
        }

        None
    }

    fn resolve_parent_app_icon(parent_app: &str) -> Option<PathBuf> {
        for candidate in Self::icon_candidates_for_parent(parent_app) {
            if let Some(path) = resolve_icon(candidate) {
                return Some(path);
            }
        }
        None
    }

    fn icon_candidates_for_parent(parent_app: &str) -> &'static [&'static str] {
        match parent_app {
            "code" | "Code" | "vscode" | "Visual Studio Code" => &[
                "com.visualstudio.code",
                "com.visualstudio.code-oss",
                "code",
                "visual-studio-code",
                "vscode",
            ],
            "codium" | "Codium" | "vscodium" | "VSCodium" => {
                &["com.vscodium.codium", "vscodium", "codium"]
            }
            "subl" | "Subl" | "sublime" | "Sublime" | "sublime-text" => {
                &["sublime-text", "com.sublimetext.three", "subl"]
            }
            "zed" | "Zed" => &["dev.zed.Zed", "zed"],
            _ => &[],
        }
    }
}

impl Default for ResultsList {
    fn default() -> Self {
        Self::new()
    }
}
