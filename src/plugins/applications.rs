use super::traits::{Plugin, PluginContext, PluginResult};
use crate::desktop::DesktopEntry;
use crate::usage::UsageTracker;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Plugin for searching desktop applications
pub struct ApplicationsPlugin {
    entries: Vec<DesktopEntry>,
    matcher: SkimMatcherV2,
    usage_tracker: Option<UsageTracker>,
}

impl std::fmt::Debug for ApplicationsPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationsPlugin")
            .field("entries", &self.entries.len())
            .field("usage_tracker", &self.usage_tracker.is_some())
            .finish()
    }
}

impl ApplicationsPlugin {
    /// Create a new applications plugin
    pub fn new(entries: Vec<DesktopEntry>) -> Self {
        Self {
            entries,
            matcher: SkimMatcherV2::default(),
            usage_tracker: None,
        }
    }

    /// Create with usage tracking
    pub fn with_usage_tracking(entries: Vec<DesktopEntry>, usage_tracker: UsageTracker) -> Self {
        Self {
            entries,
            matcher: SkimMatcherV2::default(),
            usage_tracker: Some(usage_tracker),
        }
    }

    /// Calculate fuzzy match score for an entry
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
            best_score = best_score.max(score * 3);
        }

        // 3. Fuzzy match on generic name (secondary field)
        if let Some(ref generic) = entry.generic_name {
            let generic_lower = generic.to_lowercase();
            if generic_lower.contains(query) {
                best_score = best_score.max(5000);
            }

            if let Some(score) = self.matcher.fuzzy_match(generic, query) {
                best_score = best_score.max(score * 2);
            }
        }

        // 4. Fuzzy match on keywords (tertiary field)
        for keyword in &entry.keywords {
            if let Some(score) = self.matcher.fuzzy_match(keyword, query) {
                best_score = best_score.max(score);
            }
        }

        // 5. Fuzzy match on categories (low priority - only if query is >3 chars)
        if query.len() > 3 {
            for category in &entry.categories {
                if let Some(score) = self.matcher.fuzzy_match(category, query) {
                    best_score = best_score.max(score / 2);
                }
            }
        }

        best_score
    }
}

impl Plugin for ApplicationsPlugin {
    fn name(&self) -> &str {
        "applications"
    }

    fn description(&self) -> &str {
        "Search installed desktop applications"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@app"]
    }

    fn should_handle(&self, query: &str) -> bool {
        // Don't interfere with other @ commands (unless it's @app)
        if query.starts_with('@') {
            return query.starts_with("@app");
        }

        // Applications plugin handles all non-@ queries (fallback)
        true
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        let query_lower = query.to_lowercase();

        // If empty query, return most used apps
        if query.is_empty() {
            let mut results: Vec<&DesktopEntry> = self.entries.iter().collect();

            if let Some(tracker) = &self.usage_tracker {
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

            return Ok(results
                .into_iter()
                .take(context.max_results)
                .map(|entry| {
                    PluginResult::new(
                        entry.name.clone(),
                        entry.exec.clone(),
                        self.name().to_string(),
                    )
                    .with_subtitle(entry.generic_name.clone().unwrap_or_default())
                    .with_icon(entry.icon.clone().unwrap_or_default())
                    .with_terminal(entry.terminal)
                    .with_score(0)
                })
                .collect());
        }

        // Score entries using fuzzy matching + usage boost
        let mut results: Vec<(&DesktopEntry, f64)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let fuzzy_score = self.calculate_fuzzy_score(entry, &query_lower);

                if fuzzy_score > 0 {
                    let final_score = if let Some(tracker) = &self.usage_tracker {
                        let usage_score = tracker.get_score(&entry.path.to_string_lossy());
                        fuzzy_score as f64 * (1.0 + usage_score * 0.1)
                    } else {
                        fuzzy_score as f64
                    };

                    Some((entry, final_score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score
        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.name.cmp(&b.0.name))
        });

        // Convert to PluginResult
        Ok(results
            .into_iter()
            .take(context.max_results)
            .map(|(entry, score)| {
                PluginResult::new(
                    entry.name.clone(),
                    entry.exec.clone(),
                    self.name().to_string(),
                )
                .with_subtitle(entry.generic_name.clone().unwrap_or_default())
                .with_icon(entry.icon.clone().unwrap_or_default())
                .with_terminal(entry.terminal)
                .with_score(score as i64)
            })
            .collect())
    }

    fn priority(&self) -> i32 {
        1000 // Highest priority - main functionality
    }
}
