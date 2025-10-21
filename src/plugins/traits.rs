use anyhow::Result;
use std::fmt::Debug;

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
    pub plugin_name: String,
    /// Sub-results (e.g., workspaces under VS Code app)
    pub sub_results: Vec<PluginResult>,
    /// Parent app name (for linked entries like workspaces, recent files)
    /// When set, the UI will use this app's icon for the entry
    pub parent_app: Option<String>,
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
}

/// Context provided to plugins during search
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Maximum number of results requested
    pub max_results: usize,
    /// Whether to include low-score results
    pub include_low_scores: bool,
}

impl PluginContext {
    pub fn new(max_results: usize) -> Self {
        Self {
            max_results,
            #[allow(dead_code)]

            include_low_scores: false,
        }
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
}
