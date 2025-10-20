# Native Launcher - Quick Start Guide

## What is Native Launcher?

Native Launcher is a keyboard-driven application launcher for Linux, similar to Spotlight on macOS or Rofi on Linux. It provides a fast, beautiful overlay that lets you quickly search and launch applications.

## Why Another Launcher?

- **Modern Rust**: Memory-safe, fast, and maintainable
- **Wayland First**: Built for modern Linux compositors using layer shell
- **Excellent UX**: Fuzzy search, smart ranking, beautiful interface
- **Extensible**: Plugin system for custom functionality
- **Active Development**: Modern codebase with clear roadmap

## Quick Commands

```bash
# Build the project
cargo build --release

# Run in development mode
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open
```

## Development Phases

### ✅ Setup Complete

- Project structure initialized
- Dependencies configured
- Documentation created
- Development plan established

### 🚧 Phase 1: MVP (Current)

**Goal**: Basic launcher that searches and launches apps

**Tasks**:

1. Desktop file parsing
2. GTK4 UI with layer shell
3. Simple substring search
4. Keyboard navigation
5. App launching
6. Global shortcut integration

**ETA**: 3 weeks

### 📋 Phase 2: Enhanced Features

- Fuzzy search
- Icon support
- Usage tracking
- Configuration system

### 📋 Phase 3: Advanced Features

- Plugin system
- Custom themes
- Performance optimization
- Desktop actions

### 📋 Phase 4+: Optional Extensions

- X11 support
- Extended plugin ecosystem
- Community contributions

## File Structure Overview

```
native-launcher/
├── Cargo.toml              # Rust project manifest with dependencies
├── README.md               # Main documentation
├── plans.md               # Detailed development roadmap
├── CONTRIBUTING.md        # Contribution guidelines
├── LICENSE                # MIT license
│
├── src/
│   └── main.rs           # Entry point (currently placeholder)
│
├── config/
│   └── default.toml      # Example configuration
│
└── docs/                 # Future documentation
```

## Next Steps for Development

### Immediate (Week 1)

1. Set up logging system with `tracing`
2. Create desktop file parser module
3. Implement file system scanner
4. Write unit tests for parser

### Week 2

1. Initialize GTK4 application
2. Set up gtk4-layer-shell
3. Create basic UI widgets
4. Handle window positioning

### Week 3

1. Implement search logic
2. Connect UI to search
3. Add keyboard navigation
4. Test app launching

## Key Technologies

- **gtk4** (0.9): GUI framework
- **gtk4-layer-shell** (0.4): Wayland overlay support
- **freedesktop-desktop-entry**: Parse .desktop files
- **fuzzy-matcher**: Fuzzy search (Phase 2)
- **tokio**: Async runtime
- **serde/toml**: Configuration
- **tracing**: Logging

## Research References

Studied these similar projects:

- **Rofi**: Layer shell protocol usage, feature set
- **Hyprshell**: Rust architecture, GTK4 patterns, desktop file parsing
- **Wofi**: Wayland integration patterns

## Common Patterns Found

### Desktop File Parsing

```rust
// Standard locations
/usr/share/applications/
/usr/local/share/applications/
~/.local/share/applications/

// Key fields to parse
Name, Exec, Icon, Categories, Keywords, Terminal
```

### Wayland Layer Shell Setup

```rust
window.init_layer_shell();
window.set_layer(Layer::Overlay);
window.set_keyboard_mode(KeyboardMode::Exclusive);
window.set_namespace("native-launcher");
```

### Search Algorithm

1. Parse search query
2. Match against Name, Keywords, Categories
3. Score by relevance (exact > prefix > fuzzy)
4. Boost by usage frequency
5. Sort and display top N results

## Performance Targets

| Metric         | Target | Phase   |
| -------------- | ------ | ------- |
| Cold start     | <100ms | Phase 2 |
| Search latency | <10ms  | Phase 2 |
| Memory usage   | <30MB  | Phase 3 |
| Disk cache     | <5MB   | Phase 2 |

## Testing Strategy

- **Unit tests**: Parser, search, config
- **Integration tests**: Full workflows
- **Benchmarks**: Performance-critical paths
- **Manual tests**: Various compositors and themes

## Getting Help

- Read `plans.md` for detailed architecture
- Check `README.md` for user documentation
- See `CONTRIBUTING.md` for development guidelines
- Open GitHub issues for questions

## Tips

1. **Start small**: Get basic functionality working first
2. **Test often**: Run tests frequently during development
3. **Profile early**: Use benchmarks to catch regressions
4. **Document as you go**: Update docs with code changes
5. **Follow the phases**: Don't skip ahead, build solid foundation

---

**Happy Coding! 🚀**

For the complete development roadmap, see [plans.md](plans.md)
