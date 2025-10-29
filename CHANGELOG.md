# Native Launcher Changelog

## [Unreleased]

### Added

- Dynamic plugin timing based on actual performance measurements
- Empty state on launch (Spotlight-style compact window)
- Incremental search with fast/slow plugin separation
- Daemon mode for instant appearance
- Interactive installation with theme selection
- Self-updater system with background version checks

### Changed

- Window dynamically resizes based on search input
- Plugins categorized by performance (< 10ms = fast)
- Performance metrics logged every 10 searches

### Fixed

- All compilation warnings resolved
- Type mismatches in daemon mode

## [0.1.2] - 2025-10-29

## [0.1.1] - 2025-10-28

## [0.1.0] - 2025-10-28

### Added

- Initial release
- GTK4 + Wayland native launcher
- Plugin system (Apps, Calculator, Files, SSH, Web Search, etc.)
- Advanced calculator with units, currency, time, timezone
- File indexing with plocate/fd/find
- Usage-based ranking
- Fuzzy search with nucleo
- Desktop actions support
- Keyboard-driven navigation
- Modern UI with coral accents (#FF6363) on dark theme
- Configuration system with TOML
- Icon support with theme detection
- Compositor integration (Hyprland, Sway, KDE, GNOME)

### Performance

- <100ms startup time (target: <50ms)
- <10ms search latency for 500 apps
- <30MB memory usage
- Debounced search (150ms)
- Smart triggering to skip expensive operations
