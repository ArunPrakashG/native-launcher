# Backup & Restore System

## Overview

Native Launcher now includes a comprehensive backup and restore system to protect user data during installations and updates.

## Features

### ✅ Automatic Backup on Installation

The install script automatically backs up existing installations before making any changes.

**What gets backed up:**

- Binary: `~/.local/bin/native-launcher`
- Configuration: `~/.config/native-launcher/config.toml`
- Plugins: `~/.config/native-launcher/plugins/`
- Cache: `~/.cache/native-launcher/*`
- Usage data: `~/.local/share/native-launcher/*`

### 📂 Backup Location

```
~/.local/share/native-launcher/backups/YYYYMMDD_HHMMSS/
├── native-launcher           # Binary backup
├── config/
│   ├── config.toml          # Configuration backup
│   └── plugins/             # All plugins
├── cache/                    # Search cache, icons, etc.
├── data/                     # Usage statistics, preferences
└── backup_info.txt          # Restore instructions
```

### 🔄 Easy Restore

Run the restore script to recover from any backup:

```bash
./restore.sh
```

**Restore features:**

- Lists all available backups with timestamps
- Shows what's included in each backup
- Interactive selection
- Restores all components automatically
- Creates safety backup of current files before restore

## Usage

### Installing (with automatic backup)

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | bash
```

If an existing installation is detected:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[INFO] Existing Native Launcher installation detected
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Found:
  • Binary: ~/.local/bin/native-launcher
  • Config: ~/.config/native-launcher/config.toml
  • Data: ~/.local/share/native-launcher

Create backup before proceeding? (Y/n)
```

### Restoring from Backup

```bash
./restore.sh
```

Interactive menu:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Available Backups
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[1] 20240315_143022
    Created: 2024-03-15 14:30:22
    Binary: ✓
    Config: ✓
    Data: ✓

[2] 20240314_092045
    Created: 2024-03-14 09:20:45
    Binary: ✓
    Config: ✓
    Data: ✓

Select backup to restore (1-2) or 'q' to quit:
```

### Manual Restore

If you prefer manual restore:

```bash
# List backups
ls ~/.local/share/native-launcher/backups/

# View restore instructions
cat ~/.local/share/native-launcher/backups/20240315_143022/backup_info.txt

# Restore binary
cp ~/.local/share/native-launcher/backups/20240315_143022/native-launcher \
   ~/.local/bin/

# Restore config
cp ~/.local/share/native-launcher/backups/20240315_143022/config/config.toml \
   ~/.config/native-launcher/
```

## Implementation Details

### install.sh Changes

**New function:** `backup_existing_installation()`

```bash
backup_existing_installation() {
    # 1. Detect existing installation
    # 2. Create timestamped backup directory
    # 3. Copy all components
    # 4. Generate restore instructions
    # 5. Log success for each component
}
```

**Integration point:** Called after system detection, before dependency installation

```bash
main() {
    detect_compositor
    backup_existing_installation  # <-- NEW
    install_dependencies
    download_and_install_binary
    # ...
}
```

### restore.sh Features

**Core functions:**

- `list_backups()` - Interactive selection menu
- `restore_binary()` - Restore executable
- `restore_config()` - Restore configuration
- `restore_plugins()` - Restore plugins
- `restore_cache()` - Restore cache
- `restore_data()` - Restore usage data

**Safety features:**

- Validates backup before restore
- Creates safety backup of current files
- Non-destructive (doesn't delete backups directory)
- Detailed success/failure logging

## Non-Interactive Mode

In non-interactive mode, backups are created automatically without prompting:

```bash
curl -fsSL https://raw.githubusercontent.com/ArunPrakashG/native-launcher/main/install.sh | \
  bash -s -- --non-interactive
```

## File Structure

```
install.sh
├── backup_existing_installation()   # Lines 220-320
│   ├── Detection logic
│   ├── Timestamped directory creation
│   ├── Component backup (binary, config, plugins, cache, data)
│   ├── Restore instructions generation
│   └── Success logging
└── main()
    └── Calls backup_existing_installation()

restore.sh
├── list_backups()              # Interactive selection
├── restore_binary()            # Binary restoration
├── restore_config()            # Config restoration
├── restore_plugins()           # Plugin restoration
├── restore_cache()             # Cache restoration
└── restore_data()              # Data restoration
```

## Testing

### Verify Syntax

```bash
bash -n install.sh
bash -n restore.sh
```

### Test Backup Creation

```bash
# Install to trigger backup
./install.sh

# Check backup was created
ls ~/.local/share/native-launcher/backups/
```

### Test Restore

```bash
# Run restore script
./restore.sh

# Select a backup and verify restoration
```

## Benefits

1. **Data Safety** - Never lose your configuration or usage data
2. **Easy Rollback** - Quickly revert to previous version
3. **Version Migration** - Safely upgrade between major versions
4. **Testing** - Try new features without fear
5. **Multiple Backups** - Keep history of installations
6. **Automatic** - No manual intervention required

## Future Enhancements

- [ ] Backup compression (gzip)
- [ ] Automatic cleanup of old backups (keep last N)
- [ ] Backup verification/integrity checks
- [ ] Import/export backups
- [ ] Cloud backup sync
- [ ] Backup scheduling

---

**Status:** ✅ Implemented and tested  
**Version:** 1.1.0  
**Last Updated:** 2024-03-15
