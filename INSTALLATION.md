# Installation Guide

This guide covers all installation methods for Native Launcher.

## Table of Contents

- [Quick Install](#quick-install)
- [Backup & Restore](#backup--restore)
- [Manual Installation](#manual-installation)
- [Building from Source](#building-from-source)
- [Compositor Configuration](#compositor-configuration)
- [Theme Selection](#theme-selection)
- [Troubleshooting](#troubleshooting)

---

## Quick Install

The easiest way to install Native Launcher is using the automated installer:

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

### What the Installer Does

The installation process follows these steps:

1. **Backup** - Automatically backs up existing installation (if found)
2. **System Detection** - Detects your Linux distribution and Wayland compositor
3. **Dependency Installation** - Installs required packages (GTK4, gtk4-layer-shell, wl-clipboard)
4. **Download Binary** - Fetches the latest release from GitHub
5. **Theme Selection** - Interactive theme chooser (6 themes available)
6. **Configuration** - Generates config files with your chosen theme
7. **Compositor Setup** - Automatically configures keybinds (Hyprland/Sway)

### Interactive vs Non-Interactive Mode

```bash
# Interactive (default) - asks for confirmation and theme choice
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash

# Non-interactive - uses defaults, no prompts
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash -s -- --non-interactive
```

---

## Backup & Restore

### Automatic Backups

The installer **automatically backs up** your existing installation before making changes. This includes:

- ✅ Binary (`~/.local/bin/native-launcher`)
- ✅ Configuration (`~/.config/native-launcher/config.toml`)
- ✅ Plugins (`~/.config/native-launcher/plugins/`)
- ✅ Cache (`~/.cache/native-launcher/*`)
- ✅ Usage data (`~/.local/share/native-launcher/*`)

Backups are stored in:

```
~/.local/share/native-launcher/backups/YYYYMMDD_HHMMSS/
```

Example:

```
~/.local/share/native-launcher/backups/20240315_143022/
├── native-launcher        (binary)
├── config/
│   ├── config.toml
│   └── plugins/
├── cache/
└── data/
```

### Restoring from Backup

If you need to restore a previous version:

```bash
# Download and run restore script
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/restore.sh | bash

# Or if you cloned the repo
./restore.sh
```

The restore script will:

1. List all available backups with timestamps
2. Show what's included in each backup
3. Let you select which backup to restore
4. Restore all components (binary, config, plugins, cache, data)

### Manual Restore

You can also manually restore from a backup:

```bash
# Find your backup
ls ~/.local/share/native-launcher/backups/

# Restore binary
cp ~/.local/share/native-launcher/backups/20240315_143022/native-launcher ~/.local/bin/

# Restore config
cp ~/.local/share/native-launcher/backups/20240315_143022/config/config.toml ~/.config/native-launcher/

# Restore plugins
cp -r ~/.local/share/native-launcher/backups/20240315_143022/config/plugins/* ~/.config/native-launcher/plugins/
```

---

## Manual Installation

If you prefer to install manually:

### 1. Install Dependencies

**Arch Linux / Manjaro:**

```bash
sudo pacman -S gtk4 gtk4-layer-shell wl-clipboard
```

**Ubuntu / Debian:**

```bash
sudo apt update
sudo apt install libgtk-4-dev gtk4-layer-shell wl-clipboard
```

**Fedora:**

```bash
sudo dnf install gtk4-devel gtk4-layer-shell wl-clipboard
```

### 2. Download Binary

```bash
# Create installation directory
mkdir -p ~/.local/bin

# Download latest release
curl -L -o ~/.local/bin/native-launcher \
  https://github.com/ArunPrakashG/native-launcher/releases/latest/download/native-launcher

# Make executable
chmod +x ~/.local/bin/native-launcher
```

### 3. Create Configuration

```bash
# Create config directory
mkdir -p ~/.config/native-launcher

# Download default config
curl -L -o ~/.config/native-launcher/config.toml \
  https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/config/default.toml
```

### 4. Configure Compositor

See [Compositor Configuration](#compositor-configuration) below.

---

## Building from Source

### Prerequisites

- Rust 1.75 or later
- GTK4 development libraries
- gtk4-layer-shell development libraries

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Clone and Build

```bash
# Clone repository
git clone https://github.com/ArunPrakashG/native-launcher.git
cd native-launcher

# Build release binary
cargo build --release

# Install binary
cp target/release/native-launcher ~/.local/bin/

# Generate default config
mkdir -p ~/.config/native-launcher
cp config/default.toml ~/.config/native-launcher/config.toml
```

### Development Build

```bash
# Build and run in debug mode
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run benchmarks
cargo bench
```

---

## Compositor Configuration

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```bash
bind = SUPER, SPACE, exec, ~/.local/bin/native-launcher
```

Reload config:

```bash
hyprctl reload
```

### Sway

Add to `~/.config/sway/config`:

```bash
bindsym Mod4+Space exec ~/.local/bin/native-launcher
```

Reload config:

```bash
swaymsg reload
```

### River

Add to `~/.config/river/init`:

```bash
riverctl map normal Super Space spawn ~/.local/bin/native-launcher
```

### KDE Plasma (Wayland)

1. Open System Settings → Shortcuts → Custom Shortcuts
2. Edit → New → Global Shortcut → Command/URL
3. Trigger: **Meta+Space**
4. Command: `~/.local/bin/native-launcher`

### GNOME (Wayland)

Install and use a keybind extension:

```bash
# Using gsettings
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings \
  "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
  name 'Native Launcher'

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
  command '~/.local/bin/native-launcher'

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
  binding '<Super>space'
```

---

## Theme Selection

Native Launcher includes 6 beautiful themes:

### Available Themes

| #   | Theme           | Accent             | Background         |
| --- | --------------- | ------------------ | ------------------ |
| 1   | **Default**     | Coral `#FF6363`    | Charcoal `#1C1C1E` |
| 2   | **Nord**        | Frost `#88C0D0`    | Polar `#2E3440`    |
| 3   | **Dracula**     | Purple `#BD93F9`   | Dark `#282A36`     |
| 4   | **Catppuccin**  | Lavender `#B4BEFE` | Mocha `#1E1E2E`    |
| 5   | **Gruvbox**     | Orange `#FE8019`   | Dark `#282828`     |
| 6   | **Tokyo Night** | Blue `#7AA2F7`     | Night `#1A1B26`    |

### Changing Themes

#### During Installation

The installer will ask you to choose a theme interactively.

#### After Installation

Edit `~/.config/native-launcher/config.toml`:

```toml
[ui]
theme = "Nord"  # Change to: Default, Nord, Dracula, Catppuccin, Gruvbox, or TokyoNight
```

Then restart Native Launcher.

#### Reinstall with Theme

```bash
# Reinstall and choose a different theme
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

Your previous installation will be backed up automatically.

---

## Troubleshooting

### Binary not found

Ensure `~/.local/bin` is in your `$PATH`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
source ~/.bashrc
```

### Keybind not working

1. Check compositor configuration
2. Reload compositor config
3. Test binary manually: `~/.local/bin/native-launcher`
4. Check for conflicts with existing keybinds

### Window not appearing

1. Verify GTK4 and gtk4-layer-shell are installed
2. Check you're running Wayland (not X11):

```bash
echo $XDG_SESSION_TYPE  # Should output: wayland
```

3. Check logs:

```bash
RUST_LOG=debug ~/.local/bin/native-launcher
```

### Missing dependencies

Run the installer again to install dependencies:

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

### Config file errors

Reset to default config:

```bash
curl -L -o ~/.config/native-launcher/config.toml \
  https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/config/default.toml
```

### Updates not detected

The updater checks for new versions every 24 hours. To force a check:

1. Delete update cache: `rm ~/.cache/native-launcher/update_check.json`
2. Restart Native Launcher

Or manually check: [Releases Page](https://github.com/ArunPrakashG/native-launcher/releases)

### Restore from backup not working

If the restore script fails:

1. Check backup directory: `ls ~/.local/share/native-launcher/backups/`
2. Manually restore (see [Manual Restore](#manual-restore))
3. Report issue with error details

---

## Uninstallation

To completely remove Native Launcher:

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/uninstall.sh | bash
```

This will remove:

- Binary: `~/.local/bin/native-launcher`
- Config: `~/.config/native-launcher/`
- Cache: `~/.cache/native-launcher/`
- Data: `~/.local/share/native-launcher/`

**Note:** Backups are kept by default. Delete manually if needed:

```bash
rm -rf ~/.local/share/native-launcher/backups/
```

---

## Support

- 📖 [Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)
- 🐛 [Report Bug](https://github.com/ArunPrakashG/native-launcher/issues)
- 💡 [Request Feature](https://github.com/ArunPrakashG/native-launcher/issues)
- 💬 [Discussions](https://github.com/ArunPrakashG/native-launcher/discussions)
