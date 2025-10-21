use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use super::cache::DesktopCache;
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
    #[allow(dead_code)]

    pub fn add_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Get the configured search paths
    #[allow(dead_code)]

    pub fn paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Scan all configured paths and return desktop entries
    #[allow(dead_code)]

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

    /// Scan with caching for faster startup
    pub fn scan_cached(&self) -> Result<Vec<DesktopEntry>> {
        info!("Starting cached desktop file scan");

        // Try to load existing cache
        let mut cache = DesktopCache::load().unwrap_or_else(|e| {
            warn!("Failed to load cache: {}, building new cache", e);
            DesktopCache::new()
        });

        // Prune deleted files
        cache.prune();

        let mut entries = Vec::new();
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        for path in &self.search_paths {
            if !path.exists() {
                debug!("Skipping non-existent path: {}", path.display());
                continue;
            }

            info!("Scanning directory: {}", path.display());

            for entry in WalkDir::new(path)
                .follow_links(true)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let file_path = entry.path();

                // Only process .desktop files
                if file_path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }

                // Try cache first
                if let Some(cached_entry) = cache.get(file_path) {
                    cache_hits += 1;
                    if !cached_entry.no_display {
                        entries.push(cached_entry.clone());
                    }
                } else {
                    // Cache miss - parse file
                    cache_misses += 1;
                    match DesktopEntry::from_file(file_path.to_path_buf()) {
                        Ok(desktop_entry) => {
                            if !desktop_entry.no_display {
                                debug!("Parsed: {}", desktop_entry.name);
                                entries.push(desktop_entry.clone());
                            }
                            // Update cache
                            if let Err(e) = cache.insert(file_path.to_path_buf(), desktop_entry) {
                                warn!("Failed to cache {}: {}", file_path.display(), e);
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Cache stats: {} hits, {} misses", cache_hits, cache_misses);

        // Save updated cache
        if let Err(e) = cache.save() {
            warn!("Failed to save cache: {}", e);
        }

        // Remove duplicates
        entries = self.deduplicate_entries(entries);

        info!("Scan complete: {} total entries", entries.len());
        Ok(entries)
    }

    /// Scan a single directory for .desktop files
    #[allow(dead_code)]

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
            if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(key) {
                e.insert(());
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
