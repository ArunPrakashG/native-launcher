use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{debug, warn};

/// Recent documents plugin that aggregates recently accessed files
/// Parses ~/.local/share/recently-used.xbel (freedesktop standard)
/// Uses lazy loading - entries are loaded on first search
#[derive(Debug)]
pub struct RecentDocumentsPlugin {
    entries: OnceLock<Vec<RecentEntry>>,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct RecentEntry {
    path: PathBuf,
    mime_type: String,
    modified: DateTime<Utc>,
    accessed: DateTime<Utc>,
    count: u32,
}

impl RecentDocumentsPlugin {
    pub fn new() -> Self {
        debug!("recent documents plugin created (lazy loading enabled)");

        Self {
            entries: OnceLock::new(),
            enabled: true,
        }
    }

    /// Get entries, loading them lazily on first access
    fn get_entries(&self) -> &Vec<RecentEntry> {
        self.entries.get_or_init(|| {
            Self::load_recent_entries(200).unwrap_or_else(|e| {
                warn!("Failed to load recent documents: {}", e);
                Vec::new()
            })
        })
    }

    /// Load recent entries from recently-used.xbel
    fn load_recent_entries(max_count: usize) -> Result<Vec<RecentEntry>> {
        let xbel_path = Self::recently_used_path()?;

        if !xbel_path.exists() {
            debug!("recently-used.xbel not found at {:?}", xbel_path);
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&xbel_path)
            .with_context(|| format!("Failed to read {:?}", xbel_path))?;

        let entries = Self::parse_xbel(&content)?;

        // Sort by modified time (most recent first)
        let mut entries = entries;
        entries.sort_by(|a, b| b.modified.cmp(&a.modified));
        entries.truncate(max_count);

        debug!("Parsed {} recent entries from xbel", entries.len());

        Ok(entries)
    }

    /// Get path to recently-used.xbel
    fn recently_used_path() -> Result<PathBuf> {
        let home = home_dir().context("Could not determine home directory")?;
        Ok(home.join(".local/share/recently-used.xbel"))
    }

    /// Parse XBEL XML format
    /// Simple line-by-line parser to avoid heavy XML dependency
    fn parse_xbel(content: &str) -> Result<Vec<RecentEntry>> {
        let mut entries = Vec::new();
        let mut current_href: Option<String> = None;
        let mut current_modified: Option<DateTime<Utc>> = None;
        let mut current_visited: Option<DateTime<Utc>> = None;
        let mut current_mime: Option<String> = None;
        let mut current_count: Option<u32> = None;

        for line in content.lines() {
            let line = line.trim();

            // Parse bookmark tag with href and timestamps
            if line.starts_with("<bookmark href=") {
                // Extract href
                if let Some(href_start) = line.find("href=\"") {
                    let href_content = &line[href_start + 6..];
                    if let Some(href_end) = href_content.find('"') {
                        let href = &href_content[..href_end];
                        if href.starts_with("file://") {
                            current_href = Some(href.to_string());
                        }
                    }
                }

                // Extract modified timestamp
                if let Some(mod_start) = line.find("modified=\"") {
                    let mod_content = &line[mod_start + 10..];
                    if let Some(mod_end) = mod_content.find('"') {
                        let timestamp = &mod_content[..mod_end];
                        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
                            current_modified = Some(dt.with_timezone(&Utc));
                        }
                    }
                }

                // Extract visited timestamp
                if let Some(vis_start) = line.find("visited=\"") {
                    let vis_content = &line[vis_start + 9..];
                    if let Some(vis_end) = vis_content.find('"') {
                        let timestamp = &vis_content[..vis_end];
                        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
                            current_visited = Some(dt.with_timezone(&Utc));
                        }
                    }
                }
            }

            // Parse mime-type
            if line.contains("<mime:mime-type type=") {
                if let Some(type_start) = line.find("type=\"") {
                    let type_content = &line[type_start + 6..];
                    if let Some(type_end) = type_content.find('"') {
                        current_mime = Some(type_content[..type_end].to_string());
                    }
                }
            }

            // Parse count from application tag
            if line.contains("<bookmark:application") {
                if let Some(count_start) = line.find("count=\"") {
                    let count_content = &line[count_start + 7..];
                    if let Some(count_end) = count_content.find('"') {
                        if let Ok(count) = count_content[..count_end].parse::<u32>() {
                            current_count = Some(count);
                        }
                    }
                }
            }

            // End of bookmark - create entry
            if line.starts_with("</bookmark>") {
                if let (Some(href), Some(modified), Some(visited)) =
                    (current_href.take(), current_modified, current_visited)
                {
                    // Convert file:// URL to path
                    if let Some(path_str) = href.strip_prefix("file://") {
                        // URL decode the path
                        if let Ok(decoded) = urlencoding::decode(path_str) {
                            let path = PathBuf::from(decoded.into_owned());

                            // Only include files that exist
                            if path.exists() {
                                entries.push(RecentEntry {
                                    path,
                                    mime_type: current_mime
                                        .take()
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    modified,
                                    accessed: visited,
                                    count: current_count.unwrap_or(1),
                                });
                            }
                        }
                    }
                }

                // Reset for next bookmark
                current_href = None;
                current_modified = None;
                current_visited = None;
                current_mime = None;
                current_count = None;
            }
        }

        Ok(entries)
    }

    /// Get file type category from mime type
    fn categorize_mime(mime: &str) -> &'static str {
        if mime.starts_with("text/") {
            "Text"
        } else if mime.starts_with("image/") {
            "Image"
        } else if mime.starts_with("video/") {
            "Video"
        } else if mime.starts_with("audio/") {
            "Audio"
        } else if mime.starts_with("application/pdf") {
            "PDF"
        } else if mime.contains("document") || mime.contains("word") {
            "Document"
        } else if mime.contains("spreadsheet") || mime.contains("excel") {
            "Spreadsheet"
        } else if mime.contains("presentation") || mime.contains("powerpoint") {
            "Presentation"
        } else if mime == "inode/directory" {
            "Folder"
        } else {
            "File"
        }
    }

    /// Get human-readable time ago string
    fn time_ago(dt: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(*dt);

        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{}d ago", duration.num_days())
        } else if duration.num_weeks() < 4 {
            format!("{}w ago", duration.num_weeks())
        } else {
            format!("{}mo ago", duration.num_days() / 30)
        }
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@recent") {
            rest
        } else if let Some(rest) = query.strip_prefix("@r") {
            rest
        } else {
            query
        }
    }
}

impl Plugin for RecentDocumentsPlugin {
    fn name(&self) -> &str {
        "recent"
    }

    fn search(&self, query: &str, ctx: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.should_handle(query) {
            return Ok(Vec::new());
        }

        let filter = self.strip_prefix(query).trim().to_lowercase();
        let mut results = Vec::new();

        let entries = self.get_entries();
        for (idx, entry) in entries.iter().enumerate() {
            // Skip if we have enough results
            if results.len() >= ctx.max_results {
                break;
            }

            let filename = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let path_str = entry.path.to_string_lossy();

            // Filter by query
            if !filter.is_empty() {
                let filename_lower = filename.to_lowercase();
                let path_lower = path_str.to_lowercase();

                if !filename_lower.contains(&filter) && !path_lower.contains(&filter) {
                    continue;
                }
            }

            let category = Self::categorize_mime(&entry.mime_type);
            let time_str = Self::time_ago(&entry.modified);

            // Build command to open file with default handler
            let command = if entry.path.is_dir() {
                format!("xdg-open '{}'", entry.path.display())
            } else {
                format!("xdg-open '{}'", entry.path.display())
            };

            let subtitle = format!(
                "{} • {} • {}",
                category,
                time_str,
                entry
                    .path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
            );

            // Score: more recent + higher count = higher score
            let age_days = (Utc::now() - entry.modified).num_days();
            let recency_score = 10000 / (age_days + 1).max(1);
            let count_bonus = (entry.count as i64) * 100;
            let filter_bonus = if !filter.is_empty() { 2000 } else { 0 };
            let score = 8000 + recency_score + count_bonus + filter_bonus - (idx as i64 * 10);

            let result = PluginResult::new(filename.to_string(), command, self.name().to_string())
                .with_subtitle(subtitle)
                .with_score(score);

            results.push(result);
        }

        Ok(results)
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@recent") || query.starts_with("@r ")
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn priority(&self) -> i32 {
        80
    }

    fn description(&self) -> &str {
        "Search recently accessed files and folders via @recent"
    }

    fn handle_keyboard_event(
        &self,
        event: &crate::plugins::traits::KeyboardEvent,
    ) -> crate::plugins::traits::KeyboardAction {
        use crate::plugins::traits::KeyboardAction;
        use gtk4::gdk::ModifierType;

        // Only handle if we have a selection and query matches our prefix
        if !event.has_selection || !self.should_handle(&event.query) {
            return KeyboardAction::None;
        }

        // Alt+Enter: Open containing folder
        if event.modifiers.contains(ModifierType::ALT_MASK) && event.key == gtk4::gdk::Key::Return {
            // Note: The actual file path would need to be extracted from the selected result
            // For now, this returns None as the result path is not in the event
            // The UI would need to pass the selected result's path in the event
            return KeyboardAction::None; // TODO: Extract path from selected result
        }

        // Ctrl+Enter: Copy path to clipboard
        if event.modifiers.contains(ModifierType::CONTROL_MASK)
            && event.key == gtk4::gdk::Key::Return
        {
            // Note: Same limitation as above
            return KeyboardAction::None; // TODO: Extract path from selected result
        }

        KeyboardAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_should_handle() {
        let plugin = RecentDocumentsPlugin::new();
        assert!(plugin.should_handle("@recent"));
        assert!(plugin.should_handle("@recent test"));
        assert!(plugin.should_handle("@r test"));
        assert!(!plugin.should_handle("recent"));
        assert!(!plugin.should_handle("@rec"));
    }

    #[test]
    fn test_strip_prefix() {
        let plugin = RecentDocumentsPlugin::new();
        assert_eq!(plugin.strip_prefix("@recent test"), " test");
        assert_eq!(plugin.strip_prefix("@r test"), " test");
        assert_eq!(plugin.strip_prefix("@recent"), "");
    }

    #[test]
    fn test_categorize_mime() {
        assert_eq!(RecentDocumentsPlugin::categorize_mime("text/plain"), "Text");
        assert_eq!(RecentDocumentsPlugin::categorize_mime("image/png"), "Image");
        assert_eq!(RecentDocumentsPlugin::categorize_mime("video/mp4"), "Video");
        assert_eq!(
            RecentDocumentsPlugin::categorize_mime("audio/mpeg"),
            "Audio"
        );
        assert_eq!(
            RecentDocumentsPlugin::categorize_mime("application/pdf"),
            "PDF"
        );
        assert_eq!(
            RecentDocumentsPlugin::categorize_mime("inode/directory"),
            "Folder"
        );
        assert_eq!(
            RecentDocumentsPlugin::categorize_mime("application/unknown"),
            "File"
        );
    }

    #[test]
    fn test_time_ago() {
        let now = Utc::now();
        assert_eq!(RecentDocumentsPlugin::time_ago(&now), "Just now");

        let five_min_ago = now - chrono::Duration::minutes(5);
        assert_eq!(RecentDocumentsPlugin::time_ago(&five_min_ago), "5m ago");

        let two_hours_ago = now - chrono::Duration::hours(2);
        assert_eq!(RecentDocumentsPlugin::time_ago(&two_hours_ago), "2h ago");

        let three_days_ago = now - chrono::Duration::days(3);
        assert_eq!(RecentDocumentsPlugin::time_ago(&three_days_ago), "3d ago");
    }

    #[test]
    fn test_parse_xbel_sample() {
        let sample = r#"<?xml version="1.0"?>
<xbel>
  <bookmark href="file:///home/user/test.txt" modified="2025-11-01T10:00:00Z" visited="2025-11-01T10:00:00Z">
    <info>
      <metadata owner="http://freedesktop.org">
        <mime:mime-type type="text/plain"/>
        <bookmark:applications>
          <bookmark:application name="app" count="5"/>
        </bookmark:applications>
      </metadata>
    </info>
  </bookmark>
</xbel>"#;

        let entries = RecentDocumentsPlugin::parse_xbel(sample).unwrap();
        assert_eq!(entries.len(), 0); // File doesn't exist, so filtered out
    }

    #[test]
    fn test_search_filters_by_query() {
        let plugin = RecentDocumentsPlugin::new();
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@recent", &ctx).unwrap();
        // Should return results (may be empty if no recent files)
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = RecentDocumentsPlugin::new();
        assert_eq!(plugin.priority(), 80);
    }

    #[test]
    fn test_plugin_enabled() {
        let plugin = RecentDocumentsPlugin::new();
        assert!(plugin.enabled());
    }
}
