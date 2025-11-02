# Native Launcher Changelog

## [Unreleased]

## [0.2.0] - 2025-11-02

## [0.2.0] - 2025-11-02

### 🎉 Major Features

#### Session Management & Window Control

- **Session Switcher Plugin** (`@switch`, `@sw`) - 11 tests

  - Auto-detects Hyprland/Sway compositor
  - Aggregates running windows from compositor
  - Scans VS Code workspaces from storage
  - 3-second cache with lazy loading
  - Focus windows instantly via compositor commands

- **Window Management Plugin** (`@wm`, `@window`) - 11 tests
  - Move windows to workspaces 1-5
  - Center, fullscreen, floating toggles
  - Pin windows to all workspaces
  - Window movement (left/right/up/down)
  - Close window action
  - Works with Hyprland and Sway

#### Git & Development Tools

- **Git Projects Plugin** (`@git`, `@repo`) - 7 tests
  - Scans ~/code, ~/projects, ~/dev, ~/workspace (depth 2)
  - Detects git repositories and current branch
  - Opens in default editor (VISUAL, EDITOR, or common editors)
  - Lazy loading with 10ms budget
  - Smart editor detection (VS Code, Sublime, Atom, Neovim, etc.)

#### Enhanced User Experience

- **Inline Result Actions** - Keyboard shortcuts for quick operations

  - `Alt+Enter`: Open containing folder
  - `Ctrl+Enter`: Copy path to clipboard (doesn't close window)
  - Works in files and recent plugins
  - Updated keyboard hints footer

- **Icon Badges** - Visual indicators on result rows

  - Terminal badge (🖥️) for Terminal=true apps and SSH
  - Web badge (🌐) for web search results
  - Document badge (📄) for file results
  - Folder badge (📁) for directories and git repos
  - 16px symbolic icons with opacity transitions

- **Category-Based Icon Fallback** - 9 new tests
  - Intelligent icon resolution for apps without Icon= field
  - 150+ freedesktop.org categories mapped
  - Three-tier fallback: explicit → category-based → generic
  - 100% local, privacy-respecting (no external APIs)
  - Examples: Development→🔧, WebBrowser→🌐, TextEditor→📝

#### UI Customization

- **Density Toggle** - Adjust UI spacing

  - Compact mode: Tighter spacing (10px padding)
  - Comfortable mode: Default spacious layout (14px padding)
  - CSS-based instant switching
  - Config: `ui.density = "compact" | "comfortable"`

- **Theme Accent Variants** - 7 color options
  - Coral (default), Teal, Violet, Blue, Green, Orange, Pink
  - CSS variables for instant switching
  - Config: `ui.accent = "color"`
  - No relaunch required

#### Advanced Features

- **Screenshot Annotate Mode** (`@ss annotate`) - 6 tests

  - Swappy integration for annotation
  - Full screen, window, and area annotation modes
  - Pipeline: screenshot → annotate → copy to clipboard
  - Graceful degradation when swappy not installed
  - Keywords: "annotate", "edit", "draw", "markup"

- **Recent Documents Plugin** (`@recent`, `@r`) - 8 tests
  - Parses ~/.local/share/recently-used.xbel
  - Top 200 recent entries sorted by modified time
  - Human-readable timestamps ("5m ago", "3d ago")
  - File categorization (Text, Image, Video, PDF, etc.)
  - Opens with default handler (xdg-open)

### 🚀 Performance Improvements

- **Usage Learning v2** - Time-decay system (9 tests)

  - Hour-of-day boost (up to 30% for frequently used hours)
  - Launch history tracking (last 100 launches)
  - Exponential time decay (7-day half-life)
  - Precomputed scores for O(1) runtime
  - Smart pattern recognition for daily workflows

- **Enhanced Fuzzy Search** - Phase 2 (8 new tests)

  - Acronym matching (vsc → Visual Studio Code)
  - Word boundary detection prioritization
  - Case-sensitivity bonus (+2000 for exact case)
  - Exec field matching (google-chrome → Chrome)
  - Minimum score threshold filtering
  - Performance optimizations with caching

- **Compiler Optimizations**
  - Fat LTO for aggressive cross-crate optimization
  - panic = "abort" for smaller binary
  - Inline optimization hints for hot paths
  - Lazy loading with OnceLock patterns
  - Memory pre-allocation in hot paths

### 📊 Statistics

- **150 tests passing** (up from 118)
- **32 new tests** across all new features
- **Startup time**: 34-35ms (maintained)
- **Binary size**: 7.8MB (optimized with fat LTO)
- **Build time**: 59s with full optimizations

### 🔧 Technical Improvements

- All new plugins follow performance-first philosophy
- Comprehensive test coverage for new features
- Detailed documentation for each feature
- Backward-compatible configuration changes
- Zero breaking changes to existing APIs

### 📚 Documentation

- Added CATEGORY_ICON_FALLBACK.md
- Updated plugin development guide
- Enhanced performance documentation
- Added inline actions documentation
- Session switcher implementation guide

### Changed

- Applications plugin now uses category-based icon fallback
- Results list renders badge icons in title row
- Icon resolution prioritizes category mapping over generic fallback
- Keyboard hints updated to show Alt+Enter and Ctrl+Enter

### Fixed

- Icon resolution for desktop entries without Icon= field
- Missing badge_icon field in all plugins
- Test assertions for icon fallback system

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
