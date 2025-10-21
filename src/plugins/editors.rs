use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Recent workspace/project from code editors
#[derive(Debug, Clone)]
struct RecentWorkspace {
    /// Workspace path (folder or .code-workspace file)
    path: PathBuf,
    /// Display name
    name: String,
    /// Editor that opened it
    editor: String,
    /// Editor command to open
    command: String,
}

/// VS Code storage.json structure (partial)
#[derive(Debug, Deserialize)]
struct VSCodeStorage {
    #[serde(rename = "openedPathsList")]
    opened_paths_list: Option<OpenedPathsList>,
}

#[derive(Debug, Deserialize)]
struct OpenedPathsList {
    workspaces3: Option<Vec<String>>,
    folders2: Option<Vec<String>>,
}

/// Plugin for code editor workspace detection
#[derive(Debug)]
pub struct EditorsPlugin {
    recent_workspaces: Vec<RecentWorkspace>,
    enabled: bool,
}

impl EditorsPlugin {
    /// Create a new editors plugin
    pub fn new(enabled: bool) -> Self {
        let recent_workspaces = Self::load_recent_workspaces(50).unwrap_or_else(|e| {
            warn!("Failed to load recent workspaces: {}", e);
            Vec::new()
        });

        debug!(
            "Editors plugin initialized with {} workspaces",
            recent_workspaces.len()
        );

        Self {
            recent_workspaces,
            enabled,
        }
    }

    /// Load recent workspaces from various editors
    fn load_recent_workspaces(max_count: usize) -> Result<Vec<RecentWorkspace>> {
        let mut workspaces = Vec::new();

        // Load VS Code workspaces
        if let Ok(vscode_workspaces) = Self::load_vscode_workspaces(max_count) {
            debug!("Loaded {} VS Code workspaces", vscode_workspaces.len());
            workspaces.extend(vscode_workspaces);
        }

        // Load VSCodium workspaces
        if let Ok(codium_workspaces) = Self::load_vscodium_workspaces(max_count) {
            debug!("Loaded {} VSCodium workspaces", codium_workspaces.len());
            workspaces.extend(codium_workspaces);
        }

        // Load Sublime Text workspaces
        if let Ok(sublime_workspaces) = Self::load_sublime_workspaces(max_count) {
            debug!(
                "Loaded {} Sublime Text workspaces",
                sublime_workspaces.len()
            );
            workspaces.extend(sublime_workspaces);
        }

        // Load Zed workspaces
        if let Ok(zed_workspaces) = Self::load_zed_workspaces(max_count) {
            debug!("Loaded {} Zed workspaces", zed_workspaces.len());
            workspaces.extend(zed_workspaces);
        }

        // Sort by path and deduplicate
        workspaces.sort_by(|a, b| a.path.cmp(&b.path));
        workspaces.dedup_by(|a, b| a.path == b.path);

        // Limit to max_count
        workspaces.truncate(max_count);

        debug!(
            "Loaded {} total workspaces across all editors",
            workspaces.len()
        );
        Ok(workspaces)
    }

    /// Load recent workspaces from VS Code
    fn load_vscode_workspaces(max_count: usize) -> Result<Vec<RecentWorkspace>> {
        Self::load_vscode_like_workspaces("Code", "code", "code", max_count)
    }

    /// Load recent workspaces from VSCodium
    fn load_vscodium_workspaces(max_count: usize) -> Result<Vec<RecentWorkspace>> {
        Self::load_vscode_like_workspaces("VSCodium", "codium", "codium", max_count)
    }

    /// Load workspaces from VS Code-like editors
    fn load_vscode_like_workspaces(
        config_dir: &str,
        command: &str,
        editor_name: &str,
        max_count: usize,
    ) -> Result<Vec<RecentWorkspace>> {
        let mut workspaces = Vec::new();

        // Try the storage.json format
        let config_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join(config_dir)
            .join("User")
            .join("globalStorage")
            .join("storage.json");

        if config_path.exists() {
            let content = match fs::read_to_string(&config_path) {
                Ok(c) => c,
                Err(_) => return Ok(workspaces),
            };

            let storage = match serde_json::from_str::<VSCodeStorage>(&content) {
                Ok(s) => s,
                Err(_) => return Ok(workspaces),
            };

            let Some(opened_paths) = storage.opened_paths_list else {
                return Ok(workspaces);
            };

            // Process workspace files (.code-workspace)
            if let Some(workspace_paths) = opened_paths.workspaces3 {
                for workspace_uri in workspace_paths.iter().take(max_count) {
                    let Some(path) = Self::parse_vscode_uri(workspace_uri) else {
                        continue;
                    };

                    if !path.exists() {
                        continue;
                    }

                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    workspaces.push(RecentWorkspace {
                        path: path.clone(),
                        name,
                        editor: editor_name.to_string(),
                        command: format!("{} '{}'", command, path.display()),
                    });
                }
            }

            // Process regular folders
            if let Some(folder_paths) = opened_paths.folders2 {
                for folder_uri in folder_paths.iter().take(max_count) {
                    let Some(path) = Self::parse_vscode_uri(folder_uri) else {
                        continue;
                    };

                    if !path.exists() {
                        continue;
                    }

                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    workspaces.push(RecentWorkspace {
                        path: path.clone(),
                        name,
                        editor: editor_name.to_string(),
                        command: format!("{} '{}'", command, path.display()),
                    });
                }
            }
        }

        // Also scan workspaceStorage directories
        let workspace_storage_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join(config_dir)
            .join("User")
            .join("workspaceStorage");

        if !workspace_storage_dir.exists() {
            return Ok(workspaces);
        }

        let entries = match fs::read_dir(&workspace_storage_dir) {
            Ok(e) => e,
            Err(_) => return Ok(workspaces),
        };

        for entry in entries.flatten().take(max_count - workspaces.len()) {
            let workspace_json = entry.path().join("workspace.json");

            if !workspace_json.exists() {
                continue;
            }

            let content = match fs::read_to_string(&workspace_json) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let json = match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(j) => j,
                Err(_) => continue,
            };

            let Some(folder_uri) = json.get("folder").and_then(|v| v.as_str()) else {
                continue;
            };

            let Some(path) = Self::parse_vscode_uri(folder_uri) else {
                continue;
            };

            if !path.exists() || workspaces.iter().any(|w| w.path == path) {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            workspaces.push(RecentWorkspace {
                path: path.clone(),
                name,
                editor: editor_name.to_string(),
                command: format!("{} '{}'", command, path.display()),
            });
        }

        Ok(workspaces)
    }

    /// Load Sublime Text workspaces
    fn load_sublime_workspaces(max_count: usize) -> Result<Vec<RecentWorkspace>> {
        let mut workspaces = Vec::new();

        // Sublime Text stores recent workspaces in Session.sublime_session
        let config_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("sublime-text")
            .join("Local")
            .join("Session.sublime_session");

        if !config_path.exists() {
            return Ok(workspaces);
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Ok(workspaces),
        };

        let json = match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(j) => j,
            Err(_) => return Ok(workspaces),
        };

        let Some(workspaces_arr) = json
            .get("workspaces")
            .and_then(|w| w.get("recent_workspaces"))
            .and_then(|r| r.as_array())
        else {
            return Ok(workspaces);
        };

        for workspace_path in workspaces_arr.iter().take(max_count) {
            let Some(path_str) = workspace_path.as_str() else {
                continue;
            };

            let path = PathBuf::from(path_str);
            if !path.exists() {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            workspaces.push(RecentWorkspace {
                path: path.clone(),
                name,
                editor: "subl".to_string(),
                command: format!("subl '{}'", path.display()),
            });
        }

        Ok(workspaces)
    }

    /// Load Zed workspaces
    fn load_zed_workspaces(max_count: usize) -> Result<Vec<RecentWorkspace>> {
        let mut workspaces = Vec::new();

        // Zed stores recent workspaces in workspace db
        let config_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("zed")
            .join("workspace")
            .join("workspace-db.json");

        if !config_path.exists() {
            return Ok(workspaces);
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Ok(workspaces),
        };

        let json = match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(j) => j,
            Err(_) => return Ok(workspaces),
        };

        let Some(workspaces_obj) = json.as_object() else {
            return Ok(workspaces);
        };

        for (path_str, _) in workspaces_obj.iter().take(max_count) {
            let path = PathBuf::from(path_str);
            if !path.exists() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            workspaces.push(RecentWorkspace {
                path: path.clone(),
                name,
                editor: "zed".to_string(),
                command: format!("zed '{}'", path.display()),
            });
        }

        Ok(workspaces)
    }

    /// Parse VS Code URI (file://path or just path)
    fn parse_vscode_uri(uri: &str) -> Option<PathBuf> {
        let decoded = urlencoding::decode(uri).ok()?;
        let path_str = if decoded.starts_with("file://") {
            &decoded[7..]
        } else {
            &decoded
        };

        Some(PathBuf::from(path_str.to_string()))
    }
}

impl Default for EditorsPlugin {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Plugin for EditorsPlugin {
    fn name(&self) -> &str {
        "editors"
    }

    fn description(&self) -> &str {
        "Code editor workspaces (VS Code, VSCodium, Sublime, Zed)"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@code", "@zed", "@editor"]
    }

    fn priority(&self) -> i32 {
        700 // Higher priority than files
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn should_handle(&self, query: &str) -> bool {
        if !self.enabled || query.is_empty() {
            return false;
        }

        // Handle explicit commands or participate in global search for short queries
        query.starts_with('@') || query.len() >= 2
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Extract search term
        let search_term = if query.starts_with('@') {
            query_lower
                .strip_prefix("@code")
                .or_else(|| query_lower.strip_prefix("@zed"))
                .or_else(|| query_lower.strip_prefix("@editor"))
                .unwrap_or(&query_lower)
                .trim()
        } else {
            query_lower.trim()
        };

        for workspace in &self.recent_workspaces {
            // Filter by search term
            if !search_term.is_empty()
                && !workspace.name.to_lowercase().contains(search_term)
                && !workspace
                    .path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(search_term)
            {
                continue;
            }

            let subtitle = Some(format!(
                "{} - {}",
                workspace.editor,
                workspace.path.display()
            ));

            results.push(PluginResult {
                title: workspace.name.clone(),
                subtitle,
                icon: None, // Icon will be resolved from parent_app
                command: workspace.command.clone(),
                terminal: false,
                score: 800, // High score for workspaces
                plugin_name: self.name().to_string(),
                sub_results: Vec::new(),
                parent_app: Some(workspace.editor.clone()),
            });

            if results.len() >= context.max_results {
                break;
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.cmp(&a.score));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vscode_uri() {
        let uri = "file:///home/user/project";
        let path = EditorsPlugin::parse_vscode_uri(uri);
        assert_eq!(path, Some(PathBuf::from("/home/user/project")));

        let uri = "/home/user/project";
        let path = EditorsPlugin::parse_vscode_uri(uri);
        assert_eq!(path, Some(PathBuf::from("/home/user/project")));

        let uri = "file:///home/user/my%20project";
        let path = EditorsPlugin::parse_vscode_uri(uri);
        assert_eq!(path, Some(PathBuf::from("/home/user/my project")));
    }

    #[test]
    fn test_should_handle() {
        let plugin = EditorsPlugin::new(true);

        assert!(plugin.should_handle("@workspace test"));
        assert!(plugin.should_handle("@code"));
        assert!(plugin.should_handle("project"));
        assert!(plugin.should_handle("co")); // 2 chars

        assert!(!plugin.should_handle("a")); // Too short
        assert!(!plugin.should_handle(""));

        let disabled = EditorsPlugin::new(false);
        assert!(!disabled.should_handle("test"));
    }
}
