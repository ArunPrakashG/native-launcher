# Native Launcher 🚀

A modern, fast, and beautifully designed application launcher for Linux, written in Rust. Taking design inspiration from modern launchers like Raycast, built natively for Wayland with GTK4.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

## ✨ Features

### Core Functionality

- ⚡ **Lightning Fast**: Sub-100ms startup, <10ms search latency
- 🎨 **Modern Design**: Clean, minimal UI with coral accents (#FF6363) on charcoal (#1C1C1E)
- 🌊 **Fluid Interactions**: Fast 0.15s transitions, 60fps rendering
- 🔍 **Smart Search**: Intelligent fuzzy matching with relevance scoring
- 📊 **Usage Learning**: Tracks frequently used apps for better results

### Desktop Integration

- 🎯 **Wayland Native**: Built on gtk4-layer-shell for seamless integration
- 🖥️ **Multi-Monitor**: Smart positioning across displays
- 🔌 **Desktop Actions**: Right-click menus inline (e.g., Firefox → New Private Window)
- 🎨 **Icon Support**: Automatic icon resolution from system themes

### Plugin System

- 📁 **Workspace Detection**: Find VS Code/VSCodium workspaces (searches `.vscode`, `.code-workspace` files)
- 📂 **Recent Files**: Access recently opened files from all editors
- 🔧 **Calculator**: Instant math evaluation (`2+2`, `sqrt(16)`)
- 🌐 **Web Search**: Quick search shortcuts (@google, @github, @stackoverflow)
- 🐚 **Shell Commands**: Execute terminal commands directly
- 🔌 **Extensible**: Plugin API for custom functionality

### User Experience

- ⌨️ **Keyboard-Driven**: Full keyboard navigation (↑/↓ arrows, Enter, Escape)
- 🎨 **Themeable**: Custom CSS styling support
- 🏃 **Background Loading**: Icon cache preloads in background for instant display

## 🎨 Design Language

Native Launcher follows **Raycast's design principles**:

- **Minimal & Clean**: Flat design with subtle depth
- **Professional Dark Theme**: Charcoal backgrounds (#1C1C1E)
- **Coral/Red Accents**: Distinctive selection color (#FF6363)
- **Fast Animations**: Quick 0.15s transitions
- **Clear Typography**: High contrast white text on dark
- **Subtle Borders**: Minimal separation, maximum clarity

See [RAYCAST_DESIGN.md](RAYCAST_DESIGN.md) for detailed design documentation.

## 🎯 What Makes Native Launcher Special?

### Inline Desktop Actions

Unlike traditional launchers, Native Launcher displays application actions inline:

- **No mode switching**: Actions appear directly under parent apps
- **Visual clarity**: Indented with coral highlights for easy identification
- **Fast workflow**: No need to navigate into submenus

### Intelligent Workspace Detection

Automatically finds and displays workspaces from code editors:

- **VS Code/VSCodium**: Searches `~/.config/Code/storage.json` and `.code-workspace` files
- **Recent projects**: Shows recently opened workspaces first
- **Parent app icons**: Workspace entries show greyed VS Code icon for context

### Performance First Philosophy

Every feature prioritizes speed:

- **<100ms cold start**: Optimized startup sequence
- **<10ms search**: Fast fuzzy matching with caching
- **Background loading**: Icon cache preloads without blocking UI
- **No heavy animations**: Smooth 60fps transitions

## Screenshots

_Coming soon..._

## Installation

### Prerequisites

#### Build Dependencies

```bash
# Arch Linux
sudo pacman -S rust gtk4 gtk4-layer-shell pkg-config

# Ubuntu/Debian
sudo apt install cargo libgtk-4-dev libgtk4-layer-shell-dev pkg-config

# Fedora
sudo dnf install rust cargo gtk4-devel gtk4-layer-shell-devel pkg-config
```

#### Runtime Requirements

- GTK4
- gtk4-layer-shell
- A Wayland compositor with layer shell support (Sway, Hyprland, KDE Plasma, GNOME)

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/native-launcher
cd native-launcher

# Build in release mode
cargo build --release

# Install to system (optional)
sudo cp target/release/native-launcher /usr/local/bin/
```

See [RUNNING.md](RUNNING.md) for detailed build and execution instructions.

### Package Managers

_Coming soon: AUR, .deb, .rpm packages_

## Usage

### Basic Usage

1. **Start the launcher:**

   ```bash
   native-launcher
   ```

2. **Keyboard Shortcut:** Configure in your compositor to open on `Super+Space`

   See [HOTKEY_SETUP.md](HOTKEY_SETUP.md) for detailed setup instructions for all compositors.

3. **Search & Launch:**

   - Type to search applications
   - Use ↑/↓ arrow keys to navigate
   - Press Enter to launch
   - Press Escape to close

4. **Advanced Features:**
   - **Desktop Actions**: Actions appear inline under parent apps (e.g., Firefox shows "New Window", "Private Window")
   - **Workspace Search**: Type "code" to see VS Code workspaces from your projects
   - **Command Search**: Use `@` prefixes for specialized searches:
     - `@ws` or `@workspace` - Search only workspaces
     - `@recent` or `@file` - Search recent files
     - `@calc 2+2` - Calculator
     - `@google query` - Web search
     - `@shell ls -la` - Execute shell commands

### Configuration

Configuration file location: `~/.config/native-launcher/config.toml`

Generate default configuration:

```bash
native-launcher --generate-config
```

Example configuration:

```toml
[appearance]
width = 800
max_results = 10
show_icons = true
icon_size = 48

[behavior]
fuzzy_search = true
remember_usage = true

[keyboard]
activation_key = "Super_L+space"
```

See [docs/configuration.md](docs/configuration.md) for full configuration reference.

### Compositor Integration

#### Sway

Add to `~/.config/sway/config`:

```
bindsym $mod+Space exec native-launcher
```

#### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
bind = SUPER, SPACE, exec, native-launcher
```

#### Other Compositors

Consult your compositor's documentation for setting custom keybindings.

## Development

### Project Structure

```
native-launcher/
├── src/
│   ├── main.rs           # Entry point
│   ├── config/           # Configuration management
│   ├── desktop/          # Desktop file parsing
│   ├── search/           # Search engine
│   ├── ui/               # GTK4 user interface
│   ├── keyboard/         # Input handling
│   ├── cache/            # Caching system
│   └── utils/            # Utilities
├── plans.md              # Detailed development roadmap
└── docs/                 # Documentation
```

### Building for Development

```bash
# Build and run in debug mode
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint
cargo clippy
```

### Development Roadmap

See [plans.md](plans.md) for the complete phase-by-phase development plan including:

- **Phase 1 (MVP)**: Core functionality - desktop file parsing, basic search, GTK4 UI
- **Phase 2**: Enhanced search with fuzzy matching, icons, usage tracking
- **Phase 3**: Advanced features - plugins, theming, performance optimization
- **Phase 4**: X11 support (optional)
- **Phase 5**: Extended plugin ecosystem

## Similar Projects

Native Launcher is inspired by these excellent projects:

- [Rofi](https://github.com/davatorium/rofi) - The classic window switcher and launcher
- [Wofi](https://hg.sr.ht/~scoopta/wofi) - Wayland-native GTK launcher
- [Hyprshell](https://github.com/H3rmt/hyprshell) - Modern Rust-based launcher for Hyprland
- [Walker](https://github.com/abenz1267/walker) - Another great Wayland launcher
- [Ulauncher](https://ulauncher.io/) - Feature-rich Python launcher

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

### Ways to Contribute

- 🐛 Report bugs and issues
- 💡 Suggest new features
- 📝 Improve documentation
- 🔧 Submit bug fixes
- ✨ Develop new plugins
- 🌍 Add translations

## Performance

Target benchmarks:

- **Startup**: <100ms (cold start)
- **Search**: <10ms (500+ applications)
- **Memory**: <30MB (idle)

Current performance: _TBD (under development)_

## Troubleshooting

### Issue: Launcher doesn't show up

**Solution**: Ensure your compositor supports the layer shell protocol:

```bash
# Check if wlr-layer-shell is available
wayland-info | grep layer_shell
```

### Issue: Keyboard shortcut not working

**Solution**: Configure the shortcut in your compositor's config file, not in native-launcher.

### Issue: No applications showing

**Solution**: Verify desktop files exist:

```bash
ls /usr/share/applications/
ls ~/.local/share/applications/
```

For more issues, see [TESTING.md](TESTING.md) or open an issue.

## Documentation

- **[RUNNING.md](RUNNING.md)** - Building, running, and testing the launcher
- **[HOTKEY_SETUP.md](HOTKEY_SETUP.md)** - Setting up global keyboard shortcuts
- **[RAYCAST_DESIGN.md](RAYCAST_DESIGN.md)** - Design language and visual style
- **[TESTING.md](TESTING.md)** - Comprehensive testing guide
- **[CHANGES.md](CHANGES.md)** - Recent changes and fixes
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[plans.md](plans.md)** - Development roadmap

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Thanks to the Rust and GTK communities
- Inspired by the excellent work on Rofi, Wofi, and Hyprshell
- Design inspired by [Raycast](https://www.raycast.com/)
- Built on the freedesktop.org specifications

## Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/native-launcher/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/native-launcher/discussions)

---

**Star ⭐ this repo if you find it useful!**
