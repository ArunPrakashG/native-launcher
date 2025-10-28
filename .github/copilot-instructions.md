# Native Launcher - AI Agent Instructions

## Project Overview

Native Launcher is a **Wayland-native GTK4 application launcher** for Linux, written in Rust. A modern, fast launcher with a clean design aesthetic featuring coral accents on a dark charcoal background. The project prioritizes speed (<100ms startup) and keyboard-driven workflows.

## 🚨 CRITICAL: Performance First Philosophy

**Performance is the #1 priority. Features are negotiable, speed is not.**

All decisions must prioritize performance:

- ❌ Don't add features that degrade startup time or search latency
- ❌ Don't add dependencies without profiling their impact
- ❌ Don't add visual effects that block the UI thread
- ✅ Profile before and after every significant change
- ✅ Question any feature that adds >10ms to critical paths
- ✅ Cache aggressively, compute lazily, render on-demand

**Performance Targets** (hard limits):

- Startup: <100ms cold start (target: <50ms)
- Search: <10ms for 500 apps (target: <5ms)
- Memory: <30MB idle (target: <20MB)
- UI latency: <16ms (60fps, target: 120fps)

**Measure everything**:

```bash
# Startup time
time ./target/release/native-launcher

# Search benchmarks
cargo bench

# Memory profiling
/usr/bin/time -v ./target/release/native-launcher
```

If a feature conflicts with performance, **cut the feature**.

## Architecture

### Core Components (src/)

```
main.rs          → GTK4 app initialization, event loop, keyboard handlers
lib.rs           → Public module exports for testing
desktop/         → .desktop file parsing (freedesktop standard)
  ├─ entry.rs    → DesktopEntry/DesktopAction models with inline action parsing
  ├─ scanner.rs  → Scans /usr/share/applications and ~/.local/share/applications
search/          → Fuzzy search with nucleo/fuzzy-matcher
  └─ mod.rs      → SearchEngine with substring matching (fuzzy coming in Phase 2)
ui/              → GTK4 widgets following modern design principles
  ├─ window.rs   → LauncherWindow (gtk4-layer-shell overlay)
  ├─ search_entry.rs → SearchWidget (input field)
  ├─ results_list.rs → ResultsList (inline action display with ListItem enum)
  └─ style.css   → Design system: #1C1C1E bg, #FF6363 coral accents
utils/           → Execute commands, icon loading
```

### Data Flow

1. **Startup**: `DesktopScanner::scan()` → parses all .desktop files → `SearchEngine::new(entries)`
2. **Search**: User types → `search_widget.entry.connect_changed()` → `SearchEngine::search()` → `results_list.update_results()`
3. **Launch**: Enter key → `results_list.get_selected_command()` → `execute_command(exec, terminal)`

### Key Pattern: Inline Actions Display

**Recent refactor**: Actions now display inline under parent apps (no separate mode). See `src/ui/results_list.rs`:

```rust
enum ListItem {
    App { entry: DesktopEntry, index: usize },
    Action { action: DesktopAction, parent_entry: DesktopEntry, action_index: usize }
}
```

This creates a flat list where each app is followed immediately by its actions (if any). Actions are visually distinguished with:

- 24px left indentation
- Coral text color (#ff6363)
- Left border that highlights on selection

**Why**: Eliminates mode switching (no Right/Left arrow navigation), faster workflow, matches UX of macOS Spotlight and Windows Jump Lists.

## Development Workflow

### Build & Run

```bash
# Debug build with logging
RUST_LOG=debug cargo run

# Release build (fast)
cargo build --release && ./target/release/native-launcher

# Run tests (15+ test cases in tests/desktop_tests.rs)
cargo test

# Benchmarks (search performance in benches/search_benchmark.rs)
cargo bench
```

### Testing Desktop Actions

Apps with built-in actions (test inline display):

- Firefox: New Window, Private Window, Profile Manager
- Chrome/Chromium: New Window, Incognito Window
- VS Code: New Window, New Empty Window

### CSS Hot Reload

Edit `src/ui/style.css` → Rebuild → Changes apply. No live reload (GTK limitation).

## Project-Specific Conventions

### 1. GTK4 State Management: Rc<RefCell<T>>

GTK signals require `'static` lifetime. We use `Rc<RefCell<T>>` for shared mutable state:

```rust
// src/main.rs
let search_engine = Rc::new(RefCell::new(SearchEngine::new(entries)));
search_widget.entry.connect_changed(move |entry| {
    let results = search_engine.borrow().search(&query, 10);
    // ...
});
```

**Pattern**: Clone `Rc` before move into closure, borrow mutably when accessing.

### 2. Desktop Actions Parsing

`freedesktop-desktop-entry` crate doesn't expose `[Desktop Action ...]` sections. We manually parse:

```rust
// src/desktop/entry.rs
fn parse_actions(entry: &FdEntry, path: &PathBuf) -> Result<Vec<DesktopAction>> {
    let contents = std::fs::read_to_string(path)?;
    // Manual [Desktop Action ID] section parsing...
}
```

**Why**: Spec compliance. Desktop actions enable context menus (right-click actions).

### 3. Modern Launcher Design (Inspired by Contemporary Launchers)

### 3. Design System

**Colors** (see `src/ui/style.css`):

- Background: `#1C1C1E` (charcoal)
- Accent: `#FF6363` (coral/red)
- Text: `#EBEBF5` (off-white)
- Transitions: `0.15s cubic-bezier(0.4, 0, 0.2, 1)`

**Rules**:

- Rounded corners (16px window, 10px inputs)
- Minimal borders, subtle shadows
- Coral highlights for selected items
- No heavy animations (performance first)

### 4. Terminal App Handling

Apps with `Terminal=true` in .desktop files need terminal wrapper:

```rust
// src/utils/exec.rs
if terminal {
    Command::new("alacritty").args(["-e", "sh", "-c", exec]).spawn()?;
}
```

**Auto-detection strategy** (planned):

1. Check `$TERMINAL` env var (user preference)
2. Check `x-terminal-emulator` symlink (Debian/Ubuntu)
3. Check common terminals in order: alacritty, kitty, wezterm, foot, gnome-terminal, konsole
4. Fall back to xterm if nothing found
5. Cache detected terminal to avoid repeated detection

### 5. Error Handling

- Use `anyhow::Result` for functions that can fail
- Use `tracing::error!` for runtime errors (don't panic in GTK callbacks)
- Desktop files with missing fields are skipped (logged at DEBUG level)

## Phase Status (See plans.md)

- ✅ **Phase 1 MVP**: Core launcher working (search, launch, keyboard nav)
- 🔄 **Phase 2**: Fuzzy search, icons, usage tracking (in progress)
- 📋 **Phase 3**: Desktop actions (✅ inline display complete), plugin system
- 📋 **Phase 4**: X11 support (optional)

## Integration Points

### Wayland Compositors

Uses `gtk4-layer-shell` for overlay window. Requires layer shell protocol support:

- ✅ Sway, Hyprland, River (wlroots-based)
- ✅ KDE Plasma (Wayland)
- ✅ GNOME (Mutter)

**Hotkey setup**: Compositor-specific. User must configure `Super+Space` → launch native-launcher. No global hotkey in Wayland (security model).

### Desktop File Spec

Follows freedesktop.org standard:

- Parses: Name, GenericName, Exec, Icon, Categories, Keywords, Terminal, Actions
- Supports Exec field codes: `%f` (file), `%u` (URL) - currently stripped
- Filters: `NoDisplay=true`, `Hidden=true` entries excluded

### Icon Themes

Uses `freedesktop-icons` crate (Phase 2). Implementation approach:

**Icon Resolution Chain**:

1. **Desktop entry icon**: Check `Icon=` field from .desktop file
2. **Absolute paths**: If starts with `/`, use directly
3. **Icon name lookup**: Search in XDG icon theme directories:
   - `~/.local/share/icons` (user icons)
   - `/usr/share/icons` (system icons)
   - `~/.icons` (legacy)
4. **Theme hierarchy**: Follow theme inheritance (`index.theme` files)
5. **Size matching**: Find icon closest to target size (48x48 default)
6. **Fallback chain**:
   - Themed icon → hicolor (default theme) → generic icon → hardcoded fallback

**Implementation pattern**:

```rust
// src/utils/icons.rs (planned)
pub fn resolve_icon(icon_name: &str, size: u32, theme: &str) -> Option<PathBuf> {
    // 1. Check if absolute path
    if icon_name.starts_with('/') && Path::new(icon_name).exists() {
        return Some(PathBuf::from(icon_name));
    }

    // 2. Use freedesktop-icons crate for theme lookup
    let lookup = freedesktop_icons::lookup(icon_name)
        .with_size(size)
        .with_theme(theme)
        .with_cache();

    // 3. Try multiple extensions
    for ext in &["svg", "png", "xpm"] {
        if let Some(path) = lookup.find_with_extension(ext) {
            return Some(path);
        }
    }

    None
}
```

**Caching strategy**: Build icon path cache on startup, refresh when theme changes (monitor `~/.config/gtk-4.0/settings.ini`).

**GTK4 Integration**:

```rust
// src/ui/results_list.rs
let image = gtk4::Image::from_file(&icon_path);
image.set_pixel_size(48);
row.prepend(&image);
```

**Performance notes**:

- Icon lookup: ~1-5ms per icon (cached)
- Load on-demand as results scroll into view
- Use `gtk4::IconTheme::for_display()` for system theme detection

## Common Tasks

### Add a New UI Widget

1. Create `src/ui/new_widget.rs`
2. Implement struct with GTK container (e.g., `pub container: GtkBox`)
3. Add CSS classes in `src/ui/style.css` (follow design system colors)
4. Export in `src/ui/mod.rs`
5. Instantiate in `src/main.rs` build_ui()

### Add Desktop Entry Field

1. Update `DesktopEntry` struct in `src/desktop/entry.rs`
2. Parse field in `DesktopEntry::from_file()` using `freedesktop_desktop_entry` API
3. Add to search scoring in `src/desktop/entry.rs::match_score()`
4. Update tests in `tests/desktop_tests.rs`

### Modify Search Algorithm

Edit `src/search/mod.rs`. Current implementation is substring matching. Phase 2 will integrate `nucleo` for fuzzy matching with:

- Multi-field search (name, generic_name, keywords)
- Relevance scoring (exact > prefix > fuzzy)
- Usage-based boosting (frequency tracking)

## Testing Notes

- **Unit tests**: `tests/desktop_tests.rs` covers parser, scanner, search
- **Manual testing**: Launch on Wayland compositor, test with apps like Firefox (has actions)
- **Performance**: Use `cargo bench` for search latency (target <10ms for 500 apps)
- **No integration tests yet**: Would require headless GTK setup

## Documentation

- `plans.md`: Phased development roadmap (read first!)
- `README.md`: User-facing quick start
- `docs/INLINE_ACTIONS_UPDATE.md`: Recent refactor details
- `docs/VISUAL_EXAMPLE.md`: UI mockups for inline actions
- `CONTRIBUTING.md`: Contribution guidelines
- Additional documentation planned as wiki submodule

## Critical Files for Context

When working on features, read these first:

- `src/main.rs` (150 lines): Event loop, keyboard handlers
- `src/desktop/entry.rs` (250 lines): Core data model
- `src/ui/results_list.rs` (240 lines): Complex widget with inline actions
- `src/ui/style.css` (240 lines): Complete visual theme
- `plans.md`: Architecture and phase goals

## Anti-Patterns to Avoid

❌ Don't use unwrap() in GTK callbacks (use `if let` or log errors)  
❌ Don't add animations >0.2s (breaks "fast" feel)  
❌ Don't diverge from design system colors without discussion  
❌ Don't use sync file I/O in main thread (blocks UI)  
❌ Don't parse .desktop files on every search (use cached entries)  
❌ **Don't add features without profiling impact first**  
❌ **Don't import large dependencies without measuring binary size increase**  
❌ **Don't allocate in hot paths (search loop, render loop)**  
❌ **Don't add "nice to have" features that hurt startup time**

## Questions to Ask

- **"Does this maintain <100ms startup time?"** (profile with `time ./target/release/native-launcher`)
- **"Does this maintain <10ms search latency?"** (run `cargo bench` before and after)
- **"Have I profiled this change?"** (use `cargo flamegraph` or `perf`)
- "Does this follow the design system?" (check style.css colors)
- "Does this work on Sway/Hyprland?" (test on wlroots compositor)
- "Does this handle missing .desktop fields gracefully?" (log and skip, don't crash)
- **"Can this be done lazily/cached instead?"** (defer work until needed)
- **"Is this feature worth the performance cost?"** (if no, cut the feature)
