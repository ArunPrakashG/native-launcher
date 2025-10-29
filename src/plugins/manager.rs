use super::traits::{Plugin, PluginContext, PluginResult};
use super::LauncherPlugin;
use super::{
    AdvancedCalculatorPlugin, ApplicationsPlugin, CalculatorPlugin, EditorsPlugin,
    FileBrowserPlugin, ScreenshotPlugin, ShellPlugin, SshPlugin, ThemeSwitcherPlugin,
    WebSearchPlugin,
};
use crate::config::Config;
use crate::desktop::DesktopEntryArena;
use crate::usage::UsageTracker;
use crate::utils::exec::{register_open_handler, CommandOpenHandler, OpenHandlerPriority};
use anyhow::Result;
use dirs::home_dir;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tracing::debug;
use urlencoding::decode;

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

fn ensure_builtin_open_handlers_registered() {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| {
        register_filesystem_open_handler();
    });
}

fn register_filesystem_open_handler() {
    let handler = CommandOpenHandler {
        command: "xdg-open".to_string(),
        args: Vec::new(),
        pass_target: true,
    };

    register_open_handler(
        "filesystem-open",
        OpenHandlerPriority::Last,
        move |target, merge_login_env| {
            if let Some(path) = resolve_filesystem_path(target) {
                if path.exists() {
                    debug!("filesystem open handler launching {}", path.display());
                    let path_string = path.to_string_lossy().to_string();
                    return handler.execute(&path_string, merge_login_env);
                }
            }

            Ok(false)
        },
    );
}

fn resolve_filesystem_path(target: &str) -> Option<PathBuf> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return None;
    }

    if let Some(stripped) = trimmed.strip_prefix("file://") {
        if let Ok(decoded) = decode(stripped) {
            return Some(PathBuf::from(decoded.into_owned()));
        }

        return Some(PathBuf::from(stripped));
    }

    if trimmed.starts_with("~/") {
        if let Some(home) = home_dir() {
            return Some(home.join(&trimmed[2..]));
        }
        return None;
    }

    if trimmed == "~" {
        return home_dir();
    }

    if Path::new(trimmed).is_absolute() {
        return Some(PathBuf::from(trimmed));
    }

    None
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
        entry_arena: DesktopEntryArena,
        usage_tracker: Option<UsageTracker>,
        config: &Config,
    ) -> Self {
        ensure_builtin_open_handlers_registered();

        let usage_tracker = if config.search.usage_ranking {
            usage_tracker
        } else {
            None
        };

        let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

        // Applications plugin (always enabled, highest priority)
        let apps_plugin = usage_tracker
            .map(|tracker| ApplicationsPlugin::with_usage_tracking(entry_arena.clone(), tracker))
            .unwrap_or_else(|| ApplicationsPlugin::new(entry_arena.clone()));
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

        // Screenshot plugin
        if config.plugins.screenshot {
            plugins.push(Box::new(ScreenshotPlugin::new()));
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
    use crate::desktop::{DesktopEntry, DesktopEntryArena};
    use crate::utils::exec::{handler_counts_for_test, reset_open_handlers_for_test};
    use std::path::PathBuf;
    use urlencoding::encode;

    fn create_test_config() -> Config {
        Config::default()
    }

    fn reset_handlers_to_builtin() {
        reset_open_handlers_for_test();
        super::register_filesystem_open_handler();
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
        reset_handlers_to_builtin();
        let entries = vec![create_test_entry("Firefox")];
        let arena = DesktopEntryArena::from_vec(entries);
        let config = create_test_config();
        let manager = PluginManager::new(arena, None, &config);

        let enabled = manager.enabled_plugins();
        assert!(enabled.contains(&"applications"));
        assert!(enabled.contains(&"calculator"));
        assert!(enabled.contains(&"shell"));
        assert!(enabled.contains(&"web_search"));
        assert!(enabled.contains(&"screenshot"));
        // Filesystem open handler should be registered exactly once
        assert_eq!(handler_counts_for_test(), (1, 0));
        reset_handlers_to_builtin();
    }

    #[test]
    fn test_calculator_search() {
        reset_handlers_to_builtin();
        let entries = Vec::new();
        let arena = DesktopEntryArena::from_vec(entries);
        let config = create_test_config();
        let manager = PluginManager::new(arena, None, &config);

        let results = manager.search("2+2", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "4");
        reset_handlers_to_builtin();
    }

    #[test]
    fn test_shell_search() {
        reset_handlers_to_builtin();
        let entries = Vec::new();
        let arena = DesktopEntryArena::from_vec(entries);
        let config = create_test_config();
        let manager = PluginManager::new(arena, None, &config);

        let results = manager.search(">ls -la", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].title.contains("ls -la"));
        reset_handlers_to_builtin();
    }

    #[test]
    fn registers_filesystem_handler_once() {
        reset_handlers_to_builtin();

        let arena = DesktopEntryArena::from_vec(Vec::new());
        let config = Config::default();
        let _manager = PluginManager::new(arena, None, &config);
        assert_eq!(handler_counts_for_test(), (1, 0));

        // Creating another manager should not add duplicates
        let arena2 = DesktopEntryArena::from_vec(Vec::new());
        let _manager2 = PluginManager::new(arena2, None, &config);
        assert_eq!(handler_counts_for_test(), (1, 0));

        reset_handlers_to_builtin();
    }

    #[test]
    fn resolve_filesystem_path_covers_common_inputs() {
        // Absolute path
        let abs = std::env::temp_dir().join("launcher-open-test");
        let abs_str = abs.to_string_lossy().to_string();
        assert_eq!(resolve_filesystem_path(&abs_str).unwrap(), abs);

        // file:// scheme with encoding
        let encoded = format!("file://{}", encode(&abs_str).into_owned());
        assert_eq!(resolve_filesystem_path(&encoded).unwrap(), abs);

        // file:// without encoding
        let plain_scheme = format!("file://{}", abs_str);
        assert_eq!(resolve_filesystem_path(&plain_scheme).unwrap(), abs);

        // Tilde expansion
        if let Some(home) = dirs::home_dir() {
            assert_eq!(resolve_filesystem_path("~").unwrap(), home);
            let tilde_path = resolve_filesystem_path("~/Documents");
            if tilde_path.is_some() {
                assert!(tilde_path.unwrap().starts_with(&home));
            }
        }

        // URLs should not resolve
        assert!(resolve_filesystem_path("https://example.com").is_none());

        // Relative paths without scheme should be ignored
        assert!(resolve_filesystem_path("relative/path").is_none());
    }
}
