# Native Launcher 🚀

> **A blazing-fast, beautiful application launcher for Linux**  
> Built natively for Wayland with GTK4 · Designed for speed and elegance

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![Wayland](https://img.shields.io/badge/wayland-native-green.svg)

[Features](#-features) • [Quick Install](#-quick-install) • [Themes](#-themes) • [Documentation](https://github.com/ArunPrakashG/native-launcher/wiki)

</div>

---

## ✨ Features

- ⚡ **Lightning Fast** - <100ms startup, <10ms search
- 🎨 **Beautiful Themes** - 6 built-in themes (Default, Nord, Dracula, Catppuccin, Gruvbox, Tokyo Night)
- 🔍 **Smart Search** - Fuzzy matching with intelligent ranking
- 🧮 **Advanced Calculator** - Time, units, currency, timezone conversions
- 📁 **File Search** - System-wide indexing with plocate/fd/find
- 🌐 **Web Search** - Instant web search with Ctrl+Enter
- 🔌 **Plugin System** - Extensible with dynamic plugins
- ⌨️ **Keyboard-Driven** - Full keyboard navigation
- 🪟 **Wayland Native** - Built on gtk4-layer-shell
- 🔄 **Auto-Updates** - Background update checking

## 🚀 Quick Install

**One-line installation** (recommended):

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

The installer will:

- ✅ Backup existing installation (if found)
- ✅ Detect your system and compositor
- ✅ Install required dependencies
- ✅ Download the latest release
- ✅ Let you choose a theme interactively
- ✅ Configure compositor keybinds (Hyprland/Sway)

## 🎨 Themes

Choose from 6 beautiful themes during installation:

| Theme           | Accent             | Background         |
| --------------- | ------------------ | ------------------ |
| **Default**     | Coral `#FF6363`    | Charcoal `#1C1C1E` |
| **Nord**        | Frost `#88C0D0`    | Polar `#2E3440`    |
| **Dracula**     | Purple `#BD93F9`   | Dark `#282A36`     |
| **Catppuccin**  | Lavender `#B4BEFE` | Mocha `#1E1E2E`    |
| **Gruvbox**     | Orange `#FE8019`   | Dark `#282828`     |
| **Tokyo Night** | Blue `#7AA2F7`     | Night `#1A1B26`    |

## 📋 System Requirements

### Supported Distributions

- ✅ **Arch Linux** / Manjaro / EndeavourOS (primary support)
- ✅ Ubuntu / Debian / Pop!\_OS
- ✅ Fedora
- ✅ openSUSE

### Supported Compositors

- ✅ **Hyprland** (automatic setup)
- ✅ **Sway** (automatic setup)
- ✅ KDE Plasma (Wayland)
- ✅ GNOME (Wayland)
- ✅ River, Wayfire, etc.

### Dependencies

- GTK4
- gtk4-layer-shell
- wl-clipboard

## 🎯 Usage

1. Press **Super+Space** (default keybind)
2. Type to search applications
3. Use **↑/↓** to navigate
4. Press **Enter** to launch
5. Press **Ctrl+Enter** for web search
6. Press **Escape** to close

### Command Prefixes

| Prefix        | Plugin             | Example                |
| ------------- | ------------------ | ---------------------- |
| `@app`        | Applications       | `@app firefox`         |
| `@cal`        | Calculator         | `@cal 2+2`             |
| `@convert`    | Unit Conversion    | `@convert 10kg to lbs` |
| `@time`       | Time/Timezone      | `@time Tokyo`          |
| `@files`      | File Search        | `@files config`        |
| `$ or @shell` | Shell Commands     | `$ ls -la`             |
| `@ssh`        | SSH Connections    | `@ssh server`          |
| `@code`       | VS Code Workspaces | `@code my-project`     |

## 📚 Documentation

- 📖 [Full Documentation](https://github.com/ArunPrakashG/native-launcher/wiki)
- ⚙️ [Configuration Guide](https://github.com/ArunPrakashG/native-launcher/wiki/Configuration)
- 🔌 [Plugin Development](https://github.com/ArunPrakashG/native-launcher/wiki/Plugin-Development)
- 🎨 [UI Design](https://github.com/ArunPrakashG/native-launcher/wiki/UI-Design)
- 🚀 [Performance](https://github.com/ArunPrakashG/native-launcher/wiki/Performance)

## 🛠️ Advanced

<details>
<summary><b>Build from Source</b></summary>

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ArunPrakashG/native-launcher.git
cd native-launcher
cargo build --release

# Install
cp target/release/native-launcher ~/.local/bin/
```

</details>

<details>
<summary><b>Manual Compositor Setup</b></summary>

**Hyprland** (`~/.config/hypr/hyprland.conf`):

```bash
bind = SUPER, SPACE, exec, ~/.local/bin/native-launcher
```

**Sway** (`~/.config/sway/config`):

```bash
bindsym Mod4+Space exec ~/.local/bin/native-launcher
```

**River** (`~/.config/river/init`):

```bash
riverctl map normal Super Space spawn ~/.local/bin/native-launcher
```

</details>

<details>
<summary><b>Restore from Backup</b></summary>

If you need to restore from a previous backup:

```bash
# Run the restore script
./restore.sh

# Or if downloaded separately
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/restore.sh | bash
```

The restore script will:

- 📂 List all available backups with timestamps
- 🔍 Show what's included in each backup
- ✅ Let you select which backup to restore
- 🔄 Restore binary, config, plugins, cache, and data

Backups are stored in: `~/.local/share/native-launcher/backups/`

</details>

<details>
<summary><b>Uninstall</b></summary>

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/uninstall.sh | bash
```

</details>

## 🤝 Contributing

Contributions are welcome! See our [Contributing Guide](https://github.com/ArunPrakashG/native-launcher/wiki/Contributing) for details.

## 📄 License

MIT License - see [LICENSE](LICENSE) file

## 🙏 Acknowledgments

- Inspired by [Raycast](https://www.raycast.com/), [Rofi](https://github.com/davatorium/rofi), and [Wofi](https://hg.sr.ht/~scoopta/wofi)
- Built with the amazing Rust and GTK communities

---

<div align="center">

**⭐ Star this repo if you find it useful!**

[Report Bug](https://github.com/ArunPrakashG/native-launcher/issues) · [Request Feature](https://github.com/ArunPrakashG/native-launcher/issues) · [Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)

</div>
