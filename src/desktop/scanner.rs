use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use super::entry::DesktopEntry;

/// Scans system directories for .desktop files
pub struct DesktopScanner {
    search_paths: Vec<PathBuf>,
}

impl DesktopScanner {
    /// Create a new scanner with default search paths
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // System-wide applications
        search_paths.push(PathBuf::from("/usr/share/applications"));
        search_paths.push(PathBuf::from("/usr/local/share/applications"));

        // User-specific applications
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".local/share/applications"));
        }

        // XDG data dirs
        if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_data_dirs.split(':') {
                if !dir.is_empty() {
                    search_paths.push(PathBuf::from(dir).join("applications"));
                }
            }
        }

        Self { search_paths }
    }

    /// Add a custom search path
    pub fn add_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Scan all configured paths and return desktop entries
    pub fn scan(&self) -> Result<Vec<DesktopEntry>> {
        info!("Starting desktop file scan");
        let mut entries = Vec::new();

        for path in &self.search_paths {
            if !path.exists() {
                debug!("Skipping non-existent path: {}", path.display());
                continue;
            }

            info!("Scanning directory: {}", path.display());
            match self.scan_directory(path) {
                Ok(mut dir_entries) => {
                    info!("Found {} entries in {}", dir_entries.len(), path.display());
                    entries.append(&mut dir_entries);
                }
                Err(e) => {
                    warn!("Error scanning {}: {}", path.display(), e);
                }
            }
        }

        // Remove duplicates (prefer user entries over system entries)
        entries = self.deduplicate_entries(entries);

        info!("Scan complete: {} total entries", entries.len());
        Ok(entries)
    }

    /// Scan a single directory for .desktop files
    fn scan_directory(&self, path: &Path) -> Result<Vec<DesktopEntry>> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only process .desktop files
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            match DesktopEntry::from_file(path.to_path_buf()) {
                Ok(desktop_entry) => {
                    // Skip entries marked as NoDisplay
                    if !desktop_entry.no_display {
                        debug!("Parsed: {}", desktop_entry.name);
                        entries.push(desktop_entry);
                    } else {
                        debug!("Skipping NoDisplay entry: {}", desktop_entry.name);
                    }
                }
                Err(e) => {
                    debug!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        Ok(entries)
    }

    /// Remove duplicate entries, preferring entries from later paths
    fn deduplicate_entries(&self, entries: Vec<DesktopEntry>) -> Vec<DesktopEntry> {
        use std::collections::HashMap;

        let mut seen = HashMap::new();
        let mut result = Vec::new();

        // Process in reverse order so user entries (which come later) override system entries
        for entry in entries.into_iter().rev() {
            let key = entry.name.clone();
            if !seen.contains_key(&key) {
                seen.insert(key, ());
                result.push(entry);
            }
        }

        result.reverse();
        result
    }
}

impl Default for DesktopScanner {
    fn default() -> Self {
        Self::new()
    }
}
