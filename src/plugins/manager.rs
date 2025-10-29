use super::traits::{Plugin, PluginContext, PluginResult};
use super::LauncherPlugin;
use super::{
    AdvancedCalculatorPlugin, ApplicationsPlugin, CalculatorPlugin, EditorsPlugin,
    FileBrowserPlugin, ShellPlugin, SshPlugin, ThemeSwitcherPlugin, WebSearchPlugin,
};
use crate::config::Config;
use crate::desktop::DesktopEntry;
use crate::usage::UsageTracker;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a plugin
#[derive(Debug, Clone)]
struct PluginMetrics {
    total_time: Duration,
    call_count: u32,
}

impl PluginMetrics {
    fn new() -> Self {
        Self {
            total_time: Duration::ZERO,
            call_count: 0,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.total_time += duration;
        self.call_count += 1;
    }

    fn average_ms(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        self.total_time.as_micros() as f64 / self.call_count as f64 / 1000.0
    }
}

/// Manages all plugins and coordinates search across them
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    performance_metrics: RefCell<HashMap<String, PluginMetrics>>,
    config: Config,
}

impl PluginManager {
    /// Create a new plugin manager with default plugins
    pub fn new(
        entries: Vec<DesktopEntry>,
        usage_tracker: Option<UsageTracker>,
        config: &Config,
    ) -> Self {
        let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

        // Applications plugin (always enabled, highest priority)
        let apps_plugin = if let Some(tracker) = usage_tracker {
            ApplicationsPlugin::with_usage_tracking(entries.clone(), tracker)
        } else {
            ApplicationsPlugin::new(entries.clone())
        };
        plugins.push(Box::new(apps_plugin));

        // Calculator plugin (basic math)
        if config.plugins.calculator {
            plugins.push(Box::new(CalculatorPlugin::new()));
        }

        // Advanced calculator plugin (time, units, currency, timezone)
        // Always enabled alongside basic calculator
        if config.plugins.calculator {
            plugins.push(Box::new(AdvancedCalculatorPlugin::new()));
        }

        // Shell plugin
        if config.plugins.shell {
            let shell = ShellPlugin::with_prefix(config.plugins.shell_prefix.clone());
            plugins.push(Box::new(shell));
        }

        // Editors plugin (workspaces from VS Code, VSCodium, Sublime, Zed)
        if config.plugins.editors {
            plugins.push(Box::new(EditorsPlugin::new(true)));
        }

        // File browser plugin
        if config.plugins.files {
            plugins.push(Box::new(FileBrowserPlugin::new(true)));
        }

        // Web search plugin
        if config.plugins.web_search {
            plugins.push(Box::new(WebSearchPlugin::new()));
        }

        // Launcher (self-update) plugin
        if config.plugins.launcher {
            plugins.push(Box::new(LauncherPlugin::new(true)));
        }

        // SSH plugin
        if config.plugins.ssh {
            plugins.push(Box::new(SshPlugin::new(true)));
        }

        // Theme switcher plugin (always enabled)
        plugins.push(Box::new(ThemeSwitcherPlugin::new(config.clone())));

        // Sort plugins by priority (highest first)
        plugins.sort_by(|a, b| b.priority().cmp(&a.priority()));

        Self {
            plugins,
            performance_metrics: RefCell::new(HashMap::new()),
            config: config.clone(),
        }
    }

    /// Register a dynamic plugin
    /// Plugins are automatically sorted by priority after registration
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
        // Re-sort by priority
        self.plugins.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Search across all plugins
    /// If query starts with @ or $, route to specific plugin(s) matching the command prefix
    /// Otherwise, perform global search across all plugins
    pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<PluginResult>> {
        let mut context = PluginContext::new(max_results, &self.config);
        // Pre-allocate for max_results to avoid reallocations
        let mut all_results = Vec::with_capacity(max_results);

        // Check if query starts with @ or $ command prefix
        let is_command_query = query.starts_with('@') || query.starts_with('$');

        if is_command_query {
            // Command-based search: only query plugins that match the command prefix
            for plugin in &self.plugins {
                if !plugin.enabled() {
                    continue;
                }

                // Check if this plugin's command prefixes match
                let matches_prefix = plugin
                    .command_prefixes()
                    .iter()
                    .any(|prefix| query.starts_with(prefix));

                if matches_prefix {
                    let results = plugin.search(query, &context)?;
                    all_results.extend(results);
                }
            }
        } else {
            // Global search: query ALL enabled plugins
            // Use two-pass approach for smart triggering:
            // 1. Query app plugin first to get app matches
            // 2. Pass app count to other plugins so they can optimize

            let mut app_results_count = 0;

            // First pass: Applications plugin only
            for plugin in &self.plugins {
                if plugin.enabled() && plugin.name() == "Applications" {
                    if plugin.should_handle(query) {
                        let results = plugin.search(query, &context)?;
                        // Count high-quality app matches (score >= 700)
                        app_results_count = results.iter().filter(|r| r.score >= 700).count();
                        all_results.extend(results);
                    }
                    break;
                }
            }

            // Update context with app results count
            context = context.with_app_results(app_results_count);

            // Second pass: All other plugins
            for plugin in &self.plugins {
                if plugin.enabled()
                    && plugin.name() != "Applications"
                    && plugin.should_handle(query)
                {
                    let results = plugin.search(query, &context)?;
                    all_results.extend(results);
                }
            }
        }

        // Sort all results by score (descending)
        // Use unstable sort for better performance (order of equal elements doesn't matter)
        all_results
            .sort_unstable_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));

        // Limit to max_results
        Ok(all_results.into_iter().take(max_results).collect())
    }

    /// Incremental search - returns fast results immediately, then slow results
    /// Dynamically categorizes plugins based on their actual performance (measured timing)
    /// Callbacks:
    /// - on_fast_results: Called with results from fast plugins (< 10ms average)
    /// - on_slow_results: Called with results from slow plugins (>= 10ms average)
    pub fn search_incremental<F1, F2>(
        &self,
        query: &str,
        max_results: usize,
        on_fast_results: F1,
        on_slow_results: F2,
    ) -> Result<()>
    where
        F1: FnOnce(Vec<PluginResult>),
        F2: FnOnce(Vec<PluginResult>),
    {
        const FAST_THRESHOLD_MS: f64 = 10.0; // Plugins faster than 10ms are "fast"
        let mut context = PluginContext::new(max_results, &self.config);

        // Categorize plugins based on their historical performance
        let num_plugins = self.plugins.len();
        let mut fast_plugins = Vec::with_capacity(num_plugins);
        let mut slow_plugins = Vec::with_capacity(num_plugins);

        {
            let metrics = self.performance_metrics.borrow();

            for plugin in &self.plugins {
                if !plugin.enabled() {
                    continue;
                }

                let plugin_name = plugin.name();
                let avg_time = metrics
                    .get(plugin_name)
                    .map(|m| m.average_ms())
                    .unwrap_or(0.0);

                // If no historical data, assume Applications and calculators are fast
                // Everything else starts as slow until measured
                if avg_time == 0.0 {
                    if plugin_name == "Applications"
                        || plugin_name == "calculator"
                        || plugin_name == "advanced_calculator"
                        || plugin_name == "web_search"
                    {
                        fast_plugins.push(plugin.as_ref());
                    } else {
                        slow_plugins.push(plugin.as_ref());
                    }
                } else if avg_time < FAST_THRESHOLD_MS {
                    fast_plugins.push(plugin.as_ref());
                } else {
                    slow_plugins.push(plugin.as_ref());
                }
            }
        }

        // Phase 1: Fast plugins
        let mut fast_results = Vec::with_capacity(max_results);
        let mut app_results_count = 0;

        for plugin in fast_plugins {
            if plugin.should_handle(query) {
                let start = Instant::now();
                let results = plugin.search(query, &context)?;
                let elapsed = start.elapsed();

                // Record timing
                {
                    let mut metrics = self.performance_metrics.borrow_mut();
                    metrics
                        .entry(plugin.name().to_string())
                        .or_insert_with(PluginMetrics::new)
                        .record(elapsed);
                }

                // Track app matches for smart triggering
                if plugin.name() == "Applications" {
                    app_results_count = results.iter().filter(|r| r.score >= 700).count();
                }

                fast_results.extend(results);
            }
        }

        // Sort and limit fast results - use unstable sort for performance
        fast_results
            .sort_unstable_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        let fast_results: Vec<_> = fast_results.into_iter().take(max_results).collect();

        // Call fast callback immediately
        on_fast_results(fast_results);

        // Phase 2: Slow plugins
        context = context.with_app_results(app_results_count);
        let mut slow_results = Vec::with_capacity(max_results);

        for plugin in slow_plugins {
            if plugin.should_handle(query) {
                let start = Instant::now();
                let results = plugin.search(query, &context)?;
                let elapsed = start.elapsed();

                // Record timing
                {
                    let mut metrics = self.performance_metrics.borrow_mut();
                    metrics
                        .entry(plugin.name().to_string())
                        .or_insert_with(PluginMetrics::new)
                        .record(elapsed);
                }

                slow_results.extend(results);
            }
        }

        // Sort and limit slow results - use unstable sort for performance
        slow_results
            .sort_unstable_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));

        let slow_results: Vec<_> = slow_results.into_iter().take(max_results).collect();

        // Call slow callback
        on_slow_results(slow_results);

        Ok(())
    }

    /// Insert workspaces as separate entries right after VS Code/VSCodium
    /// Dispatch keyboard event to plugins in priority order
    /// Returns the action from the first plugin that handles the event
    pub fn dispatch_keyboard_event(
        &self,
        event: &super::traits::KeyboardEvent,
    ) -> super::traits::KeyboardAction {
        // Dispatch to plugins in priority order (already sorted)
        for plugin in &self.plugins {
            if !plugin.enabled() {
                continue;
            }

            let action = plugin.handle_keyboard_event(event);
            match action {
                super::traits::KeyboardAction::None => continue, // Try next plugin
                _ => return action,                              // First handler wins
            }
        }

        super::traits::KeyboardAction::None
    }

    /// Get list of enabled plugins
    pub fn enabled_plugins(&self) -> Vec<&str> {
        self.plugins
            .iter()
            .filter(|p| p.enabled())
            .map(|p| p.name())
            .collect()
    }

    /// Get performance metrics for all plugins (for debugging/logging)
    pub fn get_performance_metrics(&self) -> Vec<(String, f64, u32)> {
        let metrics = self.performance_metrics.borrow();
        let mut result: Vec<(String, f64, u32)> = metrics
            .iter()
            .map(|(name, m)| (name.clone(), m.average_ms(), m.call_count))
            .collect();

        // Sort by average time (slowest first)
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopEntry;
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        Config::default()
    }

    fn create_test_entry(name: &str) -> DesktopEntry {
        DesktopEntry {
            name: name.to_string(),
            generic_name: None,
            exec: format!("{}", name.to_lowercase()),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal: false,
            path: PathBuf::from(format!("/{}.desktop", name)),
            no_display: false,
            actions: vec![],
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let entries = vec![create_test_entry("Firefox")];
        let config = create_test_config();
        let manager = PluginManager::new(entries, None, &config);

        let enabled = manager.enabled_plugins();
        assert!(enabled.contains(&"applications"));
        assert!(enabled.contains(&"calculator"));
        assert!(enabled.contains(&"shell"));
        assert!(enabled.contains(&"web_search"));
    }

    #[test]
    fn test_calculator_search() {
        let entries = vec![];
        let config = create_test_config();
        let manager = PluginManager::new(entries, None, &config);

        let results = manager.search("2+2", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "4");
    }

    #[test]
    fn test_shell_search() {
        let entries = vec![];
        let config = create_test_config();
        let manager = PluginManager::new(entries, None, &config);

        let results = manager.search(">ls -la", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].title.contains("ls -la"));
    }
}
