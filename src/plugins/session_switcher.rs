use crate::plugins::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Session/workspace item that can be switched to
#[derive(Debug, Clone)]
pub struct SessionItem {
    /// Display name
    pub name: String,
    /// Subtitle (window class, workspace path, etc.)
    pub subtitle: String,
    /// Command to execute to switch to this session
    pub command: String,
    /// Icon name
    pub icon: String,
    /// Session type for categorization
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Running window/application
    Window,
    /// VS Code workspace
    VSCodeWorkspace,
    /// Terminal session
    Terminal,
    /// Browser tab/window
    BrowserWindow,
}

impl SessionType {
    fn icon(&self) -> &'static str {
        match self {
            SessionType::Window => "preferences-system-windows",
            SessionType::VSCodeWorkspace => "code",
            SessionType::Terminal => "utilities-terminal",
            SessionType::BrowserWindow => "web-browser",
        }
    }

    fn category(&self) -> &'static str {
        match self {
            SessionType::Window => "Window",
            SessionType::VSCodeWorkspace => "VS Code",
            SessionType::Terminal => "Terminal",
            SessionType::BrowserWindow => "Browser",
        }
    }
}

/// Cache for session data with TTL
#[derive(Debug)]
struct SessionCache {
    items: Vec<SessionItem>,
    cached_at: Instant,
    ttl: Duration,
}

impl SessionCache {
    fn new(ttl: Duration) -> Self {
        Self {
            items: Vec::new(),
            cached_at: Instant::now()
                .checked_sub(ttl * 2)
                .unwrap_or_else(Instant::now),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    fn update(&mut self, items: Vec<SessionItem>) {
        self.items = items;
        self.cached_at = Instant::now();
    }

    fn get(&self) -> &[SessionItem] {
        &self.items
    }
}

#[derive(Debug)]
pub struct SessionSwitcherPlugin {
    enabled: bool,
    cache: OnceLock<std::sync::Mutex<SessionCache>>,
    cache_ttl: Duration,
    compositor: Option<CompositorType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompositorType {
    Hyprland,
    Sway,
    Unknown,
}

impl SessionSwitcherPlugin {
    pub fn new(enabled: bool) -> Self {
        let compositor = Self::detect_compositor();
        debug!("Session switcher detected compositor: {:?}", compositor);

        Self {
            enabled,
            cache: OnceLock::new(),
            cache_ttl: Duration::from_secs(3), // 3 second cache
            compositor,
        }
    }

    fn detect_compositor() -> Option<CompositorType> {
        // Check for Hyprland
        if Command::new("which")
            .arg("hyprctl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(CompositorType::Hyprland);
        }

        // Check for Sway
        if Command::new("which")
            .arg("swaymsg")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(CompositorType::Sway);
        }

        None
    }

    fn get_or_refresh_sessions(&self) -> Vec<SessionItem> {
        let cache = self
            .cache
            .get_or_init(|| std::sync::Mutex::new(SessionCache::new(self.cache_ttl)));

        let mut cache = cache.lock().unwrap();

        if cache.is_expired() {
            debug!("Session cache expired, refreshing...");
            let mut items = Vec::new();

            // Collect windows from compositor
            if let Some(windows) = self.get_windows() {
                items.extend(windows);
            }

            // Collect VS Code workspaces
            if let Some(workspaces) = self.get_vscode_workspaces() {
                items.extend(workspaces);
            }

            cache.update(items);
        }

        cache.get().to_vec()
    }

    fn get_windows(&self) -> Option<Vec<SessionItem>> {
        match self.compositor? {
            CompositorType::Hyprland => self.get_hyprland_windows(),
            CompositorType::Sway => self.get_sway_windows(),
            CompositorType::Unknown => None,
        }
    }

    fn get_hyprland_windows(&self) -> Option<Vec<SessionItem>> {
        let output = Command::new("hyprctl")
            .args(["clients", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // Simple JSON parsing for window list
        // Format: [{"address":"0x...","title":"...","class":"...","workspace":{"id":1}}, ...]
        let mut items = Vec::new();

        for line in json_str.lines() {
            if let Some(title) = Self::extract_json_field(line, "title") {
                let class = Self::extract_json_field(line, "class")
                    .unwrap_or_else(|| "Unknown".to_string());
                let address =
                    Self::extract_json_field(line, "address").unwrap_or_else(|| "0x0".to_string());

                items.push(SessionItem {
                    name: title.clone(),
                    subtitle: format!("{} • Hyprland", class),
                    command: format!("hyprctl dispatch focuswindow address:{}", address),
                    icon: Self::get_icon_for_class(&class),
                    session_type: SessionType::Window,
                });
            }
        }

        Some(items)
    }

    fn get_sway_windows(&self) -> Option<Vec<SessionItem>> {
        let output = Command::new("swaymsg")
            .args(["-t", "get_tree"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        // Parse sway tree for windows
        for line in json_str.lines() {
            if line.contains("\"type\": \"con\"") || line.contains("\"type\":\"con\"") {
                if let Some(name) = Self::extract_json_field(line, "name") {
                    let app_id = Self::extract_json_field(line, "app_id")
                        .or_else(|| {
                            Self::extract_json_field(line, "window_properties")
                                .and_then(|_| Self::extract_json_field(line, "class"))
                        })
                        .unwrap_or_else(|| "Unknown".to_string());

                    if let Some(id) = Self::extract_json_field(line, "id") {
                        items.push(SessionItem {
                            name: name.clone(),
                            subtitle: format!("{} • Sway", app_id),
                            command: format!("swaymsg '[con_id={}] focus'", id),
                            icon: Self::get_icon_for_class(&app_id),
                            session_type: SessionType::Window,
                        });
                    }
                }
            }
        }

        Some(items)
    }

    fn get_vscode_workspaces(&self) -> Option<Vec<SessionItem>> {
        // Check common VS Code workspace locations
        let home = std::env::var("HOME").ok()?;
        let vscode_storage = format!("{}/.config/Code/User/workspaceStorage", home);

        let entries = std::fs::read_dir(&vscode_storage).ok()?;
        let mut items = Vec::new();

        for entry in entries.flatten() {
            let workspace_json = entry.path().join("workspace.json");
            if workspace_json.exists() {
                if let Ok(content) = std::fs::read_to_string(&workspace_json) {
                    if let Some(folder) = Self::extract_json_field(&content, "folder") {
                        // Extract workspace name from path
                        let name = folder.split('/').last().unwrap_or(&folder).to_string();

                        items.push(SessionItem {
                            name: name.clone(),
                            subtitle: format!("{} • VS Code Workspace", folder),
                            command: format!("code '{}'", folder),
                            icon: "code".to_string(),
                            session_type: SessionType::VSCodeWorkspace,
                        });
                    }
                }
            }
        }

        Some(items)
    }

    /// Simple JSON field extraction (no serde dependency)
    fn extract_json_field(json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":", field);
        let start = json.find(&pattern)?;
        let value_start = json[start + pattern.len()..].trim_start();

        if value_start.starts_with('"') {
            // String value
            let end = value_start[1..].find('"')?;
            Some(value_start[1..end + 1].to_string())
        } else {
            // Number value
            let end = value_start.find([',', '}', ']'].as_ref())?;
            Some(value_start[..end].trim().to_string())
        }
    }

    fn get_icon_for_class(class: &str) -> String {
        match class.to_lowercase().as_str() {
            s if s.contains("firefox") => "firefox",
            s if s.contains("chrome") => "google-chrome",
            s if s.contains("code") || s.contains("vscode") => "code",
            s if s.contains("terminal") || s.contains("alacritty") || s.contains("kitty") => {
                "utilities-terminal"
            }
            s if s.contains("nautilus") || s.contains("thunar") => "system-file-manager",
            _ => "preferences-system-windows",
        }
        .to_string()
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@switch") || query.starts_with("@sw")
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@switch") {
            rest.trim()
        } else if let Some(rest) = query.strip_prefix("@sw") {
            rest.trim()
        } else {
            query
        }
    }
}

impl Plugin for SessionSwitcherPlugin {
    fn name(&self) -> &str {
        "Session Switcher"
    }

    fn description(&self) -> &str {
        "Switch between open windows and VS Code workspaces"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn priority(&self) -> i32 {
        50 // Medium priority
    }

    fn should_handle(&self, query: &str) -> bool {
        self.should_handle(query)
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        if !self.should_handle(query) {
            return Ok(Vec::new());
        }

        // Check if compositor is available
        if self.compositor.is_none() {
            return Ok(vec![PluginResult {
                title: "Session Switcher Unavailable".to_string(),
                subtitle: Some(
                    "No supported compositor detected (Hyprland/Sway required)".to_string(),
                ),
                command: String::new(),
                icon: Some("dialog-warning".to_string()),
                terminal: false,
                score: 1000,
                plugin_name: self.name().to_string(),
                sub_results: vec![],
                parent_app: None,
                desktop_path: None,
                badge_icon: None,
            }]);
        }

        let filter = self.strip_prefix(query);
        let sessions = self.get_or_refresh_sessions();

        debug!(
            "Session switcher: found {} sessions, filter: '{}'",
            sessions.len(),
            filter
        );

        let filter_lower = filter.to_lowercase();
        let mut results: Vec<PluginResult> = sessions
            .into_iter()
            .filter_map(|session| {
                // Filter by query
                if !filter.is_empty() {
                    let name_lower = session.name.to_lowercase();
                    let subtitle_lower = session.subtitle.to_lowercase();

                    if !name_lower.contains(&filter_lower)
                        && !subtitle_lower.contains(&filter_lower)
                    {
                        return None;
                    }
                }

                // Calculate score based on match quality
                let mut score = 500; // Base score

                if !filter.is_empty() {
                    let name_lower = session.name.to_lowercase();

                    // Exact match
                    if name_lower == filter_lower {
                        score += 3000;
                    }
                    // Starts with
                    else if name_lower.starts_with(&filter_lower) {
                        score += 2000;
                    }
                    // Contains
                    else if name_lower.contains(&filter_lower) {
                        score += 1000;
                    }
                }

                Some(PluginResult {
                    title: session.name,
                    subtitle: Some(session.subtitle),
                    command: session.command,
                    icon: Some(session.icon),
                    terminal: false,
                    score,
                    plugin_name: self.name().to_string(),
                    sub_results: vec![],
                    parent_app: None,
                    desktop_path: None,
                    badge_icon: None, // No badge for sessions
                })
            })
            .take(context.max_results)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_handle() {
        let plugin = SessionSwitcherPlugin::new(true);

        assert!(plugin.should_handle("@switch"));
        assert!(plugin.should_handle("@switch foo"));
        assert!(plugin.should_handle("@sw"));
        assert!(plugin.should_handle("@sw bar"));

        assert!(!plugin.should_handle("switch"));
        assert!(!plugin.should_handle("@wm"));
        assert!(!plugin.should_handle("foo"));
    }

    #[test]
    fn test_strip_prefix() {
        let plugin = SessionSwitcherPlugin::new(true);

        assert_eq!(plugin.strip_prefix("@switch foo"), "foo");
        assert_eq!(plugin.strip_prefix("@switch  bar  "), "bar");
        assert_eq!(plugin.strip_prefix("@sw baz"), "baz");
        assert_eq!(plugin.strip_prefix("no prefix"), "no prefix");
    }

    #[test]
    fn test_plugin_enabled() {
        let plugin = SessionSwitcherPlugin::new(true);
        assert!(plugin.enabled());

        let plugin = SessionSwitcherPlugin::new(false);
        assert!(!plugin.enabled());
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = SessionSwitcherPlugin::new(true);
        assert_eq!(plugin.priority(), 50);
    }

    #[test]
    fn test_extract_json_field_string() {
        let json = r#"{"title":"My Window","class":"firefox"}"#;

        assert_eq!(
            SessionSwitcherPlugin::extract_json_field(json, "title"),
            Some("My Window".to_string())
        );
        assert_eq!(
            SessionSwitcherPlugin::extract_json_field(json, "class"),
            Some("firefox".to_string())
        );
    }

    #[test]
    fn test_extract_json_field_number() {
        let json = r#"{"id":123,"workspace":{"id":1}}"#;

        assert_eq!(
            SessionSwitcherPlugin::extract_json_field(json, "id"),
            Some("123".to_string())
        );
    }

    #[test]
    fn test_get_icon_for_class() {
        assert_eq!(
            SessionSwitcherPlugin::get_icon_for_class("firefox"),
            "firefox"
        );
        assert_eq!(
            SessionSwitcherPlugin::get_icon_for_class("google-chrome"),
            "google-chrome"
        );
        assert_eq!(SessionSwitcherPlugin::get_icon_for_class("code"), "code");
        assert_eq!(
            SessionSwitcherPlugin::get_icon_for_class("alacritty"),
            "utilities-terminal"
        );
        assert_eq!(
            SessionSwitcherPlugin::get_icon_for_class("unknown-app"),
            "preferences-system-windows"
        );
    }

    #[test]
    fn test_session_cache_expiry() {
        let mut cache = SessionCache::new(Duration::from_millis(100));

        // Fresh cache should be expired (initial time set in past)
        assert!(cache.is_expired());

        // Update cache
        cache.update(vec![]);
        assert!(!cache.is_expired());

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));
        assert!(cache.is_expired());
    }

    #[test]
    fn test_search_without_compositor() {
        use crate::config::Config;

        let plugin = SessionSwitcherPlugin {
            enabled: true,
            cache: OnceLock::new(),
            cache_ttl: Duration::from_secs(3),
            compositor: None,
        };

        let config = Config::default();
        let context = PluginContext::new(10, &config);
        let results = plugin.search("@switch", &context).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Unavailable"));
    }

    #[test]
    fn test_session_type_icon() {
        assert_eq!(SessionType::Window.icon(), "preferences-system-windows");
        assert_eq!(SessionType::VSCodeWorkspace.icon(), "code");
        assert_eq!(SessionType::Terminal.icon(), "utilities-terminal");
        assert_eq!(SessionType::BrowserWindow.icon(), "web-browser");
    }

    #[test]
    fn test_session_type_category() {
        assert_eq!(SessionType::Window.category(), "Window");
        assert_eq!(SessionType::VSCodeWorkspace.category(), "VS Code");
        assert_eq!(SessionType::Terminal.category(), "Terminal");
        assert_eq!(SessionType::BrowserWindow.category(), "Browser");
    }
}
