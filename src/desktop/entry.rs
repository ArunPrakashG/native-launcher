use anyhow::Result;
use std::path::PathBuf;

/// Represents a parsed desktop application entry
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    /// Application name
    pub name: String,
    /// Generic name (optional)
    pub generic_name: Option<String>,
    /// Executable command
    pub exec: String,
    /// Icon name or path
    pub icon: Option<String>,
    /// Categories
    pub categories: Vec<String>,
    /// Keywords for searching
    pub keywords: Vec<String>,
    /// Whether to launch in terminal
    pub terminal: bool,
    /// Original .desktop file path
    pub path: PathBuf,
    /// Whether the entry should be shown
    pub no_display: bool,
}

impl DesktopEntry {
    /// Parse a desktop entry from a .desktop file
    pub fn from_file(path: PathBuf) -> Result<Self> {
        use freedesktop_desktop_entry::DesktopEntry as FdEntry;

        let entry = FdEntry::from_path(path.clone(), &[] as &[&str])?;

        // Get the Desktop Entry section
        let name = entry
            .name(&[] as &[&str])
            .ok_or_else(|| anyhow::anyhow!("Desktop entry missing Name field"))?
            .to_string();

        let generic_name = entry.generic_name(&[] as &[&str]).map(|s| s.to_string());

        let exec = entry
            .exec()
            .ok_or_else(|| anyhow::anyhow!("Desktop entry missing Exec field"))?
            .to_string();

        let icon = entry.icon().map(|s| s.to_string());

        let categories = entry
            .categories()
            .map(|cats| {
                cats.iter()
                    .flat_map(|s| s.split(';'))
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let keywords = entry
            .keywords(&[] as &[&str])
            .map(|kws| kws.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let terminal = entry.terminal();
        let no_display = entry.no_display();

        Ok(DesktopEntry {
            name,
            generic_name,
            exec,
            icon,
            categories,
            keywords,
            terminal,
            path,
            no_display,
        })
    }

    /// Check if this entry matches a search query
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check name
        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check generic name
        if let Some(ref generic) = self.generic_name {
            if generic.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        // Check keywords
        for keyword in &self.keywords {
            if keyword.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        // Check categories
        for category in &self.categories {
            if category.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        false
    }

    /// Get a score for how well this entry matches the query (0-100)
    pub fn match_score(&self, query: &str) -> u32 {
        if query.is_empty() {
            return 50; // Neutral score for empty query
        }

        let query_lower = query.to_lowercase();
        let name_lower = self.name.to_lowercase();

        // Exact match
        if name_lower == query_lower {
            return 100;
        }

        // Starts with query
        if name_lower.starts_with(&query_lower) {
            return 90;
        }

        // Contains query at word boundary
        if name_lower.contains(&format!(" {}", query_lower)) {
            return 80;
        }

        // Contains query anywhere
        if name_lower.contains(&query_lower) {
            return 70;
        }

        // Check generic name
        if let Some(ref generic) = self.generic_name {
            let generic_lower = generic.to_lowercase();
            if generic_lower.starts_with(&query_lower) {
                return 60;
            }
            if generic_lower.contains(&query_lower) {
                return 50;
            }
        }

        // Check keywords
        for keyword in &self.keywords {
            let kw_lower = keyword.to_lowercase();
            if kw_lower.starts_with(&query_lower) {
                return 40;
            }
            if kw_lower.contains(&query_lower) {
                return 30;
            }
        }

        0
    }
}
