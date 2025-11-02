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

### 🚀 Core Features

- ⚡ **Lightning Fast** - <35ms startup, <10ms search, optimized for responsive typing
- ⭐ **Pins/Favorites** - Pin apps with Ctrl+P; pinned apps show ★ indicator
- 🎨 **Theme System** - 6 themes + 7 accent colors (coral, teal, violet, blue, green, orange, pink)
- 🔍 **Smart Search** - Enhanced fuzzy matching with acronym support and query highlighting
- 🎯 **Usage Learning** - Hour-of-day boost and time-decay ranking (learns your patterns)
- 🎨 **Density Modes** - Compact or comfortable UI spacing (configurable)
- 🖼️ **Smart Icons** - Category-based fallback for 150+ app types (all apps get appropriate icons)

### 🪟 Session & Window Management

- 🔄 **Session Switcher** - `@switch` - Switch between windows and VS Code workspaces (Hyprland/Sway)
- 🪟 **Window Management** - `@wm` - Move, center, fullscreen, float, pin windows (Hyprland/Sway)
- 📂 **Recent Documents** - `@recent` - Access recently opened files with timestamps

### 💻 Development Tools

- 🔧 **Git Projects** - `@git` - Find and open git repositories in your editor
- 💻 **VS Code Workspaces** - `@code` - Quick access to coding projects
- 🐚 **SSH Manager** - `@ssh` - Connect to configured SSH hosts

### 🔍 Search & Productivity

- 🧮 **Advanced Calculator** - `@cal` - Math, units, currency, time, timezone conversions
- 📁 **File Search** - `@files` - System-wide file indexing with plocate/fd/find
- 🌐 **Web Search** - Instant web search with Ctrl+Enter (5+ search engines)
- 🌐 **Browser History** - `@tabs` / `@history` - Search across 6 browsers (Chrome, Brave, Firefox, Edge, Vivaldi, Opera)
- 📋 **Clipboard History** - `@clip` - Paste recent items (cliphist integration)
- 😀 **Emoji Picker** - `@emoji` - Search and copy 3000+ emojis

### 📸 Media & Screenshots

- 📷 **Screenshots** - `@ss` - Capture screen/window/area
- ✏️ **Screenshot Annotate** - `@ss annotate` - Edit screenshots with Swappy

### ⌨️ Enhanced Keyboard Actions

- `Alt+Enter` - Open containing folder (file results)
- `Ctrl+Enter` - Copy path to clipboard (doesn't close window)
- `Ctrl+P` - Pin/unpin selected app
- `Ctrl+1` - Execute first result instantly

### 🎨 Visual Polish

- 🏷️ **Icon Badges** - Visual indicators (terminal🖥️, web🌐, file📄, folder📁)
- ✨ **Match Highlighting** - Coral-colored query matches in results
- 🎭 **Smooth Animations** - 60fps+ transitions with cubic-bezier easing

### 🔧 Technical Excellence

- 🔌 **Plugin System** - Extensible with dynamic plugins and script support
- 🪟 **Wayland Native** - Built on gtk4-layer-shell
- 🔄 **Auto-Updates** - Background version checking
- 📊 **150 Tests** - Comprehensive test coverage
- 🎯 **Performance First** - Every feature optimized for speed

## 🚀 Quick Install

**One-line installation** (recommended):

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

**System-wide installation** (optional - creates symlink for all users):

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash -s -- --link
```

The installer will:

- ✅ Backup existing installation (if found)
- ✅ Detect your system and compositor
- ✅ Install required dependencies
- ✅ Download the latest release to `~/.local/bin`
- ✅ Let you choose a theme interactively
- ✅ Configure compositor keybinds (Hyprland/Sway)
- ✅ Install man page to `~/.local/share/man`
- ✅ (Optional) Create system-wide symlink with `--link`

### Installation Options

| Option    | Description                                             | Requires Sudo |
| --------- | ------------------------------------------------------- | ------------- |
| (default) | Install to `~/.local/bin` with absolute paths in config | No            |
| `--link`  | Create symlink in `/usr/local/bin` for system-wide use  | Yes (once)    |
| `--help`  | Show installation help and all available options        | No            |

**Why use `--link`?**

- Makes `native-launcher` available system-wide (all users)
- Simpler compositor configs (uses `native-launcher` instead of full path)
- Binary updates don't require sudo (only the initial symlink creation does)
- Configs automatically updated to use short command name

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
5. Press **Ctrl+1** to execute first result (fast workflow - no navigation needed)
6. Press **Ctrl+Enter** for web search
7. Press **Ctrl+P** to Pin/Unpin selected app (appears first in default results)
8. Press **Escape** to close

### Command Prefixes

| Prefix                | Plugin                 | Example                |
| --------------------- | ---------------------- | ---------------------- |
| `@app`                | Applications           | `@app firefox`         |
| `@switch` / `@sw`     | Session Switcher       | `@switch code`         |
| `@wm` / `@window`     | Window Management      | `@wm workspace 2`      |
| `@git` / `@repo`      | Git Projects           | `@git my-project`      |
| `@recent` / `@r`      | Recent Documents       | `@recent config`       |
| `@tabs` / `@history`  | Browser History        | `@tabs github`         |
| `@clip`               | Clipboard History      | `@clip password`       |
| `@emoji`              | Emoji Picker           | `@emoji smile`         |
| `@cal`                | Calculator             | `@cal 2+2`             |
| `@convert`            | Unit Conversion        | `@convert 10kg to lbs` |
| `@time`               | Time/Timezone          | `@time Tokyo`          |
| `@files`              | File Search            | `@files config`        |
| `$ or @shell`         | Shell Commands         | `$ ls -la`             |
| `@ssh`                | SSH Connections        | `@ssh server`          |
| `@code`               | VS Code Workspaces     | `@code my-project`     |
| `@screenshot` / `@ss` | Screenshots & Annotate | `@ss annotate`         |

### Screenshots

- Use `@screenshot` or the short `@ss` prefix to list capture options (full screen, active window, selection)
- Images are saved to `~/Pictures/Screenshots` (the folder is created automatically)
- Launcher detects common tools automatically (`grimshot`, `hyprshot`, `gnome-screenshot`, `spectacle`, `maim`, `scrot`, `grim` + `slurp`)

## 📚 Documentation

**Man Page** - Comprehensive reference manual (installed with launcher):

```bash
man native-launcher
```

**Online Wiki** - Detailed guides and tutorials:

- 📖 [Full Documentation](https://github.com/ArunPrakashG/native-launcher/wiki)
- ⚙️ [Configuration Guide](https://github.com/ArunPrakashG/native-launcher/wiki/Configuration)
- 🔌 [Plugin Development](https://github.com/ArunPrakashG/native-launcher/wiki/Plugin-Development)
- 🎨 [UI Design](https://github.com/ArunPrakashG/native-launcher/wiki/UI-Design)
- 🚀 [Performance](https://github.com/ArunPrakashG/native-launcher/wiki/Performance)
- ⭐ [Pins & Favorites](https://github.com/ArunPrakashG/native-launcher/wiki/Keyboard-Shortcuts#navigation)

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
# If installed with --link
bind = SUPER, SPACE, exec, native-launcher

# If installed without --link (default)
bind = SUPER, SPACE, exec, ~/.local/bin/native-launcher
```

**Sway** (`~/.config/sway/config`):

```bash
# If installed with --link
bindsym Mod4+Space exec native-launcher

# If installed without --link (default)
bindsym Mod4+Space exec ~/.local/bin/native-launcher
```

**River** (`~/.config/river/init`):

```bash
# If installed with --link
riverctl map normal Super Space spawn native-launcher

# If installed without --link (default)
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

The uninstaller will remove:

- ✅ Binary from `~/.local/bin`
- ✅ System-wide symlink (if created with `--link`)
- ✅ Man page from `~/.local/share/man`
- ✅ Configuration files (with confirmation)
- ✅ Cache and data (with confirmation)
- ✅ Compositor keybinds (with confirmation)

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
