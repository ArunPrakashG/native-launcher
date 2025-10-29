# Fixed Window Size Issue - Oct 29, 2025

## Problem

The application window was automatically expanding in all directions when:

- User typed into the search box
- Content/results became larger
- Text overflowed

This created an unpredictable, jarring user experience where the window would grow dynamically.

## Root Cause

Multiple GTK properties were working together to allow window expansion:

1. **Window resizable by default**: `ApplicationWindow` is resizable unless explicitly disabled
2. **hexpand/vexpand on widgets**: Child widgets were requesting expansion space
3. **No size constraints**: Only `default_width`/`default_height` set (which are just hints, not constraints)

### Specific Issues

```rust
// search_entry.rs - Entry was requesting horizontal expansion
let entry = Entry::builder()
    .hexpand(true)  // ❌ CAUSED HORIZONTAL EXPANSION
    .build();

// results_list.rs - ScrolledWindow was requesting both expansions
let container = ScrolledWindow::builder()
    .hexpand(true)  // ❌ CAUSED HORIZONTAL EXPANSION
    .vexpand(true)  // ❌ CAUSED VERTICAL EXPANSION
    .build();

// main.rs - Window allowed resizing
launcher_window.window.set_default_width(700);   // Just a hint
launcher_window.window.set_default_height(550);  // Just a hint
// No set_resizable(false) ❌
```

## Solution

### 1. Disable Window Resizing

**File**: `src/main.rs`

```rust
// Apply window config - use FIXED size to prevent expansion
launcher_window.window.set_default_width(config.window.width);
launcher_window.window.set_default_height(config.window.height);

// CRITICAL: Prevent window from resizing beyond default size
launcher_window.window.set_resizable(false);
```

### 2. Remove Widget Expansion Properties

**File**: `src/ui/search_entry.rs`

```rust
// BEFORE
let entry = Entry::builder()
    .placeholder_text("Search applications...")
    .hexpand(true)  // ❌ Removed
    .build();

// AFTER
let entry = Entry::builder()
    .placeholder_text("Search applications...")
    .build(); // ✅ No expansion request
```

**File**: `src/ui/results_list.rs`

```rust
// BEFORE
let container = ScrolledWindow::builder()
    .hexpand(true)  // ❌ Removed
    .vexpand(true)  // ❌ Removed
    .child(&list)
    .has_frame(false)
    .build();

// AFTER
let container = ScrolledWindow::builder()
    .child(&list)
    .has_frame(false)
    .build(); // ✅ No expansion request
```

## How GTK Window Sizing Works

### Window Size Properties

1. **`default_width`/`default_height`**: Initial size hints (can be overridden)
2. **`set_resizable(false)`**: Prevents user and programmatic resizing
3. **Child `hexpand`/`vexpand`**: Widgets request extra space from parent

### Expansion Hierarchy

```
ApplicationWindow (resizable=true)
    ↓ (allows growth)
Main GtkBox
    ↓ (distributes space)
ScrolledWindow (hexpand=true, vexpand=true)
    ↓ (requests expansion)
Window grows to accommodate! ❌
```

### Fixed Size Hierarchy

```
ApplicationWindow (resizable=false) ← LOCKED SIZE
    ↓ (fixed container)
Main GtkBox
    ↓ (constrained space)
ScrolledWindow (no expand flags)
    ↓ (uses available space only)
Window stays fixed! ✅
```

## Testing

### Before Fix

1. Open launcher (700×550)
2. Type "code" → Window expands horizontally ❌
3. Long file paths appear → Window expands ❌
4. Unpredictable sizing

### After Fix

1. Open launcher (700×550)
2. Type anything → Window stays 700×550 ✅
3. Any content → Window stays 700×550 ✅
4. Predictable, fixed sizing

## Files Modified

- `src/main.rs`: Added `set_resizable(false)`
- `src/ui/search_entry.rs`: Removed `hexpand(true)` from Entry
- `src/ui/results_list.rs`: Removed `hexpand(true)` and `vexpand(true)` from ScrolledWindow

## Benefits

### User Experience

- **Predictable**: Window always same size
- **Stable**: No jarring expansions
- **Professional**: Matches behavior of Spotlight, Raycast, Alfred

### Performance

- **No resize calculations**: Window size is fixed
- **No layout recalculations**: Children work within fixed bounds
- **Simpler rendering**: GTK doesn't need to handle dynamic sizing

### Consistency

- Works with our fixed min-height CSS (400px)
- Matches the "always visible, fixed height" design decision
- Complements the removal of empty_state_on_launch logic

## Design Philosophy

Modern launchers follow a fixed-size paradigm:

| Launcher            | Window Behavior                      |
| ------------------- | ------------------------------------ |
| macOS Spotlight     | Fixed width, slight height variation |
| Raycast             | Fixed size                           |
| Alfred              | Fixed width, slight height variation |
| PowerToys Run       | Fixed size                           |
| **Native Launcher** | **Fixed size** ✅                    |

This creates a predictable, muscle-memory-friendly experience where users always know where the window will be and how large it will be.

## Additional Notes

### CSS Still Provides Visual Boundaries

Even with fixed window size, CSS provides internal structure:

- `min-height: 400px` on scrolledwindow ensures minimum content area
- Scrolling handles overflow content
- No need for dynamic window sizing

### Future Considerations

If we ever want to allow user-configurable sizes:

- Keep `set_resizable(false)`
- Allow size changes only via config file
- Restart required for new size (intentional friction prevents accidental changes)
