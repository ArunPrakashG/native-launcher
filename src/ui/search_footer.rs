use gtk4::pango::EllipsizeMode;
use gtk4::{prelude::*, Align, Box as GtkBox, Label, Orientation, Stack, StackTransitionType};

/// Footer widget showing web search information
#[derive(Clone)]
pub struct SearchFooter {
    pub container: GtkBox,
    stack: Stack,
    loading_label: Label,
    hints_label: Label,
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

        // Loading state indicator
        let loading_box = GtkBox::new(Orientation::Horizontal, 6);
        loading_box.set_halign(Align::Center);
        loading_box.set_hexpand(true);

        let loading_label = Label::new(Some("⏳ Searching files..."));
        loading_label.set_css_classes(&["search-footer-text", "search-footer-loading"]);
        loading_box.append(&loading_label);

        // Hints mode content
        let hints_box = GtkBox::new(Orientation::Horizontal, 6);
        hints_box.set_halign(Align::Center);
        hints_box.set_hexpand(true);
        let hints_label = Label::new(None);
        hints_label.set_css_classes(&["search-footer-text"]);
        hints_label.set_halign(Align::Center);
        hints_label.set_wrap(true);
        hints_label.set_ellipsize(EllipsizeMode::End);
        hints_label.set_max_width_chars(100);
        hints_box.append(&hints_label);

        stack.add_named(&placeholder_box, Some("placeholder"));
        stack.add_named(&loading_box, Some("loading"));
        stack.add_named(&hints_box, Some("hints"));

        container.append(&stack);

        let footer = Self {
            container,
            stack,
            loading_label,
            hints_label,
        };
        // Start hidden; becomes visible when showing web hint or loading
        footer.container.set_visible(false);

        footer
    }

    /// Show loading indicator
    pub fn show_loading(&self) {
        self.loading_label.set_text("⏳ Searching files...");
        self.container.set_visible(true);
        self.stack.set_visible_child_name("loading");
    }

    /// Hide loading indicator
    pub fn hide_loading(&self) {
        // Prefer showing hints if present; otherwise fallback to placeholder
        if self.hints_label.text().is_empty() {
            self.stack.set_visible_child_name("placeholder");
        } else {
            self.stack.set_visible_child_name("hints");
        }
    }

    /// Show contextual hints text (lightweight guidance)
    pub fn show_hints(&self, text: &str) {
        self.hints_label.set_text(text);
        self.container.set_visible(true);
        self.stack.set_visible_child_name("hints");
    }

    /// Show default placeholder text (footer always visible)
    pub fn show_placeholder(&self) {
        self.container.set_visible(true);
        self.stack.set_visible_child_name("placeholder");
    }
}

impl Default for SearchFooter {
    fn default() -> Self {
        Self::new()
    }
}
