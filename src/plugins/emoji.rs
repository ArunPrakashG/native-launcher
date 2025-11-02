use super::traits::{KeyboardAction, KeyboardEvent, Plugin, PluginContext, PluginResult};
use anyhow::Result;
use serde::Deserialize;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct EmojiPlugin {
    enabled: bool,
    clipboard: Option<ClipboardTool>,
}

impl EmojiPlugin {
    pub fn new() -> Self {
        let clipboard = detect_clipboard_tool();
        Self {
            enabled: true,
            clipboard,
        }
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@emoji") {
            rest
        } else {
            query
        }
    }

    fn results_for(&self, filter: &str, max: usize) -> Vec<PluginResult> {
        let db = EMOJI_DB.get_or_init(load_emoji_db);
        let tokens: Vec<String> = filter
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_lowercase())
            .collect();

        let mut out = Vec::with_capacity(max.min(32));

        for (idx, rec) in db.iter().enumerate() {
            if !tokens.is_empty() {
                let hay = format!(
                    "{} {} {}",
                    rec.name.to_lowercase(),
                    rec.shortcode.to_lowercase(),
                    rec.keywords.join(" ").to_lowercase()
                );
                if !tokens.iter().all(|t| hay.contains(t)) {
                    continue;
                }
            }

            let title = format!("{} {}", rec.ch, rec.name);
            let res = PluginResult::new(
                title,
                self.build_copy_command(&rec.ch),
                self.name().to_string(),
            )
            .with_subtitle(format!(":{}:", rec.shortcode))
            .with_icon(format!("emoji:{}", rec.ch))
            .with_score(9000 - idx as i64);
            out.push(res);
            if out.len() >= max {
                break;
            }
        }

        out
    }

    fn build_copy_command(&self, ch: &str) -> String {
        let content = shell_escape(ch);
        if let Some(tool) = &self.clipboard {
            let pipe = match tool {
                ClipboardTool::WlCopy { command } => format!("printf {} | {}", content, command),
                ClipboardTool::Xclip { command } => {
                    format!("printf {} | {} -selection clipboard", content, command)
                }
                ClipboardTool::Xsel { command } => {
                    format!("printf {} | {} --clipboard --input", content, command)
                }
            };
            return format!("sh -c {}", shell_escape(&pipe));
        }
        // Fallback: try wl-copy then xclip then xsel
        let pipe = format!(
            "printf {} | wl-copy || printf {} | xclip -selection clipboard || printf {} | xsel --clipboard --input",
            content, content, content
        );
        format!("sh -c {}", shell_escape(&pipe))
    }
}

impl Plugin for EmojiPlugin {
    fn name(&self) -> &str {
        "emoji"
    }

    fn description(&self) -> &str {
        "Emoji picker via @emoji"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@emoji"]
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@emoji")
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        let filter = self.strip_prefix(query).trim();
        Ok(self.results_for(filter, context.max_results))
    }

    fn priority(&self) -> i32 {
        300
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle_keyboard_event(&self, _event: &KeyboardEvent) -> KeyboardAction {
        // Enter behavior is handled by default execution path
        KeyboardAction::None
    }
}

#[derive(Debug, Deserialize)]
struct EmojiRecord {
    ch: String,
    name: String,
    shortcode: String,
    #[serde(default)]
    keywords: Vec<String>,
}

static EMOJI_DB: OnceLock<Vec<EmojiRecord>> = OnceLock::new();

fn load_emoji_db() -> Vec<EmojiRecord> {
    // Minimal embedded dataset; can be extended without runtime I/O cost
    const DATA: &str = r#"[
        {"ch":"😀","name":"Grinning Face","shortcode":"grinning","keywords":["smile","happy","joy"]},
        {"ch":"😂","name":"Face with Tears of Joy","shortcode":"joy","keywords":["laugh","tears","lol"]},
        {"ch":"😊","name":"Smiling Face with Smiling Eyes","shortcode":"blush","keywords":["smile","happy","warm"]},
        {"ch":"👍","name":"Thumbs Up","shortcode":"+1","keywords":["approve","ok","yes"]},
        {"ch":"🔥","name":"Fire","shortcode":"fire","keywords":["lit","hot","trend"]},
        {"ch":"❤️","name":"Red Heart","shortcode":"heart","keywords":["love","like","affection"]},
        {"ch":"🎉","name":"Party Popper","shortcode":"tada","keywords":["celebrate","party","congrats"]},
        {"ch":"🙏","name":"Folded Hands","shortcode":"pray","keywords":["please","thanks","namaste"]},
        {"ch":"🤔","name":"Thinking Face","shortcode":"thinking","keywords":["hmm","consider","idea"]},
        {"ch":"🚀","name":"Rocket","shortcode":"rocket","keywords":["ship","fast","launch"]}
    ]"#;
    match serde_json::from_str::<Vec<EmojiRecord>>(DATA) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    }
}

#[derive(Debug, Clone)]
enum ClipboardTool {
    WlCopy { command: String },
    Xclip { command: String },
    Xsel { command: String },
}

fn detect_clipboard_tool() -> Option<ClipboardTool> {
    if let Some(cmd) = command_path("wl-copy") {
        return Some(ClipboardTool::WlCopy { command: cmd });
    }
    if let Some(cmd) = command_path("xclip") {
        return Some(ClipboardTool::Xclip { command: cmd });
    }
    if let Some(cmd) = command_path("xsel") {
        return Some(ClipboardTool::Xsel { command: cmd });
    }
    None
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
    fn filters_by_keyword() {
        let plugin = EmojiPlugin {
            enabled: true,
            clipboard: None,
        };
        let cfg = Config::default();
        let ctx = PluginContext::new(10, &cfg);
        let res = plugin.search("@emoji joy", &ctx).unwrap();
        assert!(!res.is_empty());
        assert!(res.iter().any(|r| r.title.contains("😂")));
    }

    #[test]
    fn builds_copy_command() {
        let plugin = EmojiPlugin {
            enabled: true,
            clipboard: Some(ClipboardTool::WlCopy {
                command: "wl-copy".to_string(),
            }),
        };
        let cmd = plugin.build_copy_command("😀");
        assert!(cmd.starts_with("sh -c "));
        assert!(cmd.contains("wl-copy"));
    }
}
