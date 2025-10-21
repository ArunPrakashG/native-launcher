#!/usr/bin/env bash
# Memory profiling script for Native Launcher

echo "=== Memory Profiling for Native Launcher ==="
echo

# Build release if not already built
if [ ! -f "target/release/native-launcher" ]; then
    echo "Building release binary..."
    cargo build --release
    echo
fi

# Check binary size
echo "Binary size:"
ls -lh target/release/native-launcher | awk '{print $5, $9}'
echo

# Run with time measurement (startup only, exit immediately)
echo "Quick startup test (loads and exits):"
echo "Note: This measures initialization without GTK window creation"
echo

# Measure with hyperfine if available
if command -v hyperfine &> /dev/null; then
    echo "Running hyperfine benchmark (10 runs)..."
    hyperfine --runs 10 --warmup 2 \
        --export-markdown /tmp/native-launcher-startup.md \
        'timeout 0.5s ./target/release/native-launcher || true'
    echo
    cat /tmp/native-launcher-startup.md
    echo
fi

# Measure with time (more detailed)
echo "Memory usage with /usr/bin/time:"
/usr/bin/time -v timeout 1s ./target/release/native-launcher 2>&1 | grep -E "Maximum resident|User time|System time|Elapsed|Percent of CPU"
echo

# Check cache sizes
echo "Cache directory sizes:"
if [ -d "$HOME/.cache/native-launcher" ]; then
    du -sh "$HOME/.cache/native-launcher"
    du -sh "$HOME/.cache/native-launcher"/*
else
    echo "Cache directory not found"
fi
echo

# Memory profiling with valgrind if available
if command -v valgrind &> /dev/null; then
    echo "Running valgrind massif (heap profiling)..."
    echo "This may take a minute..."
    valgrind --tool=massif --massif-out-file=/tmp/native-launcher.massif \
        timeout 2s ./target/release/native-launcher 2>&1 | grep -E "total heap usage"
    
    if command -v ms_print &> /dev/null && [ -f /tmp/native-launcher.massif ]; then
        echo
        echo "Peak memory usage:"
        ms_print /tmp/native-launcher.massif | head -30 | tail -10
    fi
fi

echo
echo "=== Profiling Complete ==="
