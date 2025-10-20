use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tracing::{debug, info};

const APP_ID: &str = "com.github.native-launcher";

/// Main application window
pub struct LauncherWindow {
    pub window: ApplicationWindow,
}

impl LauncherWindow {
    /// Create a new launcher window
    pub fn new(app: &Application) -> Self {
        info!("Creating launcher window");

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Native Launcher")
            .default_width(700)
            .default_height(550)
            .build();

        // Initialize layer shell for Wayland
        window.init_layer_shell();

        // Configure layer shell
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_namespace("native-launcher");

        // Don't anchor to any edges (this centers the window)
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Don't reserve exclusive space
        window.set_exclusive_zone(-1);

        debug!("Layer shell configured");

        Self { window }
    }

    /// Show the window
    pub fn show(&self) {
        debug!("Showing window");
        self.window.present();
    }

    /// Hide the window
    pub fn hide(&self) {
        debug!("Hiding window");
        self.window.set_visible(false);
    }

    /// Toggle window visibility
    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }
}
