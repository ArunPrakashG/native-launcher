/// Script-based plugin system for Native Launcher
/// Allows users to create plugins using shell scripts, Python, or other executables
/// without compiling Rust code.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents the result from a script plugin execution
#[derive(Debug, Clone)]
pub struct PluginResult {
    pub title: String,
    pub subtitle: Option<String>,
    pub command: String,
    pub icon: Option<String>,
}

/// Script plugin manifest (plugin.toml)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScriptPluginManifest {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Command triggers (what user types to activate)
    pub triggers: Vec<String>,

    /// Execution configuration
    pub execution: ExecutionConfig,

    /// Optional environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginMetadata {
    /// Plugin name (displayed in results)
    pub name: String,

    /// Short description
    pub description: String,

    /// Author name
    pub author: String,

    /// Version string
    pub version: String,

    /// Priority (higher = searched first)
    pub priority: u32,

    /// Optional icon path
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionConfig {
    /// Script/executable path (relative to plugin directory or absolute)
    pub script: String,

    /// Interpreter (e.g., "bash", "python3", "node")
    #[serde(default)]
    pub interpreter: Option<String>,

    /// Output format: "json" or "text"
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Whether to show results when query is empty
    #[serde(default)]
    pub show_on_empty: bool,
}

fn default_output_format() -> String {
    "json".to_string()
}

fn default_timeout() -> u64 {
    3000 // 3 seconds
}

/// Script output in JSON format
#[derive(Debug, Deserialize)]
struct ScriptOutput {
    results: Vec<ScriptResult>,
}

#[derive(Debug, Deserialize)]
struct ScriptResult {
    title: String,
    #[serde(default)]
    subtitle: Option<String>,
    command: String,
    #[serde(default)]
    icon: Option<String>,
}

/// A script-based plugin
pub struct ScriptPlugin {
    manifest: ScriptPluginManifest,
    plugin_dir: PathBuf,
}

impl ScriptPlugin {
    /// Load a script plugin from a directory
    pub fn load_from_dir(plugin_dir: &Path) -> anyhow::Result<Self> {
        let manifest_path = plugin_dir.join("plugin.toml");

        if !manifest_path.exists() {
            anyhow::bail!("Plugin manifest not found: {}", manifest_path.display());
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: ScriptPluginManifest = toml::from_str(&manifest_content)?;

        // Validate manifest
        if manifest.triggers.is_empty() {
            anyhow::bail!("Plugin must define at least one trigger");
        }

        Ok(Self {
            manifest,
            plugin_dir: plugin_dir.to_path_buf(),
        })
    }

    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.manifest.metadata.name
    }

    /// Get plugin priority
    pub fn priority(&self) -> u32 {
        self.manifest.metadata.priority
    }

    /// Check if this plugin should handle the query
    pub fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            return self.manifest.execution.show_on_empty;
        }

        // Check if query starts with any trigger
        self.manifest
            .triggers
            .iter()
            .any(|trigger| query.starts_with(trigger))
    }

    /// Execute the plugin script with the query
    pub fn execute(&self, query: &str) -> anyhow::Result<Vec<PluginResult>> {
        // Extract the actual query (remove trigger prefix)
        let actual_query = self.extract_query(query);

        // Build script path
        let script_path = if Path::new(&self.manifest.execution.script).is_absolute() {
            PathBuf::from(&self.manifest.execution.script)
        } else {
            self.plugin_dir.join(&self.manifest.execution.script)
        };

        if !script_path.exists() {
            anyhow::bail!("Script not found: {}", script_path.display());
        }

        // Build command
        let mut cmd = if let Some(interpreter) = &self.manifest.execution.interpreter {
            let mut c = Command::new(interpreter);
            c.arg(&script_path);
            c
        } else {
            Command::new(&script_path)
        };

        // Add query as argument
        cmd.arg(actual_query);

        // Add environment variables
        for (key, value) in &self.manifest.environment {
            cmd.env(key, value);
        }

        // Set working directory to plugin directory
        cmd.current_dir(&self.plugin_dir);

        // Execute with timeout
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Script execution failed: {}", stderr);
        }

        // Parse output based on format
        match self.manifest.execution.output_format.as_str() {
            "json" => self.parse_json_output(&output.stdout),
            "text" => self.parse_text_output(&output.stdout),
            _ => anyhow::bail!(
                "Unsupported output format: {}",
                self.manifest.execution.output_format
            ),
        }
    }

    /// Extract the actual query by removing trigger prefix
    fn extract_query<'a>(&self, query: &'a str) -> &'a str {
        for trigger in &self.manifest.triggers {
            if let Some(rest) = query.strip_prefix(trigger) {
                return rest.trim();
            }
        }
        query
    }

    /// Parse JSON output from script
    fn parse_json_output(&self, output: &[u8]) -> anyhow::Result<Vec<PluginResult>> {
        let output_str = String::from_utf8_lossy(output);
        let script_output: ScriptOutput = serde_json::from_str(&output_str)?;

        Ok(script_output
            .results
            .into_iter()
            .map(|r| PluginResult {
                title: r.title,
                subtitle: r.subtitle,
                command: r.command,
                icon: r.icon.or_else(|| self.manifest.metadata.icon.clone()),
            })
            .collect())
    }

    /// Parse plain text output from script (one result per line)
    fn parse_text_output(&self, output: &[u8]) -> anyhow::Result<Vec<PluginResult>> {
        let output_str = String::from_utf8_lossy(output);
        let lines: Vec<&str> = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        if lines.is_empty() {
            return Ok(vec![]);
        }

        // Text format: each line is "title|subtitle|command"
        // Or simple format: each line is just a title (command = title)
        Ok(lines
            .into_iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();

                match parts.len() {
                    1 => Some(PluginResult {
                        title: parts[0].to_string(),
                        subtitle: None,
                        command: parts[0].to_string(),
                        icon: self.manifest.metadata.icon.clone(),
                    }),
                    2 => Some(PluginResult {
                        title: parts[0].to_string(),
                        subtitle: Some(parts[1].to_string()),
                        command: parts[0].to_string(),
                        icon: self.manifest.metadata.icon.clone(),
                    }),
                    3 => Some(PluginResult {
                        title: parts[0].to_string(),
                        subtitle: Some(parts[1].to_string()),
                        command: parts[2].to_string(),
                        icon: self.manifest.metadata.icon.clone(),
                    }),
                    _ => None,
                }
            })
            .collect())
    }
}

/// Script plugin manager - loads and manages all script plugins
pub struct ScriptPluginManager {
    plugins: Vec<ScriptPlugin>,
}

impl Default for ScriptPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptPluginManager {
    /// Create a new plugin manager and load all plugins
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
        };

        manager.load_plugins();
        manager
    }

    /// Load all plugins from the plugin directories
    fn load_plugins(&mut self) {
        let plugin_dirs = Self::get_plugin_directories();

        for dir in plugin_dirs {
            if !dir.exists() {
                tracing::debug!("Plugin directory does not exist: {}", dir.display());
                continue;
            }

            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        match ScriptPlugin::load_from_dir(&path) {
                            Ok(plugin) => {
                                tracing::info!(
                                    "Loaded script plugin: {} (priority: {})",
                                    plugin.name(),
                                    plugin.priority()
                                );
                                self.plugins.push(plugin);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to load plugin from {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Sort by priority (highest first)
        self.plugins.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Get list of directories to scan for plugins
    fn get_plugin_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User plugins: ~/.config/native-launcher/plugins/
        if let Some(config_dir) = dirs::config_dir() {
            dirs.push(config_dir.join("native-launcher").join("plugins"));
        }

        // System plugins: /usr/share/native-launcher/plugins/
        dirs.push(PathBuf::from("/usr/share/native-launcher/plugins"));

        // Development plugins: ./plugins/ (for testing)
        if let Ok(current_dir) = std::env::current_dir() {
            dirs.push(current_dir.join("plugins"));
        }

        dirs
    }

    /// Search across all loaded plugins
    pub fn search(&self, query: &str) -> Vec<PluginResult> {
        let mut all_results = Vec::new();

        for plugin in &self.plugins {
            if plugin.matches(query) {
                match plugin.execute(query) {
                    Ok(results) => {
                        all_results.extend(results);
                    }
                    Err(e) => {
                        tracing::error!("Plugin {} execution failed: {}", plugin.name(), e);
                    }
                }
            }
        }

        all_results
    }

    /// Get number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get list of loaded plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins
            .iter()
            .map(|p| {
                format!(
                    "{} v{}",
                    p.manifest.metadata.name, p.manifest.metadata.version
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_parsing() {
        let toml = r#"
            [metadata]
            name = "Test Plugin"
            description = "A test plugin"
            author = "Test Author"
            version = "1.0.0"
            priority = 500
            
            triggers = ["test ", "t "]
            
            [execution]
            script = "test.sh"
            interpreter = "bash"
            output_format = "json"
            timeout_ms = 2000
        "#;

        let manifest: ScriptPluginManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.metadata.name, "Test Plugin");
        assert_eq!(manifest.triggers.len(), 2);
        assert_eq!(manifest.execution.script, "test.sh");
    }

    #[test]
    fn test_query_extraction() {
        let manifest = ScriptPluginManifest {
            metadata: PluginMetadata {
                name: "Test".to_string(),
                description: "Test".to_string(),
                author: "Test".to_string(),
                version: "1.0".to_string(),
                priority: 500,
                icon: None,
            },
            triggers: vec!["weather ".to_string(), "w ".to_string()],
            execution: ExecutionConfig {
                script: "script.sh".to_string(),
                interpreter: None,
                output_format: "json".to_string(),
                timeout_ms: 3000,
                show_on_empty: false,
            },
            environment: HashMap::new(),
        };

        let plugin = ScriptPlugin {
            manifest,
            plugin_dir: PathBuf::from("/tmp"),
        };

        assert_eq!(plugin.extract_query("weather Tokyo"), "Tokyo");
        assert_eq!(plugin.extract_query("w Tokyo"), "Tokyo");
    }
}
