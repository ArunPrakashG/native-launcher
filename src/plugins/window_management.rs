use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;
use std::process::Command;
use tracing::{debug, warn};

/// Window management plugin for Hyprland/Sway compositors
/// Provides quick window management actions like move to workspace, center, etc.
#[derive(Debug)]
pub struct WindowManagementPlugin {
    compositor: Option<Compositor>,
    enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Compositor {
    Hyprland,
    Sway,
}

impl Compositor {
    fn command(&self) -> &'static str {
        match self {
            Compositor::Hyprland => "hyprctl",
            Compositor::Sway => "swaymsg",
        }
    }

    fn dispatch_prefix(&self) -> &'static str {
        match self {
            Compositor::Hyprland => "hyprctl dispatch",
            Compositor::Sway => "swaymsg",
        }
    }
}

#[derive(Debug, Clone)]
struct WindowAction {
    title: String,
    subtitle: String,
    command: String,
    keywords: Vec<&'static str>,
}

impl WindowManagementPlugin {
    pub fn new() -> Self {
        let compositor = Self::detect_compositor();

        if let Some(comp) = compositor {
            debug!("window management plugin detected compositor: {:?}", comp);
        } else {
            debug!("window management plugin: no supported compositor detected");
        }

        Self {
            compositor,
            enabled: true,
        }
    }

    /// Detect which compositor is running
    fn detect_compositor() -> Option<Compositor> {
        // Check for Hyprland first
        if Self::command_exists("hyprctl") {
            return Some(Compositor::Hyprland);
        }

        // Check for Sway
        if Self::command_exists("swaymsg") {
            return Some(Compositor::Sway);
        }

        None
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get available window actions for the detected compositor
    fn get_actions(&self, compositor: Compositor) -> Vec<WindowAction> {
        let prefix = compositor.dispatch_prefix();

        match compositor {
            Compositor::Hyprland => vec![
                WindowAction {
                    title: "Move Window to Workspace 1".to_string(),
                    subtitle: "Move active window to workspace 1".to_string(),
                    command: format!("{} movetoworkspace 1", prefix),
                    keywords: vec!["move", "workspace", "1"],
                },
                WindowAction {
                    title: "Move Window to Workspace 2".to_string(),
                    subtitle: "Move active window to workspace 2".to_string(),
                    command: format!("{} movetoworkspace 2", prefix),
                    keywords: vec!["move", "workspace", "2"],
                },
                WindowAction {
                    title: "Move Window to Workspace 3".to_string(),
                    subtitle: "Move active window to workspace 3".to_string(),
                    command: format!("{} movetoworkspace 3", prefix),
                    keywords: vec!["move", "workspace", "3"],
                },
                WindowAction {
                    title: "Move Window to Workspace 4".to_string(),
                    subtitle: "Move active window to workspace 4".to_string(),
                    command: format!("{} movetoworkspace 4", prefix),
                    keywords: vec!["move", "workspace", "4"],
                },
                WindowAction {
                    title: "Move Window to Workspace 5".to_string(),
                    subtitle: "Move active window to workspace 5".to_string(),
                    command: format!("{} movetoworkspace 5", prefix),
                    keywords: vec!["move", "workspace", "5"],
                },
                WindowAction {
                    title: "Center Window".to_string(),
                    subtitle: "Center the active window".to_string(),
                    command: format!("{} centerwindow", prefix),
                    keywords: vec!["center", "window"],
                },
                WindowAction {
                    title: "Toggle Fullscreen".to_string(),
                    subtitle: "Toggle fullscreen for active window".to_string(),
                    command: format!("{} fullscreen 0", prefix),
                    keywords: vec!["fullscreen", "full", "toggle"],
                },
                WindowAction {
                    title: "Toggle Floating".to_string(),
                    subtitle: "Toggle floating mode for active window".to_string(),
                    command: format!("{} togglefloating", prefix),
                    keywords: vec!["float", "floating", "toggle"],
                },
                WindowAction {
                    title: "Pin Window".to_string(),
                    subtitle: "Pin window to all workspaces".to_string(),
                    command: format!("{} pin active", prefix),
                    keywords: vec!["pin", "sticky", "all"],
                },
                WindowAction {
                    title: "Close Window".to_string(),
                    subtitle: "Close the active window".to_string(),
                    command: format!("{} killactive", prefix),
                    keywords: vec!["close", "kill", "quit"],
                },
                WindowAction {
                    title: "Move Window Left".to_string(),
                    subtitle: "Move focus and window left".to_string(),
                    command: format!("{} movewindow l", prefix),
                    keywords: vec!["move", "left"],
                },
                WindowAction {
                    title: "Move Window Right".to_string(),
                    subtitle: "Move focus and window right".to_string(),
                    command: format!("{} movewindow r", prefix),
                    keywords: vec!["move", "right"],
                },
                WindowAction {
                    title: "Move Window Up".to_string(),
                    subtitle: "Move focus and window up".to_string(),
                    command: format!("{} movewindow u", prefix),
                    keywords: vec!["move", "up"],
                },
                WindowAction {
                    title: "Move Window Down".to_string(),
                    subtitle: "Move focus and window down".to_string(),
                    command: format!("{} movewindow d", prefix),
                    keywords: vec!["move", "down"],
                },
            ],
            Compositor::Sway => vec![
                WindowAction {
                    title: "Move Window to Workspace 1".to_string(),
                    subtitle: "Move active window to workspace 1".to_string(),
                    command: format!("{} move container to workspace 1", prefix),
                    keywords: vec!["move", "workspace", "1"],
                },
                WindowAction {
                    title: "Move Window to Workspace 2".to_string(),
                    subtitle: "Move active window to workspace 2".to_string(),
                    command: format!("{} move container to workspace 2", prefix),
                    keywords: vec!["move", "workspace", "2"],
                },
                WindowAction {
                    title: "Move Window to Workspace 3".to_string(),
                    subtitle: "Move active window to workspace 3".to_string(),
                    command: format!("{} move container to workspace 3", prefix),
                    keywords: vec!["move", "workspace", "3"],
                },
                WindowAction {
                    title: "Move Window to Workspace 4".to_string(),
                    subtitle: "Move active window to workspace 4".to_string(),
                    command: format!("{} move container to workspace 4", prefix),
                    keywords: vec!["move", "workspace", "4"],
                },
                WindowAction {
                    title: "Move Window to Workspace 5".to_string(),
                    subtitle: "Move active window to workspace 5".to_string(),
                    command: format!("{} move container to workspace 5", prefix),
                    keywords: vec!["move", "workspace", "5"],
                },
                WindowAction {
                    title: "Toggle Fullscreen".to_string(),
                    subtitle: "Toggle fullscreen for active window".to_string(),
                    command: format!("{} fullscreen toggle", prefix),
                    keywords: vec!["fullscreen", "full", "toggle"],
                },
                WindowAction {
                    title: "Toggle Floating".to_string(),
                    subtitle: "Toggle floating mode for active window".to_string(),
                    command: format!("{} floating toggle", prefix),
                    keywords: vec!["float", "floating", "toggle"],
                },
                WindowAction {
                    title: "Toggle Sticky".to_string(),
                    subtitle: "Pin window to all workspaces".to_string(),
                    command: format!("{} sticky toggle", prefix),
                    keywords: vec!["pin", "sticky", "all"],
                },
                WindowAction {
                    title: "Close Window".to_string(),
                    subtitle: "Close the active window".to_string(),
                    command: format!("{} kill", prefix),
                    keywords: vec!["close", "kill", "quit"],
                },
                WindowAction {
                    title: "Move Window Left".to_string(),
                    subtitle: "Move focus and window left".to_string(),
                    command: format!("{} move left", prefix),
                    keywords: vec!["move", "left"],
                },
                WindowAction {
                    title: "Move Window Right".to_string(),
                    subtitle: "Move focus and window right".to_string(),
                    command: format!("{} move right", prefix),
                    keywords: vec!["move", "right"],
                },
                WindowAction {
                    title: "Move Window Up".to_string(),
                    subtitle: "Move focus and window up".to_string(),
                    command: format!("{} move up", prefix),
                    keywords: vec!["move", "up"],
                },
                WindowAction {
                    title: "Move Window Down".to_string(),
                    subtitle: "Move focus and window down".to_string(),
                    command: format!("{} move down", prefix),
                    keywords: vec!["move", "down"],
                },
            ],
        }
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@wm") {
            rest
        } else if let Some(rest) = query.strip_prefix("@window") {
            rest
        } else {
            query
        }
    }
}

impl Plugin for WindowManagementPlugin {
    fn name(&self) -> &str {
        "window-management"
    }

    fn search(&self, query: &str, ctx: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.should_handle(query) {
            return Ok(Vec::new());
        }

        // If no compositor detected, return help message
        let compositor = match self.compositor {
            Some(comp) => comp,
            None => {
                let result = PluginResult::new(
                    "Window Management Unavailable".to_string(),
                    "echo 'No supported compositor detected'".to_string(),
                    self.name().to_string(),
                )
                .with_subtitle("Requires Hyprland or Sway compositor".to_string())
                .with_score(9000);

                return Ok(vec![result]);
            }
        };

        let filter = self.strip_prefix(query).trim().to_lowercase();
        let actions = self.get_actions(compositor);
        let mut results = Vec::new();

        for (idx, action) in actions.iter().enumerate() {
            if results.len() >= ctx.max_results {
                break;
            }

            // Filter by query
            if !filter.is_empty() {
                let title_lower = action.title.to_lowercase();
                let matches = title_lower.contains(&filter)
                    || action.keywords.iter().any(|k| k.contains(&filter));

                if !matches {
                    continue;
                }
            }

            // Score: filtered results higher, then by order
            let filter_bonus = if !filter.is_empty() { 2000 } else { 0 };
            let score = 8500 + filter_bonus - (idx as i64 * 10);

            let result = PluginResult::new(
                action.title.clone(),
                action.command.clone(),
                self.name().to_string(),
            )
            .with_subtitle(action.subtitle.clone())
            .with_score(score);

            results.push(result);
        }

        Ok(results)
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@wm") || query.starts_with("@window")
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn priority(&self) -> i32 {
        75
    }

    fn description(&self) -> &str {
        "Window management shortcuts for Hyprland/Sway via @wm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_should_handle() {
        let plugin = WindowManagementPlugin::new();
        assert!(plugin.should_handle("@wm"));
        assert!(plugin.should_handle("@wm move"));
        assert!(plugin.should_handle("@window"));
        assert!(plugin.should_handle("@window center"));
        assert!(!plugin.should_handle("window"));
        assert!(!plugin.should_handle("@w"));
    }

    #[test]
    fn test_strip_prefix() {
        let plugin = WindowManagementPlugin::new();
        assert_eq!(plugin.strip_prefix("@wm test"), " test");
        assert_eq!(plugin.strip_prefix("@window test"), " test");
        assert_eq!(plugin.strip_prefix("@wm"), "");
    }

    #[test]
    fn test_compositor_detection() {
        // Just test that detection doesn't crash
        let comp = WindowManagementPlugin::detect_compositor();
        // Will be Some on Hyprland/Sway systems, None otherwise
        assert!(comp.is_some() || comp.is_none());
    }

    #[test]
    fn test_get_actions_hyprland() {
        let plugin = WindowManagementPlugin::new();
        let actions = plugin.get_actions(Compositor::Hyprland);

        assert!(!actions.is_empty());
        assert!(actions.len() >= 10);

        // Check some key actions exist
        assert!(actions.iter().any(|a| a.title.contains("Workspace")));
        assert!(actions.iter().any(|a| a.title.contains("Center")));
        assert!(actions.iter().any(|a| a.title.contains("Fullscreen")));
    }

    #[test]
    fn test_get_actions_sway() {
        let plugin = WindowManagementPlugin::new();
        let actions = plugin.get_actions(Compositor::Sway);

        assert!(!actions.is_empty());
        assert!(actions.len() >= 10);

        // Check some key actions exist
        assert!(actions.iter().any(|a| a.title.contains("Workspace")));
        assert!(actions.iter().any(|a| a.title.contains("Fullscreen")));
    }

    #[test]
    fn test_search_filters_by_query() {
        let plugin = WindowManagementPlugin::new();
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@wm move", &ctx).unwrap();

        // Should return filtered results (or message if no compositor)
        if plugin.compositor.is_some() {
            assert!(!results.is_empty());
            // All results should contain "move" in title or be move actions
            for result in &results {
                let title_lower = result.title.to_lowercase();
                assert!(title_lower.contains("move"));
            }
        }
    }

    #[test]
    fn test_search_workspace_filter() {
        let plugin = WindowManagementPlugin::new();
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@wm workspace 2", &ctx).unwrap();

        if plugin.compositor.is_some() {
            assert!(!results.is_empty());
            // Should prioritize workspace 2 actions
            if !results.is_empty() {
                assert!(results[0].title.contains("Workspace"));
            }
        }
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = WindowManagementPlugin::new();
        assert_eq!(plugin.priority(), 75);
    }

    #[test]
    fn test_plugin_enabled() {
        let plugin = WindowManagementPlugin::new();
        assert!(plugin.enabled());
    }

    #[test]
    fn test_hyprland_commands_format() {
        let plugin = WindowManagementPlugin::new();
        let actions = plugin.get_actions(Compositor::Hyprland);

        for action in &actions {
            assert!(action.command.starts_with("hyprctl dispatch"));
        }
    }

    #[test]
    fn test_sway_commands_format() {
        let plugin = WindowManagementPlugin::new();
        let actions = plugin.get_actions(Compositor::Sway);

        for action in &actions {
            assert!(action.command.starts_with("swaymsg"));
        }
    }
}
