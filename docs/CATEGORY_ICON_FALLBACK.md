# Category-Based Icon Fallback System

## Overview

Native Launcher now includes an intelligent icon fallback system that automatically resolves icons for desktop entries that don't specify an `Icon=` field. This system uses the freedesktop.org category specification to map applications to appropriate icons.

## Problem

Some desktop entries (`.desktop` files) don't include an `Icon=` field, resulting in generic fallback icons being displayed. This makes the launcher less visually intuitive.

## Solution

The launcher now uses a three-tier icon resolution strategy:

1. **Explicit Icon** - If `Icon=` field exists, use it (existing behavior)
2. **Category-Based Fallback** - Map application categories to appropriate icons
3. **Generic Fallback** - Use default application icon as last resort

## Implementation

### Core Function

```rust
pub fn resolve_icon_with_category_fallback(
    icon_name: Option<&str>,
    categories: &[String],
) -> PathBuf
```

This function is the main entry point used by the applications plugin.

### Category Mapping

The system supports **150+ freedesktop.org categories**, mapping them to appropriate symbolic icons. Examples:

| Category      | Icon Name                  | Visual |
| ------------- | -------------------------- | ------ |
| `Development` | `applications-development` | 🔧     |
| `WebBrowser`  | `web-browser`              | 🌐     |
| `TextEditor`  | `accessories-text-editor`  | 📝     |
| `AudioVideo`  | `applications-multimedia`  | 🎵     |
| `Game`        | `applications-games`       | 🎮     |
| `Graphics`    | `applications-graphics`    | 🖼️     |
| `Network`     | `applications-internet`    | 🌐     |
| `Office`      | `applications-office`      | 📄     |
| `System`      | `applications-system`      | ⚙️     |
| `Utility`     | `applications-utilities`   | 🔨     |

### Priority Handling

When multiple categories are present, the **first matching category wins**. This allows more specific categories to take precedence:

```
Categories=WebBrowser;Network;
→ Resolves to "web-browser" (more specific than "applications-internet")
```

## Technical Details

### Performance

- **Zero overhead** for entries with explicit icons
- **Cached resolution** - Icon lookups are cached in memory
- **Local-only** - No network requests or external APIs
- **Fast lookup** - Simple string matching in category list

### Open Source Compliance

- **100% local** - No external services or APIs
- **Freedesktop.org standard** - Based on official specification
- **GTK icon themes** - Uses system-installed icon themes
- **No proprietary dependencies**

### Icon Theme Support

The system respects your GTK icon theme settings and searches in:

1. Current GTK theme (e.g., Adwaita, Papirus, etc.)
2. Hicolor theme (fallback)
3. Custom user icon directories

## Category Reference

### Main Categories (Most Common)

- **AudioVideo** - Multimedia applications
- **Audio** - Audio-specific tools
- **Video** - Video players/editors
- **Development** - IDEs, debuggers, compilers
- **Education** - Educational software
- **Game** - Games and entertainment
- **Graphics** - Image editors, viewers
- **Network** - Network/internet applications
- **Office** - Office productivity
- **Science** - Scientific software
- **Settings** - System settings/preferences
- **System** - System tools
- **Utility** - General utilities

### Specialized Categories

Over 130 additional categories are supported, including:

- **Development**: IDE, Debugger, RevisionControl, Profiling
- **Office**: WordProcessor, Spreadsheet, Presentation, Database
- **Graphics**: 2DGraphics, 3DGraphics, VectorGraphics, Photography
- **Internet**: WebBrowser, Email, Chat, Feed, FileTransfer, P2P
- **Multimedia**: Player, Recorder, DiscBurning, AudioVideoEditing
- **Games**: ActionGame, StrategyGame, Simulation, RolePlaying
- **Science**: Astronomy, Biology, Chemistry, Physics, Math

See `src/utils/icons.rs` for the complete mapping table.

## Usage

### Automatic (Applications Plugin)

The applications plugin automatically uses category-based fallback:

```rust
use crate::utils::icons::resolve_icon_with_category_fallback;

let icon_path = resolve_icon_with_category_fallback(
    entry.icon.as_deref(),  // Option<&str>
    &entry.categories,       // &[String]
);
```

### Manual Usage

For custom plugins:

```rust
use crate::utils::icons::category_to_icon;

// Get icon name from categories
if let Some(icon_name) = category_to_icon(&["Development", "IDE"]) {
    println!("Icon: {}", icon_name); // "applications-development"
}
```

## Testing

The system includes 11 comprehensive tests:

- Category-to-icon mapping validation
- Priority handling (specific vs general)
- Fallback chain testing
- Edge cases (empty categories, unknown categories)

Run tests:

```bash
cargo test --lib icons
```

## Examples

### Before (Without Category Fallback)

```
App: "MyCustomIDE"
Icon: (none specified)
Result: Generic app icon ⚪
```

### After (With Category Fallback)

```
App: "MyCustomIDE"
Icon: (none specified)
Categories: ["Development", "IDE"]
Result: Development icon 🔧
```

## Benefits

1. **Better UX** - All apps now have appropriate icons
2. **No Configuration** - Works automatically with existing `.desktop` files
3. **Standards Compliant** - Uses freedesktop.org category specification
4. **Performance** - Minimal overhead with caching
5. **Privacy** - No external API calls or tracking
6. **Open Source** - 100% local, transparent implementation

## Future Enhancements

Possible improvements:

- Custom category → icon mapping in config file
- User-defined icon overrides per app
- Icon search path customization
- Theme-specific category mappings

## References

- [Freedesktop.org Menu Specification](https://specifications.freedesktop.org/menu-spec/latest/)
- [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [Icon Theme Specification](https://specifications.freedesktop.org/icon-theme-spec/latest/)
