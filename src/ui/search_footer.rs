use std::cell::Cell;

use gtk4::pango::EllipsizeMode;
use gtk4::{prelude::*, Align, Box as GtkBox, Label, Orientation, Stack, StackTransitionType};

/// Footer widget showing web search information
#[derive(Clone)]
pub struct SearchFooter {
    pub container: GtkBox,
    stack: Stack,
    engine_label: Label,
    query_label: Label,
    browser_label: Label,
    loading_label: Label,
    has_content: Cell<bool>,
}

impl SearchFooter {
    /// Create a new search footer
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_css_classes(&["search-footer"]);

        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(0);
        stack.set_hexpand(true);
        stack.set_halign(Align::Center);

        // Placeholder content: "Press Ctrl+Enter to search the web"
        let placeholder_box = GtkBox::new(Orientation::Horizontal, 6);
        placeholder_box.set_halign(Align::Center);
        placeholder_box.set_hexpand(true);

        let placeholder_prefix = Label::new(Some("Press"));
        placeholder_prefix.set_css_classes(&["search-footer-text"]);
        placeholder_box.append(&placeholder_prefix);

        let placeholder_shortcut = Label::new(Some("Ctrl+Enter"));
        placeholder_shortcut.set_css_classes(&["search-footer-text", "search-footer-accent"]);
        placeholder_box.append(&placeholder_shortcut);

        let placeholder_suffix = Label::new(Some("to search the web"));
        placeholder_suffix.set_css_classes(&["search-footer-text"]);
        placeholder_box.append(&placeholder_suffix);

        // Content for active web search hint
        let content_box = GtkBox::new(Orientation::Horizontal, 6);
        content_box.set_halign(Align::Center);
        content_box.set_hexpand(true);

        let prefix_label = Label::new(Some("Search"));
        prefix_label.set_css_classes(&["search-footer-text"]);
        content_box.append(&prefix_label);

        let engine_label = Label::new(None);
        engine_label.set_css_classes(&["search-footer-text", "search-footer-accent"]);
        content_box.append(&engine_label);

        let for_label = Label::new(Some("for"));
        for_label.set_css_classes(&["search-footer-text"]);
        content_box.append(&for_label);

        let query_label = Label::new(None);
        query_label.set_css_classes(&["search-footer-text", "search-footer-query"]);
        query_label.set_ellipsize(EllipsizeMode::End);
        query_label.set_max_width_chars(32);
        content_box.append(&query_label);

        let in_label = Label::new(Some("in"));
        in_label.set_css_classes(&["search-footer-text"]);
        content_box.append(&in_label);

        let browser_label = Label::new(None);
        browser_label.set_css_classes(&["search-footer-text", "search-footer-browser"]);
        browser_label.set_ellipsize(EllipsizeMode::End);
        browser_label.set_max_width_chars(20);
        content_box.append(&browser_label);

        let shortcut_label = Label::new(Some("(Ctrl+Enter)"));
        shortcut_label.set_css_classes(&["search-footer-text", "search-footer-accent"]);
        content_box.append(&shortcut_label);

        // Loading state indicator
        let loading_box = GtkBox::new(Orientation::Horizontal, 6);
        loading_box.set_halign(Align::Center);
        loading_box.set_hexpand(true);

        let loading_label = Label::new(Some("⏳ Searching files..."));
        loading_label.set_css_classes(&["search-footer-text", "search-footer-loading"]);
        loading_box.append(&loading_label);

        stack.add_named(&placeholder_box, Some("placeholder"));
        stack.add_named(&content_box, Some("content"));
        stack.add_named(&loading_box, Some("loading"));

        container.append(&stack);

        let footer = Self {
            container,
            stack,
            engine_label,
            query_label,
            browser_label,
            loading_label,
            has_content: Cell::new(false),
        };

        footer.show_placeholder();

        footer
    }

    /// Update footer with web search information
    pub fn update(&self, engine: &str, query: &str, browser: &str) {
        self.engine_label.set_text(engine);
        self.query_label.set_text(&format!("'{}'", query));
        self.browser_label.set_text(browser);
        self.has_content.set(true);
        self.container.set_visible(true);
        self.stack.set_visible_child_name("content");
    }

    /// Show loading indicator
    pub fn show_loading(&self) {
        self.loading_label.set_text("⏳ Searching files...");
        self.container.set_visible(true);
        self.stack.set_visible_child_name("loading");
    }

    /// Hide loading indicator
    pub fn hide_loading(&self) {
        if self.has_content.get() {
            self.stack.set_visible_child_name("content");
        } else {
            self.stack.set_visible_child_name("placeholder");
        }
    }

    /// Show the footer
    pub fn show(&self) {
        self.container.set_visible(true);
        if self.has_content.get() {
            self.stack.set_visible_child_name("content");
        } else {
            self.stack.set_visible_child_name("placeholder");
        }
    }

    /// Hide the footer
    pub fn hide(&self) {
        self.show_placeholder();
    }

    /// Check if footer is visible
    #[allow(dead_code)] // Part of widget API
    pub fn is_visible(&self) -> bool {
        self.container.is_visible()
    }

    /// Show default placeholder text (footer always visible)
    pub fn show_placeholder(&self) {
        self.has_content.set(false);
        self.container.set_visible(true);
        self.stack.set_visible_child_name("placeholder");
    }
}

impl Default for SearchFooter {
    fn default() -> Self {
        Self::new()
    }
}
