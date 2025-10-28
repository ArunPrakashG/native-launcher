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
        read -p "Create backup before installing? (Y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            log_warning "Skipping backup (not recommended)"
            return 0
        fi
    fi
    
    # Create backup directory
    mkdir -p "$backup_dir"
    log_info "Creating backup at $backup_dir..."
    
    # Backup binary
    if [ -f "$INSTALL_DIR/native-launcher" ]; then
        cp "$INSTALL_DIR/native-launcher" "$backup_dir/native-launcher"
        log_success "Backed up binary"
    fi
    
    # Backup configuration
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        mkdir -p "$backup_dir/config"
        cp "$CONFIG_DIR/config.toml" "$backup_dir/config/config.toml"
        log_success "Backed up configuration"
    fi
    
    # Backup plugins directory
    if [ -d "$CONFIG_DIR/plugins" ]; then
        cp -r "$CONFIG_DIR/plugins" "$backup_dir/config/"
        log_success "Backed up plugins"
    fi
    
    # Backup cache
    if [ -d "$HOME/.cache/native-launcher" ]; then
        mkdir -p "$backup_dir/cache"
        cp -r "$HOME/.cache/native-launcher/"* "$backup_dir/cache/" 2>/dev/null || true
        log_success "Backed up cache"
    fi
    
    # Backup usage data
    if [ -d "$HOME/.local/share/native-launcher" ]; then
        mkdir -p "$backup_dir/data"
        cp -r "$HOME/.local/share/native-launcher/"* "$backup_dir/data/" 2>/dev/null || true
        log_success "Backed up usage data"
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

# Interactive theme selection
select_theme() {
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
    echo "  1) Default    - Coral (#FF6363) on Charcoal (#1C1C1E)"
    echo "  2) Nord       - Nord Frost (#88C0D0) on Nord Polar (#2E3440)"
    echo "  3) Dracula    - Dracula Purple (#BD93F9) on Dark (#282A36)"
    echo "  4) Catppuccin - Catppuccin Lavender (#B4BEFE) on Mocha (#1E1E2E)"
    echo "  5) Gruvbox    - Gruvbox Orange (#FE8019) on Dark (#282828)"
    echo "  6) Tokyo Night- Tokyo Night Blue (#7AA2F7) on Night (#1A1B26)"
    echo ""
    read -p "Enter your choice (1-6) [default: 1]: " theme_choice
    
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
    
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        log_warning "Configuration already exists at $CONFIG_DIR/config.toml"
        if [ "$INTERACTIVE" = "true" ]; then
            read -p "Overwrite existing configuration? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Keeping existing configuration"
                return
            fi
        else
            log_info "Keeping existing configuration (non-interactive mode)"
            return
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
        log_info "Add the following to your ~/.bashrc or ~/.zshrc:"
        echo ""
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
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
    echo ""
    echo "Quick Start:"
    echo "  1. Press Super+Space to launch (if keybind configured)"
    echo "  2. Or run: $INSTALL_DIR/native-launcher"
    echo "  3. Edit config: $CONFIG_DIR/config.toml"
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
    
    # Detect system
    detect_distro
    detect_compositor
    
    # Backup existing installation before making changes
    backup_existing_installation
    
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
    
    # Verify installation
    if verify_installation; then
        print_completion
    else
        log_error "Installation verification failed"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Native Launcher Installation Script"
        echo ""
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --help, -h          Show this help message"
        echo "  --version, -v       Show script version"
        echo "  --non-interactive   Skip interactive prompts (use defaults)"
        echo ""
        echo "This script will:"
        echo "  1. Detect your system and compositor"
        echo "  2. Install required dependencies"
        echo "  3. Download the latest release"
        echo "  4. Install the binary to ~/.local/bin"
        echo "  5. Let you choose a theme (interactive mode)"
        echo "  6. Create configuration with selected theme"
        echo "  7. Setup compositor keybinds (if supported)"
        exit 0
        ;;
    --version|-v)
        echo "Native Launcher Installation Script v$SCRIPT_VERSION"
        exit 0
        ;;
    --non-interactive)
        INTERACTIVE=false
        main
        ;;
    *)
        main
        ;;
esac
