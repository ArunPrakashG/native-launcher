use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

/// Usage statistics for a single application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    /// Number of times the app has been launched
    pub launch_count: usize,
    /// Timestamp of last launch (Unix timestamp in seconds)
    pub last_used: u64,
    /// Timestamp of first launch
    pub first_used: u64,
}

impl AppUsage {
    pub fn new() -> Self {
        let now = current_timestamp();
        Self {
            launch_count: 0,
            last_used: now,
            first_used: now,
        }
    }

    /// Record a launch
    pub fn record_launch(&mut self) {
        self.launch_count += 1;
        self.last_used = current_timestamp();
    }

    /// Calculate a usage score for ranking (higher = more relevant)
    pub fn score(&self) -> f64 {
        let now = current_timestamp();
        let days_since_last = ((now - self.last_used) as f64) / 86400.0; // seconds to days

        // Score formula: launch_count * recency_factor
        // Recency factor decays exponentially (half-life of 7 days)
        let recency_factor = 2.0_f64.powf(-days_since_last / 7.0);

        (self.launch_count as f64) * recency_factor
    }
}

impl Default for AppUsage {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks application usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTracker {
    /// Map from desktop file path to usage stats
    usage_data: HashMap<String, AppUsage>,

    /// Path to the cache file
    #[serde(skip)]
    cache_path: PathBuf,
}

impl UsageTracker {
    /// Create a new usage tracker
    pub fn new() -> Self {
        let cache_path = Self::default_cache_path();

        Self {
            usage_data: HashMap::new(),
            cache_path,
        }
    }

    /// Load usage data from disk
    pub fn load() -> Result<Self> {
        let cache_path = Self::default_cache_path();

        if !cache_path.exists() {
            info!("No usage cache found, starting fresh");
            return Ok(Self {
                usage_data: HashMap::new(),
                cache_path,
            });
        }

        debug!("Loading usage data from {:?}", cache_path);

        let data = fs::read(&cache_path)?;
        let mut tracker: UsageTracker = bincode::deserialize(&data)?;
        tracker.cache_path = cache_path;

        info!("Loaded usage data for {} apps", tracker.usage_data.len());
        Ok(tracker)
    }

    /// Save usage data to disk
    pub fn save(&self) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        debug!("Saving usage data to {:?}", self.cache_path);

        let encoded = bincode::serialize(&self.usage_data)?;
        fs::write(&self.cache_path, encoded)?;

        debug!("Usage data saved successfully");
        Ok(())
    }

    /// Record a launch for an application
    pub fn record_launch(&mut self, desktop_path: &str) {
        let entry = self
            .usage_data
            .entry(desktop_path.to_string())
            .or_default();

        entry.record_launch();

        debug!(
            "Recorded launch for {} (count: {}, last: {})",
            desktop_path, entry.launch_count, entry.last_used
        );

        // Save immediately (async save would be better, but keep it simple)
        if let Err(e) = self.save() {
            error!("Failed to save usage data: {}", e);
        }
    }

    /// Get usage score for an application (higher = more frequently/recently used)
    pub fn get_score(&self, desktop_path: &str) -> f64 {
        self.usage_data
            .get(desktop_path)
            .map(|usage| usage.score())
            .unwrap_or(0.0)
    }

    /// Get usage stats for an application
    #[allow(dead_code)]

    pub fn get_usage(&self, desktop_path: &str) -> Option<&AppUsage> {
        self.usage_data.get(desktop_path)
    }

    /// Default cache file path
    fn default_cache_path() -> PathBuf {
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        cache_dir.join("native-launcher").join("usage.bin")
    }

    /// Clear all usage data
    #[allow(dead_code)]

    pub fn clear(&mut self) {
        self.usage_data.clear();
        debug!("Cleared all usage data");
    }

    /// Get total number of tracked apps
    pub fn app_count(&self) -> usize {
        self.usage_data.len()
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_score_increases_with_launches() {
        let mut usage = AppUsage::new();
        let initial_score = usage.score();

        usage.record_launch();
        usage.record_launch();

        assert!(usage.score() > initial_score);
        assert_eq!(usage.launch_count, 2);
    }

    #[test]
    fn test_tracker_records_launches() {
        let mut tracker = UsageTracker::new();

        tracker.record_launch("/test/app1.desktop");
        tracker.record_launch("/test/app1.desktop");
        tracker.record_launch("/test/app2.desktop");

        assert_eq!(tracker.app_count(), 2);
        assert_eq!(
            tracker
                .get_usage("/test/app1.desktop")
                .unwrap()
                .launch_count,
            2
        );
        assert_eq!(
            tracker
                .get_usage("/test/app2.desktop")
                .unwrap()
                .launch_count,
            1
        );
    }

    #[test]
    fn test_usage_score_nonzero() {
        let usage = AppUsage::new();
        assert_eq!(usage.score(), 0.0); // No launches yet

        let mut usage2 = AppUsage::new();
        usage2.record_launch();
        assert!(usage2.score() > 0.0);
    }
}
