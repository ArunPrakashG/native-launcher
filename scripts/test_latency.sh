#!/usr/bin/env bash

# Input Latency Measurement Script
# This script measures search latency to identify performance issues

echo "🔍 Native Launcher - Input Latency Test"
echo "======================================="
echo ""

# Build the project
echo "📦 Building project..."
cargo build --release --quiet 2>&1 | grep -E "error|warning" || echo "✅ Build successful"
echo ""

# Run performance tests
echo "🧪 Running Performance Tests..."
echo ""

echo "1️⃣  Testing keystroke latency (target: <16ms)..."
cargo test --release --test performance_tests test_typing_performance_target -- --nocapture 2>&1

echo ""
echo "2️⃣  Testing short query performance (target: <5ms)..."
cargo test --release --test performance_tests test_short_query_performance -- --nocapture 2>&1

echo ""
echo "3️⃣  Testing app search performance (target: <10ms)..."
cargo test --release --test performance_tests test_app_search_performance -- --nocapture 2>&1

echo ""
echo "4️⃣  Testing file index cache (target: <5ms cached)..."
cargo test --release --test performance_tests test_file_index_cache_performance -- --nocapture 2>&1

echo ""
echo "5️⃣  Progressive typing analysis..."
cargo test --release --test performance_tests test_progressive_typing_analysis -- --nocapture 2>&1

echo ""
echo "======================================="
echo "📊 Test Complete"
echo ""
echo "Expected Results:"
echo "  ✅ <16ms per keystroke (60fps)"
echo "  🎯 <8ms per keystroke (120fps goal)"
echo ""
echo "If tests fail, the issue is likely:"
echo "  1. File index search on every keystroke (>=3 chars)"
echo "  2. No debouncing on search input"
echo "  3. Blocking I/O in UI thread"
