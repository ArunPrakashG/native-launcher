# Inline Actions Display - Update Summary

## Overview

Modified the desktop actions feature to display actions directly inline under each application entry, rather than requiring navigation to a separate view. This provides immediate visibility of available actions similar to macOS Spotlight or Windows Jump Lists.

## Changes Made

### 1. `src/ui/results_list.rs` - Complete Refactor

- **Removed**: `DisplayMode` enum (Results/Actions modes)
- **Added**: `ListItem` enum with two variants:
  - `App { entry: DesktopEntry, index: usize }` - Main application entries
  - `Action { action: DesktopAction, parent_entry: DesktopEntry, action_index: usize }` - Action sub-items

#### Key Method Changes:

- **`update_results()`**: Now builds a flat list of items where each app is followed immediately by its actions (if any)
- **`get_selected_command()`**: Simplified to check current item type and return appropriate command
- **`show_actions()`, `show_results()`, `is_action_mode()`**: Kept for compatibility but return `false` (no-ops)

#### Display Logic:

```rust
// Before: Manual navigation between Results and Actions views
App 1
App 2  (press Right to see actions)
App 3

// After: Inline display with visual hierarchy
App 1
  ↳ Action 1A (highlighted in coral)
  ↳ Action 1B (highlighted in coral)
App 2 (no actions)
App 3
  ↳ Action 3A (highlighted in coral)
```

### 2. `src/ui/style.css` - Updated Action Styling

- **Removed**: `.action-indicator` (the "→ N actions" label)
- **Removed**: `.action-header` (header row when in action mode)
- **Updated**: `.action-name` - Now uses `var(--raycast-primary)` (coral #ff6363) for high visibility
- **Enhanced**: Indented action rows with left border that highlights on hover/selection

#### Visual Design:

- Action names are highlighted in coral by default
- Indented 24px with a subtle left border
- Border color transitions:
  - Default: `var(--raycast-text-quaternary)` (dim)
  - Hover: `var(--raycast-primary)` (coral)
  - Selected: `rgba(255, 255, 255, 0.8)` (bright white)

### 3. `src/main.rs` - No Changes Required

The existing keyboard handlers still call `show_actions()`, `show_results()`, etc., but these now no-op since actions are always visible inline.

## User Experience

### Before (Separate Views):

1. Search for "firefox"
2. See "Firefox" with "→ 3 actions" indicator
3. Press Right arrow to enter action mode
4. See actions listed
5. Press Left arrow to go back

### After (Inline Display):

1. Search for "firefox"
2. See:
   ```
   Firefox
     New Window (coral)
     New Private Window (coral)
     Profile Manager (coral)
   ```
3. Use Up/Down arrows to navigate directly to any action
4. Press Enter to launch app or action

## Benefits

1. **Immediate Visibility**: Users can see all available actions without extra navigation
2. **Faster Navigation**: Direct selection with Up/Down arrows, no mode switching
3. **Visual Hierarchy**: Clear parent-child relationship with indentation and color
4. **Consistent UX**: Similar to macOS Spotlight's Quick Actions and Windows Jump Lists
5. **Less Complexity**: Removed mode management code, simpler state

## Technical Notes

- Actions are expanded for all apps in search results automatically
- Each action retains reference to its parent entry for correct terminal mode detection
- The flat list structure allows GTK ListBox to handle selection naturally
- Color highlighting ensures actions stand out while maintaining Raycast aesthetic

## Example Desktop Entry

Firefox's `.desktop` file might include:

```ini
[Desktop Entry]
Name=Firefox
Exec=firefox

[Desktop Action new-window]
Name=New Window
Exec=firefox --new-window

[Desktop Action new-private-window]
Name=New Private Window
Exec=firefox --private-window
```

This will now display as:

```
Firefox
  New Window (coral, indented)
  New Private Window (coral, indented)
```

## Testing

To test with apps that have actions:

```bash
# Run the launcher
cargo run --release

# Search for apps with actions:
# - Firefox (New Window, Private Window, etc.)
# - Chrome/Chromium (New Window, Incognito, etc.)
# - VS Code (New Window, etc.)
# - Many other GUI applications
```

Use Up/Down arrows to navigate through apps and their actions, then press Enter to launch.

## Future Enhancements

- Add icons for actions (using the `icon` field in DesktopAction)
- Add keyboard shortcuts to actions (e.g., Cmd+N for New Window)
- Collapsible action groups for apps with many actions
- Action filtering/search within app context
