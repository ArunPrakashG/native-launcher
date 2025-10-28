# Keybind Detection and Update Feature

## Overview

The installer now includes intelligent keybind detection and update functionality. It automatically detects existing Super+Space keybinds in compositor config files and offers to update them to launch Native Launcher, with full backup and validation support.

## Features

### 1. **Automatic Detection**

Detects Super+Space keybinds across all supported compositors:

- **Hyprland**: `bind = SUPER, Space, exec, ...`
- **Sway**: `bindsym $mod+Space exec ...`
- **i3**: `bindsym $mod+Space exec ...`
- **River**: `riverctl map normal Super Space spawn ...`
- **Wayfire**: `binding_launcher = <super> KEY_SPACE`

### 2. **Smart Update Logic**

- ✅ Detects if keybind already launches Native Launcher (skips update)
- ✅ Detects if keybind launches other apps (rofi, wofi, etc.)
- ✅ Offers to update existing keybinds or add new ones
- ✅ Shows exact line numbers and content before updating
- ✅ **Displays full file paths for easy verification**
- ✅ **Provides shell commands to manually inspect keybinds**

### 3. **Safety Features**

- **Automatic Backups**: Creates timestamped backups before any modification
- **Config Validation**: Validates compositor config after changes
- **Auto-Restore**: Automatically restores backup if validation fails
- **User Confirmation**: Requires explicit confirmation for destructive changes
- **Verification Commands**: Shows `sed` and `tail` commands to verify changes

### 4. **Compositor Integration**

Automatically generates correct keybind syntax for each compositor:

| Compositor | Keybind Syntax                                                                 |
| ---------- | ------------------------------------------------------------------------------ |
| Hyprland   | `bind = SUPER, Space, exec, native-launcher`                                   |
| Sway       | `bindsym $mod+Space exec native-launcher`                                      |
| i3         | `bindsym $mod+Space exec native-launcher`                                      |
| River      | `riverctl map normal Super Space spawn native-launcher`                        |
| Wayfire    | `binding_launcher = <super> KEY_SPACE`<br>`command_launcher = native-launcher` |

## User Experience Flow

### Scenario 1: No Existing Keybind

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Keybind Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ No Super+Space keybind detected in:
  File: ~/.config/hypr/hyprland.conf

Recommended keybind for hyprland:
  bind = SUPER, Space, exec, native-launcher

Add this keybind to your config? (Y/n)
```

**Actions:**

1. User confirms
2. Backup created: `~/.config/hypr/hyprland.conf.backup-keybind-20251028_213045`
3. Keybind added to config with comment
4. Config validated
5. Success message with file path and verification command

### Scenario 2: Existing Keybind (Other App)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Keybind Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ Found existing Super+Space keybind(s) in:
  File: ~/.config/hypr/hyprland.conf

  Line 15: bind = SUPER, Space, exec, rofi -show drun

You can verify with:
  sed -n '15p' ~/.config/hypr/hyprland.conf

⚠️  Super+Space is bound to another application

This keybind will be updated to launch native-launcher.
The old configuration will be backed up.

Update keybind to launch native-launcher? (y/N)
```

**Actions:**

1. User confirms
2. Backup created: `~/.config/hypr/hyprland.conf.backup-keybind-20251028_213045`
3. Line 15 updated in-place
4. Config validated
5. Success message with file path, line number, and verification command

### Scenario 3: Already Configured

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Keybind Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ Found existing Super+Space keybind(s) in:
  File: ~/.config/hypr/hyprland.conf

  Line 15: bind = SUPER, Space, exec, native-launcher

✓ Already configured for native-launcher

You can verify with:
  sed -n '15p' ~/.config/hypr/hyprland.conf
```

**Actions:**

- No changes made
- Shows file path and verification command
- Installation continues

## Implementation Details

### Functions Added

#### `get_keybind_patterns()`

Returns regex patterns for detecting Super+Space keybinds in each compositor's config format.

```bash
# Hyprland example
'bind.*SUPER.*Space|bind.*\$mainMod.*Space'
```

#### `get_recommended_keybind()`

Returns the proper keybind syntax for the detected compositor.

```bash
# Hyprland example
"bind = SUPER, Space, exec, native-launcher"
```

#### `detect_existing_keybinds(config_file)`

Searches config file for Super+Space keybinds and returns line numbers and content.

```bash
# Output format
"3:bind = SUPER, Space, exec, rofi -show drun"
```

#### `is_native_launcher_keybind(keybind_line)`

Checks if a keybind line already launches native-launcher.

Returns 0 (true) if it contains "native-launcher", 1 (false) otherwise.

#### `update_keybind(config_file, line_number, new_keybind)`

Updates a specific line in the config file using `sed`.

```bash
sed -i "${line_number}s/.*/${new_keybind}/" "$config_file"
```

#### `setup_keybinds()`

Main orchestration function that handles the entire keybind setup workflow.

### Integration with Installer

The keybind setup runs after compositor auto-start configuration:

```bash
# In main() function
setup_compositor_integration
setup_compositor_autostart
setup_keybinds          # ← NEW
restart_daemon
```

## Validation and Safety

### Config Validation

After updating keybinds, the installer validates the compositor config:

- **Sway**: `sway --validate --config <file>`
- **i3**: `i3 -C -c <file>`
- **Hyprland/River/Wayfire**: File existence and readability check

### Auto-Restore on Failure

If validation fails:

```bash
✗ Config validation failed!
⚠ Restoring backup...
✓ Backup restored

Manual setup required. Add to ~/.config/hypr/hyprland.conf:
  bind = SUPER, Space, exec, native-launcher
```

### Backup File Naming

Backups use timestamped filenames:

```
~/.config/hypr/hyprland.conf.backup-keybind-20251028_213045
```

Format: `{config_file}.backup-keybind-{YYYYMMDD_HHMMSS}`

## Verification Commands

The installer provides easy-to-use shell commands for users to verify keybind configuration:

### Check Specific Line

When a keybind is detected or updated on a specific line (e.g., line 15):

```bash
sed -n '15p' ~/.config/hypr/hyprland.conf
```

### Check Last Few Lines

When a keybind is added to the end of the file:

```bash
tail -n 3 ~/.config/hypr/hyprland.conf
```

### Full File Path Display

All messages include the complete file path:

```
File: ~/.config/hypr/hyprland.conf
Line 15: bind = SUPER, Space, exec, native-launcher
```

This makes it easy for users to:

- Open the file in their editor
- Manually inspect the configuration
- Verify changes were applied correctly
- Debug any issues that may occur

## Testing

Comprehensive tests verify keybind detection across all compositors:

```bash
./test-keybind-detection.sh
```

Test coverage:

- ✅ Hyprland with rofi
- ✅ Sway with wofi
- ✅ i3 with rofi
- ✅ River with wofi
- ✅ Wayfire with rofi
- ✅ Hyprland already configured with native-launcher

All tests pass successfully.

## Error Handling

### Compositor Not Supported

```
⚠ Compositor auto-start not supported for unknown
```

### Config File Not Found

```
✗ Config file not found: /path/to/config
```

### Backup Creation Failed

```
✗ Failed to create backup
```

### User Declined

```
✓ Keybind setup skipped

To manually configure, add to ~/.config/hypr/hyprland.conf:
  bind = SUPER, Space, exec, native-launcher
```

## Manual Configuration

If automatic setup is skipped or fails, users can manually add keybinds:

**Hyprland** (`~/.config/hypr/hyprland.conf`):

```
bind = SUPER, Space, exec, native-launcher
```

**Sway** (`~/.config/sway/config`):

```
bindsym $mod+Space exec native-launcher
```

**i3** (`~/.config/i3/config`):

```
bindsym $mod+Space exec native-launcher
```

**River** (`~/.config/river/init`):

```bash
riverctl map normal Super Space spawn native-launcher
```

**Wayfire** (`~/.config/wayfire.ini`):

```ini
[command]
binding_launcher = <super> KEY_SPACE
command_launcher = native-launcher
```

## Benefits

1. **Zero Manual Config**: Users don't need to manually edit config files
2. **Safe Updates**: Never loses user configuration with backups
3. **Smart Detection**: Knows when update is needed vs already configured
4. **Transparent**: Shows exactly what will change before modifying
5. **Validated**: Ensures config remains valid after changes
6. **Flexible**: Works with custom config paths and session managers

## Future Enhancements

Potential improvements:

- [ ] Detect custom keybinds (not just Super+Space)
- [ ] Support for multiple keybind options
- [ ] Backup management UI (list/restore old backups)
- [ ] Config diff preview before applying changes
- [ ] Support for more compositors (GNOME, KDE keybinds)
