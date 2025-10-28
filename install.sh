#!/bin/bash
# Native Launcher - Automated Installation Script
# Primary support: Arch Linux + Hyprland
# Additional support: Debian/Ubuntu, Fedora, and other Wayland compositors

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script version
SCRIPT_VERSION="1.1.0"

# GitHub repository info
GITHUB_REPO="ArunPrakashG/native-launcher"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/native-launcher"
DESKTOP_DIR="$HOME/.local/share/applications"

# Interactive mode flag
INTERACTIVE=true
SELECTED_THEME="default"
INSTALL_MODE="upgrade"  # "upgrade" or "fresh"
SKIP_BACKUP=false

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --non-interactive)
                INTERACTIVE=false
                shift
                ;;
            --fresh)
                INSTALL_MODE="fresh"
                shift
                ;;
            --upgrade)
                INSTALL_MODE="upgrade"
                shift
                ;;
            --skip-backup)
                SKIP_BACKUP=true
                shift
                ;;
            --theme)
                SELECTED_THEME="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Show help message
show_help() {
    cat << EOF
Native Launcher Installation Script v${SCRIPT_VERSION}

Usage: $0 [OPTIONS]

Options:
  --non-interactive    Skip all interactive prompts (use defaults)
  --fresh              Fresh installation (remove existing configs)
  --upgrade            Upgrade installation (keep existing configs) [default]
  --skip-backup        Skip backup creation (not recommended)
  --theme THEME        Set theme without prompt (default, nord, dracula, 
                       catppuccin, gruvbox, tokyonight)
  -h, --help           Show this help message

Examples:
  # Interactive installation (recommended)
  $0

  # Non-interactive upgrade with Nord theme
  $0 --non-interactive --upgrade --theme nord

  # Fresh install without prompts
  $0 --non-interactive --fresh --theme default

EOF
}


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

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        DISTRO_VERSION=$VERSION_ID
    else
        log_error "Cannot detect distribution"
        exit 1
    fi
    log_info "Detected distribution: $DISTRO $DISTRO_VERSION"
}

# Detect Wayland compositor
detect_compositor() {
    if [ -n "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
        COMPOSITOR="hyprland"
    elif [ "$XDG_CURRENT_DESKTOP" = "sway" ]; then
        COMPOSITOR="sway"
    elif [ "$XDG_SESSION_DESKTOP" = "KDE" ] && [ "$XDG_SESSION_TYPE" = "wayland" ]; then
        COMPOSITOR="kde-wayland"
    elif [ "$XDG_CURRENT_DESKTOP" = "GNOME" ] && [ "$XDG_SESSION_TYPE" = "wayland" ]; then
        COMPOSITOR="gnome-wayland"
    elif [ "$XDG_SESSION_TYPE" = "wayland" ]; then
        COMPOSITOR="wayland-generic"
    else
        COMPOSITOR="unknown"
        log_warning "Could not detect Wayland compositor. Manual configuration may be needed."
    fi
    log_info "Detected compositor: $COMPOSITOR"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Required tools
    command -v curl >/dev/null 2>&1 || missing_deps+=("curl")
    command -v tar >/dev/null 2>&1 || missing_deps+=("tar")
    command -v jq >/dev/null 2>&1 || missing_deps+=("jq")
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Installing dependencies..."
        install_dependencies "${missing_deps[@]}"
    else
        log_success "All required dependencies are installed"
    fi
}

# Install dependencies based on distribution
install_dependencies() {
    local deps=("$@")
    
    case $DISTRO in
        arch|manjaro|endeavouros)
            sudo pacman -S --noconfirm "${deps[@]}"
            ;;
        ubuntu|debian|pop)
            sudo apt-get update
            sudo apt-get install -y "${deps[@]}"
            ;;
        fedora)
            sudo dnf install -y "${deps[@]}"
            ;;
        opensuse*)
            sudo zypper install -y "${deps[@]}"
            ;;
        *)
            log_error "Unsupported distribution for automatic dependency installation"
            log_info "Please install manually: ${deps[*]}"
            exit 1
            ;;
    esac
}

# Install GTK4 and layer-shell dependencies
install_gtk_dependencies() {
    log_info "Installing GTK4 and layer-shell dependencies..."
    
    case $DISTRO in
        arch|manjaro|endeavouros)
            sudo pacman -S --noconfirm gtk4 gtk4-layer-shell
            log_success "GTK4 dependencies installed"
            ;;
        ubuntu|debian|pop)
            sudo apt-get update
            sudo apt-get install -y libgtk-4-1 libgtk-4-dev gtk4-layer-shell
            log_success "GTK4 dependencies installed"
            ;;
        fedora)
            sudo dnf install -y gtk4 gtk4-devel gtk4-layer-shell
            log_success "GTK4 dependencies installed"
            ;;
        opensuse*)
            sudo zypper install -y gtk4 gtk4-devel gtk4-layer-shell
            log_success "GTK4 dependencies installed"
            ;;
        *)
            log_warning "Please install GTK4 and gtk4-layer-shell manually for your distribution"
            ;;
    esac
}

# Get latest release from GitHub
get_latest_release() {
    log_info "Fetching latest release information..."
    
    RELEASE_DATA=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
    
    if echo "$RELEASE_DATA" | jq -e '.message' >/dev/null 2>&1; then
        log_error "Failed to fetch release data from GitHub"
        log_info "Response: $(echo "$RELEASE_DATA" | jq -r '.message')"
        exit 1
    fi
    
    LATEST_VERSION=$(echo "$RELEASE_DATA" | jq -r '.tag_name')
    DOWNLOAD_URL=$(echo "$RELEASE_DATA" | jq -r '.assets[] | select(.name | test("native-launcher.*linux.*tar.gz")) | .browser_download_url')
    
    if [ -z "$DOWNLOAD_URL" ]; then
        log_error "No Linux binary found in latest release"
        log_info "You may need to build from source"
        exit 1
    fi
    
    log_success "Latest version: $LATEST_VERSION"
    log_info "Download URL: $DOWNLOAD_URL"
}

# Download and extract binary
download_and_install() {
    log_info "Downloading native-launcher $LATEST_VERSION..."
    
    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    # Download release
    if ! curl -L -o native-launcher.tar.gz "$DOWNLOAD_URL"; then
        log_error "Failed to download release"
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    log_success "Downloaded successfully"
    log_info "Extracting..."
    
    # Extract archive
    tar -xzf native-launcher.tar.gz
    
    # Find the binary
    BINARY=$(find . -name "native-launcher" -type f -executable | head -n 1)
    
    if [ -z "$BINARY" ]; then
        log_error "Binary not found in archive"
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Install binary
    log_info "Installing to $INSTALL_DIR..."
    cp "$BINARY" "$INSTALL_DIR/native-launcher"
    chmod +x "$INSTALL_DIR/native-launcher"
    
    # Cleanup
    cd - > /dev/null
    rm -rf "$TMP_DIR"
    
    log_success "Binary installed to $INSTALL_DIR/native-launcher"
}

# Backup existing installation
backup_existing_installation() {
    # Skip backup if flag is set
    if [ "$SKIP_BACKUP" = true ]; then
        log_warning "Skipping backup (--skip-backup flag set)"
        return 0
    fi
    
    log_info "Checking for existing installation..."
    
    local backup_needed=false
    local backup_dir="$HOME/.local/share/native-launcher/backups/$(date +%Y%m%d_%H%M%S)"
    
    # Check if binary exists
    if [ -f "$INSTALL_DIR/native-launcher" ]; then
        backup_needed=true
        log_info "Found existing binary"
    fi
    
    # Check if config exists
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        backup_needed=true
        log_info "Found existing configuration"
    fi
    
    # Check if data exists
    if [ -d "$HOME/.local/share/native-launcher" ] || [ -d "$HOME/.cache/native-launcher" ]; then
        backup_needed=true
        log_info "Found existing data/cache"
    fi
    
    if [ "$backup_needed" = false ]; then
        log_info "No existing installation found, skipping backup"
        return 0
    fi
    
    log_warning "Existing installation detected"
    
    if [ "$INTERACTIVE" = "true" ]; then
        read -p "Create backup before installing? (Y/n) " -n 1 -r < /dev/tty
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            log_warning "Skipping backup (not recommended)"
            return 0
        fi
    fi
    
    # Create backup directory
    mkdir -p "$backup_dir" || {
        log_error "Failed to create backup directory"
        return 1
    }
    log_info "Creating backup at $backup_dir..."
    
    # Backup binary
    if [ -f "$INSTALL_DIR/native-launcher" ]; then
        cp "$INSTALL_DIR/native-launcher" "$backup_dir/native-launcher" 2>/dev/null && \
            log_success "Backed up binary" || \
            log_warning "Failed to backup binary"
    fi
    
    # Backup configuration
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        mkdir -p "$backup_dir/config" 2>/dev/null
        cp "$CONFIG_DIR/config.toml" "$backup_dir/config/config.toml" 2>/dev/null && \
            log_success "Backed up configuration" || \
            log_warning "Failed to backup configuration"
    fi
    
    # Backup plugins directory
    if [ -d "$CONFIG_DIR/plugins" ]; then
        cp -r "$CONFIG_DIR/plugins" "$backup_dir/config/" 2>/dev/null && \
            log_success "Backed up plugins" || \
            log_warning "Failed to backup plugins"
    fi
    
    # Backup cache
    if [ -d "$HOME/.cache/native-launcher" ]; then
        mkdir -p "$backup_dir/cache" 2>/dev/null
        cp -r "$HOME/.cache/native-launcher/"* "$backup_dir/cache/" 2>/dev/null && \
            log_success "Backed up cache" || \
            log_warning "Cache backup skipped (empty or inaccessible)"
    fi
    
    # Backup usage data (exclude backups directory to avoid recursion)
    if [ -d "$HOME/.local/share/native-launcher" ]; then
        mkdir -p "$backup_dir/data" 2>/dev/null
        # Copy everything except the backups directory
        local backed_up=false
        for item in "$HOME/.local/share/native-launcher/"*; do
            [ -e "$item" ] || continue  # Skip if no files exist
            if [ "$(basename "$item")" != "backups" ]; then
                if cp -r "$item" "$backup_dir/data/" 2>/dev/null; then
                    backed_up=true
                fi
            fi
        done
        if [ "$backed_up" = true ]; then
            log_success "Backed up usage data"
        else
            log_warning "Usage data backup skipped (empty or inaccessible)"
        fi
    fi
    
    # Create backup info file
    cat > "$backup_dir/backup_info.txt" << EOF
Native Launcher Backup
Created: $(date)
Backup Location: $backup_dir

Contents:
- Binary: $([ -f "$backup_dir/native-launcher" ] && echo "✓" || echo "✗")
- Config: $([ -f "$backup_dir/config/config.toml" ] && echo "✓" || echo "✗")
- Plugins: $([ -d "$backup_dir/config/plugins" ] && echo "✓" || echo "✗")
- Cache: $([ -d "$backup_dir/cache" ] && echo "✓" || echo "✗")
- Data: $([ -d "$backup_dir/data" ] && echo "✓" || echo "✗")

To restore:
  cp $backup_dir/native-launcher $INSTALL_DIR/
  cp $backup_dir/config/config.toml $CONFIG_DIR/
  cp -r $backup_dir/config/plugins $CONFIG_DIR/ (if exists)
  cp -r $backup_dir/cache/* ~/.cache/native-launcher/
  cp -r $backup_dir/data/* ~/.local/share/native-launcher/
EOF
    
    log_success "Backup completed successfully"
    log_info "Backup location: $backup_dir"
    echo ""
}

# Choose installation mode (fresh vs upgrade)
choose_install_mode() {
    # Skip if already set via CLI or non-interactive
    if [ "$INTERACTIVE" != "true" ]; then
        return
    fi
    
    # Only ask if existing installation found
    if [ ! -f "$CONFIG_DIR/config.toml" ] && [ ! -f "$INSTALL_DIR/native-launcher" ]; then
        log_info "No existing installation found, proceeding with fresh install"
        INSTALL_MODE="fresh"
        return
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Installation Mode"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Existing installation detected. How would you like to proceed?"
    echo ""
    echo "  [1] Upgrade - Keep existing configs and data (recommended)"
    echo "      • Binary will be updated"
    echo "      • Your configuration will be preserved"
    echo "      • Usage data and cache remain intact"
    echo "      • Backup will be created"
    echo ""
    echo "  [2] Fresh Install - Remove all existing data"
    echo "      • All configs will be deleted"
    echo "      • Cache and usage data will be cleared"
    echo "      • You'll configure theme again"
    echo "      • Backup will be created first"
    echo ""
    
    while true; do
        read -p "Choose installation mode (1=upgrade, 2=fresh) [1]: " choice < /dev/tty
        choice=${choice:-1}
        
        case $choice in
            1)
                INSTALL_MODE="upgrade"
                log_info "Selected: Upgrade (keep existing configs)"
                break
                ;;
            2)
                INSTALL_MODE="fresh"
                log_warning "Selected: Fresh install (will remove configs)"
                echo ""
                read -p "Are you sure? This will delete all existing configs! (yes/no) [no]: " confirm < /dev/tty
                if [ "$confirm" = "yes" ]; then
                    break
                else
                    log_info "Cancelled fresh install, switching to upgrade mode"
                    INSTALL_MODE="upgrade"
                    break
                fi
                ;;
            *)
                log_error "Invalid choice. Please enter 1 or 2"
                ;;
        esac
    done
    
    echo ""
}

# Clean existing installation (for fresh install mode)
clean_existing_installation() {
    if [ "$INSTALL_MODE" != "fresh" ]; then
        return
    fi
    
    log_warning "Performing fresh install - removing existing configs..."
    
    # Remove config directory
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        log_success "Removed configuration directory"
    fi
    
    # Remove cache
    if [ -d "$HOME/.cache/native-launcher" ]; then
        rm -rf "$HOME/.cache/native-launcher"
        log_success "Removed cache directory"
    fi
    
    # Remove data (but keep backups)
    if [ -d "$HOME/.local/share/native-launcher" ]; then
        # Move backups temporarily
        local temp_backups=""
        if [ -d "$HOME/.local/share/native-launcher/backups" ]; then
            temp_backups=$(mktemp -d)
            mv "$HOME/.local/share/native-launcher/backups" "$temp_backups/"
        fi
        
        # Remove data directory
        rm -rf "$HOME/.local/share/native-launcher"
        
        # Restore backups
        if [ -n "$temp_backups" ]; then
            mkdir -p "$HOME/.local/share/native-launcher"
            mv "$temp_backups/backups" "$HOME/.local/share/native-launcher/"
            rm -rf "$temp_backups"
        fi
        
        log_success "Removed data directory (backups preserved)"
    fi
    
    echo ""
}

# Stop daemon if running
stop_daemon() {
    log_info "Checking for running daemon..."
    
    # Find daemon process
    local daemon_pid=$(pgrep -f "native-launcher.*--daemon" 2>/dev/null)
    
    if [ -n "$daemon_pid" ]; then
        log_warning "Found running daemon (PID: $daemon_pid)"
        
        # Mark that daemon was running (for restart after installation)
        touch /tmp/native-launcher-daemon-was-running
        
        if [ "$INTERACTIVE" = "true" ]; then
            read -p "Stop daemon before installation? (Y/n) " -n 1 -r < /dev/tty
            echo
            if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                kill "$daemon_pid" 2>/dev/null && \
                    log_success "Daemon stopped" || \
                    log_warning "Failed to stop daemon"
                sleep 1
            else
                log_warning "Daemon still running - installation may require manual restart"
                rm -f /tmp/native-launcher-daemon-was-running
            fi
        else
            # Non-interactive: always stop daemon
            kill "$daemon_pid" 2>/dev/null && \
                log_success "Daemon stopped" || \
                log_warning "Failed to stop daemon"
            sleep 1
        fi
    else
        log_info "No daemon running"
    fi
}

# Check if daemon should be restarted after installation
should_restart_daemon() {
    # Check if daemon was running before (marked by stop_daemon)
    if [ -f "/tmp/native-launcher-daemon-was-running" ]; then
        return 0
    fi
    
    return 1
}

# Restart daemon if it was running before
restart_daemon() {
    if ! should_restart_daemon; then
        return
    fi
    
    log_info "Restarting daemon..."
    
    # Start daemon manually (compositor auto-start will handle it on next restart)
    nohup "$INSTALL_DIR/native-launcher" --daemon >/dev/null 2>&1 &
    log_success "Daemon started in background"
    
    # Clean up marker file
    rm -f /tmp/native-launcher-daemon-was-running
}

# Detect compositor config file
get_compositor_config_path() {
    case "$COMPOSITOR" in
        hyprland)
            echo "$HOME/.config/hypr/hyprland.conf"
            ;;
        sway)
            echo "$HOME/.config/sway/config"
            ;;
        i3)
            echo "$HOME/.config/i3/config"
            ;;
        river)
            echo "$HOME/.config/river/init"
            ;;
        wayfire)
            echo "$HOME/.config/wayfire.ini"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Detect session manager (uwsm, etc.)
detect_session_manager() {
    if command -v uwsm >/dev/null 2>&1; then
        echo "uwsm"
    else
        echo ""
    fi
}

# Get auto-start command format for compositor
get_autostart_command() {
    local binary="native-launcher"  # Use binary name from PATH, not absolute path
    local use_session_mgr="$1"
    
    # Wrap with session manager if requested
    if [ "$use_session_mgr" = "yes" ]; then
        local session_mgr=$(detect_session_manager)
        if [ "$session_mgr" = "uwsm" ]; then
            binary="uwsm app -- $binary"
        fi
    fi
    
    case "$COMPOSITOR" in
        hyprland)
            echo "exec-once = $binary --daemon"
            ;;
        sway)
            echo "exec $binary --daemon"
            ;;
        i3)
            echo "exec --no-startup-id $binary --daemon"
            ;;
        river)
            echo "riverctl spawn \"$binary --daemon\""
            ;;
        wayfire)
            echo "autostart_native_launcher = $binary --daemon"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Remove old daemon entries from compositor config
remove_old_daemon_entries() {
    local config_file="$1"
    
    if [ ! -f "$config_file" ]; then
        return
    fi
    
    # Create temp file without daemon entries
    grep -v "native-launcher.*--daemon" "$config_file" > "${config_file}.tmp" || true
    mv "${config_file}.tmp" "$config_file"
}

# Validate compositor config
validate_compositor_config() {
    local config_file="$1"
    
    case "$COMPOSITOR" in
        sway)
            # Sway has built-in validation
            if command -v sway >/dev/null 2>&1; then
                sway --validate --config "$config_file" >/dev/null 2>&1
                return $?
            fi
            ;;
        i3)
            # i3 has built-in validation
            if command -v i3 >/dev/null 2>&1; then
                i3 -C -c "$config_file" >/dev/null 2>&1
                return $?
            fi
            ;;
        hyprland|river|wayfire)
            # No built-in validators, just check file exists and is readable
            [ -f "$config_file" ] && [ -r "$config_file" ]
            return $?
            ;;
    esac
    
    # Default: assume valid if file exists
    [ -f "$config_file" ]
    return $?
}

# Setup compositor auto-start (replaces systemd approach)
setup_compositor_autostart() {
    if [ "$INTERACTIVE" != "true" ]; then
        return
    fi
    
    # Get default compositor config path
    local detected_config=$(get_compositor_config_path)
    local config_file=""
    
    if [ -z "$detected_config" ]; then
        log_warning "Compositor auto-start not supported for $COMPOSITOR"
        log_info "To enable daemon mode, add to your compositor config:"
        echo "  native-launcher --daemon"
        return
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Compositor Auto-Start Configuration"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Detected compositor: $COMPOSITOR"
    echo "Detected config file: $detected_config"
    
    # Check if detected config exists
    if [ -f "$detected_config" ]; then
        echo ""
        read -p "Is this the correct config file? (Y/n) " -n 1 -r < /dev/tty
        echo
        
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            echo ""
            read -p "Enter path to your compositor config: " config_file < /dev/tty
            config_file="${config_file/#\~/$HOME}"  # Expand ~ to $HOME
            
            if [ ! -f "$config_file" ]; then
                log_error "Config file not found: $config_file"
                return
            fi
            
            log_success "Using custom config: $config_file"
        else
            config_file="$detected_config"
        fi
    else
        log_warning "Detected config not found: $detected_config"
        echo ""
        read -p "Enter path to your compositor config (or press Enter to skip): " config_file < /dev/tty
        
        if [ -z "$config_file" ]; then
            log_info "Skipping daemon auto-start setup"
            log_info "To enable manually, add to your compositor config:"
            echo "  native-launcher --daemon"
            return
        fi
        
        config_file="${config_file/#\~/$HOME}"  # Expand ~ to $HOME
        
        if [ ! -f "$config_file" ]; then
            log_error "Config file not found: $config_file"
            return
        fi
    fi
    
    # Check if already configured
    local existing_entry=$(grep -n "native-launcher.*--daemon" "$config_file" 2>/dev/null | head -1)
    local update_mode=false
    
    if [ -n "$existing_entry" ]; then
        log_info "Found existing daemon entry in config"
        echo "  Line: $existing_entry"
        echo ""
        read -p "Update existing entry? (Y/n) " -n 1 -r < /dev/tty
        echo
        
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            log_info "Keeping existing daemon configuration"
            return
        fi
        
        update_mode=true
    fi
    
    # Detect session manager
    local session_mgr=$(detect_session_manager)
    local use_session_mgr="no"
    
    if [ -n "$session_mgr" ]; then
        echo ""
        log_info "Detected session manager: $session_mgr"
        echo "Session managers provide better Wayland session integration."
        echo ""
        read -p "Use $session_mgr to launch daemon? (Y/n) " -n 1 -r < /dev/tty
        echo
        
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            use_session_mgr="yes"
            log_success "Will use $session_mgr for launching"
        fi
    fi
    
    # Get auto-start command
    local autostart_cmd=$(get_autostart_command "$use_session_mgr")
    
    if [ -z "$autostart_cmd" ]; then
        return
    fi
    
    echo ""
    echo "Configuration Summary:"
    echo "  Config file: $config_file"
    if [ "$update_mode" = true ]; then
        echo "  Mode: Update existing entry"
    else
        echo "  Mode: Add new entry"
    fi
    echo ""
    echo "Auto-start command:"
    echo "  $autostart_cmd"
    echo ""
    echo "Benefits:"
    echo "  • Launcher pre-loads on compositor startup"
    echo "  • Instant appearance when pressing Super+Space"
    echo "  • No manual daemon management needed"
    echo ""
    echo "Trade-offs:"
    echo "  • Uses ~20-30MB RAM constantly"
    echo ""
    echo "⚠️  WARNING: This will modify your compositor config"
    echo ""
    
    # Create backup filename
    local backup_file="${config_file}.backup-$(date +%Y%m%d_%H%M%S)"
    echo "Backup will be created at:"
    echo "  $backup_file"
    echo ""
    echo "If validation fails, backup will be auto-restored."
    echo ""
    
    if [ "$update_mode" = true ]; then
        read -p "Update daemon configuration? (Y/n) " -n 1 -r < /dev/tty
    else
        read -p "Add daemon to auto-start? (y/N) " -n 1 -r < /dev/tty
    fi
    echo
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Compositor auto-start skipped"
        return
    fi
    
    # Create backup
    log_info "Creating backup..."
    cp "$config_file" "$backup_file" || {
        log_error "Failed to create backup"
        return 1
    }
    log_success "Backup created: $backup_file"
    
    if [ "$update_mode" = true ]; then
        # Update existing entry
        log_info "Updating existing daemon entry..."
        sed -i "/native-launcher.*--daemon/c\\$autostart_cmd" "$config_file"
    else
        # Remove old entries (avoid duplicates)
        log_info "Removing old daemon entries (if any)..."
        remove_old_daemon_entries "$config_file"
        
        # Add new entry
        log_info "Adding daemon to compositor config..."
        echo "" >> "$config_file"
        echo "# Native Launcher Daemon - Added by installer on $(date +%Y-%m-%d)" >> "$config_file"
        echo "$autostart_cmd" >> "$config_file"
    fi
    
    # Validate config
    log_info "Validating compositor config..."
    if validate_compositor_config "$config_file"; then
        log_success "Config validation passed"
        log_success "Daemon auto-start configured!"
        echo ""
        echo "Restart your compositor to apply changes:"
        case "$COMPOSITOR" in
            hyprland)
                echo "  hyprctl reload"
                ;;
            sway)
                echo "  swaymsg reload"
                ;;
            i3)
                echo "  i3-msg reload"
                ;;
            river)
                echo "  Restart River"
                ;;
        esac
    else
        log_error "Config validation failed!"
        log_warning "Restoring backup..."
        mv "$backup_file" "$config_file"
        log_success "Backup restored"
        echo ""
        echo "Manual setup required. Add to $config_file:"
        echo "  $autostart_cmd"
    fi
}

# Interactive theme selection
select_theme() {
    # Skip theme selection in upgrade mode (keep existing config)
    if [ "$INSTALL_MODE" = "upgrade" ] && [ -f "$CONFIG_DIR/config.toml" ]; then
        log_info "Upgrade mode: Skipping theme selection (keeping existing theme)"
        return
    fi
    
    if [ "$INTERACTIVE" != "true" ]; then
        SELECTED_THEME="default"
        return
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Theme Selection"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Choose your preferred theme:"
    echo ""
    # Display themes with their actual colors
    echo -e "  1) Default     - \033[38;2;255;99;99m●\033[0m Coral \033[38;2;255;99;99m(#FF6363)\033[0m on \033[38;2;28;28;30m●\033[0m Charcoal \033[38;2;28;28;30m(#1C1C1E)\033[0m"
    echo -e "  2) Nord        - \033[38;2;136;192;208m●\033[0m Frost \033[38;2;136;192;208m(#88C0D0)\033[0m on \033[38;2;46;52;64m●\033[0m Polar \033[38;2;46;52;64m(#2E3440)\033[0m"
    echo -e "  3) Dracula     - \033[38;2;189;147;249m●\033[0m Purple \033[38;2;189;147;249m(#BD93F9)\033[0m on \033[38;2;40;42;54m●\033[0m Dark \033[38;2;40;42;54m(#282A36)\033[0m"
    echo -e "  4) Catppuccin  - \033[38;2;180;190;254m●\033[0m Lavender \033[38;2;180;190;254m(#B4BEFE)\033[0m on \033[38;2;30;30;46m●\033[0m Mocha \033[38;2;30;30;46m(#1E1E2E)\033[0m"
    echo -e "  5) Gruvbox     - \033[38;2;254;128;25m●\033[0m Orange \033[38;2;254;128;25m(#FE8019)\033[0m on \033[38;2;40;40;40m●\033[0m Dark \033[38;2;40;40;40m(#282828)\033[0m"
    echo -e "  6) Tokyo Night - \033[38;2;122;162;247m●\033[0m Blue \033[38;2;122;162;247m(#7AA2F7)\033[0m on \033[38;2;26;27;38m●\033[0m Night \033[38;2;26;27;38m(#1A1B26)\033[0m"
    echo ""
    read -p "Enter your choice (1-6) [default: 1]: " theme_choice < /dev/tty
    
    case ${theme_choice:-1} in
        1) SELECTED_THEME="default" ;;
        2) SELECTED_THEME="nord" ;;
        3) SELECTED_THEME="dracula" ;;
        4) SELECTED_THEME="catppuccin" ;;
        5) SELECTED_THEME="gruvbox" ;;
        6) SELECTED_THEME="tokyonight" ;;
        *)
            log_warning "Invalid choice, using default theme"
            SELECTED_THEME="default"
            ;;
    esac
    
    log_success "Selected theme: $SELECTED_THEME"
}

# Get theme colors
get_theme_colors() {
    case $SELECTED_THEME in
        nord)
            BG_COLOR="#2E3440"
            ACCENT_COLOR="#88C0D0"
            TEXT_COLOR="#ECEFF4"
            ;;
        dracula)
            BG_COLOR="#282A36"
            ACCENT_COLOR="#BD93F9"
            TEXT_COLOR="#F8F8F2"
            ;;
        catppuccin)
            BG_COLOR="#1E1E2E"
            ACCENT_COLOR="#B4BEFE"
            TEXT_COLOR="#CDD6F4"
            ;;
        gruvbox)
            BG_COLOR="#282828"
            ACCENT_COLOR="#FE8019"
            TEXT_COLOR="#EBDBB2"
            ;;
        tokyonight)
            BG_COLOR="#1A1B26"
            ACCENT_COLOR="#7AA2F7"
            TEXT_COLOR="#C0CAF5"
            ;;
        *)
            BG_COLOR="#1C1C1E"
            ACCENT_COLOR="#FF6363"
            TEXT_COLOR="#EBEBF5"
            ;;
    esac
}

# Create default configuration
create_config() {
    log_info "Creating configuration..."
    
    mkdir -p "$CONFIG_DIR"
    
    # In upgrade mode, keep existing config
    if [ "$INSTALL_MODE" = "upgrade" ] && [ -f "$CONFIG_DIR/config.toml" ]; then
        log_info "Upgrade mode: Keeping existing configuration"
        return
    fi
    
    # In fresh mode or if no config exists, create new one
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        log_warning "Configuration already exists at $CONFIG_DIR/config.toml"
        if [ "$INTERACTIVE" = "true" ] && [ "$INSTALL_MODE" != "fresh" ]; then
            read -p "Overwrite existing configuration? (y/N) " -n 1 -r < /dev/tty
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Keeping existing configuration"
                return
            fi
        fi
    fi
    
    # Get theme colors
    get_theme_colors
    
    cat > "$CONFIG_DIR/config.toml" << EOF
# Native Launcher Configuration
# Theme: $SELECTED_THEME

[window]
width = 700
height = 550
position = "top"
transparency = true

[search]
max_results = 10
fuzzy_matching = true
usage_ranking = true
min_score_threshold = 0

[ui]
show_icons = true
show_keyboard_hints = true
empty_state_on_launch = true
theme = "$SELECTED_THEME"

[theme.colors]
background = "$BG_COLOR"
accent = "$ACCENT_COLOR"
text = "$TEXT_COLOR"

[plugins]
calculator = true
files = true
web_search = true
ssh = true
editors = true
shell = true
shell_prefix = "\$"

[updater]
check_on_startup = true
auto_download = false
EOF
    
    log_success "Configuration created at $CONFIG_DIR/config.toml with $SELECTED_THEME theme"
}

# Setup compositor integration
setup_compositor_integration() {
    log_info "Setting up compositor integration..."
    
    case $COMPOSITOR in
        hyprland)
            setup_hyprland
            ;;
        sway)
            setup_sway
            ;;
        kde-wayland)
            setup_kde
            ;;
        gnome-wayland)
            setup_gnome
            ;;
        *)
            log_warning "Automatic setup not available for $COMPOSITOR"
            show_manual_setup
            ;;
    esac
}

# Setup Hyprland integration
setup_hyprland() {
    local HYPRLAND_CONFIG="$HOME/.config/hypr/hyprland.conf"
    
    if [ ! -f "$HYPRLAND_CONFIG" ]; then
        log_warning "Hyprland config not found at $HYPRLAND_CONFIG"
        return
    fi
    
    local KEYBIND="bind = SUPER, SPACE, exec, $INSTALL_DIR/native-launcher"
    
    if grep -q "native-launcher" "$HYPRLAND_CONFIG"; then
        log_info "Hyprland keybind already exists"
    else
        log_info "Adding keybind to Hyprland config..."
        echo "" >> "$HYPRLAND_CONFIG"
        echo "# Native Launcher" >> "$HYPRLAND_CONFIG"
        echo "$KEYBIND" >> "$HYPRLAND_CONFIG"
        log_success "Added Super+Space keybind to Hyprland config"
        log_warning "Run 'hyprctl reload' to apply changes"
    fi
}

# Setup Sway integration
setup_sway() {
    local SWAY_CONFIG="$HOME/.config/sway/config"
    
    if [ ! -f "$SWAY_CONFIG" ]; then
        log_warning "Sway config not found at $SWAY_CONFIG"
        return
    fi
    
    local KEYBIND="bindsym Mod4+Space exec $INSTALL_DIR/native-launcher"
    
    if grep -q "native-launcher" "$SWAY_CONFIG"; then
        log_info "Sway keybind already exists"
    else
        log_info "Adding keybind to Sway config..."
        echo "" >> "$SWAY_CONFIG"
        echo "# Native Launcher" >> "$SWAY_CONFIG"
        echo "$KEYBIND" >> "$SWAY_CONFIG"
        log_success "Added Super+Space keybind to Sway config"
        log_warning "Reload Sway config (Mod+Shift+C) to apply changes"
    fi
}

# Setup KDE integration
setup_kde() {
    log_info "For KDE Plasma, please set up the keybind manually:"
    echo ""
    echo "1. Open System Settings"
    echo "2. Go to Shortcuts → Custom Shortcuts"
    echo "3. Add new → Global Shortcut → Command/URL"
    echo "4. Set trigger: Meta+Space"
    echo "5. Set command: $INSTALL_DIR/native-launcher"
}

# Setup GNOME integration
setup_gnome() {
    log_info "For GNOME, setting up custom keybind..."
    
    # Try to set up using gsettings
    if command -v gsettings >/dev/null 2>&1; then
        gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/native-launcher/']"
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/native-launcher/ name 'Native Launcher'
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/native-launcher/ command "$INSTALL_DIR/native-launcher"
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/native-launcher/ binding '<Super>space'
        log_success "GNOME keybind configured"
    else
        log_warning "gsettings not found. Please configure keybind manually in Settings → Keyboard"
    fi
}

# Show manual setup instructions
show_manual_setup() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Manual Setup Instructions:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Add the following keybind to your compositor configuration:"
    echo ""
    echo "  Command: $INSTALL_DIR/native-launcher"
    echo "  Keybind: Super+Space (or your preferred key)"
    echo ""
    echo "Configuration file locations:"
    echo "  - Hyprland: ~/.config/hypr/hyprland.conf"
    echo "  - Sway: ~/.config/sway/config"
    echo "  - River: ~/.config/river/init"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    if [ ! -f "$INSTALL_DIR/native-launcher" ]; then
        log_error "Binary not found at $INSTALL_DIR/native-launcher"
        return 1
    fi
    
    if [ ! -x "$INSTALL_DIR/native-launcher" ]; then
        log_error "Binary is not executable"
        return 1
    fi
    
    # Check if directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warning "$INSTALL_DIR is not in your PATH"
        log_info "Adding $INSTALL_DIR to PATH in shell config..."
        
        # Detect shell config file
        if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
            SHELL_RC="$HOME/.zshrc"
        elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
            SHELL_RC="$HOME/.bashrc"
        else
            SHELL_RC="$HOME/.profile"
        fi
        
        # Add to PATH if not already there
        if ! grep -q "export PATH=\"\$HOME/.local/bin:\$PATH\"" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Added by native-launcher installer" >> "$SHELL_RC"
            echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$SHELL_RC"
            log_success "Added to PATH in $SHELL_RC"
            log_info "Run: source $SHELL_RC (or restart your terminal)"
        fi
        
        # Add to current session
        export PATH="$HOME/.local/bin:$PATH"
    fi
    
    log_success "Installation verified successfully!"
    return 0
}

# Print completion message
print_completion() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}Native Launcher Installation Complete!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Installation Summary:"
    echo "  Version: $LATEST_VERSION"
    echo "  Theme: $SELECTED_THEME"
    echo "  Binary: $INSTALL_DIR/native-launcher"
    echo "  Config: $CONFIG_DIR/config.toml"
    
    # Show daemon status
    local config_file=$(get_compositor_config_path)
    if [ -n "$config_file" ] && [ -f "$config_file" ] && grep -q "native-launcher.*--daemon" "$config_file" 2>/dev/null; then
        echo "  Daemon: Enabled (compositor auto-start)"
    elif pgrep -f "native-launcher.*--daemon" >/dev/null 2>&1; then
        echo "  Daemon: Running (manual)"
    else
        echo "  Daemon: Disabled"
    fi
    
    echo ""
    echo "Quick Start:"
    echo "  1. Press Super+Space to launch (if keybind configured)"
    echo "  2. Or run: $INSTALL_DIR/native-launcher"
    echo "  3. Edit config: $CONFIG_DIR/config.toml"
    
    # Add daemon management tips if enabled
    if [ -n "$config_file" ] && [ -f "$config_file" ] && grep -q "native-launcher.*--daemon" "$config_file" 2>/dev/null; then
        echo ""
        echo "Daemon Management:"
        echo "  Config: $config_file"
        echo "  To disable: Remove daemon line from config and restart compositor"
        echo "  Check status: pgrep -f 'native-launcher.*--daemon'"
    fi
    
    echo ""
    echo "Documentation: https://github.com/$GITHUB_REPO/wiki"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Main installation flow
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Native Launcher - Automated Installation"
    echo "  Version: $SCRIPT_VERSION"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Detect system
    detect_distro
    detect_compositor
    
    # Choose installation mode (upgrade vs fresh)
    choose_install_mode
    
    # Backup existing installation before making changes
    backup_existing_installation
    
    # Clean existing installation if fresh mode
    clean_existing_installation
    
    # Stop daemon before installation
    stop_daemon
    
    # Check and install dependencies
    check_dependencies
    install_gtk_dependencies
    
    # Get latest release and install
    get_latest_release
    download_and_install
    
    # Interactive theme selection
    select_theme
    
    # Create configuration
    create_config
    
    # Setup compositor integration
    setup_compositor_integration
    
    # Setup compositor auto-start (daemon mode)
    setup_compositor_autostart
    
    # Restart daemon if it was running before
    restart_daemon
    
    # Verify installation
    if verify_installation; then
        print_completion
    else
        log_error "Installation verification failed"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
