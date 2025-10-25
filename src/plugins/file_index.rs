/// System-wide file indexing using native Linux tools
///
/// This module provides fast file search by leveraging native Linux indexing tools:
/// - plocate/mlocate/locate (primary - pre-indexed database)
/// - fd-find (optional - fast modern find alternative)  
/// - find (fallback - always available)
///
/// Performance targets:
/// - Search latency: <100ms for locate, <500ms for find
/// - Result limit: 50 files max
/// - Cache TTL: 2 minutes
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};

/// Available file indexing backend
#[derive(Debug, Clone, Copy, PartialEq)]
enum IndexBackend {
    /// plocate - fastest, modern locate implementation
    Plocate,
    /// mlocate - traditional locate with security
    Mlocate,
    /// locate - classic file index
    Locate,
    /// fd - modern find alternative (optional)
    Fd,
    /// find - fallback, always available but slow
    Find,
}

impl IndexBackend {
    /// Get command name for this backend
    fn command(&self) -> &str {
        match self {
            IndexBackend::Plocate => "plocate",
            IndexBackend::Mlocate => "mlocate",
            IndexBackend::Locate => "locate",
            IndexBackend::Fd => "fd",
            IndexBackend::Find => "find",
        }
    }

    /// Check if this backend is available
    fn is_available(&self) -> bool {
        Command::new(self.command())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get search performance tier (1=fastest, 4=slowest)
    fn performance_tier(&self) -> u8 {
        match self {
            IndexBackend::Plocate => 1,
            IndexBackend::Mlocate | IndexBackend::Locate => 2,
            IndexBackend::Fd => 2,
            IndexBackend::Find => 4,
        }
    }
}

/// Cached search result
#[derive(Debug, Clone)]
struct CachedSearch {
    results: Vec<PathBuf>,
    timestamp: SystemTime,
}

impl CachedSearch {
    /// Check if cache entry is still valid
    fn is_valid(&self, ttl: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.timestamp)
            .map(|age| age < ttl)
            .unwrap_or(false)
    }
}

/// File indexing service for system-wide file search
pub struct FileIndexService {
    /// Active backend
    backend: IndexBackend,
    /// Search result cache (query -> results)
    cache: Arc<Mutex<HashMap<String, CachedSearch>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// Max results to return
    max_results: usize,
    /// Search timeout
    timeout: Duration,
}

impl std::fmt::Debug for FileIndexService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileIndexService")
            .field("backend", &self.backend)
            .field("cache_ttl", &self.cache_ttl)
            .field("max_results", &self.max_results)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl FileIndexService {
    /// Create a new file index service with auto-detected backend
    pub fn new() -> Self {
        let backend = Self::detect_backend();
        debug!("File index backend: {:?} ({})", backend, backend.command());

        Self {
            backend,
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_secs(120), // 2 minutes
            max_results: 50,
            timeout: Duration::from_secs(3),
        }
    }

    /// Detect best available indexing backend
    fn detect_backend() -> IndexBackend {
        // Try backends in order of preference
        let backends = [
            IndexBackend::Plocate,
            IndexBackend::Mlocate,
            IndexBackend::Locate,
            IndexBackend::Fd,
            IndexBackend::Find,
        ];

        for backend in backends {
            if backend.is_available() {
                return backend;
            }
        }

        // Fallback to find (should always be available)
        IndexBackend::Find
    }

    /// Search for files matching query
    ///
    /// Returns paths sorted by relevance (exact match > prefix > contains)
    pub fn search(&self, query: &str) -> Result<Vec<PathBuf>> {
        if query.is_empty() || query.len() < 2 {
            return Ok(Vec::new());
        }

        // Check cache first
        let cache_key = format!("{}:{}", self.backend.command(), query);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                if cached.is_valid(self.cache_ttl) {
                    debug!("Cache hit for query: {}", query);
                    return Ok(cached.results.clone());
                }
            }
        }

        // Perform search
        let results = match self.backend {
            IndexBackend::Plocate | IndexBackend::Mlocate | IndexBackend::Locate => {
                self.search_locate(query)?
            }
            IndexBackend::Fd => self.search_fd(query)?,
            IndexBackend::Find => self.search_find(query)?,
        };

        // Cache results
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                cache_key,
                CachedSearch {
                    results: results.clone(),
                    timestamp: SystemTime::now(),
                },
            );

            // Limit cache size
            if cache.len() > 100 {
                // Remove oldest entries
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, v)| v.timestamp);
                if let Some((key, _)) = entries.first() {
                    let key = (*key).clone();
                    cache.remove(&key);
                }
            }
        }

        Ok(results)
    }

    /// Search using locate/mlocate/plocate
    fn search_locate(&self, query: &str) -> Result<Vec<PathBuf>> {
        let output = Command::new(self.backend.command())
            .arg("--limit")
            .arg(self.max_results.to_string())
            .arg("--ignore-case")
            .arg("--basename") // Match only filename, not full path
            .arg(query)
            .output()
            .context("Failed to execute locate")?;

        if !output.status.success() {
            // Database might not exist yet (updatedb not run)
            debug!("locate search failed, falling back to find");
            return self.search_find(query);
        }

        let results = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<PathBuf> = results
            .lines()
            .filter_map(|line| {
                let path = PathBuf::from(line.trim());
                // Only return existing, accessible files
                if path.exists() && !Self::is_hidden(&path) {
                    Some(path)
                } else {
                    None
                }
            })
            .take(self.max_results)
            .collect();

        debug!("locate found {} results for '{}'", paths.len(), query);
        Ok(self.sort_by_relevance(paths, query))
    }

    /// Search using fd (if available)
    fn search_fd(&self, query: &str) -> Result<Vec<PathBuf>> {
        // Use home directory as search root for performance
        let search_root = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let output = Command::new("fd")
            .arg("--max-results")
            .arg(self.max_results.to_string())
            .arg("--ignore-case")
            .arg("--type")
            .arg("f") // Files only
            .arg("--hidden") // Include hidden files
            .arg("--follow") // Follow symlinks
            .arg(query)
            .arg(&search_root)
            .output()
            .context("Failed to execute fd")?;

        if !output.status.success() {
            warn!("fd search failed, falling back to find");
            return self.search_find(query);
        }

        let results = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<PathBuf> = results
            .lines()
            .filter_map(|line| {
                let path = PathBuf::from(line.trim());
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            })
            .take(self.max_results)
            .collect();

        debug!("fd found {} results for '{}'", paths.len(), query);
        Ok(self.sort_by_relevance(paths, query))
    }

    /// Search using find (fallback, slower but always available)
    fn search_find(&self, query: &str) -> Result<Vec<PathBuf>> {
        // Search in common directories only (not entire filesystem - too slow)
        let search_paths = Self::get_search_paths();

        let mut all_results = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            let output = Command::new("find")
                .arg(&search_path)
                .arg("-type")
                .arg("f") // Files only
                .arg("-iname") // Case-insensitive name match
                .arg(format!("*{}*", query))
                .arg("-maxdepth")
                .arg("5") // Limit depth for performance
                .arg("-print")
                .output()
                .context("Failed to execute find")?;

            if output.status.success() {
                let results = String::from_utf8_lossy(&output.stdout);
                for line in results.lines() {
                    if all_results.len() >= self.max_results {
                        break;
                    }

                    let path = PathBuf::from(line.trim());
                    if path.exists() && !Self::is_hidden(&path) {
                        all_results.push(path);
                    }
                }
            }

            if all_results.len() >= self.max_results {
                break;
            }
        }

        debug!("find found {} results for '{}'", all_results.len(), query);
        Ok(self.sort_by_relevance(all_results, query))
    }

    /// Get common search paths for find fallback
    fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // User home directory (highest priority)
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Documents"));
            paths.push(home.join("Downloads"));
            paths.push(home.join("Desktop"));
            paths.push(home.join("Pictures"));
            paths.push(home.join("Videos"));
            paths.push(home.join("Music"));
            paths.push(home); // Home itself (last to search subfolders first)
        }

        // Common system paths
        paths.push(PathBuf::from("/usr/share"));
        paths.push(PathBuf::from("/opt"));

        paths
    }

    /// Check if path is hidden (starts with .)
    fn is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
    }

    /// Sort results by relevance to query
    ///
    /// Scoring:
    /// - Exact filename match: 1000
    /// - Filename starts with query: 800
    /// - Filename contains query: 600
    /// - Parent directory contains query: 400
    fn sort_by_relevance(&self, mut paths: Vec<PathBuf>, query: &str) -> Vec<PathBuf> {
        let query_lower = query.to_lowercase();

        paths.sort_by_cached_key(|path| {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let score = if file_name == query_lower {
                1000 // Exact match
            } else if file_name.starts_with(&query_lower) {
                800 // Prefix match
            } else if file_name.contains(&query_lower) {
                600 // Contains match
            } else {
                // Check parent directory
                let parent = path
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if parent.contains(&query_lower) {
                    400 // Parent directory match
                } else {
                    200 // Other match
                }
            };

            // Negate for descending sort
            -score
        });

        paths
    }

    /// Async search for files (non-blocking)
    ///
    /// Performs file search in background thread and calls callback with results.
    /// Checks cache synchronously first for instant results if available.
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `callback` - Function to call with results (runs on GTK main thread)
    ///
    /// # Returns
    /// * `Some(Vec<PathBuf>)` - Cached results if available immediately
    /// * `None` - Search running in background, callback will be called
    #[allow(dead_code)]
    pub fn search_async<F>(&self, query: String, callback: F) -> Option<Vec<PathBuf>>
    where
        F: FnOnce(Result<Vec<PathBuf>>) + Send + 'static,
    {
        if query.is_empty() || query.len() < 2 {
            return Some(Vec::new());
        }

        // Check cache first (synchronous, fast)
        let cache_key = format!("{}:{}", self.backend.command(), &query);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                if cached.is_valid(self.cache_ttl) {
                    debug!("Async search: cache hit for '{}'", query);
                    return Some(cached.results.clone());
                }
            }
        }

        // Cache miss - search in background
        debug!("Async search: starting background search for '{}'", query);

        let backend = self.backend;
        let max_results = self.max_results;
        let cache = self.cache.clone();
        let _cache_ttl = self.cache_ttl;

        std::thread::spawn(move || {
            // Perform search in background thread
            let results = match backend {
                IndexBackend::Plocate | IndexBackend::Mlocate | IndexBackend::Locate => {
                    Self::search_locate_static(backend, &query, max_results)
                }
                IndexBackend::Fd => Self::search_fd_static(&query, max_results),
                IndexBackend::Find => Self::search_find_static(&query, max_results),
            };

            // Cache successful results
            if let Ok(ref paths) = results {
                if let Ok(mut cache_lock) = cache.lock() {
                    let sorted = Self::sort_by_relevance_static(paths.clone(), &query);
                    cache_lock.insert(
                        cache_key.clone(),
                        CachedSearch {
                            results: sorted.clone(),
                            timestamp: SystemTime::now(),
                        },
                    );

                    // LRU cache eviction
                    if cache_lock.len() > 100 {
                        let mut entries: Vec<_> = cache_lock.iter().collect();
                        entries.sort_by_key(|(_, v)| v.timestamp);
                        if let Some((key, _)) = entries.first() {
                            let key = (*key).clone();
                            cache_lock.remove(&key);
                        }
                    }
                }
            }

            // Sort results by relevance
            let final_results = results.map(|paths| Self::sort_by_relevance_static(paths, &query));

            // Call callback on GTK main thread
            gtk4::glib::idle_add_once(move || {
                callback(final_results);
            });
        });

        None // Results will arrive via callback
    }

    /// Static helper for locate search (used by async)
    #[allow(dead_code)]
    fn search_locate_static(
        backend: IndexBackend,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<PathBuf>> {
        let output = Command::new(backend.command())
            .arg("--limit")
            .arg(max_results.to_string())
            .arg("--ignore-case")
            .arg("--basename")
            .arg(query)
            .output()
            .context("Failed to execute locate")?;

        if !output.status.success() {
            debug!("locate failed in async search, falling back to find");
            return Self::search_find_static(query, max_results);
        }

        let results = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<PathBuf> = results
            .lines()
            .filter_map(|line| {
                let path = PathBuf::from(line.trim());
                if path.exists() && !Self::is_hidden(&path) {
                    Some(path)
                } else {
                    None
                }
            })
            .take(max_results)
            .collect();

        Ok(paths)
    }

    /// Static helper for fd search (used by async)
    #[allow(dead_code)]
    fn search_fd_static(query: &str, max_results: usize) -> Result<Vec<PathBuf>> {
        let search_root = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let output = Command::new("fd")
            .arg("--max-results")
            .arg(max_results.to_string())
            .arg("--ignore-case")
            .arg("--type")
            .arg("f")
            .arg("--hidden")
            .arg("--follow")
            .arg(query)
            .arg(&search_root)
            .output()
            .context("Failed to execute fd")?;

        let results = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<PathBuf> = results
            .lines()
            .filter_map(|line| {
                let path = PathBuf::from(line.trim());
                if path.exists() && !Self::is_hidden(&path) {
                    Some(path)
                } else {
                    None
                }
            })
            .take(max_results)
            .collect();

        Ok(paths)
    }

    /// Static helper for find search (used by async)
    #[allow(dead_code)]
    fn search_find_static(query: &str, max_results: usize) -> Result<Vec<PathBuf>> {
        let search_paths = Self::get_search_paths();
        let mut all_results = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            let output = Command::new("find")
                .arg(&search_path)
                .arg("-maxdepth")
                .arg("5")
                .arg("-type")
                .arg("f")
                .arg("-iname")
                .arg(format!("*{}*", query))
                .output()
                .context("Failed to execute find")?;

            let results = String::from_utf8_lossy(&output.stdout);
            for line in results.lines() {
                let path = PathBuf::from(line.trim());
                if path.exists() && !Self::is_hidden(&path) {
                    all_results.push(path);
                    if all_results.len() >= max_results {
                        return Ok(all_results);
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// Static helper for relevance sorting (used by async)
    #[allow(dead_code)]
    fn sort_by_relevance_static(mut paths: Vec<PathBuf>, query: &str) -> Vec<PathBuf> {
        let query_lower = query.to_lowercase();

        paths.sort_by_cached_key(|path| {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let score = if file_name == query_lower {
                1000 // Exact match
            } else if file_name.starts_with(&query_lower) {
                800 // Prefix match
            } else if file_name.contains(&query_lower) {
                600 // Contains match
            } else {
                let parent = path
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if parent.contains(&query_lower) {
                    400 // Parent directory match
                } else {
                    200 // Other match
                }
            };

            -score // Descending sort
        });

        paths
    }

    /// Clear search cache
    #[allow(dead_code)] // Utility method for manual cache management
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
            debug!("File index cache cleared");
        }
    }

    /// Get backend info for debugging
    pub fn backend_info(&self) -> String {
        format!(
            "{} (tier {})",
            self.backend.command(),
            self.backend.performance_tier()
        )
    }

    /// Get cache statistics
    #[allow(dead_code)] // Utility method for debugging
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.lock() {
            let total = cache.len();
            let valid = cache
                .values()
                .filter(|c| c.is_valid(self.cache_ttl))
                .count();
            (valid, total)
        } else {
            (0, 0)
        }
    }
}

impl Default for FileIndexService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let service = FileIndexService::new();
        // Should detect at least find
        assert!(
            service.backend == IndexBackend::Find
                || service.backend == IndexBackend::Locate
                || service.backend == IndexBackend::Mlocate
                || service.backend == IndexBackend::Plocate
                || service.backend == IndexBackend::Fd
        );
    }

    #[test]
    fn test_is_hidden() {
        assert!(FileIndexService::is_hidden(&PathBuf::from(".hidden")));
        assert!(FileIndexService::is_hidden(&PathBuf::from("/home/.config")));
        assert!(!FileIndexService::is_hidden(&PathBuf::from("visible.txt")));
        assert!(!FileIndexService::is_hidden(&PathBuf::from(
            "/home/file.txt"
        )));
    }

    #[test]
    fn test_cache_invalidation() {
        let cached = CachedSearch {
            results: vec![PathBuf::from("/test")],
            timestamp: SystemTime::now() - Duration::from_secs(300), // 5 minutes ago
        };

        assert!(!cached.is_valid(Duration::from_secs(120))); // 2 minute TTL
        assert!(cached.is_valid(Duration::from_secs(600))); // 10 minute TTL
    }

    #[test]
    fn test_search_paths() {
        let paths = FileIndexService::get_search_paths();
        assert!(!paths.is_empty());
        // Should include home directory
        if let Some(home) = dirs::home_dir() {
            assert!(paths.contains(&home));
        }
    }

    #[test]
    fn test_sort_by_relevance() {
        let service = FileIndexService::new();
        let paths = vec![
            PathBuf::from("/home/user/documents/test.txt"),
            PathBuf::from("/home/user/test-file.pdf"),
            PathBuf::from("/home/user/Downloads/test"),
            PathBuf::from("/opt/testing/file.txt"),
        ];

        let sorted = service.sort_by_relevance(paths, "test");

        // Exact filename match should be first
        assert_eq!(sorted[0].file_name().unwrap().to_str().unwrap(), "test");
    }

    #[test]
    fn test_empty_query() {
        let service = FileIndexService::new();
        let results = service.search("").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_short_query() {
        let service = FileIndexService::new();
        let results = service.search("a").unwrap();
        assert!(results.is_empty());
    }
}
