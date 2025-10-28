# Installer Enhancement Changelog

## Latest Updates (Compositor Auto-Start - MAJOR IMPROVEMENT)

### 🚀 Compositor Auto-Start Integration (Replaces Systemd)

**Breaking Change**: Daemon mode now uses compositor auto-start configs instead of systemd services.

This is a **much better approach** that follows how other Wayland launchers (wofi, rofi) work:

#### Why This Change?

| **Old Approach (systemd)**           | **New Approach (compositor config)**  |
| ------------------------------------ | ------------------------------------- |
| ❌ May start before compositor ready | ✅ Starts when compositor ready       |
| ❌ May lack Wayland environment vars | ✅ Compositor sets all vars correctly |
| ❌ Requires systemd                  | ✅ Works on all init systems          |
| ❌ System-level integration          | ✅ Compositor-native integration      |
| ❌ Custom/non-standard               | ✅ Standard pattern (like wofi)       |

#### Features

**Supported Compositors**:

- ✅ Hyprland (`exec-once` in `hyprland.conf`)
- ✅ Sway (`exec` in `config`)
- ✅ i3 (`exec --no-startup-id` in `config`)
- ✅ River (`riverctl spawn` in `init`)
- ✅ Wayfire (`autostart_` in `wayfire.ini`)

**Safety Features**:

- ✅ **Timestamped backups**: `config.backup-YYYYMMDD_HHMMSS`
- ✅ **Duplicate prevention**: Removes old daemon entries first
- ✅ **Syntax validation**: Auto-validates config after modification (Sway, i3)
- ✅ **Auto-restore**: Restores backup if validation fails
- ✅ **Non-destructive**: Original config always preserved
- ✅ **Idempotent**: Safe to run multiple times

**User Experience**:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Compositor Auto-Start Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Detected compositor: Hyprland
Config file: ~/.config/hypr/hyprland.conf

This will add native-launcher daemon to auto-start:
  exec-once = ~/.local/bin/native-launcher --daemon

Benefits:
  • Launcher pre-loads on compositor startup
  • Instant appearance when pressing Super+Space
  • No manual daemon management needed

Trade-offs:
  • Uses ~20-30MB RAM constantly

⚠️  WARNING: This will modify your compositor config

Backup will be created at:
  ~/.config/hypr/hyprland.conf.backup-20251028_143022

If validation fails, backup will be auto-restored.

Add daemon to auto-start? (y/N)
```

#### Functions Added (~200 lines)

1. **`get_compositor_config_path()`** - Detects config file location
2. **`get_autostart_command()`** - Gets compositor-specific syntax
3. **`remove_old_daemon_entries()`** - Prevents duplicates
4. **`validate_compositor_config()`** - Validates syntax
5. **`setup_compositor_autostart()`** - Main orchestration

#### Migration from Systemd

**If you have existing systemd service**:

1. Disable systemd service:

   ```bash
   systemctl --user disable --now native-launcher-daemon
   rm ~/.config/systemd/user/native-launcher-daemon.service
   systemctl --user daemon-reload
   ```

2. Run installer to add compositor auto-start:

   ```bash
   ./install.sh --upgrade
   # Choose "Yes" when prompted
   ```

3. Restart compositor:
   ```bash
   hyprctl reload  # Or your compositor's reload command
   ```

---

## Previous Updates

### ✨ Colorful Theme Display

**Visual preview of themes in their actual colors**
**Smart handling of background daemon during installation**

- **Auto-detection**: Automatically detects if daemon is running before install
- **Graceful shutdown**: Stops daemon before installation (with confirmation in interactive mode)
- **Auto-restart**: Restarts daemon after installation if it was running before
- **Systemd integration**: Optional systemd user service setup with auto-start on login
- **Interactive prompt**: Asks user if they want to enable daemon mode (explains trade-offs)

**Functions added** (~150 lines):

- `stop_daemon()` - Detects and stops running daemon
- `should_restart_daemon()` - Checks if daemon should be restarted
- `restart_daemon()` - Restarts daemon after installation
- `setup_daemon_service()` - Creates systemd user service (optional)

**Installation flow integration**:

```
1. Backup existing installation
2. Clean if fresh mode
3. → Stop daemon (NEW)
4. Install dependencies
5. Download and install binary
6. Select theme
7. Create config
8. Setup compositor keybinds
9. → Setup daemon service (NEW, optional)
10. → Restart daemon (NEW, if was running)
11. Verify and complete
```

**Daemon benefits**:

- ✅ Instant window appearance (no startup delay)
- ✅ Faster search results
- ✅ Lower latency on hotkey press

**Trade-offs**:

- Uses ~20-30MB RAM constantly
- One more background process

**Management commands**:

```bash
# Status
systemctl --user status native-launcher-daemon

# Stop
systemctl --user stop native-launcher-daemon

# Disable auto-start
systemctl --user disable native-launcher-daemon

# Restart
systemctl --user restart native-launcher-daemon
```

#### 2. Colorful Theme Display

**Visual preview of themes in their actual colors**

Before:

```
  1) Default     - Coral/red accent
  2) Nord        - Frost blue accent
  3) Dracula     - Purple accent
```

After:

```
  1) Default     - ● Coral/red accent (#FF6363)
  2) Nord        - ● Frost blue accent (#88C0D0)
  3) Dracula     - ● Purple accent (#BD93F9)
```

**Implementation**:

- Uses RGB ANSI escape codes: `\033[38;2;R;G;B m●\033[0m`
- Shows colored bullet (●) in each theme's accent color
- Makes theme selection more visual and intuitive

**Colors**:

- Default: `\033[38;2;255;99;99m●` (coral)
- Nord: `\033[38;2;136;192;208m●` (frost blue)
- Dracula: `\033[38;2;189;147;249m●` (purple)
- Catppuccin: `\033[38;2;180;190;254m●` (lavender)
- Gruvbox: `\033[38;2;254;128;25m●` (orange)
- Tokyo Night: `\033[38;2;122;162;246m●` (electric blue)

### 📝 Documentation Updates

#### Updated `docs/INSTALL_OPTIONS.md` (+95 lines)

**New section: Daemon Mode**

- Benefits and trade-offs explanation
- Setup instructions (during install + manual)
- Management commands (systemctl)
- Installation behavior (upgrade vs fresh)

**Updated section: Installation Flow**

- Added daemon handling steps to both upgrade and fresh modes
- Shows when daemon is stopped/restarted

**Updated section: Troubleshooting**

- Daemon doesn't start automatically
- Daemon uses too much memory
- Want to enable daemon after install
- Daemon not detected during install

### 🔧 Technical Details

**File**: `install.sh` (1071 lines, +135 lines added)

**New global tracking**:

- `/tmp/native-launcher-daemon-was-running` - Marker file to track daemon state

**Modified functions**:

- `select_theme()` - Added RGB color display
- `print_completion()` - Shows daemon status in summary
- `main()` - Integrated daemon functions in flow

**Completion message now shows**:

```
Installation Summary:
  Version: X.Y.Z
  Theme: nord
  Binary: ~/.local/bin/native-launcher
  Config: ~/.config/native-launcher/config.toml
  Daemon: Enabled (systemd service)  ← NEW

Daemon Management:  ← NEW SECTION
  systemctl --user status native-launcher-daemon
  systemctl --user stop native-launcher-daemon
  systemctl --user disable native-launcher-daemon
```

### 🎯 Use Cases

**Standard upgrade with daemon**:

```bash
./install.sh --upgrade
# Daemon automatically stopped → updated → restarted
```

**Fresh install with daemon setup**:

```bash
./install.sh --fresh
# Prompted to enable daemon mode with explanation
```

**Non-interactive CI/CD** (daemon skipped):

```bash
./install.sh --non-interactive --upgrade
# Daemon stopped if running, but no new setup
```

**Manual daemon enable after install**:

```bash
# Run upgrade mode, it will ask about daemon
./install.sh --upgrade
```

### 🧪 Testing Checklist

- [ ] Theme colors display correctly in terminal
- [ ] Daemon detected when running
- [ ] Daemon stopped gracefully before install
- [ ] Daemon restarted after upgrade
- [ ] Systemd service created correctly
- [ ] Service auto-starts on login
- [ ] Fresh install removes old daemon service
- [ ] Completion message shows correct daemon status
- [ ] Non-interactive mode skips daemon setup
- [ ] Manual daemon start works without systemd

### 📊 Statistics

**Lines added**: ~135 lines

- Daemon functions: ~110 lines
- Theme colors: ~8 lines
- Documentation updates: ~95 lines (INSTALL_OPTIONS.md)
- Completion message: ~17 lines

**Total installer size**: 1071 lines (was 936)

**Features**:

- ✅ Colorful theme display
- ✅ Daemon detection
- ✅ Graceful daemon shutdown
- ✅ Auto-restart daemon
- ✅ Systemd service creation
- ✅ Interactive daemon setup
- ✅ Daemon status in completion
- ✅ Comprehensive documentation

### 🚀 Performance Impact

**Daemon mode benefits**:

- Startup time: ~100ms → <10ms (instant)
- Search latency: Same (already fast)
- Memory usage: +20-30MB (daemon overhead)

**Installer impact**:

- No performance impact (daemon functions only run when needed)
- ~1-2 seconds added for daemon detection/restart

---

## Previous Updates

### Install Mode Selection (--fresh / --upgrade)

- Interactive prompt to choose upgrade or fresh install
- `--fresh` flag removes all configs and data
- `--upgrade` flag preserves existing configs (default)

### Backup System with Recursion Fix

- Timestamped backups: `~/.local/share/native-launcher/backups/YYYYMMDD_HHMMSS/`
- Fixed infinite loop: excludes `backups/` directory when backing up
- `--skip-backup` flag to disable backups

### CLI Argument Parsing

- `--non-interactive` - Skip all prompts
- `--fresh` - Fresh install mode
- `--upgrade` - Upgrade mode (default)
- `--skip-backup` - Skip backup creation
- `--theme THEME` - Pre-select theme
- `--help` - Show usage

### Theme Selection

- 6 themes: default, nord, dracula, catppuccin, gruvbox, tokyo-night
- Interactive selection with descriptions
- CLI flag for automation

### Comprehensive Documentation

- `docs/INSTALL_OPTIONS.md` - Complete usage guide
- Examples for all scenarios
- Troubleshooting section
