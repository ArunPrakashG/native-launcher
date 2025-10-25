# Native Launcher 🚀 (Currently WIP)

A modern, fast, and beautifully designed application launcher for Linux, written in Rust. Taking design inspiration from modern launchers like Raycast, built natively for Wayland with GTK4.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

> **📚 Full Documentation**: Visit the **[Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)** for comprehensive guides

## ✨ Features

- ⚡ **Lightning Fast**: Sub-100ms startup, <10ms search latency with smart debouncing
- 🎨 **Modern Design**: Fully rounded UI with smooth animations, coral accents (#FF6363) on charcoal (#1C1C1E)
- 🔍 **Smart Search**: Intelligent fuzzy matching with relevance scoring and smart triggering
- 💾 **System-Wide File Search**: Integrated file indexing using native Linux tools (plocate/fd/find)
- 🔌 **Plugin System**: Extensible with keyboard event handling + dynamic plugin loading
- 📜 **Script Plugins**: Create plugins in any language (Bash, Python, etc.) without compiling Rust
- 🧮 **Advanced Calculator**: Time calculations, unit conversions, currency, timezone - with clipboard copy
- 🌐 **Web Search**: Press `Ctrl+Enter` for instant web search
- 📁 **Workspace Detection**: Find VS Code/VSCodium workspaces automatically
- ⌨️ **Keyboard-Driven**: Full keyboard navigation and shortcuts
- 🪟 **Wayland Native**: Built on gtk4-layer-shell for seamless integration
- 🎬 **Smooth Animations**: List entry/exit animations with staggered timing for modern feel

## Quick Start

### Installation

**Runtime Dependencies:**

- `wl-clipboard` - For clipboard support (advanced calculator, etc.)
  ```bash
  # Ubuntu/Debian
  sudo apt install wl-clipboard
  # Arch
  sudo pacman -S wl-clipboard
  # Fedora
  sudo dnf install wl-clipboard
  ```

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

### User Directories

Native Launcher stores configuration and data in standard XDG directories:

**Configuration:**

- `~/.config/native-launcher/config.toml` - Main configuration file
- `~/.config/native-launcher/plugins/` - Dynamic plugins directory (`.so` files)

**Cache & Data:**

- `~/.cache/native-launcher/entries.cache` - Desktop entries cache
- `~/.cache/native-launcher/icons.cache` - Icon paths cache
- `~/.local/share/native-launcher/usage.bin` - Application usage statistics

**First Run:**
On first launch, Native Launcher will automatically:

1. Create the config directory with default `config.toml`
2. Scan system applications from `/usr/share/applications` and `~/.local/share/applications`
3. Build icon and entry caches for fast subsequent startups

To reset to defaults, simply delete `~/.config/native-launcher/` and the cache files will be regenerated.

**Command Prefixes**: Use `@` commands to search specific plugins:

- `@app <query>` - Search applications only
- `@cal <expression>` - Calculate mathematical expressions
- `@convert`, `@time`, `@currency` - Advanced calculations (time, units, currency, timezone)
- `@code`, `@zed`, `@editor` - Search editor workspaces
- `@files` - Search recent files and system-wide file index
- `@shell <cmd>` or `$ <cmd>` - Execute shell commands
- `@ssh <host>` - Connect to SSH hosts
- `@web <query>` - Web search

**Performance Features**:

- 🚀 **150ms debouncing** - Prevents lag during rapid typing
- 🧠 **Smart triggering** - Skips expensive file searches when apps match
- ⚡ **Async file search** - Background indexing ready (future use)
- 📊 **Two-pass search** - Applications first, then other plugins with context

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

- **<100ms cold start**: Optimized startup sequence with async loading
- **<10ms search**: Fast fuzzy matching with caching and smart triggering
- **<30MB memory**: Minimal resource footprint
- **Background loading**: Icon cache preloads without blocking UI
- **Smart debouncing**: 150ms delay prevents input lag during rapid typing
- **Two-pass search**: Apps searched first, expensive operations only when needed
- **File index integration**: Native Linux tools (plocate → mlocate → locate → fd → find) for system-wide search

See [Wiki: Performance](https://github.com/ArunPrakashG/native-launcher/wiki/Performance) for benchmarks.

### Advanced Calculator with Clipboard

Natural language calculations beyond basic math:

**Time Calculations:**

- `1 hour ago` → Past time in local, UTC, and unix timestamp
- `in 5 hours` → Future time calculation
- `350 days ago` → Historical dates
- Press `Enter` to copy to clipboard with desktop notification

**Unit Conversions:**

- `150 days to years` → Time unit conversion
- `5 km to miles` → Distance conversion
- `100 pounds to kg` → Weight conversion
- `32 fahrenheit to celsius` → Temperature conversion

**Currency Exchange:**

- `100 USD to EUR` → Currency conversion (10+ currencies supported)
- `50 GBP to JPY` → Multi-currency support

**Timezone Info:**

- `now in UTC` → Current time in multiple timezones

**Features:**

- 📋 One-press clipboard copy (Enter key)
- 🔔 Desktop notifications confirm copy
- 🚀 Instant results as you type
- 💡 Smart natural language parsing

See [docs/ADVANCED_CALCULATOR.md](docs/ADVANCED_CALCULATOR.md) for complete guide and examples.

### Script Plugin System

**Extend the launcher with any programming language** - no Rust compilation needed!

Create custom plugins using Bash, Python, Node.js, or any executable:

**Quick Example:**

```bash
# Create plugin directory
mkdir -p ~/.config/native-launcher/plugins/hello-world

# Write manifest (plugin.toml)
cat > ~/.config/native-launcher/plugins/hello-world/plugin.toml <<EOF
[metadata]
name = "Hello World"
description = "My first plugin"
author = "Your Name"
version = "1.0.0"
priority = 600

triggers = ["hello ", "hi "]

[execution]
script = "hello.sh"
interpreter = "bash"
output_format = "json"
timeout_ms = 1000
EOF

# Write script
cat > ~/.config/native-launcher/plugins/hello-world/hello.sh <<'EOF'
#!/usr/bin/env bash
cat <<JSON
{
  "results": [
    {
      "title": "Hello, $1!",
      "subtitle": "Press Enter to copy",
      "command": "echo 'Hello, $1!' | wl-copy"
    }
  ]
}
JSON
EOF

chmod +x ~/.config/native-launcher/plugins/hello-world/hello.sh
```

**Built-in Example Plugins:**

- 🌤️ **Weather** - Get weather forecasts (`weather Tokyo` or `w London`)
- 😀 **Emoji Search** - Search 200+ emojis (`emoji smile` or `:fire`)
- 🎨 **Color Picker** - Convert colors (`color #FF5733` or `col rgb(255,87,51)`)

**Features:**

- 📝 TOML-based manifests (metadata, triggers, execution config)
- 🔄 JSON and text output formats
- 🔌 Auto-discovery from multiple directories
- ⚙️ Environment variable injection
- ⏱️ Configurable timeouts
- 🎯 Priority-based ordering
- 📋 Clipboard integration built-in

**Get Started:**

- **[Plugin Development Guide](docs/PLUGIN_DEVELOPMENT.md)** - Complete reference (600+ lines)
- **[Example Plugins](examples/plugins/)** - Weather, emoji, color picker
- **[Wiki: Script Plugins](https://github.com/ArunPrakashG/native-launcher/wiki/Script-Plugins)** - User guide

**Plugin Ideas**: Dictionary, translation, cryptocurrency prices, password generator, QR codes, system info, process manager, clipboard history, and more!

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
