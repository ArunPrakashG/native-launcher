use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

/// Widget that displays keyboard shortcuts at the bottom of the window
#[derive(Clone)]
pub struct KeyboardHints {
    pub container: GtkBox,
    hint_label: Label,
}

impl KeyboardHints {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_margin_top(0);
        container.set_margin_bottom(8);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.add_css_class("keyboard-hints");

        let hint_label = Label::new(None);
        hint_label.set_markup(&Self::get_default_hints());
        hint_label.add_css_class("hint-text");
        hint_label.set_halign(gtk4::Align::Center);
        hint_label.set_hexpand(true);

        container.append(&hint_label);

        Self {
            container,
            hint_label,
        }
    }

    fn get_default_hints() -> String {
        String::from(
            "<span size='small' alpha='60%'>\
            <b>↑↓</b> Navigate  •  \
            <b>↵</b> Launch  •  \
            <b>Alt+↵</b> Folder  •  \
            <b>Ctrl+↵</b> Copy Path  •  \
            <b>Ctrl+P</b> Pin  •  \
            <b>ESC</b> Close\
            </span>",
        )
    }

    #[allow(dead_code)]

    fn get_action_mode_hints() -> String {
        String::from(
            "<span size='small' alpha='60%'>\
            <b>↑↓</b> Navigate  •  \
            <b>↵</b> Run Action  •  \
            <b>Ctrl+1</b> First  •  \
            <b>Ctrl+P</b> Pin  •  \
            <b>Ctrl+Enter</b> Web  •  \
            <b>←</b> Back  •  \
            <b>ESC</b> Close\
            </span>",
        )
    }

    /// Update hints based on context
    #[allow(dead_code)]

    pub fn set_action_mode(&self, in_action_mode: bool) {
        let hints = if in_action_mode {
            Self::get_action_mode_hints()
        } else {
            Self::get_default_hints()
        };
        self.hint_label.set_markup(&hints);
    }

    /// Show visual feedback when a key is pressed
    #[allow(dead_code)]

    pub fn flash_key(&self, _key_name: &str) {
        // Add a CSS class temporarily to highlight the pressed key
        self.hint_label.add_css_class("key-pressed");

        // Remove the class after a short delay
        let label = self.hint_label.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            label.remove_css_class("key-pressed");
            gtk4::glib::ControlFlow::Break
        });
    }
}

impl Default for KeyboardHints {
    fn default() -> Self {
        Self::new()
    }
}
