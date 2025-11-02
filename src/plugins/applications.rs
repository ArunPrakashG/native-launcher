use super::traits::{Plugin, PluginContext, PluginResult};
use crate::desktop::{DesktopEntry, DesktopEntryArena, SharedDesktopEntry};
use crate::pins::PinsStore;
use crate::usage::UsageTracker;
use crate::utils::icons::resolve_icon_with_category_fallback;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::Arc;

/// Plugin for searching desktop applications
pub struct ApplicationsPlugin {
    entries: DesktopEntryArena,
    matcher: SkimMatcherV2,
    usage_tracker: Option<UsageTracker>,
    pins: Option<Arc<PinsStore>>,
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
    pub fn new(entries: DesktopEntryArena) -> Self {
        Self {
            entries,
            matcher: SkimMatcherV2::default(),
            usage_tracker: None,
            pins: None,
        }
    }

    /// Create with usage tracking
    pub fn with_usage_tracking(entries: DesktopEntryArena, usage_tracker: UsageTracker) -> Self {
        Self {
            entries,
            matcher: SkimMatcherV2::default(),
            usage_tracker: Some(usage_tracker),
            pins: None,
        }
    }

    /// Create with usage tracking and pins
    pub fn with_usage_and_pins(
        entries: DesktopEntryArena,
        usage_tracker: Option<UsageTracker>,
        pins: Option<Arc<PinsStore>>,
    ) -> Self {
        Self {
            entries,
            matcher: SkimMatcherV2::default(),
            usage_tracker,
            pins,
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
            let mut results: Vec<_> = self.entries.iter().cloned().collect();

            let tracker_opt = &self.usage_tracker;
            let pins_opt = &self.pins;

            // Sort by pinned first, then usage score, then name (stable across runs)
            results.sort_by(|a, b| {
                let a_path = a.path.to_string_lossy().to_string();
                let b_path = b.path.to_string_lossy().to_string();
                let a_pinned = pins_opt
                    .as_ref()
                    .map(|p| p.is_pinned(&a_path))
                    .unwrap_or(false);
                let b_pinned = pins_opt
                    .as_ref()
                    .map(|p| p.is_pinned(&b_path))
                    .unwrap_or(false);

                b_pinned
                    .cmp(&a_pinned)
                    .then_with(|| {
                        if let Some(tracker) = tracker_opt {
                            let score_a = tracker.get_score(&a_path);
                            let score_b = tracker.get_score(&b_path);
                            score_b
                                .partial_cmp(&score_a)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .then_with(|| a.name.cmp(&b.name))
            });

            // Encode sort into score so global manager sort preserves ordering
            let mapped: Vec<PluginResult> = results
                .into_iter()
                .take(context.max_results)
                .map(|entry| {
                    let entry = entry.as_ref();
                    let path = entry.path.to_string_lossy().to_string();
                    let pinned = pins_opt
                        .as_ref()
                        .map(|p| p.is_pinned(&path))
                        .unwrap_or(false);
                    let usage = tracker_opt
                        .as_ref()
                        .map(|t| t.get_score(&path))
                        .unwrap_or(0.0);
                    // Large boost for pinned to ensure they appear first globally
                    let pin_boost: i64 = if pinned { 1_000_000 } else { 0 };
                    // Scale usage to i64; usage is typically small (<10)
                    let usage_points: i64 = (usage * 1000.0).round() as i64;
                    let score = pin_boost + usage_points;

                    // Resolve icon with category fallback
                    let icon_path = resolve_icon_with_category_fallback(
                        entry.icon.as_deref(),
                        &entry.categories,
                    );

                    let mut result = PluginResult::new(
                        entry.name.clone(),
                        entry.exec.clone(),
                        self.name().to_string(),
                    )
                    .with_subtitle(entry.generic_name.clone().unwrap_or_default())
                    .with_icon(icon_path.to_string_lossy().to_string())
                    .with_terminal(entry.terminal)
                    .with_desktop_path(path)
                    .with_score(score);

                    // Add terminal badge for terminal apps
                    if entry.terminal {
                        result = result.with_badge_icon("utilities-terminal-symbolic".to_string());
                    }

                    result
                })
                .collect();

            return Ok(mapped);
        }

        // Score entries using fuzzy matching + usage boost
        let mut results: Vec<(SharedDesktopEntry, f64)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let fuzzy_score = self.calculate_fuzzy_score(entry.as_ref(), &query_lower);

                if fuzzy_score > 0 {
                    let mut final_score = if let Some(tracker) = &self.usage_tracker {
                        let usage_score = tracker.get_score(&entry.path.to_string_lossy());
                        fuzzy_score as f64 * (1.0 + usage_score * 0.1)
                    } else {
                        fuzzy_score as f64
                    };

                    // Apply pin boost if applicable
                    if let Some(pins) = &self.pins {
                        if pins.is_pinned(&entry.path.to_string_lossy()) {
                            // Lightweight boost to float pinned apps higher without breaking exact-match intent
                            final_score += 2000.0;
                        }
                    }

                    Some((entry.clone(), final_score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score
        results.sort_by(|(entry_a, score_a), (entry_b, score_b)| {
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| entry_a.name.cmp(&entry_b.name))
        });

        // Convert to PluginResult
        Ok(results
            .into_iter()
            .take(context.max_results)
            .map(|(entry, score)| {
                let entry = entry.as_ref();

                // Resolve icon with category fallback
                let icon_path =
                    resolve_icon_with_category_fallback(entry.icon.as_deref(), &entry.categories);

                let mut result = PluginResult::new(
                    entry.name.clone(),
                    entry.exec.clone(),
                    self.name().to_string(),
                )
                .with_subtitle(entry.generic_name.clone().unwrap_or_default())
                .with_icon(icon_path.to_string_lossy().to_string())
                .with_terminal(entry.terminal)
                .with_desktop_path(entry.path.to_string_lossy().to_string())
                .with_score(score as i64);

                // Add terminal badge for terminal apps
                if entry.terminal {
                    result = result.with_badge_icon("utilities-terminal-symbolic".to_string());
                }

                result
            })
            .collect())
    }

    fn priority(&self) -> i32 {
        1000 // Highest priority - main functionality
    }
}
