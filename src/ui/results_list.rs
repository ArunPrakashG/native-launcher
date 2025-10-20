use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ListBox, Orientation, ScrolledWindow};
use tracing::debug;

use crate::desktop::DesktopEntry;

/// Results list widget
#[derive(Clone)]
pub struct ResultsList {
    pub container: ScrolledWindow,
    pub list: ListBox,
}

impl ResultsList {
    pub fn new() -> Self {
        debug!("Creating results list");

        let list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        let container = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&list)
            .build();

        Self { container, list }
    }

    /// Update the list with search results
    pub fn update_results(&self, results: Vec<&DesktopEntry>) {
        debug!("Updating results: {} entries", results.len());

        // Clear existing items
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        // Add new results
        for entry in results {
            let row = self.create_result_row(entry);
            self.list.append(&row);
        }

        // Select first item
        if let Some(first_row) = self.list.row_at_index(0) {
            self.list.select_row(Some(&first_row));
        }
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
                }
            }
        }
    }
}

impl Default for ResultsList {
    fn default() -> Self {
        Self::new()
    }
}
