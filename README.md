# Native Launcher 🚀

A modern, fast, and beautifully designed application launcher for Linux, written in Rust. Taking design inspiration from modern launchers like Raycast, built natively for Wayland with GTK4.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

> **📚 Full Documentation**: Visit the **[Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)** for comprehensive guides

## ✨ Features

- ⚡ **Lightning Fast**: Sub-100ms startup, <10ms search latency
- 🎨 **Modern Design**: Clean, minimal UI with coral accents (#FF6363) on charcoal (#1C1C1E)
- 🔍 **Smart Search**: Intelligent fuzzy matching with relevance scoring
- 🔌 **Plugin System**: Extensible with keyboard event handling + dynamic plugin loading
- 🌐 **Web Search**: Press `Ctrl+Enter` for instant web search
- 📁 **Workspace Detection**: Find VS Code/VSCodium workspaces automatically
- ⌨️ **Keyboard-Driven**: Full keyboard navigation and shortcuts
- 🪟 **Wayland Native**: Built on gtk4-layer-shell for seamless integration

## Quick Start

### Installation

**Build from source:**

```bash
git clone https://github.com/ArunPrakashG/native-launcher.git
cd native-launcher
cargo build --release
sudo install -Dm755 target/release/native-launcher /usr/local/bin/
```

**Full installation guide**: See [Wiki: Installation](https://github.com/ArunPrakashG/native-launcher/wiki/Installation)

### Basic Usage

1. **Configure hotkey** in your compositor (e.g., `Super+Space`)
2. **Launch**: Press your configured hotkey
3. **Search**: Type to search applications
4. **Navigate**: Use `↑/↓` arrow keys
5. **Launch**: Press `Enter`
6. **Web Search**: Press `Ctrl+Enter` to search the web

**Full usage guide**: See [Wiki: Quick Start](https://github.com/ArunPrakashG/native-launcher/wiki/Quick-Start)

## Documentation

📚 **[Visit the Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)** for complete documentation:

### User Guides

- **[Installation](https://github.com/ArunPrakashG/native-launcher/wiki/Installation)** - Build and install
- **[Keyboard Shortcuts](https://github.com/ArunPrakashG/native-launcher/wiki/Keyboard-Shortcuts)** - Complete shortcut reference
- **[Configuration](https://github.com/ArunPrakashG/native-launcher/wiki/Configuration)** - Customize behavior
- **[Compositor Integration](https://github.com/ArunPrakashG/native-launcher/wiki/Compositor-Integration)** - Set up hotkeys

### Developer Guides

- **[Plugin Development](https://github.com/ArunPrakashG/native-launcher/wiki/Plugin-Development)** - Create custom plugins
- **[Architecture](https://github.com/ArunPrakashG/native-launcher/wiki/Architecture)** - Technical design
- **[API Reference](https://github.com/ArunPrakashG/native-launcher/wiki/API-Reference)** - Plugin trait documentation
- **[Contributing](https://github.com/ArunPrakashG/native-launcher/wiki/Contributing)** - How to contribute

## Highlights

### Plugin-Driven Keyboard Events

Unique architecture that moves keyboard handling into plugins:

- **No hardcoded shortcuts**: Plugins handle their own key combinations
- **Priority-based dispatch**: High-priority plugins get events first
- **Extensible**: Add custom shortcuts without touching core code
- **Example**: Web search plugin handles `Ctrl+Enter` independently

See [Wiki: Architecture](https://github.com/ArunPrakashG/native-launcher/wiki/Architecture#keyboard-event-system) for details.

### Dynamic Plugin System

Load external plugins at runtime without recompiling:

- **Binary plugins**: Compile to `.so` shared libraries
- **Runtime loading**: Automatically discovered from plugin directories
- **Safe FFI**: Stable C ABI for plugin compatibility
- **Performance monitoring**: Built-in metrics and warnings for slow plugins
- **Example included**: Complete plugin template in `examples/plugin-template/`

See [DYNAMIC_PLUGINS.md](DYNAMIC_PLUGINS.md) for the complete guide on creating your own plugins.

### Performance First

Every feature prioritizes speed:

- **<100ms cold start**: Optimized startup sequence
- **<10ms search**: Fast fuzzy matching with caching
- **<30MB memory**: Minimal resource footprint
- **Background loading**: Icon cache preloads without blocking UI

See [Wiki: Performance](https://github.com/ArunPrakashG/native-launcher/wiki/Performance) for benchmarks.

## Development

```bash
# Run in debug mode
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

See [Wiki: Contributing](https://github.com/ArunPrakashG/native-launcher/wiki/Contributing) for development setup.

See [Wiki: Contributing](https://github.com/ArunPrakashG/native-launcher/wiki/Contributing) for development setup.

## Similar Projects

- [Rofi](https://github.com/davatorium/rofi) - The classic window switcher and launcher
- [Wofi](https://hg.sr.ht/~scoopta/wofi) - Wayland-native GTK launcher
- [Walker](https://github.com/abenz1267/walker) - Another great Wayland launcher
- [Ulauncher](https://ulauncher.io/) - Feature-rich Python launcher

## Contributing

Contributions are welcome! See [Wiki: Contributing](https://github.com/ArunPrakashG/native-launcher/wiki/Contributing) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Thanks to the Rust and GTK communities
- Inspired by [Raycast](https://www.raycast.com/), Rofi, and Wofi
- Built on the freedesktop.org specifications

---

**⭐ Star this repo if you find it useful!**

**📚 [Visit the Wiki](https://github.com/ArunPrakashG/native-launcher/wiki) for complete documentation**
