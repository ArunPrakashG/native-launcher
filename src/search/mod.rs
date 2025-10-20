use crate::desktop::DesktopEntry;

/// Simple search engine for desktop entries
pub struct SearchEngine {
    entries: Vec<DesktopEntry>,
}

impl SearchEngine {
    /// Create a new search engine with the given entries
    pub fn new(entries: Vec<DesktopEntry>) -> Self {
        Self { entries }
    }

    /// Search for entries matching the query
    pub fn search(&self, query: &str, max_results: usize) -> Vec<&DesktopEntry> {
        if query.is_empty() {
            // Return all entries when query is empty, sorted by name
            let mut results: Vec<&DesktopEntry> = self.entries.iter().collect();
            results.sort_by(|a, b| a.name.cmp(&b.name));
            return results.into_iter().take(max_results).collect();
        }

        // Filter and score entries
        let mut results: Vec<(&DesktopEntry, u32)> = self
            .entries
            .iter()
            .filter(|entry| entry.matches(query))
            .map(|entry| (entry, entry.match_score(query)))
            .collect();

        // Sort by score (descending), then by name
        results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));

        // Return top results
        results
            .into_iter()
            .take(max_results)
            .map(|(entry, _)| entry)
            .collect()
    }

    /// Update the entries in the search engine
    pub fn update_entries(&mut self, entries: Vec<DesktopEntry>) {
        self.entries = entries;
    }

    /// Get total number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
