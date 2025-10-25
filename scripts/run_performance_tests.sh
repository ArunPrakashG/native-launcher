#!/usr/bin/env bash
set -e

echo "🔍 Native Launcher - Performance Test Suite"
echo "============================================"
echo ""

# Check if release build exists
if [ ! -f "target/release/native-launcher" ]; then
    echo "⚠️  Release build not found. Building..."
    cargo build --release
    echo ""
fi

echo "📊 Running Performance Tests..."
echo ""

# Test 1: Typing performance (most important)
echo "1️⃣  Typing Performance Test (target: <16ms per keystroke)"
echo "   With debouncing, this measures search latency after pause"
echo "   ────────────────────────────────────────────────────────"
cargo test --release --test performance_tests test_typing_performance_target -- --nocapture 2>&1 | tail -20
echo ""

# Test 2: Short query performance  
echo "2️⃣  Short Query Performance (1-2 chars, target: <5ms)"
echo "   These should be very fast (no file index)"
echo "   ────────────────────────────────────────────────────────"
cargo test --release --test performance_tests test_short_query_performance -- --nocapture 2>&1 | tail -15
echo ""

# Test 3: App search performance
echo "3️⃣  App Search Performance (target: <10ms)"
echo "   Application searches should be fast"
echo "   ────────────────────────────────────────────────────────"
cargo test --release --test performance_tests test_app_search_performance -- --nocapture 2>&1 | tail -15
echo ""

# Test 4: Cache performance
echo "4️⃣  File Index Cache Performance (target: <5ms cached)"
echo "   Repeated searches should hit cache"
echo "   ────────────────────────────────────────────────────────"
cargo test --release --test performance_tests test_file_index_cache_performance -- --nocapture 2>&1 | tail -15
echo ""

# Test 5: Progressive analysis
echo "5️⃣  Progressive Typing Analysis (detailed breakdown)"
echo "   Shows latency for each character typed"
echo "   ────────────────────────────────────────────────────────"
cargo test --release --test performance_tests test_progressive_typing_analysis -- --nocapture 2>&1 | tail -40
echo ""

echo "============================================"
echo "✅ Performance Testing Complete"
echo ""
echo "📝 Key Metrics:"
echo "   • Target: <16ms per keystroke (60fps)"
echo "   • Stretch: <8ms per keystroke (120fps)"
echo "   • With debouncing: 0ms blocking during typing"
echo ""
echo "🔧 To run detailed benchmarks (slower, more accurate):"
echo "   cargo bench --bench input_latency_bench"
echo ""
echo "📖 Documentation:"
echo "   • Analysis: docs/INPUT_LAG_ANALYSIS.md"
echo "   • Fix: docs/INPUT_LAG_FIX.md"
