# Installation Options Guide

## Interactive Installation (Recommended)

Simply run the installer without any flags:

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

You'll be asked to choose:

1. **Installation Mode**: Upgrade (keep configs) or Fresh (remove all)
2. **Backup**: Whether to create a backup (recommended: Yes)
3. **Theme**: Choose from 6 available themes

## Command Line Flags

### Installation Modes

**Upgrade Mode** (default - keeps existing configs):

```bash
./install.sh --upgrade
```

**Fresh Install** (removes all existing configs and data):

```bash
./install.sh --fresh
```

### Non-Interactive Installation

Skip all prompts and use defaults:

```bash
./install.sh --non-interactive
```

### Skip Backup

Skip backup creation (not recommended):

```bash
./install.sh --skip-backup
```

### Select Theme

Pre-select a theme without prompt:

```bash
./install.sh --theme nord
```

Available themes:

- `default` - Coral/red accent (#FF6363)
- `nord` - Frost blue accent (#88C0D0)
- `dracula` - Purple accent (#BD93F9)
- `catppuccin` - Lavender accent (#B4BEFE)
- `gruvbox` - Orange accent (#FE8019)
- `tokyo-night` - Electric blue accent (#7AA2F7)

### Help

Show all available options:

```bash
./install.sh --help
```

## Daemon Mode

Native Launcher can run as a background daemon for instant appearance.

### Benefits

- **Instant response**: No startup delay when pressing hotkey
- **Faster search**: Results appear immediately
- **Lower latency**: App is already loaded in memory

### Trade-offs

- **Memory usage**: ~20-30MB RAM constantly

### How It Works

The installer adds native-launcher daemon to your **compositor's auto-start configuration**, similar to how other launchers like wofi and rofi work. This ensures:

- ✅ Daemon starts when compositor starts (not system boot)
- ✅ Proper Wayland environment variables set
- ✅ Works on all systems (no systemd required)
- ✅ Compositor-aware initialization

### Supported Compositors

**Hyprland** (`~/.config/hypr/hyprland.conf`):

```
exec-once = ~/.local/bin/native-launcher --daemon
```

**Sway** (`~/.config/sway/config`):

```
exec ~/.local/bin/native-launcher --daemon
```

**i3** (`~/.config/i3/config`):

```
exec --no-startup-id ~/.local/bin/native-launcher --daemon
```

**River** (`~/.config/river/init`):

```
riverctl spawn "~/.local/bin/native-launcher --daemon"
```

**Wayfire** (`~/.config/wayfire.ini`):

```
autostart_native_launcher = ~/.local/bin/native-launcher --daemon
```

### Setup During Installation

The installer will:

1. **Detect** your compositor and config file location
2. **Prompt** you to enable daemon auto-start
3. **Explain** benefits, trade-offs, and what will be modified
4. **Backup** your config file with timestamp
5. **Remove** any old daemon entries (avoid duplicates)
6. **Add** the appropriate auto-start command
7. **Validate** config syntax (where supported)
8. **Auto-restore** backup if validation fails

### Manual Setup (After Installation)

If you skipped auto-start during installation, you can manually add it:

**1. Edit your compositor config**:

```bash
# Hyprland
nano ~/.config/hypr/hyprland.conf

# Sway
nano ~/.config/sway/config

# i3
nano ~/.config/i3/config

# River
nano ~/.config/river/init
```

**2. Add the appropriate command** (see "Supported Compositors" section above)

**3. Restart compositor**:

```bash
# Hyprland
hyprctl reload

# Sway
swaymsg reload

# i3
i3-msg reload

# River - Restart River
```

### Daemon Management

**Check if daemon is running**:

```bash
pgrep -f "native-launcher.*--daemon"
# If output shows PID, daemon is running
```

**Stop daemon**:

```bash
pkill -f "native-launcher.*--daemon"
```

**Start daemon manually** (without compositor restart):

```bash
nohup native-launcher --daemon >/dev/null 2>&1 &
```

**Disable auto-start**:

1. Edit your compositor config
2. Remove or comment out the daemon line
3. Restart compositor
4. Kill running daemon: `pkill -f "native-launcher.*--daemon"`

### Installation Behavior

- **Upgrade mode**: Preserves existing daemon setup in compositor config
- **Fresh mode**: Removes daemon from config, asks to set up again
- **Running daemon**: Automatically stops before install, restarts after
- **Non-interactive**: Skips daemon setup (manual setup required)
- **Config validation**: Auto-validates after modification (Sway, i3)
- **Auto-restore**: Restores backup if validation fails

### Safety Features

✅ **Timestamped backups**: Config backed up before modification  
✅ **Duplicate prevention**: Removes old daemon entries first  
✅ **Syntax validation**: Validates config after modification (where supported)  
✅ **Auto-restore**: Restores backup if validation fails  
✅ **Non-destructive**: Original config preserved in backup  
✅ **Idempotent**: Safe to run multiple times (won't create duplicates)

### Why Not Systemd?

You might wonder why we don't use systemd user services. Here's why compositor auto-start is better:

| Aspect          | Systemd Service       | Compositor Auto-Start        |
| --------------- | --------------------- | ---------------------------- |
| **Timing**      | Starts at user login  | Starts when compositor ready |
| **Environment** | May lack Wayland vars | Compositor sets all vars     |
| **Portability** | Requires systemd      | Works on all init systems    |
| **Integration** | System-level          | Compositor-native            |
| **Reliability** | May start too early   | Perfect timing               |
| **Standard**    | Custom approach       | How wofi/rofi work           |

## Common Usage Examples

### 1. Interactive Upgrade (Recommended for Updates)

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

- Keeps your existing configuration
- Creates a backup automatically
- Only updates the binary

### 2. Non-Interactive Upgrade

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash -s -- --non-interactive --upgrade
```

- No prompts
- Keeps existing configs
- Creates backup
- Uses existing theme

### 3. Fresh Install with Specific Theme

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash -s -- --fresh --theme nord
```

- Removes all existing configs
- Creates backup first
- Sets Nord theme
- Interactive confirmation for fresh install

### 4. Complete Non-Interactive Fresh Install

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash -s -- --non-interactive --fresh --theme gruvbox
```

- No prompts at all
- Removes everything (backup created)
- Sets Gruvbox theme
- **Warning**: Destructive! Use with caution

### 5. Quick Upgrade Without Backup

```bash
./install.sh --upgrade --skip-backup --non-interactive
```

- Fast upgrade
- No backup (risky!)
- Keeps configs
- No prompts

## Help

View all options:

```bash
./install.sh --help
```

## Installation Flow

### Upgrade Mode (Default)

1. Detect existing installation
2. Create backup (unless --skip-backup)
3. Stop daemon if running
4. Update binary to latest version
5. Keep existing config.toml
6. Keep existing plugins/cache/data
7. Skip theme selection
8. Restart daemon if it was running

### Fresh Install Mode

1. Detect existing installation
2. Ask for confirmation (interactive)
3. Create backup
4. Stop daemon if running
5. Remove all configs and data (preserves backups)
6. Download and install binary
7. Ask for theme selection
8. Generate new config.toml
9. Setup compositor keybinds
10. Setup daemon service (optional, interactive)
11. Restart daemon if it was running

## What Gets Backed Up?

- Binary: `~/.local/bin/native-launcher`
- Config: `~/.config/native-launcher/config.toml`
- Plugins: `~/.config/native-launcher/plugins/`
- Cache: `~/.cache/native-launcher/`
- Data: `~/.local/share/native-launcher/` (except backups directory)

Backups are stored in:

```
~/.local/share/native-launcher/backups/YYYYMMDD_HHMMSS/
```

## Restoring from Backup

If something goes wrong, use the restore script:

```bash
./restore.sh
```

This will:

1. List all available backups
2. Let you select which backup to restore
3. Restore all components

## Troubleshooting

**Issue**: "Existing installation detected" but you want fresh install

```bash
./install.sh --fresh
```

**Issue**: Want to try different theme without reinstalling

```bash
./install.sh --fresh --theme catppuccin
```

**Issue**: Automated CI/CD deployment

```bash
./install.sh --non-interactive --upgrade --skip-backup
```

**Issue**: Installation hangs on questions

```bash
# Press Ctrl+C and run:
./install.sh --non-interactive
```

**Issue**: Daemon doesn't start automatically

```bash
# Check if daemon is in compositor config
grep "native-launcher.*--daemon" ~/.config/hypr/hyprland.conf  # Hyprland
grep "native-launcher.*--daemon" ~/.config/sway/config  # Sway

# If not found, add manually (see "Manual Setup" section above)

# Check if daemon is running
pgrep -f "native-launcher.*--daemon"

# Start daemon manually
nohup native-launcher --daemon >/dev/null 2>&1 &
```

**Issue**: Daemon uses too much memory

```bash
# Disable daemon auto-start
# Edit your compositor config and remove/comment the daemon line:
nano ~/.config/hypr/hyprland.conf  # Or your compositor's config

# Kill running daemon
pkill -f "native-launcher.*--daemon"

# Restart compositor to apply changes
hyprctl reload  # Or your compositor's reload command
```

**Issue**: Want to enable daemon after initial install

```bash
# Option 1: Run installer again in upgrade mode
./install.sh --upgrade
# When asked about daemon, choose Yes

# Option 2: Manually add to compositor config (see "Manual Setup" section)
```

**Issue**: Daemon not detected during install

```bash
# Check if daemon is actually running
pgrep -f "native-launcher.*--daemon"

# If not running, start it manually
nohup native-launcher --daemon >/dev/null 2>&1 &
```

**Issue**: Compositor config validation failed

```bash
# Installer auto-restores backup, but you can manually restore:
cp ~/.config/hypr/hyprland.conf.backup-YYYYMMDD_HHMMSS ~/.config/hypr/hyprland.conf

# Check your compositor's config syntax manually
# Then add daemon line manually (see "Manual Setup" section)
```

**Issue**: Multiple daemon entries in config

```bash
# The installer removes duplicates automatically, but if needed:
# Edit config and keep only one daemon line
nano ~/.config/hypr/hyprland.conf

# Restart compositor
hyprctl reload
```
