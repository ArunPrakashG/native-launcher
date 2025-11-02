use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

use super::browser_history::HistoryEntry;

/// Persistent SQLite index for fast browser history/bookmark search
#[derive(Debug)]
pub struct BrowserIndex {
    db_path: PathBuf,
    #[allow(dead_code)]
    conn: Arc<Mutex<Connection>>,
}

impl BrowserIndex {
    /// Create or open the browser index database
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine cache directory"))?
            .join("native-launcher");

        std::fs::create_dir_all(&cache_dir)?;
        let db_path = cache_dir.join("browser_index.db");

        debug!("Opening browser index at {:?}", db_path);
        let conn = Connection::open(&db_path)?;

        let index = Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        };

        index.init_schema()?;
        Ok(index)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS browser_entries (
                url TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                domain TEXT NOT NULL,
                visit_count INTEGER DEFAULT 0,
                last_visit INTEGER NOT NULL,
                is_bookmark INTEGER DEFAULT 0,
                favicon_path TEXT,
                indexed_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create indices for fast searching
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_title ON browser_entries(title)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_domain ON browser_entries(domain)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_visit ON browser_entries(last_visit DESC)",
            [],
        )?;

        // Metadata table to track index freshness
        conn.execute(
            "CREATE TABLE IF NOT EXISTS index_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        debug!("Browser index schema initialized");
        Ok(())
    }

    /// Rebuild entire index from browser databases
    pub fn rebuild_index(&self, entries: Vec<HistoryEntry>) -> Result<()> {
        info!("Rebuilding browser index with {} entries", entries.len());
        let start = std::time::Instant::now();

        let conn = self.conn.lock().unwrap();

        // Start transaction for better performance
        conn.execute("BEGIN TRANSACTION", [])?;

        // Clear existing data
        conn.execute("DELETE FROM browser_entries", [])?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Insert new entries
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO browser_entries 
             (url, title, domain, visit_count, last_visit, is_bookmark, favicon_path, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for entry in entries {
            stmt.execute(params![
                entry.url,
                entry.title,
                entry.domain,
                entry.visit_count,
                entry.last_visit,
                entry.is_bookmark as i64,
                entry
                    .favicon_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                now,
            ])?;
        }

        drop(stmt);

        // Update metadata
        conn.execute(
            "INSERT OR REPLACE INTO index_metadata (key, value) VALUES ('last_rebuild', ?1)",
            params![now.to_string()],
        )?;

        conn.execute("COMMIT", [])?;

        let elapsed = start.elapsed();
        info!("Browser index rebuilt in {:?}", elapsed);

        Ok(())
    }

    /// Search indexed entries (fast path)
    pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<IndexedEntry>> {
        // Quick exit for very short queries
        if query.len() < 2 {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().unwrap();

        // Tokenize query - use only first token for performance
        let first_token = query
            .split_whitespace()
            .next()
            .map(|t| format!("%{}%", t.to_lowercase()));

        let token = match first_token {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        // Search in title and domain only (skip URL for performance)
        // Prepared statements are cached by SQLite for speed
        let mut stmt = conn.prepare_cached(
            "SELECT url, title, domain, visit_count, last_visit, is_bookmark, favicon_path
             FROM browser_entries
             WHERE LOWER(title) LIKE ?1 
                OR LOWER(domain) LIKE ?1
             ORDER BY 
                (is_bookmark * 100) + 
                (visit_count * 2) + 
                (last_visit / 3600) DESC
             LIMIT ?2",
        )?;

        let mut results = Vec::new();

        // Single query instead of looping through tokens
        let entries = stmt.query_map(params![token, max_results], |row| {
            Ok(IndexedEntry {
                url: row.get(0)?,
                title: row.get(1)?,
                domain: row.get(2)?,
                visit_count: row.get(3)?,
                last_visit: row.get(4)?,
                is_bookmark: row.get::<_, i64>(5)? != 0,
                favicon_path: row.get::<_, Option<String>>(6)?.map(PathBuf::from),
            })
        })?;

        for entry in entries.filter_map(Result::ok) {
            results.push(entry);
        }

        results.truncate(max_results);
        Ok(results)
    }

    /// Get recent entries when no query provided
    fn get_recent_entries(&self, conn: &Connection, limit: usize) -> Result<Vec<IndexedEntry>> {
        let mut stmt = conn.prepare(
            "SELECT url, title, domain, visit_count, last_visit, is_bookmark, favicon_path
             FROM browser_entries
             ORDER BY last_visit DESC
             LIMIT ?1",
        )?;

        let entries = stmt.query_map(params![limit], |row| {
            Ok(IndexedEntry {
                url: row.get(0)?,
                title: row.get(1)?,
                domain: row.get(2)?,
                visit_count: row.get(3)?,
                last_visit: row.get(4)?,
                is_bookmark: row.get::<_, i64>(5)? != 0,
                favicon_path: row.get::<_, Option<String>>(6)?.map(PathBuf::from),
            })
        })?;

        Ok(entries.filter_map(Result::ok).collect())
    }

    /// Get index age in seconds
    pub fn get_index_age(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        let last_rebuild: Option<i64> = conn
            .query_row(
                "SELECT value FROM index_metadata WHERE key = 'last_rebuild'",
                [],
                |row| {
                    let value: String = row.get(0)?;
                    Ok(value.parse().unwrap_or(0))
                },
            )
            .ok();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Ok(now - last_rebuild.unwrap_or(0))
    }

    /// Check if index needs rebuild (older than 1 hour)
    pub fn needs_rebuild(&self) -> bool {
        self.get_index_age().unwrap_or(i64::MAX) > 3600 // 1 hour
    }

    /// Get total indexed entries count
    pub fn entry_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM browser_entries", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

/// Entry from the browser index (optimized for search)
#[derive(Debug, Clone)]
pub struct IndexedEntry {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub visit_count: i64,
    pub last_visit: i64,
    pub is_bookmark: bool,
    pub favicon_path: Option<PathBuf>,
}

impl From<IndexedEntry> for HistoryEntry {
    fn from(entry: IndexedEntry) -> Self {
        HistoryEntry {
            url: entry.url,
            title: entry.title,
            domain: entry.domain,
            visit_count: entry.visit_count,
            last_visit: entry.last_visit,
            is_bookmark: entry.is_bookmark,
            favicon_path: entry.favicon_path,
        }
    }
}
