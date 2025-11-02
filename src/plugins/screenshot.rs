use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::{Context, Result};
use chrono::Local;
use dirs::{home_dir, picture_dir};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

#[derive(Debug)]
pub struct ScreenshotPlugin {
    backend: Option<ScreenshotBackend>,
    output_dir: PathBuf,
    enabled: bool,
    clipboard: Option<ClipboardTool>,
    annotator: Option<AnnotatorTool>,
}

impl ScreenshotPlugin {
    pub fn new() -> Self {
        let backend = detect_backend();
        let output_dir = default_output_directory();
        let clipboard = detect_clipboard_tool();
        let annotator = detect_annotator_tool();

        if let Some(ref backend) = backend {
            debug!(
                "screenshot plugin using backend '{}'",
                backend.display_name()
            );
        } else {
            debug!("screenshot plugin did not detect an available backend");
        }

        if let Some(ref clipboard) = clipboard {
            debug!(
                "screenshot plugin will copy captures to clipboard using {}",
                clipboard.display_name()
            );
        } else {
            debug!("screenshot plugin did not detect a clipboard utility");
        }

        if let Some(ref annotator) = annotator {
            debug!(
                "screenshot plugin will support annotation using {}",
                annotator.display_name()
            );
        } else {
            debug!("screenshot plugin did not detect an annotation tool");
        }

        Self {
            backend,
            output_dir,
            enabled: true,
            clipboard,
            annotator,
        }
    }

    #[cfg(test)]
    fn with_backend(backend: Option<ScreenshotBackend>, output_dir: PathBuf) -> Self {
        Self {
            backend,
            output_dir,
            enabled: true,
            clipboard: None,
            annotator: None,
        }
    }

    fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        if let Some(rest) = query.strip_prefix("@screenshot") {
            rest
        } else if let Some(rest) = query.strip_prefix("@ss") {
            rest
        } else {
            query
        }
    }

    fn ensure_output_dir(&self) -> Result<()> {
        if self.output_dir.exists() {
            return Ok(());
        }

        fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "failed to create screenshot directory {}",
                self.output_dir.display()
            )
        })
    }

    fn build_output_path(&self, mode: ScreenshotMode) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let filename = format!("screenshot-{}-{}.png", mode.file_stem(), timestamp);
        self.output_dir.join(filename)
    }

    fn score_for(&self, mode: ScreenshotMode, index: usize, has_filter: bool) -> i64 {
        let base = match mode {
            ScreenshotMode::Fullscreen => 9900,
            ScreenshotMode::Window => 9850,
            ScreenshotMode::Area => 9800,
            ScreenshotMode::AnnotateFullscreen => 9750,
            ScreenshotMode::AnnotateWindow => 9700,
            ScreenshotMode::AnnotateArea => 9650,
        };

        let filter_bonus = if has_filter { 200 } else { 0 };
        base + filter_bonus - (index as i64 * 10)
    }

    fn no_backend_result(&self) -> PluginResult {
        PluginResult::new(
            "No screenshot utility detected".to_string(),
            String::new(),
            self.name().to_string(),
        )
        .with_subtitle(
            "Install grimshot, hyprshot, gnome-screenshot, spectacle, maim, or scrot".to_string(),
        )
        .with_icon("dialog-warning".to_string())
        .with_score(1000)
    }

    fn no_results_message(&self, filter: &str) -> PluginResult {
        let filter = filter.trim();
        let subtitle = if filter.is_empty() {
            "No screenshot options available".to_string()
        } else {
            format!("No screenshot mode matches \"{}\"", filter)
        };

        PluginResult::new(
            "No matching screenshot option".to_string(),
            String::new(),
            self.name().to_string(),
        )
        .with_subtitle(subtitle)
        .with_icon("dialog-information".to_string())
        .with_score(1000)
    }
}

impl Plugin for ScreenshotPlugin {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Capture screenshots via @screenshot or @ss"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@screenshot", "@ss"]
    }

    fn should_handle(&self, query: &str) -> bool {
        query.starts_with("@screenshot") || query.starts_with("@ss")
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let backend = match &self.backend {
            Some(backend) => backend,
            None => return Ok(vec![self.no_backend_result()]),
        };

        self.ensure_output_dir()?;

        let filter = self.strip_prefix(query).trim().to_lowercase();
        let mut modes = backend.supported_modes();

        // Add annotation modes if annotator is available
        if self.annotator.is_some() {
            // Check if backend supports the corresponding capture mode
            for capture_mode in backend.supported_modes() {
                match capture_mode {
                    ScreenshotMode::Fullscreen => modes.push(ScreenshotMode::AnnotateFullscreen),
                    ScreenshotMode::Window => modes.push(ScreenshotMode::AnnotateWindow),
                    ScreenshotMode::Area => modes.push(ScreenshotMode::AnnotateArea),
                    _ => {} // Skip annotation modes themselves
                }
            }
        }

        let mut results = Vec::new();

        for (idx, mode) in modes.iter().enumerate() {
            if !filter.is_empty() && !mode.matches(&filter) {
                continue;
            }

            let output_path = self.build_output_path(*mode);
            let path_string = output_path.to_string_lossy().to_string();
            let escaped_path = shell_escape(&path_string);

            // Check if this is an annotation mode
            let is_annotation_mode = matches!(
                mode,
                ScreenshotMode::AnnotateFullscreen
                    | ScreenshotMode::AnnotateWindow
                    | ScreenshotMode::AnnotateArea
            );

            let base_command = match backend.command_for(*mode, &escaped_path) {
                Some(cmd) => cmd,
                None => continue,
            };

            let command = if is_annotation_mode {
                // For annotation mode: capture | swappy -f - -o output_path
                if let Some(ref annotator) = self.annotator {
                    match annotator {
                        AnnotatorTool::Swappy {
                            command: swappy_cmd,
                        } => {
                            let annotate_cmd = format!(
                                "{} | {} -f - -o {}",
                                base_command, swappy_cmd, escaped_path
                            );
                            if let Some(ref clipboard) = self.clipboard {
                                let copy_command = clipboard.command(&escaped_path);
                                let combined = format!("{} && {}", annotate_cmd, copy_command);
                                format!("sh -c {}", shell_escape(&combined))
                            } else {
                                format!("sh -c {}", shell_escape(&annotate_cmd))
                            }
                        }
                    }
                } else {
                    // Should not happen as we only add annotation modes if annotator exists
                    continue;
                }
            } else if let Some(ref clipboard) = self.clipboard {
                let copy_command = clipboard.command(&escaped_path);
                let combined = format!("{} && {}", base_command, copy_command);
                format!("sh -c {}", shell_escape(&combined))
            } else {
                base_command
            };

            let friendly = friendly_path(&output_path);
            let mut subtitle = if is_annotation_mode {
                format!(
                    "Using {} + {} • saves to {}",
                    backend.display_name(),
                    self.annotator.as_ref().unwrap().display_name(),
                    friendly
                )
            } else {
                format!("Using {} • saves to {}", backend.display_name(), friendly)
            };

            if let Some(ref clipboard) = self.clipboard {
                subtitle.push_str(&format!(
                    " • copies to clipboard ({})",
                    clipboard.display_name()
                ));
            }

            let score = self.score_for(*mode, idx, !filter.is_empty());

            let result = PluginResult::new(
                format!("{} screenshot", mode.label()),
                command,
                self.name().to_string(),
            )
            .with_subtitle(subtitle)
            .with_icon("camera-photo".to_string())
            .with_score(score);

            results.push(result);
        }

        if results.is_empty() {
            return Ok(vec![self.no_results_message(&filter)]);
        }

        Ok(results)
    }

    fn priority(&self) -> i32 {
        750
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Copy)]
enum ScreenshotMode {
    Fullscreen,
    Window,
    Area,
    AnnotateFullscreen,
    AnnotateWindow,
    AnnotateArea,
}

impl ScreenshotMode {
    fn label(self) -> &'static str {
        match self {
            ScreenshotMode::Fullscreen => "Full Screen",
            ScreenshotMode::Window => "Active Window",
            ScreenshotMode::Area => "Select Area",
            ScreenshotMode::AnnotateFullscreen => "Annotate Full Screen",
            ScreenshotMode::AnnotateWindow => "Annotate Active Window",
            ScreenshotMode::AnnotateArea => "Annotate Area",
        }
    }

    fn file_stem(self) -> &'static str {
        match self {
            ScreenshotMode::Fullscreen => "full",
            ScreenshotMode::Window => "window",
            ScreenshotMode::Area => "area",
            ScreenshotMode::AnnotateFullscreen => "annotate-full",
            ScreenshotMode::AnnotateWindow => "annotate-window",
            ScreenshotMode::AnnotateArea => "annotate-area",
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            ScreenshotMode::Fullscreen => &["full", "screen", "monitor", "entire", "whole"],
            ScreenshotMode::Window => &["window", "focused", "active", "app"],
            ScreenshotMode::Area => &["area", "region", "select", "selection", "frame", "partial"],
            ScreenshotMode::AnnotateFullscreen => &["annotate", "edit", "draw", "full", "screen"],
            ScreenshotMode::AnnotateWindow => &["annotate", "edit", "draw", "window"],
            ScreenshotMode::AnnotateArea => &["annotate", "edit", "draw", "area", "region"],
        }
    }

    fn matches(self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }

        filter.split_whitespace().all(|word| {
            let word = word.trim().to_lowercase();
            if word.is_empty() {
                return true;
            }
            self.keywords()
                .iter()
                .any(|keyword| keyword.contains(&word) || word.contains(keyword))
        })
    }
}

#[derive(Debug, Clone)]
struct ScreenshotBackend {
    tool: ScreenshotTool,
}

impl ScreenshotBackend {
    fn grimshot(command: String) -> Self {
        Self {
            tool: ScreenshotTool::Grimshot { command },
        }
    }

    fn hyprshot(command: String) -> Self {
        Self {
            tool: ScreenshotTool::Hyprshot { command },
        }
    }

    fn gnome(command: String) -> Self {
        Self {
            tool: ScreenshotTool::GnomeScreenshot { command },
        }
    }

    fn spectacle(command: String) -> Self {
        Self {
            tool: ScreenshotTool::Spectacle { command },
        }
    }

    fn maim(command: String, xdotool: Option<String>) -> Self {
        Self {
            tool: ScreenshotTool::Maim { command, xdotool },
        }
    }

    fn scrot(command: String) -> Self {
        Self {
            tool: ScreenshotTool::Scrot { command },
        }
    }

    fn grim_slurp(grim: String, slurp: String) -> Self {
        Self {
            tool: ScreenshotTool::GrimSlurp { grim, slurp },
        }
    }

    fn display_name(&self) -> &'static str {
        match &self.tool {
            ScreenshotTool::Grimshot { .. } => "grimshot",
            ScreenshotTool::Hyprshot { .. } => "hyprshot",
            ScreenshotTool::GnomeScreenshot { .. } => "gnome-screenshot",
            ScreenshotTool::Spectacle { .. } => "spectacle",
            ScreenshotTool::Maim { .. } => "maim",
            ScreenshotTool::Scrot { .. } => "scrot",
            ScreenshotTool::GrimSlurp { .. } => "grim + slurp",
        }
    }

    fn supported_modes(&self) -> Vec<ScreenshotMode> {
        match &self.tool {
            ScreenshotTool::Maim { xdotool, .. } if xdotool.is_none() => {
                vec![ScreenshotMode::Fullscreen, ScreenshotMode::Area]
            }
            ScreenshotTool::GrimSlurp { .. } => {
                vec![ScreenshotMode::Fullscreen, ScreenshotMode::Area]
            }
            _ => vec![
                ScreenshotMode::Fullscreen,
                ScreenshotMode::Window,
                ScreenshotMode::Area,
            ],
        }
    }

    fn command_for(&self, mode: ScreenshotMode, path: &str) -> Option<String> {
        match (&self.tool, mode) {
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} save screen {}", command, path))
            }
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::Window) => {
                Some(format!("{} save window {}", command, path))
            }
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::Area) => {
                Some(format!("{} save area {}", command, path))
            }
            (ScreenshotTool::Hyprshot { command }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} -m fullscreen -o {}", command, path))
            }
            (ScreenshotTool::Hyprshot { command }, ScreenshotMode::Window) => {
                Some(format!("{} -m active -o {}", command, path))
            }
            (ScreenshotTool::Hyprshot { command }, ScreenshotMode::Area) => {
                Some(format!("{} -m region -o {}", command, path))
            }
            (ScreenshotTool::GnomeScreenshot { command }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} --file={} --delay=0", command, path))
            }
            (ScreenshotTool::GnomeScreenshot { command }, ScreenshotMode::Window) => {
                Some(format!("{} --window --file={} --delay=0", command, path))
            }
            (ScreenshotTool::GnomeScreenshot { command }, ScreenshotMode::Area) => {
                Some(format!("{} --area --file={} --delay=0", command, path))
            }
            (ScreenshotTool::Spectacle { command }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} -b -n -o {}", command, path))
            }
            (ScreenshotTool::Spectacle { command }, ScreenshotMode::Window) => {
                Some(format!("{} -b -n -a -o {}", command, path))
            }
            (ScreenshotTool::Spectacle { command }, ScreenshotMode::Area) => {
                Some(format!("{} -b -n -r -o {}", command, path))
            }
            (
                ScreenshotTool::Maim {
                    command,
                    xdotool: Some(xdotool),
                },
                ScreenshotMode::Window,
            ) => Some(format!(
                "{} -i $({} getactivewindow) {}",
                command, xdotool, path
            )),
            (ScreenshotTool::Maim { xdotool: None, .. }, ScreenshotMode::Window) => None,
            (ScreenshotTool::Maim { command, .. }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} {}", command, path))
            }
            (ScreenshotTool::Maim { command, .. }, ScreenshotMode::Area) => {
                Some(format!("{} -s {}", command, path))
            }
            (ScreenshotTool::Scrot { command }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} {}", command, path))
            }
            (ScreenshotTool::Scrot { command }, ScreenshotMode::Window) => {
                Some(format!("{} -u {}", command, path))
            }
            (ScreenshotTool::Scrot { command }, ScreenshotMode::Area) => {
                Some(format!("{} -s {}", command, path))
            }
            (ScreenshotTool::GrimSlurp { grim, .. }, ScreenshotMode::Fullscreen) => {
                Some(format!("{} {}", grim, path))
            }
            (ScreenshotTool::GrimSlurp { grim, slurp }, ScreenshotMode::Area) => {
                Some(format!("{} -g \"$({})\" {}", grim, slurp, path))
            }
            (ScreenshotTool::GrimSlurp { .. }, ScreenshotMode::Window) => None,
            // Annotation modes - capture to stdout and pipe to annotation tool
            // Note: These commands will be wrapped with annotation tool in the search method
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::AnnotateFullscreen) => {
                Some(format!("{} save screen -", command))
            }
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::AnnotateWindow) => {
                Some(format!("{} save window -", command))
            }
            (ScreenshotTool::Grimshot { command }, ScreenshotMode::AnnotateArea) => {
                Some(format!("{} save area -", command))
            }
            (ScreenshotTool::Hyprshot { .. }, ScreenshotMode::AnnotateFullscreen) => None,
            (ScreenshotTool::Hyprshot { .. }, ScreenshotMode::AnnotateWindow) => None,
            (ScreenshotTool::Hyprshot { .. }, ScreenshotMode::AnnotateArea) => None,
            (ScreenshotTool::GnomeScreenshot { .. }, ScreenshotMode::AnnotateFullscreen) => None,
            (ScreenshotTool::GnomeScreenshot { .. }, ScreenshotMode::AnnotateWindow) => None,
            (ScreenshotTool::GnomeScreenshot { .. }, ScreenshotMode::AnnotateArea) => None,
            (ScreenshotTool::Spectacle { .. }, ScreenshotMode::AnnotateFullscreen) => None,
            (ScreenshotTool::Spectacle { .. }, ScreenshotMode::AnnotateWindow) => None,
            (ScreenshotTool::Spectacle { .. }, ScreenshotMode::AnnotateArea) => None,
            (ScreenshotTool::Maim { .. }, ScreenshotMode::AnnotateFullscreen) => None,
            (ScreenshotTool::Maim { .. }, ScreenshotMode::AnnotateWindow) => None,
            (ScreenshotTool::Maim { .. }, ScreenshotMode::AnnotateArea) => None,
            (ScreenshotTool::Scrot { .. }, ScreenshotMode::AnnotateFullscreen) => None,
            (ScreenshotTool::Scrot { .. }, ScreenshotMode::AnnotateWindow) => None,
            (ScreenshotTool::Scrot { .. }, ScreenshotMode::AnnotateArea) => None,
            (ScreenshotTool::GrimSlurp { grim, .. }, ScreenshotMode::AnnotateFullscreen) => {
                Some(format!("{} -", grim))
            }
            (ScreenshotTool::GrimSlurp { grim, slurp }, ScreenshotMode::AnnotateArea) => {
                Some(format!("{} -g \"$({})\" -", grim, slurp))
            }
            (ScreenshotTool::GrimSlurp { .. }, ScreenshotMode::AnnotateWindow) => None,
        }
    }
}

#[derive(Debug, Clone)]
enum ClipboardTool {
    WlCopy { command: String },
    Xclip { command: String },
    Xsel { command: String },
}

#[derive(Debug, Clone)]
enum AnnotatorTool {
    Swappy { command: String },
}

impl ClipboardTool {
    fn command(&self, escaped_path: &str) -> String {
        match self {
            ClipboardTool::WlCopy { command } => {
                format!("{} --type image/png < {}", command, escaped_path)
            }
            ClipboardTool::Xclip { command } => {
                format!(
                    "{} -selection clipboard -target image/png < {}",
                    command, escaped_path
                )
            }
            ClipboardTool::Xsel { command } => {
                format!(
                    "{} --clipboard --input --mime-type image/png < {}",
                    command, escaped_path
                )
            }
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ClipboardTool::WlCopy { .. } => "wl-copy",
            ClipboardTool::Xclip { .. } => "xclip",
            ClipboardTool::Xsel { .. } => "xsel",
        }
    }
}

impl AnnotatorTool {
    fn display_name(&self) -> &'static str {
        match self {
            AnnotatorTool::Swappy { .. } => "swappy",
        }
    }
}

#[derive(Debug, Clone)]
enum ScreenshotTool {
    Grimshot {
        command: String,
    },
    Hyprshot {
        command: String,
    },
    GnomeScreenshot {
        command: String,
    },
    Spectacle {
        command: String,
    },
    Maim {
        command: String,
        xdotool: Option<String>,
    },
    Scrot {
        command: String,
    },
    GrimSlurp {
        grim: String,
        slurp: String,
    },
}

fn detect_backend() -> Option<ScreenshotBackend> {
    if let Some(cmd) = command_path("grimshot") {
        debug!("screenshot plugin: detected grimshot at {}", cmd);
        return Some(ScreenshotBackend::grimshot(cmd));
    }

    if let Some(cmd) = command_path("hyprshot") {
        debug!("screenshot plugin: detected hyprshot at {}", cmd);
        return Some(ScreenshotBackend::hyprshot(cmd));
    }

    if let Some(cmd) = command_path("gnome-screenshot") {
        debug!("screenshot plugin: detected gnome-screenshot at {}", cmd);
        return Some(ScreenshotBackend::gnome(cmd));
    }

    if let Some(cmd) = command_path("spectacle") {
        debug!("screenshot plugin: detected spectacle at {}", cmd);
        return Some(ScreenshotBackend::spectacle(cmd));
    }

    if let (Some(grim), Some(slurp)) = (command_path("grim"), command_path("slurp")) {
        debug!(
            "screenshot plugin: detected grim ({}) with slurp ({})",
            grim, slurp
        );
        return Some(ScreenshotBackend::grim_slurp(grim, slurp));
    }

    if let Some(cmd) = command_path("maim") {
        let xdotool = command_path("xdotool");
        debug!(
            "screenshot plugin: detected maim at {} (xdotool: {:?})",
            cmd, xdotool
        );
        return Some(ScreenshotBackend::maim(cmd, xdotool));
    }

    if let Some(cmd) = command_path("scrot") {
        debug!("screenshot plugin: detected scrot at {}", cmd);
        return Some(ScreenshotBackend::scrot(cmd));
    }

    None
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

fn detect_annotator_tool() -> Option<AnnotatorTool> {
    if let Some(cmd) = command_path("swappy") {
        return Some(AnnotatorTool::Swappy { command: cmd });
    }

    None
}

fn command_path(command: &str) -> Option<String> {
    Command::new("which")
        .arg(command)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
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

fn friendly_path(path: &Path) -> String {
    let display = path.to_string_lossy().to_string();
    if let Some(home) = home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if display.starts_with(&home_str) {
            return display.replacen(&home_str, "~", 1);
        }
    }
    display
}

fn default_output_directory() -> PathBuf {
    if let Some(pictures) = picture_dir() {
        return pictures.join("Screenshots");
    }

    if let Some(home) = home_dir() {
        return home.join("Pictures").join("Screenshots");
    }

    env::temp_dir().join("native-launcher").join("Screenshots")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_output_dir() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        env::temp_dir().join(format!("nl-screenshot-test-{}", ts))
    }

    #[test]
    fn returns_message_when_no_backend() {
        let output = temp_output_dir();
        let plugin = ScreenshotPlugin::with_backend(None, output.clone());
        let config = Config::default();
        let ctx = PluginContext::new(5, &config);

        let results = plugin.search("@ss", &ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("No screenshot"));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn filters_window_option_with_scrot_backend() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::scrot("scrot".to_string());
        let plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        let config = Config::default();
        let ctx = PluginContext::new(5, &config);

        let results = plugin.search("@ss window", &ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Window"));
        assert!(results[0].command.contains("-u"));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn provides_multiple_modes_with_grimshot_backend() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        let config = Config::default();
        let ctx = PluginContext::new(5, &config);

        let results = plugin.search("@screenshot", &ctx).unwrap();
        assert!(results.len() >= 2);
        assert!(results.iter().any(|r| r.title.contains("Full")));
        assert!(results.iter().any(|r| r.title.contains("Area")));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn appends_clipboard_command_when_available() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let mut plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        plugin.clipboard = Some(ClipboardTool::WlCopy {
            command: "wl-copy".to_string(),
        });

        let config = Config::default();
        let ctx = PluginContext::new(5, &config);

        let results = plugin.search("@ss", &ctx).unwrap();
        assert!(!results.is_empty());

        let command = &results[0].command;
        assert!(command.starts_with("sh -c "));
        assert!(command.contains("wl-copy"));
        assert!(command.contains("&&"));

        let subtitle = results[0]
            .subtitle
            .as_ref()
            .expect("expected subtitle for screenshot result");
        assert!(subtitle.contains("copies to clipboard"));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn provides_annotation_modes_when_annotator_available() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let mut plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        plugin.annotator = Some(AnnotatorTool::Swappy {
            command: "swappy".to_string(),
        });

        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@screenshot", &ctx).unwrap();

        // Should have 3 regular modes + 3 annotation modes = 6 total
        assert!(
            results.len() >= 6,
            "Expected at least 6 modes (3 regular + 3 annotate), got {}",
            results.len()
        );

        // Check for annotation modes
        assert!(results.iter().any(|r| r.title.contains("Annotate Full")));
        assert!(results.iter().any(|r| r.title.contains("Annotate Active")));
        assert!(results.iter().any(|r| r.title.contains("Annotate Area")));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn annotation_command_includes_swappy() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let mut plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        plugin.annotator = Some(AnnotatorTool::Swappy {
            command: "swappy".to_string(),
        });

        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@ss annotate", &ctx).unwrap();
        assert!(!results.is_empty());

        // Find an annotation result
        let annotate_result = results
            .iter()
            .find(|r| r.title.contains("Annotate"))
            .expect("expected annotation result");

        // Check command contains swappy and pipe
        assert!(annotate_result.command.contains("swappy"));
        assert!(annotate_result.command.contains("-f -"));
        assert!(annotate_result.command.contains("-o"));

        // Check subtitle mentions annotation tool
        let subtitle = annotate_result
            .subtitle
            .as_ref()
            .expect("expected subtitle for annotation result");
        assert!(subtitle.contains("swappy"));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn annotation_with_clipboard_combines_both() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let mut plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        plugin.annotator = Some(AnnotatorTool::Swappy {
            command: "swappy".to_string(),
        });
        plugin.clipboard = Some(ClipboardTool::WlCopy {
            command: "wl-copy".to_string(),
        });

        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@ss annotate full", &ctx).unwrap();
        assert!(!results.is_empty());

        let result = &results[0];

        // Check command contains both swappy and clipboard
        assert!(result.command.contains("swappy"));
        assert!(result.command.contains("wl-copy"));
        assert!(result.command.contains("&&"));

        // Check subtitle mentions both tools
        let subtitle = result.subtitle.as_ref().expect("expected subtitle");
        assert!(subtitle.contains("swappy"));
        assert!(subtitle.contains("clipboard"));

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn filters_annotation_modes_by_keyword() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let mut plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        plugin.annotator = Some(AnnotatorTool::Swappy {
            command: "swappy".to_string(),
        });

        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        // Search for "edit" which is in annotation keywords
        let results = plugin.search("@ss edit", &ctx).unwrap();
        assert!(!results.is_empty());

        // All results should be annotation modes
        for result in &results {
            assert!(result.title.contains("Annotate"));
        }

        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn no_annotation_modes_without_annotator() {
        let output = temp_output_dir();
        let backend = ScreenshotBackend::grimshot("grimshot".to_string());
        let plugin = ScreenshotPlugin::with_backend(Some(backend), output.clone());
        // No annotator set

        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = plugin.search("@screenshot", &ctx).unwrap();

        // Should only have 3 regular modes
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| !r.title.contains("Annotate")));

        let _ = fs::remove_dir_all(output);
    }
}
