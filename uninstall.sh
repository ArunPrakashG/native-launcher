#!/bin/bash
# Native Launcher - Uninstall Script

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

# Detect Wayland compositor
detect_compositor() {
    if [ -n "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
        COMPOSITOR="hyprland"
        COMPOSITOR_CONFIG="$HOME/.config/hypr/hyprland.conf"
    elif [ "$XDG_CURRENT_DESKTOP" = "sway" ]; then
        COMPOSITOR="sway"
        COMPOSITOR_CONFIG="$HOME/.config/sway/config"
    else
        COMPOSITOR="unknown"
        COMPOSITOR_CONFIG=""
    fi
}

# Remove binary
remove_binary() {
    if [ -f "$INSTALL_DIR/native-launcher" ]; then
        log_info "Removing binary from $INSTALL_DIR..."
        rm -f "$INSTALL_DIR/native-launcher"
        log_success "Binary removed"
    else
        log_info "Binary not found at $INSTALL_DIR/native-launcher"
    fi
}

# Remove configuration
remove_config() {
    if [ -d "$CONFIG_DIR" ]; then
        read -p "Remove configuration directory $CONFIG_DIR? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIG_DIR"
            log_success "Configuration removed"
        else
            log_info "Configuration kept at $CONFIG_DIR"
        fi
    else
        log_info "Configuration directory not found"
    fi
}

# Remove cache
remove_cache() {
    if [ -d "$CACHE_DIR" ]; then
        read -p "Remove cache directory $CACHE_DIR? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CACHE_DIR"
            log_success "Cache removed"
        else
            log_info "Cache kept at $CACHE_DIR"
        fi
    else
        log_info "Cache directory not found"
    fi
}

# Remove data
remove_data() {
    if [ -d "$DATA_DIR" ]; then
        read -p "Remove data directory $DATA_DIR (includes usage statistics)? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$DATA_DIR"
            log_success "Data removed"
        else
            log_info "Data kept at $DATA_DIR"
        fi
    else
        log_info "Data directory not found"
    fi
}

# Remove compositor keybind
remove_compositor_keybind() {
    if [ -z "$COMPOSITOR_CONFIG" ] || [ ! -f "$COMPOSITOR_CONFIG" ]; then
        log_info "Compositor config not found, skipping keybind removal"
        return
    fi
    
    if grep -q "native-launcher" "$COMPOSITOR_CONFIG"; then
        read -p "Remove native-launcher keybind from $COMPOSITOR? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # Create backup
            cp "$COMPOSITOR_CONFIG" "$COMPOSITOR_CONFIG.backup"
            log_info "Created backup: $COMPOSITOR_CONFIG.backup"
            
            # Remove lines containing native-launcher
            sed -i '/native-launcher/d' "$COMPOSITOR_CONFIG"
            log_success "Keybind removed from $COMPOSITOR config"
            log_warning "Reload your compositor to apply changes"
        else
            log_info "Keybind kept in compositor config"
        fi
    else
        log_info "No native-launcher keybind found in compositor config"
    fi
}

# Main uninstall flow
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Native Launcher - Uninstall"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    log_warning "This will remove Native Launcher from your system"
    read -p "Continue with uninstallation? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstallation cancelled"
        exit 0
    fi
    
    echo ""
    
    # Detect compositor for keybind removal
    detect_compositor
    
    # Remove components
    remove_binary
    remove_compositor_keybind
    remove_config
    remove_cache
    remove_data
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_success "Native Launcher has been uninstalled"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Thank you for trying Native Launcher!"
    echo "Feedback: https://github.com/ArunPrakashG/native-launcher/issues"
    echo ""
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Native Launcher Uninstall Script"
        echo ""
        echo "Usage: $0"
        echo ""
        echo "This script will remove Native Launcher and optionally:"
        echo "  - Configuration files"
        echo "  - Cache files"
        echo "  - Usage statistics"
        echo "  - Compositor keybinds"
        exit 0
        ;;
    *)
        main
        ;;
esac
