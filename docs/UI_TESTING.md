# UI Testing Guide

## Overview

This guide explains how to test the Native Launcher GTK4 user interface. UI testing is crucial for ensuring that visual components work correctly across different environments and that user interactions behave as expected.

## Table of Contents

- [Testing Approaches](#testing-approaches)
- [Quick Start](#quick-start)
- [Running Tests](#running-tests)
- [Writing UI Tests](#writing-ui-tests)
- [Headless Testing](#headless-testing)
- [Visual Regression Testing](#visual-regression-testing)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

---

## Testing Approaches

Native Launcher uses multiple testing strategies for comprehensive UI coverage:

### 1. **Unit Tests for UI Components**

Test individual widgets in isolation:

- Widget creation and initialization
- Property setters and getters
- CSS class application
- Signal connections

**Location**: `tests/ui_tests.rs`

### 2. **Integration Tests**

Test complete user workflows:

- Search → select → launch flow
- Keyboard navigation
- Plugin interactions
- State management

**Location**: `tests/ui_tests.rs` + `tests/desktop_tests.rs`

### 3. **Headless Testing**

Run GTK4 tests without a display using Xvfb:

- CI/CD pipeline testing
- Automated testing environments
- Docker containers

**Tool**: `scripts/test-ui.sh`

### 4. **Manual Testing**

Interactive testing on real compositors:

- Visual appearance
- Animations and transitions
- Performance under load
- Compositor-specific features

**Guide**: See [Manual Testing Checklist](#manual-testing-checklist)

---

## Quick Start

### Prerequisites

**Required**:

```bash
# Install GTK4 development libraries
# Arch Linux
sudo pacman -S gtk4

# Ubuntu/Debian
sudo apt install libgtk-4-dev

# Fedora
sudo dnf install gtk4-devel
```

**Optional (for headless testing)**:

```bash
# Install Xvfb virtual framebuffer
# Arch Linux
sudo pacman -S xorg-server-xvfb

# Ubuntu/Debian
sudo apt install xvfb

# Fedora
sudo dnf install xorg-x11-server-Xvfb
```

### Run All Tests

```bash
# With display (Wayland/X11)
cargo test --test ui_tests

# Headless (using Xvfb script)
./scripts/test-ui.sh ui
```

---

## Running Tests

### UI Tests Only

```bash
# Run UI tests with visible display
cargo test --test ui_tests

# Run with logging
RUST_LOG=debug cargo test --test ui_tests -- --nocapture

# Run specific test
cargo test --test ui_tests test_search_widget_creation
```

### All Tests

```bash
# Using test script (recommended)
./scripts/test-ui.sh all

# Or manually
cargo test
```

### Test Output

```
running 15 tests
test test_keyboard_hints_creation ... ok
test test_results_list_clear ... ok
test test_results_list_creation ... ok
test test_results_list_navigation ... ok
test test_results_list_update_results ... ok
test test_results_list_with_actions ... ok
test test_search_widget_connect_changed ... ok
test test_search_widget_creation ... ok
test test_search_widget_grab_focus ... ok
test test_search_widget_text_input ... ok
test test_terminal_app_flag ... ok
test test_widget_css_classes ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

---

## Writing UI Tests

### Basic Widget Test

```rust
use gtk4::prelude::*;
use native_launcher::ui::SearchWidget;

#[test]
fn test_my_widget() {
    run_gtk_test(|| {
        let widget = SearchWidget::default();

        // Test widget creation
        assert!(widget.container.is_visible() || !widget.container.is_visible());

        // Test widget properties
        widget.entry.set_text("test");
        assert_eq!(widget.entry.text(), "test");
    });
}
```

### Test with User Interaction

```rust
#[test]
fn test_user_interaction() {
    run_gtk_test(|| {
        use native_launcher::ui::ResultsList;
        use native_launcher::desktop::DesktopEntry;
        use std::path::PathBuf;

        let results_list = ResultsList::new();

        // Setup test data
        let entry = DesktopEntry {
            name: "Firefox".to_string(),
            exec: "firefox".to_string(),
            // ... other fields
        };

        // Simulate user action
        results_list.update_results(&[entry]);
        results_list.select_next();

        // Verify result
        let command = results_list.get_selected_command();
        assert!(command.is_some());
    });
}
```

### Test Signal Connections

```rust
#[test]
fn test_signal_handling() {
    run_gtk_test(|| {
        use std::cell::RefCell;
        use std::rc::Rc;

        let search_widget = SearchWidget::default();
        let triggered = Rc::new(RefCell::new(false));

        let triggered_clone = triggered.clone();
        search_widget.entry.connect_changed(move |_| {
            *triggered_clone.borrow_mut() = true;
        });

        search_widget.entry.set_text("test");

        // Process events
        while gtk4::glib::MainContext::default().iteration(false) {}

        assert!(*triggered.borrow());
    });
}
```

### Test Helper Function

All UI tests should use the `run_gtk_test` helper:

```rust
fn run_gtk_test<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    init_gtk();

    let (tx, rx) = std::sync::mpsc::channel();

    glib::idle_add_once(move || {
        test_fn();
        tx.send(()).unwrap();
    });

    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("Test timed out");
}
```

**Why?**

- Ensures GTK is initialized once
- Runs tests on GTK main thread
- Provides timeout protection
- Handles event loop properly

---

## Headless Testing

### Using test-ui.sh Script

```bash
# Run all UI tests headlessly
./scripts/test-ui.sh ui

# Run specific test suite
./scripts/test-ui.sh unit        # Unit tests only
./scripts/test-ui.sh integration # Integration tests only
./scripts/test-ui.sh all         # Everything
```

### Manual Xvfb Setup

```bash
# Start virtual display
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99

# Run tests
cargo test --test ui_tests

# Cleanup
killall Xvfb
```

### Docker Testing

```dockerfile
# Dockerfile for UI testing
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    libgtk-4-dev \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Run tests
CMD ["sh", "-c", "Xvfb :99 & export DISPLAY=:99 && cargo test --test ui_tests"]
```

```bash
# Build and run
docker build -t native-launcher-test .
docker run native-launcher-test
```

---

## Visual Regression Testing

### Concept

Visual regression testing captures screenshots of UI components and compares them against baseline images to detect unintended visual changes.

### Future Implementation

**Planned approach using GTK snapshot API**:

```rust
// Future: tests/visual_tests.rs
use gtk4::prelude::*;
use gtk4::Snapshot;

#[test]
fn test_search_widget_appearance() {
    run_gtk_test(|| {
        let widget = SearchWidget::default();

        // Create snapshot
        let snapshot = Snapshot::new();
        widget.container.snapshot(&snapshot);

        // Convert to image and compare with baseline
        let image = snapshot.to_paintable(None);
        // ... comparison logic
    });
}
```

**Tools to integrate**:

- `insta` - Snapshot testing for Rust
- `image` - Image processing
- `imageproc` - Image comparison

**Example workflow**:

```bash
# Create baseline snapshots
cargo test --test visual_tests -- --update-snapshots

# Run visual regression tests
cargo test --test visual_tests

# Review differences
cargo insta review
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/ui-tests.yml
name: UI Tests

on: [push, pull_request]

jobs:
  ui-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install GTK4
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-4-dev xvfb

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run UI tests
        run: |
          Xvfb :99 -screen 0 1920x1080x24 &
          export DISPLAY=:99
          cargo test --test ui_tests

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: target/debug/test-results/
```

### GitLab CI

```yaml
# .gitlab-ci.yml
ui-tests:
  image: rust:latest

  before_script:
    - apt-get update
    - apt-get install -y libgtk-4-dev xvfb

  script:
    - Xvfb :99 -screen 0 1920x1080x24 &
    - export DISPLAY=:99
    - cargo test --test ui_tests

  artifacts:
    reports:
      junit: target/test-results/junit.xml
```

---

## Manual Testing Checklist

### Visual Appearance

- [ ] Window appears with correct dimensions (800x600 default)
- [ ] Background color is charcoal (#1C1C1E)
- [ ] Accent color is coral (#FF6363) on selected items
- [ ] Text is off-white (#EBEBF5)
- [ ] Rounded corners (16px window, 10px inputs)
- [ ] No visual glitches or artifacts

### Search Widget

- [ ] Placeholder text visible when empty
- [ ] Text input responsive to typing
- [ ] Focus indicator visible
- [ ] Clear button works (if implemented)
- [ ] Cursor visible and positioned correctly

### Results List

- [ ] First result auto-selected on search
- [ ] Selected item has coral background
- [ ] Icons load correctly (if enabled)
- [ ] App names and descriptions visible
- [ ] Desktop actions display inline under parent app
- [ ] Action items indented 24px
- [ ] Action items have coral text color

### Keyboard Navigation

- [ ] ↓ Down arrow selects next item
- [ ] ↑ Up arrow selects previous item
- [ ] Enter launches selected app and closes window
- [ ] Escape closes window
- [ ] Ctrl+Enter triggers web search (if query present)

### Animations

- [ ] Smooth transitions (0.15s cubic-bezier)
- [ ] No janky or stuttering animations
- [ ] Fade effects smooth
- [ ] No flash of unstyled content

### Performance

- [ ] Window appears in <100ms
- [ ] Search updates in <10ms
- [ ] No input lag when typing
- [ ] Smooth scrolling in results
- [ ] No memory leaks (monitor with `heaptrack`)

### Compositor-Specific

**Sway**:

- [ ] Window appears as overlay
- [ ] Keyboard focus exclusive
- [ ] No window decorations
- [ ] Correct layer (overlay)

**Hyprland**:

- [ ] Animations respect compositor settings
- [ ] Blur effects (if enabled)
- [ ] Proper focus behavior

**GNOME**:

- [ ] Works with Mutter compositor
- [ ] No interference with activities overview
- [ ] Proper scaling on HiDPI displays

---

## Troubleshooting

### Test Failures

**"Failed to initialize GTK"**

```bash
# Ensure GTK4 is installed
pkg-config --modversion gtk4

# Check display is available
echo $DISPLAY

# Try with Xvfb
./scripts/test-ui.sh ui
```

**"Test timed out"**

```rust
// Increase timeout in run_gtk_test helper
rx.recv_timeout(std::time::Duration::from_secs(10))
```

**"Widget not realized"**

```rust
// Ensure widget is shown before testing
widget.container.show();

// Or wait for realization
while !widget.container.is_realized() {
    gtk4::glib::MainContext::default().iteration(true);
}
```

### Xvfb Issues

**"Xvfb already running"**

```bash
# Kill existing Xvfb
killall Xvfb

# Use different display
Xvfb :100 -screen 0 1920x1080x24 &
export DISPLAY=:100
```

**"Can't open display"**

```bash
# Check Xvfb is running
ps aux | grep Xvfb

# Verify DISPLAY is set
echo $DISPLAY

# Wait longer for Xvfb to start
sleep 3
```

### CI/CD Issues

**"GTK4 not found in CI"**

- Update CI config to install GTK4 development packages
- Use appropriate package manager for CI environment
- Consider using Docker with pre-installed dependencies

**"Tests flaky in CI"**

- Increase timeouts
- Add `sleep` delays between steps
- Use `--test-threads=1` to run tests sequentially
- Check for race conditions in test setup

---

## Best Practices

### DO ✅

- Use `run_gtk_test` helper for all UI tests
- Test one widget property per test
- Use descriptive test names
- Clean up resources (signals, connections)
- Run tests with `--test-threads=1` for UI tests
- Mock external dependencies
- Test edge cases (empty lists, long text, etc.)
- Verify CSS classes are applied

### DON'T ❌

- Don't rely on timing (use signals/callbacks)
- Don't test GTK internals (test your code)
- Don't create windows in unit tests (use widgets)
- Don't forget to initialize GTK before tests
- Don't share state between tests
- Don't test visual appearance programmatically (use visual tests)
- Don't ignore flaky tests (fix them)

---

## Performance Testing

### Measure Widget Creation

```rust
#[test]
fn benchmark_widget_creation() {
    use std::time::Instant;

    run_gtk_test(|| {
        let start = Instant::now();

        for _ in 0..100 {
            let _widget = SearchWidget::default();
        }

        let elapsed = start.elapsed();
        let avg = elapsed / 100;

        println!("Average widget creation: {:?}", avg);
        assert!(avg.as_millis() < 10, "Widget creation too slow");
    });
}
```

### Monitor Memory

```bash
# Use valgrind for memory leaks
valgrind --leak-check=full cargo test --test ui_tests

# Use heaptrack for memory profiling
heaptrack cargo test --test ui_tests
heaptrack_gui heaptrack.*.gz
```

---

## Future Enhancements

### Planned Features

1. **Visual Regression Testing**

   - Screenshot capture API
   - Baseline image storage
   - Automated comparison
   - Diff visualization

2. **Accessibility Testing**

   - AT-SPI tree validation
   - Screen reader compatibility
   - Keyboard-only navigation tests
   - ARIA attributes verification

3. **Performance Benchmarks**

   - Criterion.rs integration
   - Render time measurements
   - Memory usage tracking
   - Frame rate monitoring

4. **Interactive Test Mode**
   - Launch app in test mode
   - Scripted user interactions
   - Record and playback sessions
   - Visual debugging

---

## Resources

### Documentation

- [GTK4 Testing Guide](https://docs.gtk.org/gtk4/testing.html)
- [gtk-rs Book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
- [Xvfb Documentation](https://www.x.org/releases/X11R7.6/doc/man/man1/Xvfb.1.xhtml)

### Tools

- [cargo-test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) - Rust test runner
- [Xvfb](https://www.x.org/releases/X11R7.6/doc/man/man1/Xvfb.1.xhtml) - Virtual framebuffer
- [insta](https://insta.rs/) - Snapshot testing
- [criterion](https://bheisler.github.io/criterion.rs/book/) - Benchmarking

### Examples

- `tests/ui_tests.rs` - UI test examples
- `scripts/test-ui.sh` - Headless testing script
- `tests/desktop_tests.rs` - Integration test patterns

---

**Last Updated**: October 25, 2025  
**Maintainer**: Native Launcher Team  
**Status**: Active Development
