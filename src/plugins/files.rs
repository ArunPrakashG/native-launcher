use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Recent file entry from recently-used.xbel
#[derive(Debug, Clone)]
struct RecentFile {
    /// File path
    path: PathBuf,
    /// Display name
    name: String,
    /// MIME type (optional)
    #[allow(dead_code)]
    mime_type: Option<String>,
    /// Last modified timestamp
    #[allow(dead_code)]
    modified: Option<i64>,
}

/// Plugin for file browsing and recent files
#[derive(Debug)]
pub struct FileBrowserPlugin {
    recent_files: Vec<RecentFile>,
    enabled: bool,
    #[allow(dead_code)]
    max_recent: usize,
}

impl FileBrowserPlugin {
    /// Create a new file browser plugin
    pub fn new(enabled: bool) -> Self {
        let recent_files = Self::load_recent_files(20).unwrap_or_else(|e| {
            warn!("Failed to load recent files: {}", e);
            Vec::new()
        });

        debug!(
            "File browser plugin initialized with {} recent files",
            recent_files.len()
        );

        Self {
            recent_files,
            enabled,
            max_recent: 20,
        }
    }

    /// Load recent files from GTK's recently-used.xbel
    fn load_recent_files(max_count: usize) -> Result<Vec<RecentFile>> {
        let xbel_path = dirs::data_local_dir()
            .context("Failed to get local data directory")?
            .join("recently-used.xbel");

        if !xbel_path.exists() {
            debug!("Recently-used.xbel not found at: {}", xbel_path.display());
            return Ok(Vec::new());
        }

        debug!("Loading recent files from: {}", xbel_path.display());
        let content =
            fs::read_to_string(&xbel_path).context("Failed to read recently-used.xbel")?;

        let mut files = Vec::new();

        // Simple XML parsing (looking for bookmark tags)
        for line in content.lines() {
            let line = line.trim();

            // Look for bookmark tags with href attribute
            if line.starts_with("<bookmark href=\"") {
                if let Some(url) = Self::extract_href(line) {
                    // Convert file:// URL to path
                    if let Some(path) = Self::url_to_path(&url) {
                        if path.exists() {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            files.push(RecentFile {
                                path: path.clone(),
                                name,
                                mime_type: None,
                                modified: Self::get_modified_time(&path),
                            });

                            if files.len() >= max_count {
                                break;
                            }
                        }
                    }
                }
            }
        }

        debug!("Loaded {} recent files", files.len());
        Ok(files)
    }

    /// Extract href attribute from bookmark tag
    fn extract_href(line: &str) -> Option<String> {
        let start = line.find("href=\"")? + 6;
        let end = line[start..].find('"')?;
        Some(line[start..start + end].to_string())
    }

    /// Convert file:// URL to PathBuf
    fn url_to_path(url: &str) -> Option<PathBuf> {
        if let Some(path_str) = url.strip_prefix("file://") {
            // Decode URL and remove file:// prefix
            let decoded = urlencoding::decode(path_str).ok()?;
            Some(PathBuf::from(decoded.as_ref()))
        } else {
            None
        }
    }

    /// Get file modified time as Unix timestamp
    fn get_modified_time(path: &Path) -> Option<i64> {
        let metadata = fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        let duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
        Some(duration.as_secs() as i64)
    }

    /// Get icon for file based on extension or type
    fn get_file_icon(path: &Path) -> String {
        if path.is_dir() {
            return "folder".to_string();
        }

        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                // Documents
                "pdf" => "application-pdf",
                "doc" | "docx" => "application-msword",
                "xls" | "xlsx" => "application-vnd.ms-excel",
                "ppt" | "pptx" => "application-vnd.ms-powerpoint",
                "txt" => "text-x-generic",
                "md" | "markdown" => "text-x-markdown",

                // Images
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => "image-x-generic",

                // Video
                "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic",

                // Audio
                "mp3" | "flac" | "wav" | "ogg" | "m4a" => "audio-x-generic",

                // Archives
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "package-x-generic",

                // Code
                "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "go" | "java" => "text-x-script",

                _ => "text-x-generic",
            }
            .to_string()
        } else {
            "text-x-generic".to_string()
        }
    }

    /// Format file size for display
    fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        if unit_idx == 0 {
            format!("{} {}", bytes, UNITS[0])
        } else {
            format!("{:.1} {}", size, UNITS[unit_idx])
        }
    }

    /// Search in a directory
    fn search_directory(dir: &Path, query: &str, max_results: usize) -> Result<Vec<PluginResult>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(results);
        }

        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry
                .file_name()
                .to_string_lossy()
                .to_string()
                .to_lowercase();

            // Skip hidden files unless query starts with .
            if !query_lower.starts_with('.') && file_name.starts_with('.') {
                continue;
            }

            // Match against query
            if file_name.contains(&query_lower) {
                let display_name = entry.file_name().to_string_lossy().to_string();
                let icon = Self::get_file_icon(&path);

                // Build subtitle with file info
                let subtitle = if let Ok(metadata) = fs::metadata(&path) {
                    if path.is_dir() {
                        "Directory".to_string()
                    } else {
                        Self::format_size(metadata.len())
                    }
                } else {
                    String::new()
                };

                // Calculate score based on match quality
                let score = if file_name == query_lower {
                    1000 // Exact match
                } else if file_name.starts_with(&query_lower) {
                    800 // Prefix match
                } else {
                    600 // Contains match
                };

                results.push(PluginResult {
                    title: display_name,
                    subtitle: Some(subtitle),
                    icon: Some(icon),
                    command: format!("xdg-open '{}'", path.display()),
                    terminal: false,
                    score,
                    plugin_name: "files".to_string(),
                    sub_results: Vec::new(),
                    parent_app: None,
                });

                if results.len() >= max_results {
                    break;
                }
            }
        }

        Ok(results)
    }
}

impl Plugin for FileBrowserPlugin {
    fn name(&self) -> &str {
        "files"
    }

    fn description(&self) -> &str {
        "File browser, recent files, and workspaces"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@files"]
    }

    fn priority(&self) -> i32 {
        650 // Between SSH (700) and Web Search (600)
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn should_handle(&self, query: &str) -> bool {
        if !self.enabled || query.is_empty() {
            return false;
        }

        // Always participate in global search (query length >= 2 for performance)
        // But be smart about what we search
        query.len() >= 2
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Determine search mode
        let is_command_query = query.starts_with('@');
        let is_file_command =
            query_lower.starts_with("@recent") || query_lower.starts_with("@file");
        let is_path_query = query.starts_with('/') || query.starts_with("~/");

        // For global search (no @ command), search files
        let search_files = !is_command_query || is_file_command;

        // RECENT FILES SEARCH
        if search_files {
            let search_term = if is_file_command {
                // Extract search term after @command
                query_lower
                    .strip_prefix("@recent")
                    .or_else(|| query_lower.strip_prefix("@file"))
                    .unwrap_or(&query_lower)
                    .trim()
            } else {
                // For global search, use the full query
                query_lower.trim()
            };

            for file in &self.recent_files {
                // Filter by search term
                if !search_term.is_empty()
                    && !file.name.to_lowercase().contains(search_term)
                    && !file
                        .path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(search_term)
                {
                    continue;
                }

                let icon = Self::get_file_icon(&file.path);
                let subtitle = file
                    .path
                    .parent()
                    .and_then(|p| p.to_str())
                    .map(String::from);

                results.push(PluginResult {
                    title: file.name.clone(),
                    subtitle,
                    icon: Some(icon),
                    command: format!("xdg-open '{}'", file.path.display()),
                    terminal: false,
                    score: 700,
                    plugin_name: self.name().to_string(),
                    sub_results: Vec::new(),
                    parent_app: None,
                });

                if results.len() >= context.max_results {
                    break;
                }
            }
        }

        // PATH-BASED SEARCH (always enabled for paths, regardless of @command)
        if is_path_query {
            let expanded_path = if query.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&query[2..])
                } else {
                    PathBuf::from(query)
                }
            } else {
                PathBuf::from(query)
            };

            // If path ends with /, search in that directory
            if query.ends_with('/') {
                if let Ok(dir_results) =
                    Self::search_directory(&expanded_path, "", context.max_results)
                {
                    results.extend(dir_results);
                }
            } else {
                // Search in parent directory for matching files
                if let Some(parent) = expanded_path.parent() {
                    if let Some(search_name) = expanded_path.file_name() {
                        let search_str = search_name.to_string_lossy();
                        if let Ok(dir_results) =
                            Self::search_directory(parent, &search_str, context.max_results)
                        {
                            results.extend(dir_results);
                        }
                    }
                }
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
    fn test_extract_href() {
        let line =
            r#"<bookmark href="file:///home/user/document.pdf" added="2024-10-20T12:00:00Z">"#;
        let href = FileBrowserPlugin::extract_href(line);
        assert_eq!(href, Some("file:///home/user/document.pdf".to_string()));
    }

    #[test]
    fn test_url_to_path() {
        let url = "file:///home/user/test.txt";
        let path = FileBrowserPlugin::url_to_path(url);
        assert_eq!(path, Some(PathBuf::from("/home/user/test.txt")));
    }

    #[test]
    fn test_get_file_icon() {
        assert_eq!(
            FileBrowserPlugin::get_file_icon(&PathBuf::from("test.pdf")),
            "application-pdf"
        );
        assert_eq!(
            FileBrowserPlugin::get_file_icon(&PathBuf::from("test.rs")),
            "text-x-script"
        );
        assert_eq!(
            FileBrowserPlugin::get_file_icon(&PathBuf::from("test.jpg")),
            "image-x-generic"
        );
    }

    #[test]
    fn test_format_size() {
        assert_eq!(FileBrowserPlugin::format_size(100), "100 B");
        assert_eq!(FileBrowserPlugin::format_size(1024), "1.0 KB");
        assert_eq!(FileBrowserPlugin::format_size(1536), "1.5 KB");
        assert_eq!(FileBrowserPlugin::format_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_should_handle() {
        let plugin = FileBrowserPlugin::new(true);

        // File browser participates in global search for all queries >= 2 chars
        assert!(plugin.should_handle("/home/user"));
        assert!(plugin.should_handle("~/Documents"));
        assert!(plugin.should_handle("recent"));
        assert!(plugin.should_handle("file test"));
        assert!(plugin.should_handle("firefox")); // Participates, but may return no results

        // Too short queries are rejected
        assert!(!plugin.should_handle("a"));
        assert!(!plugin.should_handle(""));

        // Disabled plugin doesn't handle
        let disabled = FileBrowserPlugin::new(false);
        assert!(!disabled.should_handle("test"));
    }
}
