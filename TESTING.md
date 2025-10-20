# Testing Guide

## Fixed Issues

### 1. Application Launching Fixed ✅

**Problem**: Applications were not launching when selected.

**Root Causes**:

1. Child processes were being killed when launcher exited
2. Commands with quotes and field codes weren't properly cleaned
3. Terminal detection wasn't handling all terminal emulator syntaxes

**Solutions Implemented**:

1. **Process Detachment**: Added `setsid -f` to detach spawned processes
   - Prevents child termination when parent exits
   - Redirects stdin/stdout/stderr to `/dev/null`
2. **Improved Command Cleaning**:

   - Better handling of Desktop Entry field codes (%f, %u, %i, etc.)
   - Proper quote removal from exec strings
   - Whitespace normalization

3. **Terminal-Specific Syntax**:
   - Different command syntax for each terminal emulator
   - Alacritty: `-e sh -c 'command'`
   - Kitty: `sh -c 'command'`
   - GNOME Terminal: `-- sh -c 'command'`
   - And more...

### 2. Raycast Design Language Implemented ✅

**Changes Made**:

- Replaced glassmorphism with flat, clean Raycast aesthetic
- Changed color scheme to charcoal + coral/red accents
- Reduced animation time from 0.25-0.3s to 0.15s
- Removed transform effects (scale, translate)
- Simplified shadows to single window shadow
- Updated typography for better hierarchy

## Testing the Application

### 1. Quick Test

```bash
# Build and run
cargo run

# Or run the release build
cargo run --release
```

**What to test**:

1. **Search**: Type to search for applications
2. **Navigation**: Use ↑/↓ arrow keys to move selection
3. **Launch**: Press Enter to launch selected app
4. **Exit**: Press Escape to close

### 2. Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run
```

This will show:

- Desktop files being scanned
- Search results and scoring
- Command execution details
- Any errors that occur

### 3. Test Application Launching

Try launching different types of applications:

```
# Test cases:
1. Simple GUI app (e.g., "firefox", "code")
2. Terminal app (e.g., "htop", "vim")
3. System utilities (e.g., "settings")
```

**Expected behavior**:

- App should launch immediately
- Launcher should close
- App should remain running after launcher exits

### 4. Test Search

```
# Test different search patterns:
1. Full name: "firefox"
2. Partial: "fire"
3. Keywords: "browser"
4. Case insensitive: "FIREFOX"
```

**Expected behavior**:

- Results update as you type
- Relevant apps appear at top
- Empty query shows all apps

## Common Issues & Solutions

### Issue: App launches but closes immediately

**Cause**: Process not properly detached  
**Solution**: Already fixed with `setsid -f`

### Issue: Terminal apps don't work

**Cause**: Wrong terminal command syntax  
**Solution**: Already fixed with terminal-specific commands

### Issue: Some apps have garbled commands

**Cause**: Field codes not cleaned  
**Solution**: Already fixed with improved cleaning

### Issue: Window doesn't show up

**Cause**: Wayland compositor doesn't support layer-shell  
**Solution**: Use a compositor that supports layer-shell (Hyprland, Sway, etc.)

## Design Verification

### Visual Checklist

- [ ] Window has dark charcoal background (#1C1C1E)
- [ ] Search bar has rounded corners and darker background
- [ ] Selected item has coral/red background (#FF6363)
- [ ] Hover states show slightly lighter background
- [ ] No transform effects (scaling, translating)
- [ ] Transitions are fast and snappy (0.15s)
- [ ] Text is white with good contrast
- [ ] Scrollbar is minimal and subtle

### Before/After Comparison

| Aspect     | Before                | After                |
| ---------- | --------------------- | -------------------- |
| Theme      | Blue glassmorphism    | Raycast charcoal     |
| Accent     | Blue (#6496FF)        | Coral/Red (#FF6363)  |
| Shadows    | Multi-layer glow      | Single window shadow |
| Animation  | 0.25-0.3s + transform | 0.15s color only     |
| Background | Semi-transparent blur | Solid dark           |

## Performance Testing

```bash
# Test startup time
time cargo run --release

# Should be under 100ms
```

```bash
# Test search performance with logging
RUST_LOG=debug cargo run --release

# Search latency should be under 10ms
```

## Next Steps

1. **Manual Testing**: Run the app and test all features
2. **Icon Support**: Add app icon display (Phase 2)
3. **Fuzzy Search**: Upgrade to nucleo for better matching
4. **History Tracking**: Track and boost frequently used apps
5. **Configuration**: Add config file support
6. **Hotkey Setup**: Document global hotkey configuration

## Running in Background

To run as a service or with a global hotkey:

```bash
# Example with Hyprland
# Add to ~/.config/hypr/hyprland.conf:
bind = SUPER, SPACE, exec, /path/to/native-launcher
```

```bash
# Example with Sway
# Add to ~/.config/sway/config:
bindsym $mod+Space exec /path/to/native-launcher
```

## Troubleshooting

### Enable Full Logging

```bash
RUST_LOG=trace cargo run 2>&1 | tee launcher.log
```

### Check Desktop Files

```bash
# See what apps are found
ls /usr/share/applications/*.desktop
ls ~/.local/share/applications/*.desktop
```

### Test Command Execution Manually

```bash
# Try running a command directly
setsid -f firefox

# Should launch Firefox and return immediately
```

## Success Criteria

The application is working correctly when:

- ✅ Window appears centered on screen
- ✅ Search results update as you type
- ✅ Arrow keys change selection
- ✅ Enter launches the selected app
- ✅ App continues running after launcher closes
- ✅ Escape closes the launcher
- ✅ UI matches Raycast's design aesthetic
- ✅ Animations are smooth and fast
