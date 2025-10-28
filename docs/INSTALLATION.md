# Installation Scripts

This directory contains automated installation and uninstallation scripts for Native Launcher.

## Install Script (`install.sh`)

### Features

- **Automatic System Detection**: Detects Linux distribution and Wayland compositor
- **Dependency Management**: Installs GTK4 and gtk4-layer-shell automatically
- **Latest Release**: Downloads and installs the latest GitHub release
- **Configuration Setup**: Creates default configuration file
- **Keybind Setup**: Automatically configures compositor keybinds (Hyprland, Sway)

### Supported Systems

**Linux Distributions:**

- Arch Linux / Manjaro / EndeavourOS ⭐ (primary support)
- Ubuntu / Debian / Pop!\_OS
- Fedora
- openSUSE

**Wayland Compositors:**

- Hyprland ⭐ (automatic keybind setup)
- Sway (automatic keybind setup)
- KDE Plasma Wayland (manual setup instructions)
- GNOME Wayland (automatic via gsettings)
- River, Wayfire, etc. (manual setup instructions)

### Usage

**One-line install:**

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

**Download and inspect first:**

```bash
wget https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh
chmod +x install.sh
./install.sh
```

**Show help:**

```bash
./install.sh --help
```

### What Gets Installed

- **Binary**: `~/.local/bin/native-launcher`
- **Config**: `~/.config/native-launcher/config.toml`
- **Compositor keybind**: Auto-configured for supported compositors

### Requirements

The script will automatically install:

- `curl` (for downloading)
- `tar` (for extracting)
- `jq` (for JSON parsing)
- `gtk4` (GTK4 libraries)
- `gtk4-layer-shell` (Layer shell support)

## Uninstall Script (`uninstall.sh`)

### Features

- **Clean Removal**: Removes binary and optionally config/cache/data
- **Keybind Cleanup**: Removes compositor keybinds (with backup)
- **Interactive**: Prompts before removing user data

### Usage

**One-line uninstall:**

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/uninstall.sh | bash
```

**Download and run:**

```bash
wget https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/uninstall.sh
chmod +x uninstall.sh
./uninstall.sh
```

### What Gets Removed

The script prompts before removing:

- Binary: `~/.local/bin/native-launcher`
- Config: `~/.config/native-launcher/` (optional)
- Cache: `~/.cache/native-launcher/` (optional)
- Data: `~/.local/share/native-launcher/` (optional, includes usage stats)
- Compositor keybinds (optional, with backup)

## Manual Installation

If you prefer manual installation or the scripts don't work for your system:

### 1. Install Dependencies

**Arch Linux:**

```bash
sudo pacman -S gtk4 gtk4-layer-shell wl-clipboard
```

**Ubuntu/Debian:**

```bash
sudo apt install libgtk-4-1 libgtk-4-dev gtk4-layer-shell wl-clipboard
```

**Fedora:**

```bash
sudo dnf install gtk4 gtk4-devel gtk4-layer-shell wl-clipboard
```

### 2. Download Binary

Download the latest release from [GitHub Releases](https://github.com/ArunPrakashG/native-launcher/releases):

```bash
# Get latest release URL (requires jq)
DOWNLOAD_URL=$(curl -s https://api.github.com/repos/ArunPrakashG/native-launcher/releases/latest | jq -r '.assets[] | select(.name | test("native-launcher.*linux.*tar.gz")) | .browser_download_url')

# Download and extract
curl -L -o native-launcher.tar.gz "$DOWNLOAD_URL"
tar -xzf native-launcher.tar.gz
```

### 3. Install Binary

```bash
mkdir -p ~/.local/bin
cp native-launcher ~/.local/bin/
chmod +x ~/.local/bin/native-launcher
```

### 4. Add to PATH

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### 5. Configure Compositor

See the README for compositor-specific keybind configuration.

## Build from Source

If you want to build from source instead:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/ArunPrakashG/native-launcher.git
cd native-launcher

# Build release
cargo build --release

# Install
cp target/release/native-launcher ~/.local/bin/
```

## Troubleshooting

### Script fails to download release

- Check your internet connection
- Verify GitHub API access: `curl -s https://api.github.com/repos/ArunPrakashG/native-launcher/releases/latest`
- Try manual installation instead

### Missing dependencies

The script should install dependencies automatically. If it fails:

- Check the error message
- Install dependencies manually for your distribution
- Report the issue on GitHub

### Keybind not working

- Verify the binary is in PATH: `which native-launcher`
- Check compositor config file for the keybind
- Reload compositor configuration
- Try running manually: `~/.local/bin/native-launcher`

### Permission denied

- Ensure script is executable: `chmod +x install.sh`
- Check file permissions in `~/.local/bin`
- Verify you have write access to `~/.local/bin` and `~/.config`

## Contributing

Found a bug or want to add support for another distribution/compositor?

1. Fork the repository
2. Make your changes to `install.sh` or `uninstall.sh`
3. Test on your system
4. Submit a pull request

Please test thoroughly on your target system before submitting!

## License

These scripts are part of Native Launcher and licensed under the MIT License.
