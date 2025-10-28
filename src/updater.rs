use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

const VERSION_URL: &str =
    "https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/VERSION";
const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/CHANGELOG.md";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub changelog: Option<String>,
}

/// Last update check timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateCheckCache {
    last_check: SystemTime,
    last_known_version: String,
}

impl UpdateCheckCache {
    fn new(version: String) -> Self {
        Self {
            last_check: SystemTime::now(),
            last_known_version: version,
        }
    }

    fn should_check(&self, interval: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_check)
            .map(|d| d >= interval)
            .unwrap_or(true)
    }
}

/// Check for updates in the background
pub fn check_for_updates_async() -> Result<()> {
    // Spawn background thread to avoid blocking startup
    std::thread::spawn(|| {
        if let Err(e) = check_for_updates_internal() {
            warn!("Update check failed: {}", e);
        }
    });
    Ok(())
}

/// Internal update check logic
fn check_for_updates_internal() -> Result<UpdateInfo> {
    debug!("Checking for updates...");

    // Check cache first
    let cache_path = get_cache_path()?;
    if let Ok(cache_data) = std::fs::read_to_string(&cache_path) {
        if let Ok(cache) = serde_json::from_str::<UpdateCheckCache>(&cache_data) {
            // Check once per day
            if !cache.should_check(Duration::from_secs(24 * 60 * 60)) {
                debug!("Update check skipped (last checked recently)");
                return Ok(UpdateInfo {
                    current_version: CURRENT_VERSION.to_string(),
                    latest_version: cache.last_known_version.clone(),
                    update_available: cache.last_known_version != CURRENT_VERSION,
                    changelog: None,
                });
            }
        }
    }

    // Fetch latest version
    let latest_version = fetch_latest_version()?;
    let update_available = latest_version != CURRENT_VERSION;

    let changelog = if update_available {
        fetch_changelog().ok()
    } else {
        None
    };

    // Update cache
    let cache = UpdateCheckCache::new(latest_version.clone());
    if let Ok(cache_json) = serde_json::to_string(&cache) {
        let _ = std::fs::write(&cache_path, cache_json);
    }

    let update_info = UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        update_available,
        changelog,
    };

    if update_info.update_available {
        info!(
            "Update available: {} -> {}",
            update_info.current_version, update_info.latest_version
        );
    } else {
        debug!("No updates available (current: {})", CURRENT_VERSION);
    }

    Ok(update_info)
}

/// Fetch the latest version from GitHub
fn fetch_latest_version() -> Result<String> {
    let response = ureq::get(VERSION_URL)
        .timeout(Duration::from_secs(5))
        .call()?;

    let version = response.into_string()?.trim().to_string();
    Ok(version)
}

/// Fetch the changelog from GitHub
fn fetch_changelog() -> Result<String> {
    let response = ureq::get(CHANGELOG_URL)
        .timeout(Duration::from_secs(5))
        .call()?;

    let changelog = response.into_string()?;
    Ok(changelog)
}

/// Get cache file path
fn get_cache_path() -> Result<std::path::PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("native-launcher");

    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir.join("update_check.json"))
}

/// Get update information synchronously (for UI display)
#[allow(dead_code)]
pub fn get_update_info() -> Result<Option<UpdateInfo>> {
    let cache_path = get_cache_path()?;

    if let Ok(cache_data) = std::fs::read_to_string(&cache_path) {
        if let Ok(cache) = serde_json::from_str::<UpdateCheckCache>(&cache_data) {
            if cache.last_known_version != CURRENT_VERSION {
                return Ok(Some(UpdateInfo {
                    current_version: CURRENT_VERSION.to_string(),
                    latest_version: cache.last_known_version.clone(),
                    update_available: true,
                    changelog: fetch_changelog().ok(),
                }));
            }
        }
    }

    Ok(None)
}

/// Get installation instructions for manual update
#[allow(dead_code)]
pub fn get_update_instructions() -> String {
    format!(
        "To update Native Launcher:\n\n\
        1. Via install script (recommended):\n\
           curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash\n\n\
        2. Manual download:\n\
           Visit: https://github.com/ArunPrakashG/native-launcher/releases\n\n\
        3. Build from source:\n\
           git pull && cargo build --release\n"
    )
}
