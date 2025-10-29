use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Orientation};
use tracing::debug;

/// Search entry widget
#[derive(Clone)]
pub struct SearchWidget {
    pub container: GtkBox,
    pub entry: Entry,
}

impl SearchWidget {
    pub fn new() -> Self {
        debug!("Creating search widget");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .margin_top(0)
            .margin_bottom(0)
            .margin_start(0)
            .margin_end(0)
            .build();

        let entry = Entry::builder()
            .placeholder_text("Search applications...")
            .build(); // Removed hexpand - window is fixed size

        container.append(&entry);

        Self { container, entry }
    }

    /// Get the current search text
    #[allow(dead_code)]

    pub fn text(&self) -> String {
        self.entry.text().to_string()
    }

    /// Clear the search text
    #[allow(dead_code)]

    pub fn clear(&self) {
        self.entry.set_text("");
    }

    /// Focus the search entry
    pub fn grab_focus(&self) {
        self.entry.grab_focus();
    }
}

impl Default for SearchWidget {
    fn default() -> Self {
        Self::new()
    }
}
