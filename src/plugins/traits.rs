use crate::config::Config;
use anyhow::Result;
use gtk4::gdk::{Key, ModifierType};
use std::fmt::Debug;

/// Keyboard event passed to plugins
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// The key that was pressed
    pub key: Key,
    /// Active modifiers (Ctrl, Shift, Alt, etc.)
    pub modifiers: ModifierType,
    /// Current search query text
    pub query: String,
    /// Whether there's a selected result
    pub has_selection: bool,
}

impl KeyboardEvent {
    pub fn new(key: Key, modifiers: ModifierType, query: String, has_selection: bool) -> Self {
        Self {
            key,
            modifiers,
            query,
            has_selection,
        }
    }

    /// Check if Ctrl modifier is pressed
    pub fn has_ctrl(&self) -> bool {
        self.modifiers.contains(ModifierType::CONTROL_MASK)
    }

    /// Check if Shift modifier is pressed
    #[allow(dead_code)] // Part of public API for plugins
    pub fn has_shift(&self) -> bool {
        self.modifiers.contains(ModifierType::SHIFT_MASK)
    }

    /// Check if Alt modifier is pressed
    #[allow(dead_code)] // Part of public API for plugins
    pub fn has_alt(&self) -> bool {
        self.modifiers.contains(ModifierType::ALT_MASK)
    }

    /// Check if Super/Meta modifier is pressed
    #[allow(dead_code)] // Part of public API for plugins
    pub fn has_super(&self) -> bool {
        self.modifiers.contains(ModifierType::SUPER_MASK)
            || self.modifiers.contains(ModifierType::META_MASK)
    }
}

/// Action that a plugin can take in response to a keyboard event
#[derive(Debug, Clone)]
#[allow(dead_code)] // All variants are part of public API - some used by web search plugin
pub enum KeyboardAction {
    /// Plugin didn't handle this event, pass to next plugin
    None,
    /// Execute command and close window
    Execute { command: String, terminal: bool },
    /// Open URL in default browser and close window
    OpenUrl(String),
    /// Event was handled but don't close window
    Handled,
    /// Open containing folder (for files)
    OpenFolder(String),
    /// Copy path to clipboard
    CopyPath(String),
}

/// Represents a result from a plugin search
#[derive(Debug, Clone)]
pub struct PluginResult {
    /// Display title
    pub title: String,
    /// Optional subtitle/description
    pub subtitle: Option<String>,
    /// Icon name or path
    pub icon: Option<String>,
    /// Command to execute when selected
    pub command: String,
    /// Whether to run in terminal
    pub terminal: bool,
    /// Search score (higher = better match)
    pub score: i64,
    /// Plugin that generated this result
    #[allow(dead_code)] // Used in tests
    pub plugin_name: String,
    /// Sub-results (e.g., workspaces under VS Code app)
    pub sub_results: Vec<PluginResult>,
    /// Parent app name (for linked entries like workspaces, recent files)
    /// When set, the UI will use this app's icon for the entry
    pub parent_app: Option<String>,
    /// Desktop file path if this result corresponds to an application entry
    pub desktop_path: Option<String>,
    /// Optional badge icon name (e.g., "terminal-symbolic", "folder-symbolic", "web-browser-symbolic")
    /// Uses GTK symbolic icon names for small overlay indicators
    pub badge_icon: Option<String>,
}

impl PluginResult {
    /// Create a new plugin result
    pub fn new(title: String, command: String, plugin_name: String) -> Self {
        Self {
            title,
            subtitle: None,
            icon: None,
            command,
            terminal: false,
            score: 0,
            plugin_name,
            #[allow(dead_code)]
            sub_results: Vec::new(),
            parent_app: None,
            desktop_path: None,
            badge_icon: None,
        }
    }

    /// Set subtitle
    pub fn with_subtitle(mut self, subtitle: String) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set terminal flag
    pub fn with_terminal(mut self, terminal: bool) -> Self {
        self.terminal = terminal;
        self
    }

    /// Set score
    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    /// Set sub-results (e.g., workspaces under an app)
    #[allow(dead_code)]

    pub fn with_sub_results(mut self, sub_results: Vec<PluginResult>) -> Self {
        self.sub_results = sub_results;
        self
    }

    /// Add a single sub-result
    #[allow(dead_code)]

    pub fn add_sub_result(mut self, sub_result: PluginResult) -> Self {
        self.sub_results.push(sub_result);
        self
    }

    /// Set parent app (for linked entries like workspaces, recent files)
    /// This will cause the UI to use the parent app's icon
    #[allow(dead_code)]

    pub fn with_parent_app(mut self, parent_app: String) -> Self {
        self.parent_app = Some(parent_app);
        self
    }

    /// Set desktop path for application results
    #[allow(dead_code)]
    pub fn with_desktop_path(mut self, path: String) -> Self {
        self.desktop_path = Some(path);
        self
    }

    /// Set badge icon (symbolic icon name for small indicator)
    /// Common badges: "terminal-symbolic", "folder-symbolic", "web-browser-symbolic",
    /// "document-symbolic", "video-symbolic", "audio-symbolic"
    pub fn with_badge_icon(mut self, badge: String) -> Self {
        self.badge_icon = Some(badge);
        self
    }
}

/// Context provided to plugins during search
#[derive(Debug, Clone)]
/// Context passed to plugins during search
/// Provides access to search parameters and read-only configuration
pub struct PluginContext<'a> {
    /// Maximum number of results requested
    pub max_results: usize,
    /// Whether to include low-score results
    pub include_low_scores: bool,
    /// Number of high-quality app results found so far (for smart triggering)
    pub app_results_count: usize,
    /// Read-only access to application configuration
    #[allow(dead_code)] // Available for plugins to access config
    pub config: &'a Config,
}

impl<'a> PluginContext<'a> {
    pub fn new(max_results: usize, config: &'a Config) -> Self {
        Self {
            max_results,
            include_low_scores: false,
            app_results_count: 0,
            config,
        }
    }

    /// Create context with app results count
    pub fn with_app_results(mut self, count: usize) -> Self {
        self.app_results_count = count;
        self
    }
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Debug + Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin description
    #[allow(dead_code)]

    fn description(&self) -> &str;

    /// Get command prefixes for this plugin (e.g., ["@wp", "@workspace"])
    /// Return empty vec if plugin doesn't use command prefixes
    fn command_prefixes(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Check if this plugin should handle the given query
    /// Return true if the plugin can provide results for this query
    fn should_handle(&self, query: &str) -> bool;

    /// Search for results matching the query
    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>>;

    /// Get plugin priority (higher = searched first)
    /// Default: 100 (Applications: 1000, Calculator: 500, etc.)
    fn priority(&self) -> i32 {
        100
    }

    /// Whether this plugin is enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Handle keyboard events
    /// Return KeyboardAction::None if this plugin doesn't handle the event
    /// Events are dispatched to plugins in priority order (highest first)
    /// First plugin to return non-None action wins
    fn handle_keyboard_event(&self, _event: &KeyboardEvent) -> KeyboardAction {
        KeyboardAction::None
    }
}
