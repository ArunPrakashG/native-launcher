// Future feature: hot-reload desktop files when they change
#![allow(dead_code)]

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use super::cache::DesktopCache;
use super::entry::DesktopEntry;

/// File system watcher for desktop files
pub struct DesktopWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    watched_paths: Vec<PathBuf>,
}

impl DesktopWatcher {
    /// Create a new desktop file watcher
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    warn!("Failed to send file watch event: {}", e);
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        Ok(Self {
            watcher,
            rx,
            watched_paths: paths,
        })
    }

    /// Start watching the configured paths
    pub fn start_watching(&mut self) -> Result<()> {
        for path in &self.watched_paths {
            if path.exists() {
                info!("Watching directory: {}", path.display());
                self.watcher
                    .watch(path, RecursiveMode::Recursive)
                    .context(format!("Failed to watch {}", path.display()))?;
            } else {
                debug!("Skipping non-existent path: {}", path.display());
            }
        }
        Ok(())
    }

    /// Process file system events and update cache
    pub fn process_events(&self, cache: &mut DesktopCache) -> Result<bool> {
        let mut cache_updated = false;

        // Process all pending events
        while let Ok(event_result) = self.rx.try_recv() {
            match event_result {
                Ok(event) => {
                    if self.should_process_event(&event) {
                        cache_updated |= self.handle_event(&event, cache)?;
                    }
                }
                Err(e) => {
                    warn!("File watch error: {}", e);
                }
            }
        }

        Ok(cache_updated)
    }

    /// Check if an event should be processed
    fn should_process_event(&self, event: &Event) -> bool {
        // Only process .desktop files
        event.paths.iter().any(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "desktop")
                .unwrap_or(false)
        })
    }

    /// Handle a file system event
    fn handle_event(&self, event: &Event, cache: &mut DesktopCache) -> Result<bool> {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                // File created or modified - update cache
                for path in &event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        info!("Detected change: {}", path.display());
                        match DesktopEntry::from_file(path.clone()) {
                            Ok(entry) => {
                                if !entry.no_display {
                                    cache.insert(path.clone(), entry)?;
                                    debug!("Updated cache for: {}", path.display());
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse changed file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
                Ok(true)
            }
            EventKind::Remove(_) => {
                // File removed - invalidate cache entry
                for path in &event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        info!("Detected removal: {}", path.display());
                        cache.remove(path);
                        debug!("Removed from cache: {}", path.display());
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

/// Background watcher thread manager
pub struct WatcherThread {
    cache: Arc<Mutex<DesktopCache>>,
    watcher: Arc<Mutex<DesktopWatcher>>,
}

impl WatcherThread {
    /// Create a new watcher thread
    pub fn new(paths: Vec<PathBuf>, cache: DesktopCache) -> Result<Self> {
        let mut watcher = DesktopWatcher::new(paths)?;
        watcher.start_watching()?;

        Ok(Self {
            cache: Arc::new(Mutex::new(cache)),
            watcher: Arc::new(Mutex::new(watcher)),
        })
    }

    /// Spawn a background thread to process events
    pub fn spawn(self) -> Result<()> {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let mut cache = match self.cache.lock() {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to lock cache: {}", e);
                        continue;
                    }
                };

                let watcher = match self.watcher.lock() {
                    Ok(w) => w,
                    Err(e) => {
                        warn!("Failed to lock watcher: {}", e);
                        continue;
                    }
                };

                match watcher.process_events(&mut cache) {
                    Ok(true) => {
                        // Cache was updated, save it
                        if let Err(e) = cache.save() {
                            warn!("Failed to save cache after file changes: {}", e);
                        }
                    }
                    Ok(false) => {
                        // No updates
                    }
                    Err(e) => {
                        warn!("Error processing file events: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
