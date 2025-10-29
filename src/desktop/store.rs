use std::sync::Arc;

use super::entry::DesktopEntry;

/// Shared pointer type for desktop entries allocated in the arena
pub type SharedDesktopEntry = Arc<DesktopEntry>;

/// Compact arena that stores desktop entries once and shares them across components.
///
/// Internally this holds an `Arc<[SharedDesktopEntry]>`, so cloning the arena or
/// individual entries is cheap and avoids repeatedly allocating `DesktopEntry` data.
#[derive(Clone, Debug, Default)]
pub struct DesktopEntryArena {
    entries: Arc<[SharedDesktopEntry]>,
}

impl DesktopEntryArena {
    /// Create a new arena from owned desktop entries.
    pub fn from_vec(entries: Vec<DesktopEntry>) -> Self {
        if entries.is_empty() {
            return Self {
                entries: Arc::from([]),
            };
        }

        let shared: Vec<SharedDesktopEntry> = entries.into_iter().map(Arc::new).collect();
        Self {
            entries: Arc::from(shared.into_boxed_slice()),
        }
    }

    /// Create an arena directly from already shared entries.
    #[allow(dead_code)]
    pub fn from_shared(entries: Vec<SharedDesktopEntry>) -> Self {
        if entries.is_empty() {
            return Self {
                entries: Arc::from([]),
            };
        }

        Self {
            entries: Arc::from(entries.into_boxed_slice()),
        }
    }

    /// Number of entries stored in the arena.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the arena is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all shared entries.
    pub fn iter(&self) -> impl Iterator<Item = &SharedDesktopEntry> {
        self.entries.iter()
    }

    /// Return all entries as a `Vec` of shared pointers.
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<SharedDesktopEntry> {
        self.entries.iter().cloned().collect()
    }
}
