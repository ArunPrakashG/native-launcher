# Native Launcher - Development Plan

## Project Overview

Native Launcher is a modern, fast, and extensible application launcher for Linux environments, written in Rust. It provides a keyboard-driven overlay interface for quickly finding and launching applications, inspired by tools like Rofi, Wofi, and Hyprshell.

### Core Philosophy

- **Speed First**: Sub-100ms startup time, instant search results
- **Wayland Native**: Built for modern Linux compositors using layer shell protocol
- **Extensible**: Plugin architecture for custom functionality
- **Minimal Dependencies**: Leverage Rust's ecosystem efficiently
- **User-Friendly**: Sensible defaults with powerful configuration options

## Technical Architecture

### Technology Stack

#### Core Technologies

- **Language**: Rust (stable channel)
- **GUI Framework**: GTK4 with gtk4-layer-shell
- **Display Protocol**: Wayland (layer shell protocol v1)
- **Desktop Integration**: freedesktop.org standards

#### Key Libraries

- **gtk4** (0.9+): Modern GTK4 bindings for Rust
- **gtk4-layer-shell** (0.4+): Wayland layer shell protocol support
- **freedesktop-desktop-entry**: Parse .desktop files
- **fuzzy-matcher** / **nucleo**: Fuzzy search algorithms
- **tokio**: Async runtime for non-blocking I/O
- **serde** / **toml**: Configuration management
- **tracing**: Structured logging

### Architecture Components

```
┌─────────────────────────────────────────────────────┐
│                   Main Process                       │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Config    │  │   Desktop    │  │   Cache    │ │
│  │   Manager   │  │   Parser     │  │   Manager  │ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
│                                                       │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Search    │  │   Keyboard   │  │    UI      │ │
│  │   Engine    │  │   Handler    │  │   Layer    │ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
│                                                       │
│  ┌─────────────────────────────────────────────┐   │
│  │          Plugin System (Phase 3+)            │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
          │                                   │
          ▼                                   ▼
   ┌──────────────┐                  ┌───────────────┐
   │  Wayland     │                  │  Desktop      │
   │  Compositor  │                  │  Files        │
   │  (Layer Shell)│                 │  (/usr/share) │
   └──────────────┘                  └───────────────┘
```

### Directory Structure

```
native-launcher/
├── Cargo.toml                 # Project manifest
├── Cargo.lock                 # Lock file (generated)
├── README.md                  # User documentation
├── plans.md                   # This file
├── LICENSE                    # MIT License
├── .gitignore                # Git ignore rules
│
├── src/
│   ├── main.rs               # Application entry point
│   │
│   ├── config/               # Configuration management
│   │   ├── mod.rs
│   │   ├── loader.rs         # Load/parse config files
│   │   └── schema.rs         # Config data structures
│   │
│   ├── desktop/              # Desktop file parsing
│   │   ├── mod.rs
│   │   ├── parser.rs         # Parse .desktop files
│   │   ├── scanner.rs        # Scan system directories
│   │   └── entry.rs          # Desktop entry model
│   │
│   ├── search/               # Search engine
│   │   ├── mod.rs
│   │   ├── fuzzy.rs          # Fuzzy matching algorithm
│   │   ├── indexer.rs        # Search index builder
│   │   └── scorer.rs         # Relevance scoring
│   │
│   ├── ui/                   # User interface
│   │   ├── mod.rs
│   │   ├── window.rs         # Main window setup
│   │   ├── entry_widget.rs   # Search input widget
│   │   ├── results_list.rs   # Results display
│   │   └── styles.css        # GTK CSS styling
│   │
│   ├── keyboard/             # Keyboard handling
│   │   ├── mod.rs
│   │   ├── shortcuts.rs      # Global hotkey registration
│   │   └── input.rs          # Input event processing
│   │
│   ├── cache/                # Caching system
│   │   ├── mod.rs
│   │   ├── storage.rs        # Persistent cache storage
│   │   └── usage.rs          # Usage statistics tracking
│   │
│   └── utils/                # Utilities
│       ├── mod.rs
│       ├── icons.rs          # Icon lookup/loading
│       └── exec.rs           # Process execution helpers
│
├── benches/                  # Performance benchmarks
│   └── search_benchmark.rs
│
├── tests/                    # Integration tests
│   ├── desktop_parsing.rs
│   └── search_engine.rs
│
├── config/                   # Default configurations
│   └── default.toml
│
└── docs/                     # Additional documentation
    ├── configuration.md
    ├── plugins.md
    └── keybindings.md
```

## Development Phases

---

## MVP - Phase 1: Core Functionality (Weeks 1-3)

**Goal**: Create a working launcher that can search and launch desktop applications.

### Features

- ✅ Basic GTK4 window with layer shell overlay
- ✅ Parse .desktop files from standard locations
- ✅ Simple text-based search (substring matching)
- ✅ Display search results in a list
- ✅ Launch selected application on Enter
- ✅ Basic keyboard navigation (arrow keys, Enter, Escape)
- ✅ Global activation shortcut (Super+Space)
- ✅ Center-screen overlay positioning

### Technical Tasks

#### Week 1: Project Setup & Desktop File Parsing

- [x] Initialize Rust project structure
- [ ] Set up logging with tracing
- [ ] Implement desktop file scanner
  - Scan `/usr/share/applications`
  - Scan `~/.local/share/applications`
  - Parse Name, Exec, Icon, Categories fields
- [ ] Create `DesktopEntry` data model
- [ ] Write unit tests for parser
- [ ] Benchmark parsing performance

#### Week 2: UI Foundation & Wayland Integration

- [ ] Set up GTK4 application structure
- [ ] Initialize gtk4-layer-shell
  - Configure layer (overlay)
  - Set keyboard mode (exclusive)
  - Position window (center screen)
- [ ] Create search entry widget
- [ ] Create results list widget (ListBox)
- [ ] Implement basic CSS styling
- [ ] Handle window show/hide

#### Week 3: Search & Launch Logic

- [ ] Implement substring search algorithm
- [ ] Filter desktop entries by search query
- [ ] Display filtered results in UI
- [ ] Implement keyboard navigation
  - Up/Down arrows: Navigate results
  - Enter: Launch selected app
  - Escape: Close launcher
- [ ] Execute applications via `exec()`
- [ ] Global keyboard shortcut registration
  - Listen for Super+Space
  - Show/hide window on trigger
- [ ] Basic error handling

### Deliverable

A working launcher that:

- Opens on Super+Space
- Shows all installed applications
- Filters as you type
- Launches selected application
- Closes on Escape

---

## Phase 2: Enhanced Search & UX (Weeks 4-6)

**Goal**: Improve search quality, add icons, and implement usage-based ranking.

### Features

- ✅ Fuzzy search with relevance scoring
- ✅ Icon support (load from theme)
- ✅ Usage history tracking
- ✅ Frequency-based result ranking
- ✅ Multi-field search (name, keywords, categories)
- ✅ Improved keyboard shortcuts
- ✅ Visual feedback and animations
- ✅ Configuration file support

### Technical Tasks

#### Week 4: Fuzzy Search Implementation

- [ ] Integrate fuzzy-matcher or nucleo
- [ ] Implement multi-field search
  - Match against Name, GenericName, Keywords
  - Weight fields differently
- [ ] Score results by relevance
  - Exact prefix match: highest
  - Word boundary match: high
  - Fuzzy match: medium
- [ ] Sort results by score
- [ ] Benchmark search performance (target: <10ms)

#### Week 5: Icons & Visual Polish

- [ ] Implement icon lookup
  - Use freedesktop-icons crate
  - Search system icon theme
  - Handle missing icons gracefully
- [ ] Display icons in results list
- [ ] Improve UI styling
  - Modern rounded corners
  - Proper spacing and padding
  - Highlight selected item
  - Semi-transparent background
- [ ] Add keyboard visual feedback

#### Week 6: Usage History & Configuration

- [ ] Implement usage tracking
  - Store launch counts in cache
  - Track last used timestamp
  - Persist to disk (bincode format)
- [ ] Boost frequently used apps in results
- [ ] Create configuration schema
  - Window dimensions
  - Position (top/center/bottom)
  - Max results shown
  - Keyboard shortcuts
  - Icon size
- [ ] Load config from `~/.config/native-launcher/config.toml`
- [ ] Implement config hot-reload

### Deliverable

A polished launcher with:

- Fast, intelligent fuzzy search
- Beautiful icon display
- Smart ranking based on usage
- Configurable behavior
- Professional UI/UX

---

## Phase 3: Advanced Features (Weeks 7-9)

**Goal**: Add power-user features and extensibility.

### Features

- ✅ Multi-monitor support
- ✅ Terminal application support (launch in terminal)
- ✅ Desktop actions support (right-click menu)
- ✅ Custom CSS theming
- ✅ Command execution mode (run arbitrary commands)
- ✅ Math calculator mode
- ✅ Window positioning options
- ✅ Performance optimizations

### Technical Tasks

#### Week 7: Extended Desktop Integration

- [ ] Parse and display Desktop Actions
  - Show submenu for apps with actions
  - Execute specific actions
- [ ] Handle Terminal=true applications
  - Detect user's default terminal
  - Launch with proper terminal wrapper
- [ ] Multi-monitor support
  - Detect active monitor
  - Position window on correct display
- [ ] Handle special Exec field codes
  - %f, %F: File arguments
  - %u, %U: URL arguments
  - Strip unsupported codes

#### Week 8: Plugin System Foundation

- [ ] Design plugin API
  - Search provider interface
  - Result item trait
  - Launch handler trait
- [ ] Built-in plugins:
  - **Applications**: Default app launcher
  - **Calculator**: Evaluate math expressions
  - **Shell**: Execute shell commands
  - **Web Search**: Quick web searches
- [ ] Plugin configuration in TOML
- [ ] Plugin priority/ordering

#### Week 9: Performance & Polish

- [ ] Optimize desktop file parsing
  - Cache parsed entries on disk
  - Incremental updates on file changes
- [ ] Optimize search indexing
  - Pre-build search index
  - Use efficient data structures
- [ ] Add benchmarks for all critical paths
- [ ] Memory profiling and optimization
- [ ] Custom CSS theme support
  - Load user CSS from config dir
  - Example themes included

### Deliverable

A feature-complete launcher with:

- Desktop actions support
- Terminal app handling
- Plugin system for extensions
- Excellent performance (<50ms search)
- Themeable interface

---

## Phase 4: X11 Support (Optional) (Weeks 10-11)

**Goal**: Add X11 backend for broader compatibility.

### Features

- ✅ X11 window management
- ✅ Detect Wayland vs X11 at runtime
- ✅ Fallback gracefully between backends

### Technical Tasks

#### Week 10: X11 Backend

- [ ] Integrate x11rb or xcb crate
- [ ] Create X11 window
  - Set override-redirect
  - Position at screen center
  - Handle input focus
- [ ] Abstract backend differences
  - Common window trait
  - Platform-specific implementations
- [ ] Runtime backend detection

#### Week 11: Testing & Refinement

- [ ] Test on X11 environments
- [ ] Test on Wayland environments
- [ ] Handle edge cases
- [ ] Document limitations of each backend

### Deliverable

Cross-platform launcher supporting both Wayland and X11.

---

## Phase 5: Advanced Plugin Ecosystem (Weeks 12-14)

**Goal**: Expand plugin capabilities and enable community contributions.

### Features

- ✅ SSH connection launcher
- ✅ File browser mode
- ✅ Clipboard history
- ✅ Window switcher
- ✅ Custom user scripts
- ✅ Plugin marketplace concept

### Technical Tasks

#### Week 12: Core Plugins

- [ ] **SSH Plugin**: Launch SSH connections
  - Parse ~/.ssh/config
  - Display hosts
  - Launch in terminal
- [ ] **File Browser**: Navigate filesystem
  - Quick file opening
  - Recent files
- [ ] **Window Switcher**: Switch between open windows
  - Integrate with compositor
  - Show window previews

#### Week 13: Script Plugin System

- [ ] Script plugin specification
  - JSON/TOML manifest
  - Input/output protocol
  - Example scripts provided
- [ ] Script execution sandbox
- [ ] Documentation for plugin authors

#### Week 14: Community & Documentation

- [ ] Plugin development guide
- [ ] Example plugins repository
- [ ] Community plugin listing
- [ ] Plugin installation mechanism

### Deliverable

Extensible launcher with rich plugin ecosystem and community support.

---

## Technical Considerations

### Performance Targets

- **Startup time**: <100ms (cold start)
- **Search latency**: <10ms (for 500 apps)
- **Memory usage**: <30MB (idle)
- **Disk cache**: <5MB

### Wayland Layer Shell Configuration

```rust
// Recommended settings
layer: Layer::Overlay
keyboard_mode: KeyboardMode::Exclusive
anchor: [] // No anchoring (centered)
exclusive_zone: -1 // Don't reserve space
namespace: "native-launcher"
```

### Desktop File Locations (freedesktop.org)

```
System-wide:
  /usr/share/applications/
  /usr/local/share/applications/

User-specific:
  ~/.local/share/applications/
```

### Configuration File Schema (Phase 2)

```toml
[appearance]
width = 800
height = 600
position = "center"  # top, center, bottom
max_results = 10
show_icons = true
icon_size = 48
theme = "dark"  # dark, light, or custom CSS path

[behavior]
activation_key = "Super_L+space"
fuzzy_search = true
remember_usage = true
usage_cache_days = 90

[keyboard]
move_up = "Up"
move_down = "Down"
select = "Return"
close = "Escape"
launch_1 = "Control+1"
launch_2 = "Control+2"
# ... up to launch_10

[plugins]
enabled = ["applications", "calculator", "shell"]

[plugins.applications]
show_generic_name = true
show_categories = false

[plugins.calculator]
auto_calculate = true

[plugins.shell]
show_in_terminal = false
```

### Cache Storage Format

```rust
// ~/.cache/native-launcher/
struct Cache {
    version: u32,
    desktop_entries: Vec<CachedDesktopEntry>,
    usage_stats: HashMap<String, UsageStats>,
    last_updated: SystemTime,
}

struct UsageStats {
    launch_count: u32,
    last_launched: SystemTime,
    frequency_score: f32,
}
```

---

## Testing Strategy

### Unit Tests

- Desktop file parser
- Search algorithms
- Configuration loader
- Icon resolver

### Integration Tests

- Full search workflow
- App launching
- Keyboard navigation
- Cache persistence

### Performance Tests

- Search latency benchmarks
- Memory usage profiling
- Startup time measurement
- Cache I/O performance

### Manual Testing

- Test on different compositors (Sway, Hyprland, Mutter)
- Test with different icon themes
- Test with large numbers of apps (1000+)
- Test keyboard shortcuts conflicts

---

## Dependencies & System Requirements

### Build Dependencies

- Rust 1.75+ (stable)
- GTK4 development headers
- gtk4-layer-shell library
- pkg-config

### Runtime Requirements

- GTK4
- gtk4-layer-shell
- Wayland compositor with layer shell support (or X11)
- freedesktop.org compliant desktop environment

### Recommended Compositors

- Sway (wlroots-based)
- Hyprland
- KDE Plasma (Wayland)
- GNOME (Mutter)

---

## Installation & Packaging

### From Source (Phase 1+)

```bash
git clone https://github.com/yourusername/native-launcher
cd native-launcher
cargo build --release
sudo cp target/release/native-launcher /usr/local/bin/
```

### Distribution Packages (Phase 3+)

- **Arch Linux**: AUR package
- **Ubuntu/Debian**: .deb package
- **Fedora**: .rpm package
- **Nix**: Flake

### Configuration (Phase 2+)

```bash
mkdir -p ~/.config/native-launcher
native-launcher --generate-config > ~/.config/native-launcher/config.toml
```

---

## Documentation Plan

### User Documentation

- README.md: Quick start guide
- docs/installation.md: Installation instructions
- docs/configuration.md: Configuration reference
- docs/keybindings.md: Keyboard shortcuts
- docs/plugins.md: Available plugins

### Developer Documentation

- docs/architecture.md: System design
- docs/plugin-api.md: Plugin development
- docs/contributing.md: Contribution guidelines
- Inline code documentation (rustdoc)

---

## Success Metrics

### MVP (Phase 1)

- ✅ Successfully launches apps
- ✅ <200ms startup time
- ✅ Works on Sway/Hyprland

### Phase 2

- ✅ <100ms startup time
- ✅ <10ms search latency
- ✅ Positive user feedback on UX

### Phase 3

- ✅ <50ms search latency
- ✅ 5+ working plugins
- ✅ Custom theming working

### Long-term

- 1000+ GitHub stars
- 10+ community plugins
- Packaged in major distros
- Active community contributions

---

## Known Challenges & Solutions

### Challenge 1: Fast Startup Time

**Problem**: GTK4 initialization can be slow  
**Solution**:

- Lazy-load non-critical components
- Cache parsed desktop files
- Use async I/O for file scanning

### Challenge 2: Global Keyboard Shortcuts

**Problem**: Wayland doesn't support global hotkeys natively  
**Solution**:

- Use compositor-specific protocols
- For Sway/Hyprland: Configure in compositor config
- For GNOME: Use keybinding daemon
- Document per-compositor setup

### Challenge 3: Icon Resolution

**Problem**: Finding correct icons across themes  
**Solution**:

- Use freedesktop-icons crate
- Implement fallback chain
- Cache resolved icon paths

### Challenge 4: Multi-Monitor Positioning

**Problem**: Detecting active monitor in Wayland  
**Solution**:

- Query compositor for focused output
- Use cursor position as fallback
- Allow manual override in config

---

## Future Ideas (Post-MVP)

### Nice-to-Have Features

- [ ] Preview thumbnails for apps
- [ ] Recently used files integration
- [ ] Bookmarks/favorites system
- [ ] Unicode emoji picker plugin
- [ ] Color picker plugin
- [ ] Dictionary/thesaurus lookup
- [ ] Currency converter
- [ ] Unit converter
- [ ] Snippet manager
- [ ] Password manager integration
- [ ] Browser bookmark search
- [ ] Music player control
- [ ] System actions (shutdown, reboot, logout)
- [ ] Remote machine search (via SSH)

### Technical Improvements

- [ ] IPC for scripting/automation
- [ ] DBus interface
- [ ] Wayland protocols contribution
- [ ] GPU-accelerated rendering
- [ ] Machine learning for prediction

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas for Contribution

- Icon theme compatibility
- Plugin development
- Compositor-specific features
- Performance optimizations
- Documentation improvements
- Translation/i18n

---

## License

MIT License - See [LICENSE](LICENSE) file for details.

---

## References & Inspiration

### Similar Projects

- [Rofi](https://github.com/davatorium/rofi) - The OG launcher
- [Wofi](https://hg.sr.ht/~scoopta/wofi) - Wayland-native launcher
- [Hyprshell](https://github.com/H3rmt/hyprshell) - Modern Rust launcher for Hyprland
- [Walker](https://github.com/abenz1267/walker) - Another Wayland launcher
- [Ulauncher](https://ulauncher.io/) - Python-based launcher (X11)

### Technical Resources

- [Layer Shell Protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
- [Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [Icon Theme Spec](https://specifications.freedesktop.org/icon-theme-spec/latest/)
- [GTK4 Documentation](https://docs.gtk.org/gtk4/)
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)

---

**Last Updated**: October 20, 2025  
**Version**: 0.1.0-dev  
**Status**: MVP Phase 1 - In Progress
