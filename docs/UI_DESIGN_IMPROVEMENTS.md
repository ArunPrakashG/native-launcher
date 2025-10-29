# UI Design Improvements - Oct 29, 2025

## Issues Fixed

### 1. ✅ Inconsistent Border Radius (Sharp Inner Viewport)

**Problem**: The results list viewport had sharp edges while the search input and window had rounded corners, creating visual inconsistency.

**Root Cause**:

- ScrolledWindow and ListBox lacked border-radius styling
- No unified design language for corner roundness

**Solution**:

- Added CSS variables for consistent border radius across all UI elements
- Applied rounded corners to scrolled window and viewport
- Added subtle background to results container for visual cohesion

**CSS Variables Added**:

```css
:root {
  --nl-radius-window: 20px; /* Main window corners */
  --nl-radius-large: 16px; /* Search input, results container */
  --nl-radius-medium: 14px; /* List items */
  --nl-radius-small: 10px; /* Icons, buttons */
  --nl-radius-tiny: 8px; /* Small UI elements */
}
```

**Changes Made**:

```css
/* Scrolled window now has rounded corners */
scrolledwindow {
  border-radius: var(--nl-radius-large);
  background-color: var(--nl-bg-secondary);
  padding: 4px;
}

/* Viewport inherits rounded corners */
scrolledwindow > viewport {
  border-radius: var(--nl-radius-large);
  overflow: visible; /* Prevent clipping */
}

/* List box matches the design */
listbox {
  border-radius: var(--nl-radius-large);
  padding: 8px 0; /* Prevent top/bottom clipping */
}
```

### 2. ✅ Viewport Clipping/Cutting Issue

**Problem**: When typing the second letter, the list items would get cut off at the edges.

**Root Cause**:

- ScrolledWindow had implicit `overflow: hidden` for border-radius
- No padding on listbox, causing items to touch edges
- Viewport clipping content due to border-radius

**Solution**:

1. **Set `overflow: visible` on scrolledwindow and viewport** - Prevents clipping
2. **Added padding to scrolledwindow (4px)** - Creates space between edge and content
3. **Added padding to listbox (8px vertical)** - Prevents items from touching top/bottom
4. **Set proper scrolling policy in Rust** - `PolicyType::Never` for horizontal, `PolicyType::Automatic` for vertical
5. **Disabled frame on ScrolledWindow** - Cleaner look with custom styling

**Rust Changes** (`src/ui/results_list.rs`):

```rust
let container = ScrolledWindow::builder()
    .hexpand(true)
    .vexpand(true)
    .child(&list)
    .has_frame(false) // No frame for clean rounded corners
    .build();

// Ensure scrolling policies are set correctly
container.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
```

### 3. ✅ Theme Customization for Border Radius

**Problem**: Themes couldn't customize the roundness of UI elements.

**Solution**: All border-radius values now use CSS variables that can be overridden per theme.

**Example Custom Theme**:

```css
/* Sharp corners theme */
:root {
  --nl-radius-window: 0px;
  --nl-radius-large: 0px;
  --nl-radius-medium: 0px;
  --nl-radius-small: 0px;
  --nl-radius-tiny: 0px;
}
```

```css
/* Extra rounded theme */
:root {
  --nl-radius-window: 30px;
  --nl-radius-large: 24px;
  --nl-radius-medium: 20px;
  --nl-radius-small: 16px;
  --nl-radius-tiny: 12px;
}
```

## Visual Consistency Improvements

### Updated Elements to Use CSS Variables

All UI elements now use the unified border-radius system:

| Element           | Variable             | Default | Purpose              |
| ----------------- | -------------------- | ------- | -------------------- |
| Main window       | `--nl-radius-window` | 20px    | Outer container      |
| Search entry      | `--nl-radius-large`  | 16px    | Input field          |
| Results container | `--nl-radius-large`  | 16px    | Scrolled window      |
| List items        | `--nl-radius-medium` | 14px    | Individual rows      |
| Icons             | `--nl-radius-small`  | 10px    | App icons            |
| Buttons           | `--nl-radius-small`  | 10px    | Interactive elements |
| Keyboard hints    | `--nl-radius-tiny`   | 8px     | Small labels         |
| Action borders    | `--nl-radius-tiny`   | 8px     | Inline actions       |

### Before vs After

**Before**:

- Window: 20px rounded
- Search input: 16px rounded
- Results list: **0px (sharp edges)** ❌
- List items: 14px rounded
- Inconsistent visual hierarchy

**After**:

- Window: 20px rounded (var)
- Search input: 16px rounded (var)
- Results container: **16px rounded (var)** ✅
- Viewport: **16px rounded (var)** ✅
- List items: 14px rounded (var)
- Icons: 10px rounded (var)
- Unified, customizable design language

## Design Language Benefits

### 1. **Consistency**

- All rounded corners follow the same visual hierarchy
- Larger elements = more roundness
- Smaller elements = less roundness

### 2. **Customizability**

- Themes can adjust all border radii at once
- Support for sharp, moderate, or extra-rounded styles
- No need to edit individual selectors

### 3. **Maintainability**

- Single source of truth for each radius level
- Easy to adjust globally
- Future-proof for new UI elements

### 4. **Performance**

- CSS variables are GPU-accelerated
- No runtime recalculation
- Smooth transitions between states

## Files Modified

### CSS Files

- `src/ui/style.css` - Added CSS variables, updated all border-radius usage
- `themes/dark.css` - Added border-radius variables to base theme

### Rust Files

- `src/ui/results_list.rs` - Fixed ScrolledWindow configuration

### Documentation

- `themes/README.md` - Added customization guide with examples
- `docs/UI_DESIGN_IMPROVEMENTS.md` - This document

## Testing

To verify the fixes:

1. **Visual consistency**: All rounded elements should have smooth, consistent corners
2. **No clipping**: Type in search box, results should not be cut off at edges
3. **Theme customization**: Override `--nl-radius-*` variables in custom theme
4. **Scrolling**: Vertical scroll should work smoothly without clipping

## Screenshots

Screenshots taken with `grim` are available:

- `screenshot-current.png` - Initial state
- `screenshot-fullscreen.png` - Full screen with launcher visible

## Future Improvements

1. **Shadow customization** - Add CSS variables for shadow blur/spread
2. **Spacing system** - Unified padding/margin variables
3. **Animation curves** - More easing function options
4. **Color palette generator** - Tool to generate theme from single color
