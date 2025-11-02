use super::browser_index::BrowserIndex;
use super::traits::{KeyboardAction, KeyboardEvent, Plugin, PluginContext, PluginResult};
use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

const CHROME_EPOCH: i64 = 11644473600; // Seconds between 1601-01-01 and 1970-01-01

#[derive(Debug, Clone)]
pub struct BrowserHistoryPlugin {
    enabled: bool,
    cache: Arc<std::sync::Mutex<CachedHistory>>,
    index: Option<Arc<BrowserIndex>>,
}

#[derive(Debug)]
struct CachedHistory {
    entries: Vec<HistoryEntry>,
    last_refresh: SystemTime,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub domain: String,
    pub visit_count: i64,
    pub last_visit: i64, // Unix timestamp
    pub favicon_path: Option<PathBuf>,
    pub is_bookmark: bool,
}

impl BrowserHistoryPlugin {
    pub fn new() -> Self {
        let index = match BrowserIndex::new() {
            Ok(idx) => {
                debug!("Browser index initialized");
                Some(Arc::new(idx))
            }
            Err(e) => {
                warn!("Failed to initialize browser index: {}", e);
                None
            }
        };

        Self {
            enabled: true,
            cache: Arc::new(std::sync::Mutex::new(CachedHistory {
                entries: Vec::new(),
                last_refresh: UNIX_EPOCH,
                ttl: Duration::from_secs(300), // 5 minutes
            })),
            index,
        }
    }

    /// Get reference to browser index for background updates
    pub fn get_index(&self) -> Option<Arc<BrowserIndex>> {
        self.index.clone()
    }

    /// Public method to fetch all history (used by indexer)
    pub fn fetch_all_history(&self) -> Vec<HistoryEntry> {
        self.fetch_history()
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@tabs") {
            rest
        } else if let Some(rest) = query.strip_prefix("@history") {
            rest
        } else {
            query
        }
    }

    fn get_cached_or_refresh(&self) -> Vec<HistoryEntry> {
        let mut cache = self.cache.lock().unwrap();
        let now = SystemTime::now();

        if now
            .duration_since(cache.last_refresh)
            .unwrap_or(Duration::MAX)
            > cache.ttl
        {
            debug!("Browser history cache expired, refreshing...");
            cache.entries = self.fetch_history();
            cache.last_refresh = now;
        }

        cache.entries.clone()
    }

    fn fetch_history(&self) -> Vec<HistoryEntry> {
        let mut all_entries = Vec::new();

        // Try Chromium-based browsers
        if let Some(entries) = self.fetch_chrome_history() {
            all_entries.extend(entries);
        }
        if let Some(entries) = self.fetch_brave_history() {
            all_entries.extend(entries);
        }
        if let Some(entries) = self.fetch_edge_history() {
            all_entries.extend(entries);
        }
        if let Some(entries) = self.fetch_vivaldi_history() {
            all_entries.extend(entries);
        }
        if let Some(entries) = self.fetch_opera_history() {
            all_entries.extend(entries);
        }

        // Try Firefox
        if let Some(entries) = self.fetch_firefox_history() {
            all_entries.extend(entries);
        }

        // Fetch bookmarks from all browsers
        all_entries.extend(self.fetch_all_bookmarks());

        // Deduplicate by URL, keeping most recent
        let mut seen = std::collections::HashMap::new();
        for entry in all_entries {
            seen.entry(entry.url.clone())
                .and_modify(|e: &mut HistoryEntry| {
                    if entry.last_visit > e.last_visit {
                        *e = entry.clone();
                    }
                })
                .or_insert(entry);
        }

        let mut deduplicated: Vec<_> = seen.into_values().collect();
        deduplicated.sort_by(|a, b| b.last_visit.cmp(&a.last_visit));
        deduplicated.truncate(100); // Keep top 100 most recent

        debug!(
            "Fetched {} unique browser history entries",
            deduplicated.len()
        );
        deduplicated
    }

    fn fetch_chrome_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let history_path = home.join(".config/google-chrome/Default/History");
        self.read_chromium_history(&history_path, "Chrome")
    }

    fn fetch_brave_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let history_path = home.join(".config/BraveSoftware/Brave-Browser/Default/History");
        self.read_chromium_history(&history_path, "Brave")
    }

    fn fetch_edge_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let history_path = home.join(".config/microsoft-edge/Default/History");
        self.read_chromium_history(&history_path, "Edge")
    }

    fn fetch_vivaldi_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let history_path = home.join(".config/vivaldi/Default/History");
        self.read_chromium_history(&history_path, "Vivaldi")
    }

    fn fetch_opera_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        // Opera uses different path structure
        let history_path = home.join(".config/opera/History");
        self.read_chromium_history(&history_path, "Opera")
    }

    fn read_chromium_history(
        &self,
        path: &PathBuf,
        browser_name: &str,
    ) -> Option<Vec<HistoryEntry>> {
        if !path.exists() {
            debug!("{} history not found at {:?}", browser_name, path);
            return None;
        }

        // Copy database to temp location to avoid locking issues
        let temp_path = std::env::temp_dir().join(format!(
            "{}-history-{}.db",
            browser_name.to_lowercase(),
            std::process::id()
        ));
        if let Err(e) = std::fs::copy(path, &temp_path) {
            warn!("Failed to copy {} history database: {}", browser_name, e);
            return None;
        }

        // Extract browser_dir from path for favicon resolution
        let browser_dir = path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(browser_name);

        let result = self.query_chromium_db(&temp_path, browser_dir);
        let _ = std::fs::remove_file(&temp_path); // Clean up temp file

        result
    }

    fn query_chromium_db(&self, db_path: &PathBuf, browser_dir: &str) -> Option<Vec<HistoryEntry>> {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;

        let mut stmt = conn
            .prepare(
                "SELECT url, title, visit_count, last_visit_time 
             FROM urls 
             ORDER BY last_visit_time DESC 
             LIMIT 100",
            )
            .ok()?;

        let entries = stmt
            .query_map([], |row| {
                let url: String = row.get(0)?;
                let title: String = row.get(1).unwrap_or_else(|_| url.clone());
                let visit_count: i64 = row.get(2).unwrap_or(0);
                let chrome_time: i64 = row.get(3)?;

                // Convert Chrome timestamp (microseconds since 1601) to Unix timestamp
                let unix_time = (chrome_time / 1_000_000) - CHROME_EPOCH;

                let domain = extract_domain(&url);

                Ok((url, title, domain, visit_count, unix_time))
            })
            .ok()?;

        let mut results = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let (url, title, domain, visit_count, last_visit) = entry;

            // Try to resolve favicon (cached lookups are fast)
            let favicon_path = self.resolve_favicon(&url, browser_dir);

            results.push(HistoryEntry {
                title: if title.is_empty() { url.clone() } else { title },
                url,
                domain,
                visit_count,
                last_visit,
                favicon_path,
                is_bookmark: false,
            });
        }

        Some(results)
    }

    fn fetch_firefox_history(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let firefox_dir = home.join(".mozilla/firefox");

        if !firefox_dir.exists() {
            debug!("Firefox directory not found");
            return None;
        }

        // Find default profile
        let profile = std::fs::read_dir(&firefox_dir)
            .ok()?
            .filter_map(Result::ok)
            .find(|entry| {
                entry.file_name().to_string_lossy().contains(".default")
                    || entry
                        .file_name()
                        .to_string_lossy()
                        .contains(".default-release")
            })?;

        let places_path = profile.path().join("places.sqlite");
        if !places_path.exists() {
            debug!("Firefox places.sqlite not found at {:?}", places_path);
            return None;
        }

        // Copy database to temp location
        let temp_path =
            std::env::temp_dir().join(format!("firefox-places-{}.db", std::process::id()));
        if let Err(e) = std::fs::copy(&places_path, &temp_path) {
            warn!("Failed to copy Firefox history database: {}", e);
            return None;
        }

        let result = self.query_firefox_db(&temp_path);
        let _ = std::fs::remove_file(&temp_path); // Clean up

        result
    }

    fn query_firefox_db(&self, db_path: &PathBuf) -> Option<Vec<HistoryEntry>> {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;

        let mut stmt = conn
            .prepare(
                "SELECT url, title, visit_count, last_visit_date 
             FROM moz_places 
             WHERE last_visit_date IS NOT NULL 
             ORDER BY last_visit_date DESC 
             LIMIT 100",
            )
            .ok()?;

        let entries = stmt
            .query_map([], |row| {
                let url: String = row.get(0)?;
                let title: Option<String> = row.get(1).ok();
                let visit_count: i64 = row.get(2).unwrap_or(0);
                let firefox_time: i64 = row.get(3)?;

                // Firefox stores time in microseconds since Unix epoch
                let unix_time = firefox_time / 1_000_000;

                let domain = extract_domain(&url);

                Ok(HistoryEntry {
                    title: title
                        .filter(|t| !t.is_empty())
                        .unwrap_or_else(|| url.clone()),
                    url,
                    domain,
                    visit_count,
                    last_visit: unix_time,
                    favicon_path: None,
                    is_bookmark: false,
                })
            })
            .ok()?;

        Some(entries.filter_map(Result::ok).collect())
    }

    fn search_entries(&self, filter: &str, max: usize) -> Vec<HistoryEntry> {
        let entries = self.get_cached_or_refresh();

        if filter.is_empty() {
            return entries.into_iter().take(max).collect();
        }

        let tokens: Vec<String> = filter
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_lowercase())
            .collect();

        entries
            .into_iter()
            .filter(|entry| {
                let haystack = format!(
                    "{} {} {}",
                    entry.title.to_lowercase(),
                    entry.url.to_lowercase(),
                    entry.domain.to_lowercase()
                );
                tokens.iter().all(|token| haystack.contains(token))
            })
            .take(max)
            .collect()
    }

    fn build_url_open_command(&self, url: &str) -> String {
        // Use xdg-open or equivalent to open URL in default browser
        format!("xdg-open {}", shell_escape(url))
    }

    fn fetch_all_bookmarks(&self) -> Vec<HistoryEntry> {
        let mut bookmarks = Vec::new();

        // Chromium-based browsers store bookmarks in JSON
        bookmarks.extend(
            self.fetch_chromium_bookmarks("google-chrome", "Chrome")
                .unwrap_or_default(),
        );
        bookmarks.extend(
            self.fetch_chromium_bookmarks("BraveSoftware/Brave-Browser", "Brave")
                .unwrap_or_default(),
        );
        bookmarks.extend(
            self.fetch_chromium_bookmarks("microsoft-edge", "Edge")
                .unwrap_or_default(),
        );
        bookmarks.extend(
            self.fetch_chromium_bookmarks("vivaldi", "Vivaldi")
                .unwrap_or_default(),
        );
        bookmarks.extend(
            self.fetch_chromium_bookmarks("opera", "Opera")
                .unwrap_or_default(),
        );

        // Firefox bookmarks
        bookmarks.extend(self.fetch_firefox_bookmarks().unwrap_or_default());

        bookmarks
    }

    fn fetch_chromium_bookmarks(
        &self,
        browser_dir: &str,
        browser_name: &str,
    ) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let bookmarks_path = home.join(format!(".config/{}/Default/Bookmarks", browser_dir));

        if !bookmarks_path.exists() {
            debug!(
                "{} bookmarks not found at {:?}",
                browser_name, bookmarks_path
            );
            return None;
        }

        let contents = std::fs::read_to_string(&bookmarks_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&contents).ok()?;

        let mut bookmarks = Vec::new();
        self.extract_chromium_bookmarks_recursive(&json["roots"], &mut bookmarks);

        debug!("Found {} {} bookmarks", bookmarks.len(), browser_name);
        Some(bookmarks)
    }

    fn extract_chromium_bookmarks_recursive(
        &self,
        node: &serde_json::Value,
        bookmarks: &mut Vec<HistoryEntry>,
    ) {
        if let Some(obj) = node.as_object() {
            // Check if this is a bookmark (has url)
            if let Some(url) = obj.get("url").and_then(|u| u.as_str()) {
                let title = obj
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(url)
                    .to_string();

                let domain = extract_domain(url);

                bookmarks.push(HistoryEntry {
                    title,
                    url: url.to_string(),
                    domain,
                    visit_count: 0, // Bookmarks don't have visit counts
                    last_visit: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    favicon_path: None,
                    is_bookmark: true,
                });
            }

            // Recurse into children
            if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    self.extract_chromium_bookmarks_recursive(child, bookmarks);
                }
            }
        }
    }

    fn fetch_firefox_bookmarks(&self) -> Option<Vec<HistoryEntry>> {
        let home = dirs::home_dir()?;
        let firefox_dir = home.join(".mozilla/firefox");

        if !firefox_dir.exists() {
            return None;
        }

        let profile = std::fs::read_dir(&firefox_dir)
            .ok()?
            .filter_map(Result::ok)
            .find(|entry| {
                entry.file_name().to_string_lossy().contains(".default")
                    || entry
                        .file_name()
                        .to_string_lossy()
                        .contains(".default-release")
            })?;

        let places_path = profile.path().join("places.sqlite");
        if !places_path.exists() {
            return None;
        }

        let temp_path =
            std::env::temp_dir().join(format!("firefox-bookmarks-{}.db", std::process::id()));
        if let Err(e) = std::fs::copy(&places_path, &temp_path) {
            warn!("Failed to copy Firefox bookmarks database: {}", e);
            return None;
        }

        let result = self.query_firefox_bookmarks(&temp_path);
        let _ = std::fs::remove_file(&temp_path);

        result
    }

    fn query_firefox_bookmarks(&self, db_path: &PathBuf) -> Option<Vec<HistoryEntry>> {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;

        let mut stmt = conn
            .prepare(
                "SELECT mp.url, mp.title 
             FROM moz_bookmarks mb 
             JOIN moz_places mp ON mb.fk = mp.id 
             WHERE mb.type = 1 AND mp.url IS NOT NULL",
            )
            .ok()?;

        let entries = stmt
            .query_map([], |row| {
                let url: String = row.get(0)?;
                let title: Option<String> = row.get(1).ok();

                let domain = extract_domain(&url);

                Ok(HistoryEntry {
                    title: title
                        .filter(|t| !t.is_empty())
                        .unwrap_or_else(|| url.clone()),
                    url,
                    domain,
                    visit_count: 0,
                    last_visit: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    favicon_path: None,
                    is_bookmark: true,
                })
            })
            .ok()?;

        Some(entries.filter_map(Result::ok).collect())
    }

    fn resolve_favicon(&self, url: &str, browser_dir: &str) -> Option<PathBuf> {
        let home = dirs::home_dir()?;

        // Try Chromium Favicons database
        let favicons_db = home.join(format!(".config/{}/Default/Favicons", browser_dir));
        if favicons_db.exists() {
            if let Some(path) = self.extract_favicon_from_chromium(&favicons_db, url) {
                return Some(path);
            }
        }

        None
    }

    fn extract_favicon_from_chromium(&self, db_path: &PathBuf, url: &str) -> Option<PathBuf> {
        let temp_path = std::env::temp_dir().join(format!("favicons-{}.db", std::process::id()));
        std::fs::copy(db_path, &temp_path).ok()?;

        let conn =
            Connection::open_with_flags(&temp_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;

        // Query favicon data for URL
        let mut stmt = conn
            .prepare(
                "SELECT image_data FROM favicon_bitmaps 
             JOIN icon_mapping ON favicon_bitmaps.icon_id = icon_mapping.icon_id 
             WHERE icon_mapping.page_url = ?1 
             LIMIT 1",
            )
            .ok()?;

        let favicon_data: Vec<u8> = stmt.query_row([url], |row| row.get(0)).ok()?;

        // Save favicon to temp cache
        let cache_dir = std::env::temp_dir().join("native-launcher-favicons");
        std::fs::create_dir_all(&cache_dir).ok()?;

        let domain = extract_domain(url);
        let favicon_path = cache_dir.join(format!("{}.png", domain.replace('/', "_")));
        std::fs::write(&favicon_path, favicon_data).ok()?;

        let _ = std::fs::remove_file(&temp_path);

        Some(favicon_path)
    }
}

impl Plugin for BrowserHistoryPlugin {
    fn name(&self) -> &str {
        "browser_history"
    }

    fn description(&self) -> &str {
        "Recent browser tabs and history via @tabs or @history"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@tabs", "@history"]
    }

    fn should_handle(&self, query: &str) -> bool {
        let has_prefix = query.starts_with("@tabs") || query.starts_with("@history");

        if has_prefix {
            return true; // Always handle prefixed queries
        }

        // For global search: require minimum 4 characters to reduce keystroke lag
        // Increased from 3 to 4 for better performance when typing fast
        let trimmed = query.trim();
        !trimmed.starts_with("@") && trimmed.len() >= 4
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let has_prefix = query.starts_with("@tabs") || query.starts_with("@history");
        let filter = if has_prefix {
            self.strip_prefix(query).trim()
        } else {
            query.trim()
        };

        // Quick exit for short queries in global search (performance)
        if !has_prefix && filter.len() < 4 {
            return Ok(Vec::new());
        }

        debug!(
            "Browser plugin searching for: '{}' (has_prefix: {})",
            filter, has_prefix
        );

        // Try fast path with persistent index first
        let entries = if let Some(ref index) = self.index {
            match index.search(filter, context.max_results) {
                Ok(indexed) => {
                    debug!("Retrieved {} results from browser index", indexed.len());
                    indexed.into_iter().map(|e| e.into()).collect()
                }
                Err(e) => {
                    warn!(
                        "Failed to search browser index: {}, falling back to live fetch",
                        e
                    );
                    self.search_entries(filter, context.max_results)
                }
            }
        } else {
            warn!("No browser index available, using in-memory cache");
            self.search_entries(filter, context.max_results)
        };

        // For global search (no prefix), limit results to avoid overwhelming
        let max = if has_prefix {
            context.max_results
        } else {
            2 // Only show top 2 browser results in global search (reduced from 3 for performance)
        };

        let entries: Vec<_> = entries.into_iter().take(max).collect();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut results = Vec::with_capacity(entries.len());
        for entry in entries {
            // Score based on recency and visit count
            // Boost bookmarks slightly
            let age_hours = ((now - entry.last_visit) / 3600).max(1);
            let recency_score = 1000 / age_hours; // More recent = higher score
            let popularity_score = entry.visit_count.min(100);
            let bookmark_boost = if entry.is_bookmark { 50 } else { 0 };
            let score = recency_score + popularity_score + bookmark_boost;

            // Build subtitle with bookmark indicator
            let subtitle = if entry.is_bookmark {
                format!("★ {} • Bookmarked", entry.domain)
            } else if entry.domain != entry.url {
                format!("{} • {} visits", entry.domain, entry.visit_count)
            } else {
                format!("{} visits", entry.visit_count)
            };

            // Use favicon if available, otherwise default icon
            let icon = if let Some(ref favicon_path) = entry.favicon_path {
                favicon_path.to_string_lossy().to_string()
            } else {
                "web-browser".to_string()
            };

            let result = PluginResult::new(
                entry.title.clone(),
                self.build_url_open_command(&entry.url),
                self.name().to_string(),
            )
            .with_subtitle(subtitle)
            .with_icon(icon)
            .with_score(score);

            results.push(result);
        }

        Ok(results)
    }

    fn priority(&self) -> i32 {
        280 // Between files (200) and emoji (300)
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle_keyboard_event(&self, _event: &KeyboardEvent) -> KeyboardAction {
        KeyboardAction::None
    }
}

fn extract_domain(url: &str) -> String {
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            after_protocol[..end].to_string()
        } else {
            after_protocol.to_string()
        }
    } else {
        url.to_string()
    }
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let mut escaped = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(extract_domain("http://sub.example.com/"), "sub.example.com");
        assert_eq!(extract_domain("example.com"), "example.com");
    }

    #[test]
    fn test_should_handle_prefix() {
        let plugin = BrowserHistoryPlugin::new();
        // Handles prefixed queries
        assert!(plugin.should_handle("@tabs foo"));
        assert!(plugin.should_handle("@history bar"));

        // Global search requires 4+ characters to reduce keystroke lag
        assert!(!plugin.should_handle("g")); // Too short
        assert!(!plugin.should_handle("gi")); // Too short
        assert!(!plugin.should_handle("git")); // 3 chars - still too short
        assert!(plugin.should_handle("gith")); // 4 chars - OK
        assert!(plugin.should_handle("github")); // Long enough

        // Exclude queries with OTHER prefixes
        assert!(!plugin.should_handle("@calc 1+1"));
        assert!(!plugin.should_handle("@ssh server"));
    }

    #[test]
    fn test_strip_prefix() {
        let plugin = BrowserHistoryPlugin::new();
        assert_eq!(plugin.strip_prefix("@tabs github"), " github");
        assert_eq!(plugin.strip_prefix("@history rust"), " rust");
        assert_eq!(plugin.strip_prefix("plain"), "plain");
    }

    #[test]
    fn test_build_url_command() {
        let plugin = BrowserHistoryPlugin::new();
        let cmd = plugin.build_url_open_command("https://example.com/test");
        assert!(cmd.contains("xdg-open"));
        assert!(cmd.contains("example.com"));
    }

    #[test]
    fn test_search_with_index() {
        use crate::config::ConfigLoader;
        use crate::plugins::traits::{Plugin, PluginContext};

        let plugin = BrowserHistoryPlugin::new();
        let config_loader = ConfigLoader::new();
        let context = PluginContext::new(10, config_loader.config());

        // Test should_handle
        assert!(plugin.should_handle("@tabs"));
        assert!(plugin.should_handle("@tabs mozilla"));
        assert!(plugin.should_handle("mozilla")); // 7 chars
        assert!(!plugin.should_handle("mo")); // Too short

        // Test search - will use index if available
        let results = plugin.search("@tabs test", &context);
        assert!(results.is_ok(), "Search should not error");
        println!(
            "@tabs test: {:?} results",
            results.as_ref().map(|r| r.len())
        );

        // Global search test
        let results = plugin.search("mozilla", &context);
        assert!(results.is_ok(), "Global search should not error");
        println!("mozilla: {:?} results", results.as_ref().map(|r| r.len()));
    }
}
