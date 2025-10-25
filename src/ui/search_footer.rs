use gtk4::{prelude::*, Box as GtkBox, Label, Orientation};

/// Footer widget showing web search information
#[derive(Clone)]
pub struct SearchFooter {
    pub container: GtkBox,
    label: Label,
    loading_label: Label,
}

impl SearchFooter {
    /// Create a new search footer
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_css_classes(&["search-footer"]);
        container.set_visible(false); // Hidden by default

        let label = Label::new(None);
        label.set_css_classes(&["search-footer-text"]);
        label.set_halign(gtk4::Align::Center);
        label.set_hexpand(true);

        let loading_label = Label::new(None);
        loading_label.set_css_classes(&["search-footer-loading"]);
        loading_label.set_halign(gtk4::Align::Center);
        loading_label.set_hexpand(true);
        loading_label.set_visible(false);

        container.append(&label);
        container.append(&loading_label);

        Self {
            container,
            label,
            loading_label,
        }
    }

    /// Update footer with web search information
    pub fn update(&self, engine: &str, query: &str, browser: &str) {
        // Use markup to highlight engine name and shortcut in coral
        let markup = format!(
            "Search <span foreground='#ff6363' weight='bold'>{}</span> for '{}' in {} <span foreground='#ff6363'>(Ctrl+Enter)</span>",
            engine, query, browser
        );
        self.label.set_markup(&markup);
    }

    /// Show loading indicator
    pub fn show_loading(&self) {
        self.loading_label
            .set_markup("<span foreground='#ff6363'>⏳ Searching files...</span>");
        self.loading_label.set_visible(true);
        self.label.set_visible(false);
        self.container.set_visible(true);
    }

    /// Hide loading indicator
    pub fn hide_loading(&self) {
        self.loading_label.set_visible(false);
        self.label.set_visible(true);
    }

    /// Show the footer
    pub fn show(&self) {
        self.hide_loading(); // Ensure loading is hidden when showing web search
        self.container.set_visible(true);
    }

    /// Hide the footer
    pub fn hide(&self) {
        self.container.set_visible(false);
        self.hide_loading();
    }

    /// Check if footer is visible
    #[allow(dead_code)] // Part of widget API
    pub fn is_visible(&self) -> bool {
        self.container.is_visible()
    }
}

impl Default for SearchFooter {
    fn default() -> Self {
        Self::new()
    }
}
