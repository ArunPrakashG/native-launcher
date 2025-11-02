use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{debug, info};

/// Persistent store for pinned (favorite) applications
#[derive(Debug)]
pub struct PinsStore {
    pins: RwLock<HashSet<String>>, // desktop file paths
    path: PathBuf,                 // JSON file path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinsFile {
    pins: HashSet<String>,
}

impl PinsStore {
    /// Create an empty store with default path
    pub fn new() -> Self {
        Self {
            pins: RwLock::new(HashSet::new()),
            path: Self::default_path(),
        }
    }

    /// Load pins from disk (JSON). If file doesn't exist, returns empty store.
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if !path.exists() {
            debug!("Pins file not found at {:?}, starting empty", path);
            return Ok(Self {
                pins: RwLock::new(HashSet::new()),
                path,
            });
        }

        let data = fs::read(&path)?;
        let parsed: PinsFile = serde_json::from_slice(&data)?;
        info!("Loaded {} pinned apps", parsed.pins.len());
        Ok(Self {
            pins: RwLock::new(parsed.pins),
            path,
        })
    }

    /// Save pins to disk (JSON). Creates directories if needed.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let pins = self.pins.read().unwrap().clone();
        let payload = PinsFile { pins };
        let json = serde_json::to_vec_pretty(&payload)?;
        fs::write(&self.path, json)?;
        debug!("Pins saved to {:?}", self.path);
        Ok(())
    }

    /// Check if a desktop entry path is pinned
    pub fn is_pinned(&self, desktop_path: &str) -> bool {
        self.pins
            .read()
            .unwrap()
            .contains(&desktop_path.to_string())
    }

    /// Toggle pinned state for a desktop entry path. Returns new state (true if pinned).
    pub fn toggle(&self, desktop_path: &str) -> Result<bool> {
        let mut guard = self.pins.write().unwrap();
        if guard.contains(desktop_path) {
            guard.remove(desktop_path);
            drop(guard);
            self.save()?;
            info!("Unpinned {}", desktop_path);
            Ok(false)
        } else {
            guard.insert(desktop_path.to_string());
            drop(guard);
            self.save()?;
            info!("Pinned {}", desktop_path);
            Ok(true)
        }
    }

    /// List all pinned desktop paths
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<String> {
        self.pins.read().unwrap().iter().cloned().collect()
    }

    fn default_path() -> PathBuf {
        let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        data_dir.join("native-launcher").join("pins.json")
    }
}

impl Default for PinsStore {
    fn default() -> Self {
        Self::new()
    }
}
