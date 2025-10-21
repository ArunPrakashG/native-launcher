use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tracing::{debug, info, warn};

use super::entry::DesktopEntry;

/// Cache metadata for a desktop file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedFile {
    /// Full path to the .desktop file
    path: PathBuf,
    /// Last modification time (seconds since UNIX epoch)
    mtime: u64,
    /// Parsed desktop entry
    entry: DesktopEntry,
}

/// Desktop entry cache for fast startup
#[derive(Debug, Serialize, Deserialize)]
pub struct DesktopCache {
    /// Cache format version for compatibility
    #[allow(dead_code)]

    version: u32,
    /// Cached entries keyed by file path
    entries: HashMap<PathBuf, CachedFile>,
}

impl Default for DesktopCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopCache {
    const VERSION: u32 = 1;

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            entries: HashMap::new(),
        }
    }

    /// Get the cache file path
    fn cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("native-launcher");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(cache_dir.join("entries.cache"))
    }

    /// Load cache from disk
    pub fn load() -> Result<Self> {
        let path = Self::cache_path()?;

        if !path.exists() {
            debug!("Cache file not found, creating new cache");
            return Ok(Self::new());
        }

        info!("Loading cache from: {}", path.display());
        let data = fs::read(&path).context("Failed to read cache file")?;

        let cache: DesktopCache =
            bincode::deserialize(&data).context("Failed to deserialize cache")?;

        // Check version compatibility
        if cache.version != Self::VERSION {
            warn!(
                "Cache version mismatch (expected {}, got {}), rebuilding cache",
                Self::VERSION,
                cache.version
            );
            return Ok(Self::new());
        }

        info!("Loaded {} cached entries", cache.entries.len());
        Ok(cache)
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::cache_path()?;
        debug!("Saving cache to: {}", path.display());

        let data = bincode::serialize(&self).context("Failed to serialize cache")?;

        fs::write(&path, data).context("Failed to write cache file")?;

        info!("Saved {} entries to cache", self.entries.len());
        Ok(())
    }

    /// Get modification time of a file
    fn get_mtime(path: &Path) -> Result<u64> {
        let metadata = fs::metadata(path).context("Failed to get file metadata")?;
        let mtime = metadata
            .modified()
            .context("Failed to get modification time")?
            .duration_since(UNIX_EPOCH)
            .context("Invalid modification time")?
            .as_secs();
        Ok(mtime)
    }

    /// Check if a cached entry is still valid
    pub fn is_valid(&self, path: &Path) -> bool {
        if let Some(cached) = self.entries.get(path) {
            if let Ok(current_mtime) = Self::get_mtime(path) {
                return cached.mtime == current_mtime;
            }
        }
        false
    }

    /// Get a cached entry if it's still valid
    pub fn get(&self, path: &Path) -> Option<&DesktopEntry> {
        if self.is_valid(path) {
            self.entries.get(path).map(|cached| &cached.entry)
        } else {
            None
        }
    }

    /// Insert or update a cache entry
    pub fn insert(&mut self, path: PathBuf, entry: DesktopEntry) -> Result<()> {
        let mtime = Self::get_mtime(&path)?;

        self.entries
            .insert(path.clone(), CachedFile { path, mtime, entry });

        Ok(())
    }

    /// Remove a specific entry from the cache
    #[allow(dead_code)]

    pub fn remove(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    /// Remove entries that no longer exist on disk
    pub fn prune(&mut self) {
        let mut to_remove = Vec::new();

        for path in self.entries.keys() {
            if !path.exists() {
                to_remove.push(path.clone());
            }
        }

        for path in to_remove {
            debug!("Removing deleted file from cache: {}", path.display());
            self.entries.remove(&path);
        }
    }

    /// Get all cached entries as a vector
    #[allow(dead_code)]

    pub fn get_all(&self) -> Vec<DesktopEntry> {
        self.entries
            .values()
            .map(|cached| cached.entry.clone())
            .collect()
    }

    /// Get statistics about the cache
    #[allow(dead_code)]

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            #[allow(dead_code)]

            total_entries: self.entries.len(),
            version: self.version,
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub version: u32,
}
