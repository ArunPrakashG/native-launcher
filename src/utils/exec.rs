use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock, RwLock};
use tracing::{debug, error, info, warn};
use urlencoding::{decode, encode};

/// Cached login-shell environment merged with the current process environment
static LAUNCH_ENV: OnceLock<HashMap<String, String>> = OnceLock::new();

pub const OPEN_COMMAND_PREFIX: &str = "open://";
const LEGACY_OPEN_COMMAND_PREFIX: &str = "open-file://";

type HandlerCallback = dyn Fn(&str, bool) -> Result<bool> + Send + Sync + 'static;

#[derive(Clone)]
struct HandlerEntry {
    name: String,
    callback: Arc<HandlerCallback>,
}

#[derive(Default)]
struct HandlerRegistry {
    plugin_handlers: Vec<HandlerEntry>,
    config_handlers: Vec<HandlerEntry>,
}

static OPEN_HANDLER_REGISTRY: OnceLock<RwLock<HandlerRegistry>> = OnceLock::new();

fn handler_registry() -> &'static RwLock<HandlerRegistry> {
    OPEN_HANDLER_REGISTRY.get_or_init(|| RwLock::new(HandlerRegistry::default()))
}

#[derive(Debug, Clone)]
pub struct CommandOpenHandler {
    pub command: String,
    pub args: Vec<String>,
    pub pass_target: bool,
}

impl CommandOpenHandler {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            pass_target: true,
        }
    }

    pub fn execute(&self, target: &str, merge_login_env: bool) -> Result<bool> {
        spawn_command_handler(self, target, merge_login_env)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenHandlerPriority {
    #[cfg_attr(not(test), allow(dead_code))]
    First,
    Last,
}

pub fn register_open_handler<F>(name: impl Into<String>, priority: OpenHandlerPriority, handler: F)
where
    F: Fn(&str, bool) -> Result<bool> + Send + Sync + 'static,
{
    let entry = HandlerEntry {
        name: name.into(),
        callback: Arc::new(handler) as Arc<HandlerCallback>,
    };

    let mut registry = handler_registry()
        .write()
        .expect("open handler registry poisoned");

    match priority {
        OpenHandlerPriority::First => registry.plugin_handlers.insert(0, entry),
        OpenHandlerPriority::Last => registry.plugin_handlers.push(entry),
    }
}

pub fn configure_open_handlers(handlers: Vec<CommandOpenHandler>) {
    let mut registry = handler_registry()
        .write()
        .expect("open handler registry poisoned");

    registry.config_handlers = handlers
        .into_iter()
        .enumerate()
        .map(|(idx, handler)| HandlerEntry {
            name: format!("config-handler-{}", idx),
            callback: wrap_command_handler(handler),
        })
        .collect();
}

fn wrap_command_handler(handler: CommandOpenHandler) -> Arc<HandlerCallback> {
    let handler = Arc::new(handler);
    Arc::new(move |target: &str, merge_login_env: bool| {
        spawn_command_handler(handler.as_ref(), target, merge_login_env)
    }) as Arc<HandlerCallback>
}

#[cfg(test)]
pub(crate) fn reset_open_handlers_for_test() {
    let mut registry = handler_registry()
        .write()
        .expect("open handler registry poisoned");
    registry.plugin_handlers.clear();
    registry.config_handlers.clear();
}

#[cfg(test)]
pub(crate) fn handler_counts_for_test() -> (usize, usize) {
    let registry = handler_registry()
        .read()
        .expect("open handler registry poisoned");
    (
        registry.plugin_handlers.len(),
        registry.config_handlers.len(),
    )
}

/// Execute a desktop entry's command
pub fn execute_command(exec: &str, terminal: bool, merge_login_env: bool) -> Result<()> {
    debug!("Executing command: {} (terminal: {})", exec, terminal);

    // Clean up the exec string (remove field codes)
    let cleaned_exec = clean_exec_string(exec);

    debug!("Cleaned command: {}", cleaned_exec);

    if cleaned_exec.is_empty() {
        warn!("Empty command after cleaning, skipping execution");
        return Ok(());
    }

    if let Some(encoded_target) = cleaned_exec.strip_prefix(OPEN_COMMAND_PREFIX) {
        return open_uri(encoded_target, merge_login_env);
    }

    if let Some(encoded_target) = cleaned_exec.strip_prefix(LEGACY_OPEN_COMMAND_PREFIX) {
        return open_uri(encoded_target, merge_login_env);
    }

    if terminal {
        return execute_in_terminal(&cleaned_exec, merge_login_env);
    }

    execute_direct(&cleaned_exec, merge_login_env)
}

pub fn build_open_command(target: impl AsRef<str>) -> String {
    let encoded = encode(target.as_ref());
    format!("{}{}", OPEN_COMMAND_PREFIX, encoded)
}

/// Remove desktop entry field codes from exec string
fn clean_exec_string(exec: &str) -> String {
    let mut result = exec.to_string();

    // Remove common field codes according to Desktop Entry Specification
    let field_codes = [
        "%f", "%F", // single/multiple files
        "%u", "%U", // single/multiple URLs
        "%d", "%D", // deprecated
        "%n", "%N", // deprecated
        "%i", // icon
        "%c", // translated name
        "%k", // location of desktop file
        "%v", // deprecated
        "%m", // deprecated
    ];

    for code in &field_codes {
        result = result.replace(code, "");
    }

    // Remove quotes if the entire string is quoted
    if result.starts_with('"') && result.ends_with('"') && result.len() > 1 {
        result = result[1..result.len() - 1].to_string();
    }

    if result.starts_with('\'') && result.ends_with('\'') && result.len() > 1 {
        result = result[1..result.len() - 1].to_string();
    }

    // Clean up extra whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Execute command directly with proper detachment
fn execute_direct(exec: &str, merge_login_env: bool) -> Result<()> {
    info!("Launching: {}", exec);

    // Use setsid to detach the process from the terminal
    // This prevents the child process from being killed when the launcher exits
    let full_command = format!("setsid -f {}", exec);

    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(&full_command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    apply_launch_environment(&mut command, merge_login_env);

    command.spawn().context("Failed to execute command")?;

    info!("Successfully launched: {}", exec);
    Ok(())
}

/// Execute command in terminal
fn execute_in_terminal(exec: &str, merge_login_env: bool) -> Result<()> {
    let terminal = detect_terminal()?;
    info!("Launching in terminal {}: {}", terminal, exec);

    // Different terminals have different command-line syntax
    let terminal_cmd = match terminal.as_str() {
        "alacritty" => format!("{} -e sh -c '{}'", terminal, exec),
        "kitty" => format!("{} sh -c '{}'", terminal, exec),
        "wezterm" => format!("{} start sh -c '{}'", terminal, exec),
        "foot" => format!("{} sh -c '{}'", terminal, exec),
        "gnome-terminal" => format!("{} -- sh -c '{}'", terminal, exec),
        "konsole" => format!("{} -e sh -c '{}'", terminal, exec),
        "xterm" => format!("{} -e sh -c '{}'", terminal, exec),
        _ => format!("{} -e sh -c '{}'", terminal, exec),
    };

    let full_command = format!("setsid -f {}", terminal_cmd);

    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(&full_command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    apply_launch_environment(&mut command, merge_login_env);

    command
        .spawn()
        .context("Failed to execute command in terminal")?;

    info!("Successfully launched in terminal: {}", exec);
    Ok(())
}

/// Detect available terminal emulator
fn detect_terminal() -> Result<String> {
    let terminals = [
        "alacritty",
        "kitty",
        "wezterm",
        "foot",
        "gnome-terminal",
        "konsole",
        "xterm",
    ];

    for term in &terminals {
        if Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            debug!("Detected terminal: {}", term);
            return Ok(term.to_string());
        }
    }

    error!("No terminal emulator found");
    anyhow::bail!("No terminal emulator found")
}

/// Apply the cached launch environment (user login shell + current process vars)
fn apply_launch_environment(command: &mut Command, merge_login_env: bool) {
    if !merge_login_env {
        return;
    }

    let env_map = LAUNCH_ENV.get_or_init(load_shell_environment);

    command.env_clear();
    for (key, value) in env_map {
        command.env(key, value);
    }
}

/// Load environment variables from the user's login shell and merge with current env
fn load_shell_environment() -> HashMap<String, String> {
    let mut merged: HashMap<String, String> = std::env::vars().collect();

    // Detect user shell (fallback to /bin/sh)
    let shell = merged
        .get("SHELL")
        .cloned()
        .unwrap_or_else(|| "/bin/sh".to_string());

    debug!("Loading environment from shell: {}", shell);

    let shell_output = Command::new(&shell).arg("-l").arg("-c").arg("env").output();

    match shell_output {
        Ok(output) if output.status.success() => {
            if let Ok(env_str) = String::from_utf8(output.stdout) {
                for line in env_str.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        merged.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        Ok(output) => {
            warn!(
                "Failed to load shell environment (status: {}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(err) => {
            warn!(
                "Unable to execute shell '{}' for environment loading: {}",
                shell, err
            );
        }
    }

    merged
}

fn open_uri(encoded_target: &str, merge_login_env: bool) -> Result<()> {
    let decoded = decode(encoded_target).context("Failed to decode open URI")?;
    let target = decoded.into_owned();

    info!("Opening via system handler: {}", target);

    let (plugin_handlers, config_handlers) = {
        let registry = handler_registry()
            .read()
            .expect("open handler registry poisoned");
        (
            registry.plugin_handlers.clone(),
            registry.config_handlers.clone(),
        )
    };

    for entry in plugin_handlers {
        match (entry.callback)(&target, merge_login_env) {
            Ok(true) => {
                debug!("Open handler '{}' handled target {}", entry.name, target);
                return Ok(());
            }
            Ok(false) => {
                debug!("Open handler '{}' skipped target {}", entry.name, target);
            }
            Err(err) => {
                warn!(
                    "Open handler '{}' failed on {}: {}",
                    entry.name, target, err
                );
            }
        }
    }

    for entry in config_handlers {
        match (entry.callback)(&target, merge_login_env) {
            Ok(true) => {
                debug!(
                    "Config open handler '{}' handled target {}",
                    entry.name, target
                );
                return Ok(());
            }
            Ok(false) => {
                debug!(
                    "Config open handler '{}' skipped target {}",
                    entry.name, target
                );
            }
            Err(err) => {
                warn!(
                    "Config open handler '{}' failed on {}: {}",
                    entry.name, target, err
                );
            }
        }
    }

    if let Err(err) = spawn_file_opener("gio", Some("open"), &target, merge_login_env) {
        debug!(
            "gio open unavailable or failed ({}), falling back to xdg-open",
            err
        );
        spawn_file_opener("xdg-open", None, &target, merge_login_env)?;
    }

    Ok(())
}

fn spawn_command_handler(
    handler: &CommandOpenHandler,
    target: &str,
    merge_login_env: bool,
) -> Result<bool> {
    if handler.command.is_empty() {
        warn!("Configured open handler skipped due to empty command");
        return Ok(false);
    }

    let mut command = Command::new(&handler.command);
    let mut injected = false;

    for arg in &handler.args {
        if handler.pass_target && arg.contains("{target}") {
            let replaced = arg.replace("{target}", target);
            command.arg(replaced);
            injected = true;
        } else {
            command.arg(arg);
        }
    }

    if handler.pass_target && !injected {
        command.arg(target);
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    apply_launch_environment(&mut command, merge_login_env);

    match command.spawn() {
        Ok(_) => {
            info!(
                "Successfully launched configured open handler '{}' for {}",
                handler.command, target
            );
            Ok(true)
        }
        Err(err) => {
            warn!(
                "Configured open handler '{}' failed to spawn for {}: {}",
                handler.command, target, err
            );
            Ok(false)
        }
    }
}

fn spawn_file_opener(
    command: &str,
    subcommand: Option<&str>,
    target: &str,
    merge_login_env: bool,
) -> Result<()> {
    let mut cmd = Command::new(command);

    if let Some(sub) = subcommand {
        cmd.arg(sub);
    }

    cmd.arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    apply_launch_environment(&mut cmd, merge_login_env);

    cmd.spawn()
        .with_context(|| format!("Failed to launch {} for target {}", command, target))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex, OnceLock};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("native-launcher-{}-{}", prefix, ts))
    }

    fn registry_mutex() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn plugin_handler_short_circuits_open_uri() -> Result<()> {
        let _guard = registry_mutex().lock().unwrap();
        reset_open_handlers_for_test();

        let handled = Arc::new(AtomicBool::new(false));
        let handled_flag = Arc::clone(&handled);

        register_open_handler(
            "test-plugin",
            OpenHandlerPriority::First,
            move |target, merge| {
                assert_eq!(target, "https://example.com");
                assert!(!merge);
                handled_flag.store(true, Ordering::SeqCst);
                Ok(true)
            },
        );

        configure_open_handlers(vec![CommandOpenHandler::new("true")]);
        assert_eq!(handler_counts_for_test(), (1, 1));

        let encoded = encode("https://example.com");
        open_uri(encoded.as_ref(), false)?;

        assert!(handled.load(Ordering::SeqCst));
        reset_open_handlers_for_test();
        Ok(())
    }

    #[test]
    fn config_handler_runs_when_plugins_skip() -> Result<()> {
        let _guard = registry_mutex().lock().unwrap();
        reset_open_handlers_for_test();

        register_open_handler("skipper", OpenHandlerPriority::First, |_target, _| {
            Ok(false)
        });

        let path = unique_temp_path("config-handler");
        let mut handler = CommandOpenHandler::new("sh");
        handler.args = vec![
            "-c".into(),
            format!("printf 'handled' > {}", path.display()),
        ];
        handler.pass_target = false;

        configure_open_handlers(vec![handler]);
        assert_eq!(handler_counts_for_test(), (1, 1));

        let encoded = encode("dummy-target");
        open_uri(encoded.as_ref(), false)?;

        for _ in 0..25 {
            if path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let contents = fs::read_to_string(&path).context("expected config handler output")?;
        assert!(contents.contains("handled"));

        let _ = fs::remove_file(&path);
        reset_open_handlers_for_test();
        Ok(())
    }

    #[test]
    fn command_handler_injects_target_placeholder() -> Result<()> {
        let _guard = registry_mutex().lock().unwrap();
        let path = unique_temp_path("placeholder");
        let mut handler = CommandOpenHandler::new("sh");
        handler.args = vec![
            "-c".into(),
            format!("printf '%s' \"{{target}}\" > {}", path.display()),
        ];
        handler.pass_target = true;

        let handled = spawn_command_handler(&handler, "value with spaces", false)?;
        assert!(handled);

        for _ in 0..25 {
            if path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        assert!(
            path.exists(),
            "expected command handler to create output file"
        );

        let contents = fs::read_to_string(&path)?;
        assert_eq!(contents.trim(), "value with spaces");

        let _ = fs::remove_file(&path);
        Ok(())
    }
}
