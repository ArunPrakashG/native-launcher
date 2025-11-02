use super::traits::{KeyboardAction, KeyboardEvent, Plugin, PluginContext, PluginResult};
use anyhow::Result;
use std::process::Command;

#[derive(Debug)]
pub struct ClipboardPlugin {
    enabled: bool,
    provider: Option<Provider>,
}

#[derive(Debug, Clone)]
enum Provider {
    Cliphist { path: String },
}

#[derive(Debug, Clone)]
struct Entry {
    id: String,
    preview: String,
    mime: Option<String>,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        let provider = detect_provider();
        Self {
            enabled: provider.is_some(),
            provider,
        }
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@clip") {
            rest
        } else {
            query
        }
    }

    fn search_entries(&self, filter: &str, max: usize) -> Vec<Entry> {
        match &self.provider {
            Some(Provider::Cliphist { path }) => fetch_cliphist_entries(path, filter, max),
            None => Vec::new(),
        }
    }

    fn build_copy_command(&self, entry: &Entry) -> String {
        match &self.provider {
            Some(Provider::Cliphist { path }) => {
                // cliphist decode <id> | wl-copy || xclip || xsel
                let pipe = format!(
                    "{} decode {} | wl-copy || {} decode {} | xclip -selection clipboard || {} decode {} | xsel --clipboard --input",
                    shell_escape(path),
                    shell_escape(&entry.id),
                    shell_escape(path),
                    shell_escape(&entry.id),
                    shell_escape(path),
                    shell_escape(&entry.id),
                );
                format!("sh -c {}", shell_escape(&pipe))
            }
            None => "true".to_string(),
        }
    }
}

impl Plugin for ClipboardPlugin {
    fn name(&self) -> &str {
        "clipboard"
    }

    fn description(&self) -> &str {
        "Clipboard history via @clip (cliphist)"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@clip"]
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@clip")
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            // Show one helpful row
            return Ok(vec![PluginResult::new(
                "Clipboard history not available".to_string(),
                "true".to_string(),
                self.name().to_string(),
            )
            .with_subtitle("Install 'cliphist' to enable @clip".to_string())
            .with_icon("edit-paste".to_string())
            .with_score(0)]);
        }

        let filter = self.strip_prefix(query).trim();
        let entries = self.search_entries(filter, context.max_results);

        let mut results = Vec::with_capacity(entries.len());
        for (idx, e) in entries.into_iter().enumerate() {
            let title = elide(&e.preview, 120);
            let mut pr =
                PluginResult::new(title, self.build_copy_command(&e), self.name().to_string())
                    .with_icon("edit-paste".to_string())
                    .with_score(10_000 - idx as i64);
            if let Some(mime) = &e.mime {
                pr = pr.with_subtitle(mime.clone());
            }
            results.push(pr);
        }

        Ok(results)
    }

    fn priority(&self) -> i32 {
        320
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle_keyboard_event(&self, _event: &KeyboardEvent) -> KeyboardAction {
        // Enter executes default copy command; Shift+Enter may be added later if needed
        KeyboardAction::None
    }
}

fn detect_provider() -> Option<Provider> {
    if let Some(path) = command_path("cliphist") {
        return Some(Provider::Cliphist { path });
    }
    None
}

fn fetch_cliphist_entries(path: &str, filter: &str, max: usize) -> Vec<Entry> {
    // Call: cliphist list
    let output = Command::new(path).arg("list").output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut out: Vec<Entry> = Vec::with_capacity(max);

    let tokens: Vec<String> = filter
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();

    for line in text.lines() {
        if let Some(e) = parse_cliphist_line(line) {
            if !tokens.is_empty() {
                let hay = format!(
                    "{} {}",
                    e.preview.to_lowercase(),
                    e.mime.clone().unwrap_or_default().to_lowercase()
                );
                if !tokens.iter().all(|t| hay.contains(t)) {
                    continue;
                }
            }
            out.push(e);
            if out.len() >= max {
                break;
            }
        }
    }
    out
}

fn parse_cliphist_line(line: &str) -> Option<Entry> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Heuristic: first token is id; try to split by whitespace or tab
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let id = parts.next()?.trim();
    let rest = parts.next().unwrap_or("").trim();
    if id.is_empty() {
        return None;
    }

    // Try to extract mime like "(text/plain)" from rest if present
    let (preview, mime) = if let Some(start) = rest.find('(') {
        if let Some(end) = rest[start..].find(')') {
            let m = rest[start + 1..start + end].trim().to_string();
            let p = rest[..start].trim().to_string();
            (if p.is_empty() { rest.to_string() } else { p }, Some(m))
        } else {
            (rest.to_string(), None)
        }
    } else {
        (rest.to_string(), None)
    };

    Some(Entry {
        id: id.to_string(),
        preview: if preview.is_empty() {
            rest.to_string()
        } else {
            preview
        },
        mime,
    })
}

fn command_path(command: &str) -> Option<String> {
    Command::new("which")
        .arg(command)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        })
}

fn elide(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    let mut count = 0;
    for ch in s.chars() {
        if count >= max_chars {
            break;
        }
        out.push(ch);
        count += 1;
    }
    if s.chars().count() > max_chars {
        out.push_str("…");
    }
    out
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let mut escaped = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn parse_line_extracts_id_and_preview() {
        let e = parse_cliphist_line("1234 text/plain Hello world").unwrap();
        assert_eq!(e.id, "1234");
        assert!(e.preview.contains("Hello"));
    }

    #[test]
    fn should_handle_prefix() {
        let plugin = ClipboardPlugin {
            enabled: true,
            provider: None,
        };
        assert!(plugin.should_handle("@clip foo"));
        assert!(!plugin.should_handle("clip foo"));
    }

    #[test]
    fn build_copy_command_contains_cliphist_decode() {
        let plugin = ClipboardPlugin {
            enabled: true,
            provider: Some(Provider::Cliphist {
                path: "cliphist".to_string(),
            }),
        };
        let cmd = plugin.build_copy_command(&Entry {
            id: "42".to_string(),
            preview: "x".to_string(),
            mime: None,
        });
        assert!(cmd.contains("cliphist"));
        assert!(cmd.contains("decode"));
        assert!(cmd.contains("42"));
    }

    #[test]
    fn search_without_provider_shows_info() {
        let plugin = ClipboardPlugin {
            enabled: false,
            provider: None,
        };
        let cfg = Config::default();
        let ctx = PluginContext::new(10, &cfg);
        let res = plugin.search("@clip", &ctx).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].title.contains("not available"));
    }
}
