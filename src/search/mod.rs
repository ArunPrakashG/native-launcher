use crate::desktop::{DesktopEntry, DesktopEntryArena, SharedDesktopEntry};
use crate::usage::UsageTracker;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Search engine for desktop entries with fuzzy matching and usage tracking
pub struct SearchEngine {
    entries: DesktopEntryArena,
    usage_enabled: bool,
    #[allow(dead_code)]
    matcher: SkimMatcherV2,
    #[allow(dead_code)]
    usage_tracker: Option<UsageTracker>,
}

impl SearchEngine {
    fn from_parts(
        entries: DesktopEntryArena,
        usage_enabled: bool,
        usage_tracker: Option<UsageTracker>,
    ) -> Self {
        Self {
            entries,
            usage_enabled,
            matcher: SkimMatcherV2::default(),
            usage_tracker,
        }
    }

    /// Create a new search engine with the given entries
    #[allow(dead_code)]
    pub fn new(entries: DesktopEntryArena, usage_enabled: bool) -> Self {
        Self::from_parts(entries, usage_enabled, None)
    }

    /// Create a new search engine with usage tracking enabled
    #[allow(dead_code)]
    pub fn with_usage_tracking(entries: DesktopEntryArena, usage_tracker: UsageTracker) -> Self {
        Self::from_parts(entries, true, Some(usage_tracker))
    }

    /// Create a new search engine with usage tracking and explicit usage flag
    #[allow(dead_code)]
    pub fn with_usage_tracking_config(
        entries: DesktopEntryArena,
        usage_tracker: UsageTracker,
        usage_enabled: bool,
    ) -> Self {
        Self::from_parts(entries, usage_enabled, Some(usage_tracker))
    }

    /// Search for entries matching the query with usage-based boosting
    #[allow(dead_code)]
    pub fn search(&self, query: &str, max_results: usize) -> Vec<SharedDesktopEntry> {
        let usage_tracker = if self.usage_enabled {
            self.usage_tracker.as_ref()
        } else {
            None
        };

        if query.is_empty() {
            // When query is empty, sort by usage if available, otherwise by name
            let mut results: Vec<_> = self.entries.iter().cloned().collect();

            if let Some(tracker) = usage_tracker {
                // Sort by usage score (descending), then by name
                results.sort_by(|a, b| {
                    let score_a = tracker.get_score(&a.path.to_string_lossy());
                    let score_b = tracker.get_score(&b.path.to_string_lossy());
                    score_b
                        .partial_cmp(&score_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.name.cmp(&b.name))
                });
            } else {
                results.sort_by(|a, b| a.name.cmp(&b.name));
            }

            return results.into_iter().take(max_results).collect();
        }

        let query_lower = query.to_lowercase();

        // Score entries using fuzzy matching + usage boost
        let mut results: Vec<(SharedDesktopEntry, f64)> = self
            .entries
            .iter()
            .filter(|entry| !entry.no_display) // Filter out hidden entries
            .filter_map(|entry| {
                let entry_ref = entry.as_ref();
                // Calculate fuzzy match score
                let fuzzy_score = self.calculate_fuzzy_score(entry_ref, &query_lower);

                if fuzzy_score > 0 {
                    // Apply usage boost if tracking is enabled
                    let final_score = if let Some(tracker) = usage_tracker {
                        let usage_score = tracker.get_score(&entry.path.to_string_lossy());
                        // Boost formula: fuzzy_score * (1 + usage_score * 0.1)
                        // This gives 10% boost per usage point
                        fuzzy_score as f64 * (1.0 + usage_score * 0.1)
                    } else {
                        fuzzy_score as f64
                    };

                    Some((entry.clone(), final_score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by final score (descending), then by name
        results.sort_by(|(entry_a, score_a), (entry_b, score_b)| {
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| entry_a.name.cmp(&entry_b.name))
        });

        // Return top results
        results
            .into_iter()
            .take(max_results)
            .map(|(entry, _)| entry)
            .collect()
    }

    /// Calculate fuzzy match score for an entry
    #[allow(dead_code)]

    fn calculate_fuzzy_score(&self, entry: &DesktopEntry, query: &str) -> i64 {
        let mut best_score = 0i64;

        // 1. Try exact match first (highest priority)
        let name_lower = entry.name.to_lowercase();
        if name_lower.contains(query) {
            // Exact substring match gets huge bonus
            best_score = best_score.max(10000 + (1000 / (name_lower.len() as i64 + 1)));

            // Extra bonus for prefix match
            if name_lower.starts_with(query) {
                best_score += 5000;
            }

            // Extra bonus if it's the whole name (exact match)
            if name_lower == query {
                best_score += 10000;
            }
        }

        // 2. Fuzzy match on name (primary field)
        if let Some(score) = self.matcher.fuzzy_match(&entry.name, query) {
            best_score = best_score.max(score * 3); // 3x weight for name
        }

        // 3. Fuzzy match on generic name (secondary field)
        if let Some(ref generic) = entry.generic_name {
            let generic_lower = generic.to_lowercase();
            // Check for exact match in generic name too
            if generic_lower.contains(query) {
                best_score = best_score.max(5000);
            }

            if let Some(score) = self.matcher.fuzzy_match(generic, query) {
                best_score = best_score.max(score * 2); // 2x weight for generic name
            }
        }

        // 4. Fuzzy match on keywords (tertiary field)
        for keyword in &entry.keywords {
            if let Some(score) = self.matcher.fuzzy_match(keyword, query) {
                best_score = best_score.max(score); // 1x weight for keywords
            }
        }

        // 5. Fuzzy match on categories (low priority - only if query is >3 chars)
        if query.len() > 3 {
            for category in &entry.categories {
                if let Some(score) = self.matcher.fuzzy_match(category, query) {
                    best_score = best_score.max(score / 2); // 0.5x weight for categories
                }
            }
        }

        best_score
    }

    /// Update the entries in the search engine
    #[allow(dead_code)]

    pub fn update_entries(&mut self, entries: DesktopEntryArena) {
        self.entries = entries;
    }

    /// Get total number of entries
    #[allow(dead_code)]

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopEntryArena;
    use crate::usage::UsageTracker;
    use std::path::PathBuf;

    fn create_test_entry(
        name: &str,
        generic_name: Option<&str>,
        keywords: Vec<&str>,
    ) -> DesktopEntry {
        DesktopEntry {
            name: name.to_string(),
            generic_name: generic_name.map(|s| s.to_string()),
            exec: "test".to_string(),
            icon: None,
            categories: vec![],
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            terminal: false,
            path: PathBuf::from("/test"),
            no_display: false,
            actions: vec![],
        }
    }

    #[test]
    fn test_fuzzy_search_exact_match() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("Files", Some("File Manager"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);
        let results = engine.search("Firefox", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("Thunderbird", Some("Email Client"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);
        let results = engine.search("fire", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_fuzzy_search_typo_tolerance() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("Files", Some("File Manager"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Test with minor typo
        let results = engine.search("firef", 10);

        // Should still find Firefox
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_generic_name() {
        let entries = vec![
            create_test_entry("Nautilus", Some("Files"), vec![]),
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);
        let results = engine.search("files", 10);

        assert!(!results.is_empty());
        // Should match Nautilus by generic name
        assert!(results.iter().any(|e| e.name == "Nautilus"));
    }

    #[test]
    fn test_fuzzy_search_keywords() {
        let entries = vec![
            create_test_entry("GIMP", Some("Image Editor"), vec!["photo", "graphics"]),
            create_test_entry("Firefox", Some("Web Browser"), vec!["web", "internet"]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);
        let results = engine.search("photo", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].name, "GIMP");
    }

    #[test]
    fn test_usage_boost_toggle() {
        let entries = vec![
            DesktopEntry {
                name: "Alpha Editor".to_string(),
                generic_name: None,
                exec: "alpha".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/alpha.desktop"),
                no_display: false,
                actions: vec![],
            },
            DesktopEntry {
                name: "Beta Browser".to_string(),
                generic_name: None,
                exec: "beta".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/beta.desktop"),
                no_display: false,
                actions: vec![],
            },
        ];

        let arena = DesktopEntryArena::from_vec(entries);

        let mut tracker = UsageTracker::new();
        tracker.record_launch("/beta.desktop");
        tracker.record_launch("/beta.desktop");
        let tracker_disabled = tracker.clone();

        let engine_usage_enabled =
            SearchEngine::with_usage_tracking_config(arena.clone(), tracker, true);
        let usage_results = engine_usage_enabled.search("", 10);
        assert_eq!(
            usage_results.first().map(|entry| entry.name.as_str()),
            Some("Beta Browser")
        );

        let engine_usage_disabled =
            SearchEngine::with_usage_tracking_config(arena, tracker_disabled, false);
        let non_usage_results = engine_usage_disabled.search("", 10);
        assert_eq!(
            non_usage_results.first().map(|entry| entry.name.as_str()),
            Some("Alpha Editor")
        );
    }
}
