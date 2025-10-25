use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

/// Execute a desktop entry's command
pub fn execute_command(exec: &str, terminal: bool) -> Result<()> {
    debug!("Executing command: {} (terminal: {})", exec, terminal);

    // Clean up the exec string (remove field codes)
    let cleaned_exec = clean_exec_string(exec);

    debug!("Cleaned command: {}", cleaned_exec);

    if cleaned_exec.is_empty() {
        warn!("Empty command after cleaning, skipping execution");
        return Ok(());
    }

    if terminal {
        return execute_in_terminal(&cleaned_exec);
    }

    execute_direct(&cleaned_exec)
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
fn execute_direct(exec: &str) -> Result<()> {
    info!("Launching: {}", exec);

    // Use setsid to detach the process from the terminal
    // This prevents the child process from being killed when the launcher exits
    let full_command = format!("setsid -f {}", exec);

    Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to execute command")?;

    info!("Successfully launched: {}", exec);
    Ok(())
}

/// Execute command in terminal
fn execute_in_terminal(exec: &str) -> Result<()> {
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

    Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
