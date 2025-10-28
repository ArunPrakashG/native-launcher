#!/bin/bash
# Native Launcher - Restore from Backup Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Installation paths
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/native-launcher"
CACHE_DIR="$HOME/.cache/native-launcher"
DATA_DIR="$HOME/.local/share/native-launcher"
BACKUP_BASE_DIR="$DATA_DIR/backups"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# List available backups
list_backups() {
    if [ ! -d "$BACKUP_BASE_DIR" ]; then
        log_error "No backups found at $BACKUP_BASE_DIR"
        exit 1
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Available Backups"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    local count=0
    local -a backup_dirs
    
    for backup_dir in "$BACKUP_BASE_DIR"/*; do
        if [ -d "$backup_dir" ]; then
            count=$((count + 1))
            backup_dirs+=("$backup_dir")
            local backup_name=$(basename "$backup_dir")
            local backup_info="$backup_dir/backup_info.txt"
            
            echo "[$count] $backup_name"
            
            if [ -f "$backup_info" ]; then
                echo "    $(grep "Created:" "$backup_info" | head -n 1)"
                echo "    Binary: $([ -f "$backup_dir/native-launcher" ] && echo "✓" || echo "✗")"
                echo "    Config: $([ -f "$backup_dir/config/config.toml" ] && echo "✓" || echo "✗")"
                echo "    Data: $([ -d "$backup_dir/data" ] && echo "✓" || echo "✗")"
            fi
            echo ""
        fi
    done
    
    if [ $count -eq 0 ]; then
        log_error "No backups found"
        exit 1
    fi
    
    # Ask user to select backup
    read -p "Select backup to restore (1-$count) or 'q' to quit: " selection
    
    if [ "$selection" = "q" ] || [ "$selection" = "Q" ]; then
        log_info "Restore cancelled"
        exit 0
    fi
    
    if ! [[ "$selection" =~ ^[0-9]+$ ]] || [ "$selection" -lt 1 ] || [ "$selection" -gt $count ]; then
        log_error "Invalid selection"
        exit 1
    fi
    
    SELECTED_BACKUP="${backup_dirs[$((selection - 1))]}"
    log_info "Selected backup: $(basename "$SELECTED_BACKUP")"
}

# Restore binary
restore_binary() {
    local backup_binary="$SELECTED_BACKUP/native-launcher"
    
    if [ ! -f "$backup_binary" ]; then
        log_warning "No binary found in backup"
        return
    fi
    
    log_info "Restoring binary..."
    mkdir -p "$INSTALL_DIR"
    cp "$backup_binary" "$INSTALL_DIR/native-launcher"
    chmod +x "$INSTALL_DIR/native-launcher"
    log_success "Binary restored to $INSTALL_DIR/native-launcher"
}

# Restore configuration
restore_config() {
    local backup_config="$SELECTED_BACKUP/config/config.toml"
    
    if [ ! -f "$backup_config" ]; then
        log_warning "No configuration found in backup"
        return
    fi
    
    log_info "Restoring configuration..."
    mkdir -p "$CONFIG_DIR"
    
    # Backup current config if it exists
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        cp "$CONFIG_DIR/config.toml" "$CONFIG_DIR/config.toml.before-restore"
        log_info "Current config backed up to config.toml.before-restore"
    fi
    
    cp "$backup_config" "$CONFIG_DIR/config.toml"
    log_success "Configuration restored to $CONFIG_DIR/config.toml"
}

# Restore plugins
restore_plugins() {
    local backup_plugins="$SELECTED_BACKUP/config/plugins"
    
    if [ ! -d "$backup_plugins" ]; then
        log_info "No plugins found in backup"
        return
    fi
    
    log_info "Restoring plugins..."
    mkdir -p "$CONFIG_DIR/plugins"
    cp -r "$backup_plugins/"* "$CONFIG_DIR/plugins/"
    log_success "Plugins restored to $CONFIG_DIR/plugins"
}

# Restore cache
restore_cache() {
    local backup_cache="$SELECTED_BACKUP/cache"
    
    if [ ! -d "$backup_cache" ]; then
        log_info "No cache found in backup"
        return
    fi
    
    log_info "Restoring cache..."
    mkdir -p "$CACHE_DIR"
    cp -r "$backup_cache/"* "$CACHE_DIR/"
    log_success "Cache restored to $CACHE_DIR"
}

# Restore data
restore_data() {
    local backup_data="$SELECTED_BACKUP/data"
    
    if [ ! -d "$backup_data" ]; then
        log_info "No data found in backup"
        return
    fi
    
    log_info "Restoring usage data..."
    mkdir -p "$DATA_DIR"
    
    # Don't overwrite backups directory
    for item in "$backup_data"/*; do
        if [ "$(basename "$item")" != "backups" ]; then
            cp -r "$item" "$DATA_DIR/"
        fi
    done
    
    log_success "Data restored to $DATA_DIR"
}

# Main restore flow
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Native Launcher - Restore from Backup"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # List and select backup
    list_backups
    
    echo ""
    log_warning "This will restore Native Launcher from the selected backup"
    read -p "Continue with restore? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Restore cancelled"
        exit 0
    fi
    
    echo ""
    
    # Restore components
    restore_binary
    restore_config
    restore_plugins
    restore_cache
    restore_data
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_success "Restore completed successfully!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Restored from: $(basename "$SELECTED_BACKUP")"
    echo ""
    echo "You can now run Native Launcher:"
    echo "  $INSTALL_DIR/native-launcher"
    echo ""
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Native Launcher Restore Script"
        echo ""
        echo "Usage: $0"
        echo ""
        echo "This script will:"
        echo "  1. List all available backups"
        echo "  2. Let you select a backup to restore"
        echo "  3. Restore binary, config, plugins, cache, and data"
        exit 0
        ;;
    *)
        main
        ;;
esac
