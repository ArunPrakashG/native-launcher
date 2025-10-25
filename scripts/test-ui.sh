#!/usr/bin/env bash
# Headless UI Testing Script for Native Launcher
# Runs GTK4 UI tests in a virtual display environment

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Native Launcher UI Testing ===${NC}\n"

# Check if running in CI or headless environment
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    echo -e "${YELLOW}No display detected. Setting up virtual display...${NC}"
    HEADLESS=true
else
    echo -e "${GREEN}Display detected: ${DISPLAY:-$WAYLAND_DISPLAY}${NC}"
    HEADLESS=false
fi

# Function to setup virtual display
setup_virtual_display() {
    echo "Setting up Xvfb virtual display..."
    
    # Check if Xvfb is installed
    if ! command -v Xvfb &> /dev/null; then
        echo -e "${RED}Error: Xvfb not found. Install it:${NC}"
        echo "  Arch: sudo pacman -S xorg-server-xvfb"
        echo "  Ubuntu/Debian: sudo apt install xvfb"
        echo "  Fedora: sudo dnf install xorg-x11-server-Xvfb"
        exit 1
    fi
    
    # Start Xvfb on display :99
    export DISPLAY=:99
    Xvfb :99 -screen 0 1920x1080x24 &
    XVFB_PID=$!
    
    # Wait for Xvfb to start
    sleep 2
    
    echo -e "${GREEN}Xvfb started on display :99 (PID: $XVFB_PID)${NC}"
    
    # Ensure cleanup on exit
    trap "kill $XVFB_PID 2>/dev/null || true" EXIT
}

# Function to run tests
run_tests() {
    local test_type="${1:-all}"
    
    cd "$PROJECT_ROOT"
    
    case "$test_type" in
        ui)
            echo -e "\n${GREEN}Running UI tests...${NC}"
            cargo test --test ui_tests -- --test-threads=1
            ;;
        unit)
            echo -e "\n${GREEN}Running unit tests...${NC}"
            cargo test --lib
            ;;
        integration)
            echo -e "\n${GREEN}Running integration tests...${NC}"
            cargo test --test desktop_tests
            cargo test --test advanced_calc_tests
            ;;
        all)
            echo -e "\n${GREEN}Running all tests...${NC}"
            cargo test --test ui_tests -- --test-threads=1
            cargo test --lib
            cargo test --test desktop_tests
            cargo test --test advanced_calc_tests
            ;;
        *)
            echo -e "${RED}Unknown test type: $test_type${NC}"
            echo "Usage: $0 [ui|unit|integration|all]"
            exit 1
            ;;
    esac
}

# Function to run visual regression tests
run_visual_tests() {
    echo -e "\n${GREEN}Running visual regression tests...${NC}"
    
    # This would integrate with screenshot comparison tools
    # For now, just a placeholder
    echo "Visual regression testing not yet implemented"
    echo "Future: Will use GTK snapshot API to capture and compare UI"
}

# Main execution
main() {
    local test_type="${1:-all}"
    
    # Setup virtual display if headless
    if [ "$HEADLESS" = true ]; then
        setup_virtual_display
    fi
    
    # Set GTK environment for testing
    export GTK_A11Y=none  # Disable accessibility for faster tests
    export G_DEBUG=fatal-warnings  # Treat warnings as fatal
    
    # Run tests
    run_tests "$test_type"
    
    # Check if visual tests requested
    if [ "$test_type" = "visual" ] || [ "$test_type" = "all" ]; then
        run_visual_tests
    fi
    
    echo -e "\n${GREEN}=== All tests completed successfully! ===${NC}"
}

# Run main with arguments
main "$@"
