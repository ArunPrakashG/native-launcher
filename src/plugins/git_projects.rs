use crate::plugins::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tracing::{debug, warn};

/// Git repository information
#[derive(Debug, Clone)]
pub struct GitRepo {
    /// Repository path
    pub path: PathBuf,
    /// Repository name (directory name)
    pub name: String,
    /// Current branch (if available)
    pub branch: Option<String>,
}

#[derive(Debug)]
pub struct GitProjectsPlugin {
    enabled: bool,
    repos: OnceLock<Vec<GitRepo>>,
    base_paths: Vec<PathBuf>,
}

impl GitProjectsPlugin {
    pub fn new(enabled: bool) -> Self {
        // Common code directories
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base_paths = vec![
            PathBuf::from(format!("{}/code", home)),
            PathBuf::from(format!("{}/projects", home)),
            PathBuf::from(format!("{}/dev", home)),
            PathBuf::from(format!("{}/workspace", home)),
        ];

        Self {
            enabled,
            repos: OnceLock::new(),
            base_paths,
        }
    }

    fn get_repos(&self) -> &Vec<GitRepo> {
        self.repos.get_or_init(|| {
            let start = std::time::Instant::now();
            let repos = self.scan_repos();
            let elapsed = start.elapsed();

            debug!(
                "Git repos scan completed: {} repos found in {:?}",
                repos.len(),
                elapsed
            );

            // Warn if scan took too long (>10ms budget)
            if elapsed.as_millis() > 10 {
                warn!(
                    "Git repos scan exceeded 10ms budget: {:?} ({} repos)",
                    elapsed,
                    repos.len()
                );
            }

            repos
        })
    }

    fn scan_repos(&self) -> Vec<GitRepo> {
        let mut repos = Vec::new();

        for base_path in &self.base_paths {
            if !base_path.exists() {
                continue;
            }

            // Scan depth 1 (immediate children)
            if let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check if it's a git repo
                        let git_dir = path.join(".git");
                        if git_dir.exists() {
                            if let Some(repo) = Self::create_repo_info(&path) {
                                repos.push(repo);
                            }
                        } else {
                            // Scan depth 2 (one level deeper)
                            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                                for sub_entry in sub_entries.flatten() {
                                    let sub_path = sub_entry.path();
                                    if sub_path.is_dir() {
                                        let git_dir = sub_path.join(".git");
                                        if git_dir.exists() {
                                            if let Some(repo) = Self::create_repo_info(&sub_path) {
                                                repos.push(repo);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        repos
    }

    fn create_repo_info(path: &Path) -> Option<GitRepo> {
        let name = path.file_name()?.to_str()?.to_string();

        // Get current branch (with timeout to avoid hanging)
        let branch = Command::new("git")
            .args(["-C", path.to_str()?, "branch", "--show-current"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                } else {
                    None
                }
            });

        Some(GitRepo {
            path: path.to_path_buf(),
            name,
            branch,
        })
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@git") || query.starts_with("@repo")
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@git") {
            rest.trim()
        } else if let Some(rest) = query.strip_prefix("@repo") {
            rest.trim()
        } else {
            query
        }
    }

    fn get_default_editor() -> String {
        // Check environment variables
        if let Ok(editor) = std::env::var("VISUAL") {
            return editor;
        }
        if let Ok(editor) = std::env::var("EDITOR") {
            return editor;
        }

        // Check common editors
        for editor in &["code", "subl", "atom", "nvim", "vim", "nano"] {
            if Command::new("which")
                .arg(editor)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return editor.to_string();
            }
        }

        "xdg-open".to_string() // Fallback
    }
}

impl Plugin for GitProjectsPlugin {
    fn name(&self) -> &str {
        "git-projects"
    }

    fn description(&self) -> &str {
        "Search git repositories and open them in editor"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@git", "@repo"]
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn priority(&self) -> i32 {
        60 // Medium-high priority
    }

    fn should_handle(&self, query: &str) -> bool {
        self.should_handle(query)
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled || !self.should_handle(query) {
            return Ok(Vec::new());
        }

        let filter = self.strip_prefix(query).to_lowercase();
        let repos = self.get_repos();

        debug!(
            "Git plugin searching: filter='{}', {} repos indexed",
            filter,
            repos.len()
        );

        let editor = Self::get_default_editor();

        let mut results: Vec<PluginResult> = repos
            .iter()
            .filter_map(|repo| {
                // Filter by query
                if !filter.is_empty() {
                    let name_lower = repo.name.to_lowercase();
                    let path_lower = repo.path.to_string_lossy().to_lowercase();

                    if !name_lower.contains(&filter) && !path_lower.contains(&filter) {
                        return None;
                    }
                }

                // Calculate score
                let mut score = 500;
                if !filter.is_empty() {
                    let name_lower = repo.name.to_lowercase();

                    // Exact match
                    if name_lower == filter {
                        score += 3000;
                    }
                    // Starts with
                    else if name_lower.starts_with(&filter) {
                        score += 2000;
                    }
                    // Contains
                    else {
                        score += 1000;
                    }
                }

                // Build subtitle with branch info
                let subtitle = if let Some(ref branch) = repo.branch {
                    format!("{}  •  {}", repo.path.display(), branch)
                } else {
                    format!("{}", repo.path.display())
                };

                // Command to open in editor
                let command = format!("{} '{}'", editor, repo.path.display());

                Some(PluginResult {
                    title: repo.name.clone(),
                    subtitle: Some(subtitle),
                    icon: Some("folder-git".to_string()),
                    command,
                    terminal: false,
                    score,
                    plugin_name: self.name().to_string(),
                    sub_results: vec![],
                    parent_app: None,
                    desktop_path: None,
                    badge_icon: Some("folder-symbolic".to_string()), // Git repo badge
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
        let plugin = GitProjectsPlugin::new(true);

        assert!(plugin.should_handle("@git"));
        assert!(plugin.should_handle("@git foo"));
        assert!(plugin.should_handle("@repo bar"));

        assert!(!plugin.should_handle("git"));
        assert!(!plugin.should_handle("@recent"));
    }

    #[test]
    fn test_strip_prefix() {
        let plugin = GitProjectsPlugin::new(true);

        assert_eq!(plugin.strip_prefix("@git foo"), "foo");
        assert_eq!(plugin.strip_prefix("@repo bar"), "bar");
        assert_eq!(plugin.strip_prefix("@git  test  "), "test");
    }

    #[test]
    fn test_plugin_enabled() {
        let plugin = GitProjectsPlugin::new(true);
        assert!(plugin.enabled());

        let plugin = GitProjectsPlugin::new(false);
        assert!(!plugin.enabled());
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = GitProjectsPlugin::new(true);
        assert_eq!(plugin.priority(), 60);
    }

    #[test]
    fn test_command_prefixes() {
        let plugin = GitProjectsPlugin::new(true);
        let prefixes = plugin.command_prefixes();

        assert_eq!(prefixes.len(), 2);
        assert!(prefixes.contains(&"@git"));
        assert!(prefixes.contains(&"@repo"));
    }

    #[test]
    fn test_get_default_editor() {
        let editor = GitProjectsPlugin::get_default_editor();
        assert!(!editor.is_empty());
    }

    #[test]
    fn test_create_repo_info() {
        // Test with current directory (assuming it's a git repo)
        if let Ok(current_dir) = std::env::current_dir() {
            let git_dir = current_dir.join(".git");
            if git_dir.exists() {
                let repo_info = GitProjectsPlugin::create_repo_info(&current_dir);
                assert!(repo_info.is_some());

                let repo = repo_info.unwrap();
                assert!(!repo.name.is_empty());
                assert_eq!(repo.path, current_dir);
            }
        }
    }
}
