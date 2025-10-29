use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;

/// Plugin for executing shell commands
#[derive(Debug)]
pub struct ShellPlugin {
    enabled: bool,
    prefix: String,
}

impl ShellPlugin {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: ">".to_string(),
        }
    }

    /// Create with custom prefix
    pub fn with_prefix(prefix: String) -> Self {
        Self {
            enabled: true,
            prefix,
        }
    }
}

impl Default for ShellPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ShellPlugin {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute shell commands (prefix with @shell or $)"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@shell", "$"]
    }

    fn should_handle(&self, query: &str) -> bool {
        self.enabled
            && (query.starts_with("@shell")
                || query.starts_with('$')
                || query.starts_with(&self.prefix))
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        // Remove prefix - support @shell, $, or custom prefix
        let command = if query.starts_with("@shell") {
            query["@shell".len()..].trim()
        } else if query.starts_with('$') {
            query[1..].trim()
        } else if query.starts_with(&self.prefix) {
            query[self.prefix.len()..].trim()
        } else {
            return Ok(vec![]);
        };

        if command.is_empty() {
            return Ok(vec![]);
        }

        Ok(vec![PluginResult::new(
            format!("Run: {}", command),
            command.to_string(),
            self.name().to_string(),
        )
        .with_subtitle("Execute in terminal".to_string())
        .with_icon("utilities-terminal".to_string())
        .with_terminal(true)
        .with_score(10000)]) // Very high score to show first
    }

    fn priority(&self) -> i32 {
        800 // High priority
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_handle() {
        let shell = ShellPlugin::new();
        assert!(shell.should_handle(">ls -la"));
        assert!(shell.should_handle(">git status"));
        assert!(!shell.should_handle("firefox"));
        assert!(!shell.should_handle("ls -la"));
    }

    #[test]
    fn test_search() {
        use crate::config::Config;

        let shell = ShellPlugin::new();
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = shell.search(">ls -la", &ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("ls -la"));
        assert!(results[0].terminal);
    }
}
