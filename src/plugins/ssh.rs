use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::{Context, Result};
use std::fs;
use tracing::{debug, warn};

/// SSH host configuration
#[derive(Debug, Clone)]
struct SshHost {
    /// Host alias/name
    name: String,
    /// Hostname or IP address
    hostname: String,
    /// Username (optional)
    user: Option<String>,
    /// Port (default: 22)
    port: u16,
    /// Identity file path (optional)
    identity_file: Option<String>,
}

impl SshHost {
    /// Generate SSH command string
    fn to_command(&self) -> String {
        let mut cmd = vec!["ssh".to_string()];

        // Add port if not default
        if self.port != 22 {
            cmd.push("-p".to_string());
            cmd.push(self.port.to_string());
        }

        // Add identity file if specified
        if let Some(ref identity) = self.identity_file {
            cmd.push("-i".to_string());
            cmd.push(identity.clone());
        }

        // Build user@host or just host
        let target = if let Some(ref user) = self.user {
            format!("{}@{}", user, self.hostname)
        } else {
            self.hostname.clone()
        };

        cmd.push(target);

        cmd.join(" ")
    }
}

/// Plugin for SSH connections
#[derive(Debug)]
pub struct SshPlugin {
    hosts: Vec<SshHost>,
    enabled: bool,
}

impl SshPlugin {
    /// Create a new SSH plugin
    pub fn new(enabled: bool) -> Self {
        let hosts = Self::parse_ssh_config().unwrap_or_else(|e| {
            warn!("Failed to parse SSH config: {}", e);
            Vec::new()
        });

        debug!("SSH plugin initialized with {} hosts", hosts.len());

        Self { hosts, enabled }
    }

    /// Parse SSH config file
    fn parse_ssh_config() -> Result<Vec<SshHost>> {
        let config_path = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".ssh")
            .join("config");

        if !config_path.exists() {
            debug!("SSH config not found at: {}", config_path.display());
            return Ok(Vec::new());
        }

        debug!("Parsing SSH config from: {}", config_path.display());
        let content = fs::read_to_string(&config_path).context("Failed to read SSH config")?;

        let mut hosts = Vec::new();
        let mut current_host: Option<SshHost> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key-value pairs
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let key = parts[0].to_lowercase();
            let value = parts[1..].join(" ");

            match key.as_str() {
                "host" => {
                    // Save previous host if exists
                    if let Some(host) = current_host.take() {
                        hosts.push(host);
                    }

                    // Skip wildcards
                    if !value.contains('*') && !value.contains('?') {
                        current_host = Some(SshHost {
                            name: value.clone(),
                            hostname: value, // Default to name
                            user: None,
                            port: 22,
                            identity_file: None,
                        });
                    }
                }
                "hostname" => {
                    if let Some(ref mut host) = current_host {
                        host.hostname = value;
                    }
                }
                "user" => {
                    if let Some(ref mut host) = current_host {
                        host.user = Some(value);
                    }
                }
                "port" => {
                    if let Some(ref mut host) = current_host {
                        if let Ok(port) = value.parse::<u16>() {
                            host.port = port;
                        }
                    }
                }
                "identityfile" => {
                    if let Some(ref mut host) = current_host {
                        // Expand ~ to home directory
                        let expanded = if value.starts_with('~') {
                            if let Some(home) = dirs::home_dir() {
                                home.join(value.trim_start_matches("~/"))
                                    .display()
                                    .to_string()
                            } else {
                                value
                            }
                        } else {
                            value
                        };
                        host.identity_file = Some(expanded);
                    }
                }
                _ => {}
            }
        }

        // Save last host
        if let Some(host) = current_host {
            hosts.push(host);
        }

        debug!("Parsed {} SSH hosts", hosts.len());
        Ok(hosts)
    }

    /// Parse known_hosts for additional hosts
    #[allow(dead_code)]

    fn parse_known_hosts() -> Result<Vec<String>> {
        let known_hosts_path = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".ssh")
            .join("known_hosts");

        if !known_hosts_path.exists() {
            return Ok(Vec::new());
        }

        let content =
            fs::read_to_string(&known_hosts_path).context("Failed to read known_hosts")?;

        let mut hosts = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Format: hostname,ip algorithm key [comment]
            if let Some(first_part) = line.split_whitespace().next() {
                // Extract hostname (before comma or space)
                if let Some(hostname) = first_part.split(',').next() {
                    // Skip hashed hostnames
                    if !hostname.starts_with('|') {
                        hosts.push(hostname.to_string());
                    }
                }
            }
        }

        Ok(hosts)
    }
}

impl Plugin for SshPlugin {
    fn name(&self) -> &str {
        "ssh"
    }

    fn description(&self) -> &str {
        "SSH connection launcher"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@ssh"]
    }

    fn priority(&self) -> i32 {
        700 // Between shell (800) and web search (600)
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn should_handle(&self, query: &str) -> bool {
        if !self.enabled || query.len() < 2 {
            return false;
        }

        // Don't interfere with other @ commands (unless it's @ssh)
        if query.starts_with('@') {
            return query.starts_with("@ssh");
        }

        // Trigger on "ssh" prefix or if query matches host name
        query.starts_with("ssh")
            || self.hosts.iter().any(|h| {
                h.name.to_lowercase().contains(&query.to_lowercase())
                    || h.hostname.to_lowercase().contains(&query.to_lowercase())
            })
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let search_query = query_lower
            .strip_prefix("@ssh")
            .or_else(|| query_lower.strip_prefix("ssh"))
            .unwrap_or(&query_lower)
            .trim();

        let mut results = Vec::new();

        for host in &self.hosts {
            // Skip if no match
            if !search_query.is_empty() {
                let name_match = host.name.to_lowercase().contains(search_query);
                let hostname_match = host.hostname.to_lowercase().contains(search_query);

                if !name_match && !hostname_match {
                    continue;
                }
            }

            // Calculate score
            let score = if search_query.is_empty() {
                500 // Default score for "ssh" query
            } else if host.name.to_lowercase() == search_query {
                1000 // Exact match
            } else if host.name.to_lowercase().starts_with(search_query) {
                800 // Prefix match
            } else {
                600 // Contains match
            };

            // Build subtitle
            let mut subtitle_parts = vec![host.hostname.clone()];
            if let Some(ref user) = host.user {
                subtitle_parts.insert(0, format!("{}@", user));
            }
            if host.port != 22 {
                subtitle_parts.push(format!(":{}", host.port));
            }

            let result = PluginResult {
                title: host.name.clone(),
                subtitle: Some(subtitle_parts.join("")),
                icon: Some("network-server".to_string()),
                command: host.to_command(),
                terminal: true, // SSH always runs in terminal
                score,
                plugin_name: self.name().to_string(),
                sub_results: Vec::new(),
                parent_app: None,
            };

            results.push(result);

            if results.len() >= context.max_results {
                break;
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_host_command() {
        let host = SshHost {
            name: "example".to_string(),
            hostname: "example.com".to_string(),
            user: Some("john".to_string()),
            port: 22,
            identity_file: None,
        };

        assert_eq!(host.to_command(), "ssh john@example.com");
    }

    #[test]
    fn test_ssh_host_command_with_port() {
        let host = SshHost {
            name: "example".to_string(),
            hostname: "example.com".to_string(),
            user: Some("john".to_string()),
            port: 2222,
            identity_file: None,
        };

        assert_eq!(host.to_command(), "ssh -p 2222 john@example.com");
    }

    #[test]
    fn test_ssh_host_command_with_identity() {
        let host = SshHost {
            name: "example".to_string(),
            hostname: "example.com".to_string(),
            user: Some("john".to_string()),
            port: 22,
            identity_file: Some("/home/user/.ssh/id_rsa".to_string()),
        };

        assert_eq!(
            host.to_command(),
            "ssh -i /home/user/.ssh/id_rsa john@example.com"
        );
    }

    #[test]
    fn test_ssh_plugin_should_handle() {
        let plugin = SshPlugin::new(true);

        assert!(plugin.should_handle("ssh"));
        assert!(plugin.should_handle("ssh example"));

        // Should not handle very short queries
        assert!(!plugin.should_handle("s"));
    }
}
