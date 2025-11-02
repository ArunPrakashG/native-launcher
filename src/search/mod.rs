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

        // Minimum score threshold to reduce false positives
        // For short queries (1-2 chars), require higher scores
        let min_score = if query.len() <= 2 {
            50 // Higher threshold for short queries
        } else {
            20 // Lower threshold for longer queries
        };

        // Score entries using fuzzy matching + usage boost
        let mut results: Vec<(SharedDesktopEntry, f64)> = self
            .entries
            .iter()
            .filter(|entry| !entry.no_display) // Filter out hidden entries
            .filter_map(|entry| {
                let entry_ref = entry.as_ref();
                // Calculate fuzzy match score
                let fuzzy_score = self.calculate_fuzzy_score(entry_ref, &query_lower);

                if fuzzy_score > min_score {
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
    #[inline(always)] // Force inlining for hot path
    fn calculate_fuzzy_score(&self, entry: &DesktopEntry, query: &str) -> i64 {
        let mut best_score = 0i64;

        // Cache lowercase conversions for performance
        let name_lower = entry.name.to_lowercase();
        let query_lower = query.to_lowercase();

        // 1. Try exact match first (highest priority)
        if name_lower.contains(&query_lower) {
            // Exact substring match gets huge bonus
            best_score = best_score.max(10000 + (1000 / (name_lower.len() as i64 + 1)));

            // Extra bonus for prefix match
            if name_lower.starts_with(&query_lower) {
                best_score += 5000;
            }

            // Extra bonus if it's the whole name (exact match)
            if name_lower == query_lower {
                best_score += 10000;
            }

            // Case-sensitive exact match bonus (user typed exact case)
            if entry.name.contains(query) {
                best_score += 2000;
            }
        }

        // 2. Acronym matching (e.g., "vsc" matches "Visual Studio Code")
        if query.len() >= 2 {
            let acronym_score = self.match_acronym(&entry.name, query);
            if acronym_score > 0 {
                best_score = best_score.max(8000 + acronym_score);
            }

            // Also try generic name for acronyms
            if let Some(ref generic) = entry.generic_name {
                let generic_acronym_score = self.match_acronym(generic, query);
                if generic_acronym_score > 0 {
                    best_score = best_score.max(6000 + generic_acronym_score);
                }
            }
        }

        // 3. Word boundary matching (e.g., "vs" matches "Visual Studio")
        let word_boundary_score = self.match_word_boundaries(&entry.name, &query_lower);
        if word_boundary_score > 0 {
            best_score = best_score.max(7000 + word_boundary_score);
        }

        // 4. Fuzzy match on name (primary field)
        if let Some(score) = self.matcher.fuzzy_match(&entry.name, query) {
            best_score = best_score.max(score * 3); // 3x weight for name
        }

        // 5. Fuzzy match on generic name (secondary field)
        if let Some(ref generic) = entry.generic_name {
            let generic_lower = generic.to_lowercase();
            // Check for exact match in generic name too
            if generic_lower.contains(&query_lower) {
                best_score = best_score.max(5000);

                // Word boundary bonus for generic name
                let generic_word_score = self.match_word_boundaries(generic, &query_lower);
                if generic_word_score > 0 {
                    best_score = best_score.max(5500 + generic_word_score);
                }
            }

            if let Some(score) = self.matcher.fuzzy_match(generic, query) {
                best_score = best_score.max(score * 2); // 2x weight for generic name
            }
        }

        // 6. Match on exec field (for technical users searching by command name)
        if query.len() >= 3 {
            let exec_lower = entry.exec.to_lowercase();
            if exec_lower.contains(&query_lower) {
                // Lower priority than name matches but still relevant
                best_score = best_score.max(3000);
            }
        }

        // 7. Fuzzy match on keywords (tertiary field)
        for keyword in &entry.keywords {
            let keyword_lower = keyword.to_lowercase();
            // Exact keyword match gets priority
            if keyword_lower == query_lower {
                best_score = best_score.max(4000);
            } else if keyword_lower.contains(&query_lower) {
                best_score = best_score.max(2000);
            }

            if let Some(score) = self.matcher.fuzzy_match(keyword, query) {
                best_score = best_score.max(score); // 1x weight for keywords
            }
        }

        // 8. Fuzzy match on categories (only if query is >3 chars to reduce false positives)
        if query.len() > 3 {
            for category in &entry.categories {
                if let Some(score) = self.matcher.fuzzy_match(category, query) {
                    best_score = best_score.max(score / 2); // 0.5x weight for categories
                }
            }
        }

        best_score
    }

    /// Match acronym patterns (e.g., "vsc" matches "Visual Studio Code")
    #[inline]
    fn match_acronym(&self, text: &str, query: &str) -> i64 {
        let query_chars: Vec<char> = query.chars().collect();
        if query_chars.is_empty() {
            return 0;
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < query_chars.len() {
            return 0;
        }

        // Check if query chars match word initials
        let mut query_idx = 0;
        let mut matched_positions = Vec::new();

        for (word_idx, word) in words.iter().enumerate() {
            if query_idx >= query_chars.len() {
                break;
            }

            let first_char = word.chars().next();
            if let Some(fc) = first_char {
                if fc.to_lowercase().eq(query_chars[query_idx].to_lowercase()) {
                    matched_positions.push(word_idx);
                    query_idx += 1;
                }
            }
        }

        // Full acronym match
        if query_idx == query_chars.len() {
            // Bonus for consecutive words
            let consecutiveness = if matched_positions.windows(2).all(|w| w[1] == w[0] + 1) {
                500
            } else {
                0
            };
            return 1000 + consecutiveness;
        }

        0
    }

    /// Match word boundaries (e.g., "code" matches "Visual Studio Code")
    #[inline]
    fn match_word_boundaries(&self, text: &str, query_lower: &str) -> i64 {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        for (idx, word) in words.iter().enumerate() {
            // Exact word match
            if *word == query_lower {
                // Earlier words get higher score
                return 1000 - (idx as i64 * 100);
            }

            // Word starts with query
            if word.starts_with(query_lower) {
                return 800 - (idx as i64 * 100);
            }
        }

        0
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

    #[test]
    fn test_acronym_matching() {
        let entries = vec![
            create_test_entry("Visual Studio Code", Some("Code Editor"), vec![]),
            create_test_entry("VLC Media Player", Some("Media Player"), vec![]),
            create_test_entry("Vim", Some("Text Editor"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // "vsc" should match "Visual Studio Code" via acronym
        let results = engine.search("vsc", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Visual Studio Code");

        // "vlc" should match "VLC Media Player" via acronym
        let results = engine.search("vlc", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "VLC Media Player");
    }

    #[test]
    fn test_word_boundary_matching() {
        let entries = vec![
            create_test_entry("Visual Studio", Some("IDE"), vec![]),
            create_test_entry("Android Studio", Some("IDE"), vec![]),
            create_test_entry("Studio One", Some("DAW"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // "studio" should match all entries containing word "studio"
        let results = engine.search("studio", 10);
        assert_eq!(results.len(), 3);

        // Earlier word occurrences should rank higher
        // "Visual Studio" has "Studio" in second position
        // "Android Studio" has "Studio" in second position
        // "Studio One" has "Studio" in first position - should rank highest
        assert_eq!(results[0].name, "Studio One");
    }

    #[test]
    fn test_exec_field_matching() {
        let entries = vec![
            DesktopEntry {
                name: "Firefox".to_string(),
                generic_name: Some("Web Browser".to_string()),
                exec: "firefox %u".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/firefox.desktop"),
                no_display: false,
                actions: vec![],
            },
            DesktopEntry {
                name: "Chrome".to_string(),
                generic_name: Some("Web Browser".to_string()),
                exec: "google-chrome %u".to_string(),
                icon: None,
                categories: vec![],
                keywords: vec![],
                terminal: false,
                path: PathBuf::from("/chrome.desktop"),
                no_display: false,
                actions: vec![],
            },
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // "google-chrome" should match Chrome by exec field
        let results = engine.search("google-chrome", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Chrome");
    }

    #[test]
    fn test_case_sensitivity_bonus() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("firefox-esr", Some("Web Browser ESR"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Exact case match "Firefox" should rank higher than "firefox"
        let results = engine.search("Firefox", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_minimum_score_threshold() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("Files", Some("File Manager"), vec![]),
            create_test_entry("Calculator", Some("Calculator"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Short query with weak match should not return false positives
        let results = engine.search("xyz", 10);
        // Should return no or very few results (no strong matches)
        assert!(results.len() < 2, "Short query should have high threshold");

        // Longer query with weak match
        let results = engine.search("xyza", 10);
        assert!(results.is_empty(), "No results should match random string");
    }

    #[test]
    fn test_prefix_match_priority() {
        let entries = vec![
            create_test_entry("Firefox", Some("Web Browser"), vec![]),
            create_test_entry("Firewall Configuration", Some("Security"), vec![]),
            create_test_entry("Archive Manager", Some("File Roller"), vec![]),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // "fire" should prioritize prefix matches
        let results = engine.search("fire", 10);
        assert!(!results.is_empty());
        // Both Firefox and Firewall start with "fire", should rank higher
        assert!(results[0].name == "Firefox" || results[0].name == "Firewall Configuration");
    }

    #[test]
    fn test_keyword_exact_match() {
        let entries = vec![
            create_test_entry(
                "GIMP",
                Some("Image Editor"),
                vec!["photo", "graphics", "edit"],
            ),
            create_test_entry(
                "Inkscape",
                Some("Vector Graphics"),
                vec!["vector", "svg", "draw"],
            ),
        ];

        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Exact keyword match should rank high
        let results = engine.search("photo", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "GIMP");

        let results = engine.search("vector", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Inkscape");
    }
}
