use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;

/// Launcher management plugin - self-update and maintenance helpers
/// Triggered with @launcher, @updater, or @native-launcher
#[derive(Debug)]
pub struct LauncherPlugin {
    enabled: bool,
}

impl LauncherPlugin {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Plugin for LauncherPlugin {
    fn name(&self) -> &str {
        "launcher"
    }

    fn description(&self) -> &str {
        "Self-updater and maintenance commands for native-launcher"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@launcher", "@updater", "@native-launcher"]
    }

    fn priority(&self) -> i32 {
        1200 // High priority but lower than theme switcher
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn should_handle(&self, query: &str) -> bool {
        if !self.enabled || query.is_empty() {
            return false;
        }

        if query.starts_with('@') {
            return query.starts_with("@launcher")
                || query.starts_with("@updater")
                || query.starts_with("@native-launcher");
        }

        false
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        let mut results: Vec<PluginResult> = Vec::new();

        // Normalize and strip the prefix (one of the supported prefixes)
        let q = query.trim();
        let remainder = q
            .strip_prefix("@launcher")
            .or_else(|| q.strip_prefix("@updater"))
            .or_else(|| q.strip_prefix("@native-launcher"))
            .unwrap_or("")
            .trim()
            .to_lowercase();

        // Primary action: update (install.sh)
        if remainder.is_empty()
            || remainder.starts_with("update")
            || remainder.starts_with("install")
            || remainder.starts_with("upgrade")
            || remainder == "u"
        {
            let cmd = format!("cd \"{}\" && ./install.sh", env!("CARGO_MANIFEST_DIR"));
            results.push(
                PluginResult::new(
                    "Update native-launcher".to_string(),
                    cmd,
                    self.name().to_string(),
                )
                .with_subtitle(
                    "Run the repository install script to update/reinstall the launcher"
                        .to_string(),
                )
                .with_icon("system-software-update".to_string())
                .with_terminal(true)
                .with_score(9000),
            );
        }

        // Restore action (restore.sh)
        if remainder.is_empty() || remainder.starts_with("restore") {
            let cmd = format!("cd \"{}\" && ./restore.sh", env!("CARGO_MANIFEST_DIR"));
            results.push(
                PluginResult::new(
                    "Restore native-launcher state".to_string(),
                    cmd,
                    self.name().to_string(),
                )
                .with_subtitle("Run the restore script (restores config/backups)".to_string())
                .with_icon("edit-restore".to_string())
                .with_terminal(true)
                .with_score(8000),
            );
        }

        // Uninstall action (uninstall.sh)
        if remainder.is_empty() || remainder.starts_with("uninstall") {
            let cmd = format!("cd \"{}\" && ./uninstall.sh", env!("CARGO_MANIFEST_DIR"));
            results.push(
                PluginResult::new(
                    "Uninstall native-launcher".to_string(),
                    cmd,
                    self.name().to_string(),
                )
                .with_subtitle("Run uninstall script (use with caution)".to_string())
                .with_icon("user-trash".to_string())
                .with_terminal(true)
                .with_score(7000),
            );
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::plugins::traits::PluginContext;

    #[test]
    fn test_should_handle() {
        let p = LauncherPlugin::new(true);
        assert!(p.should_handle("@launcher"));
        assert!(p.should_handle("@launcher update"));
        assert!(p.should_handle("@updater"));
        assert!(!p.should_handle("@theme"));
    }

    #[test]
    fn test_search_update() {
        let p = LauncherPlugin::new(true);
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);
        let results = p.search("@launcher update", &ctx).unwrap();
        assert!(results.iter().any(|r| r.command.contains("install.sh")));
    }
}
