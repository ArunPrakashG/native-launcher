use native_launcher::config::Config;
use native_launcher::desktop::{DesktopEntryArena, DesktopScanner};
use native_launcher::plugins::PluginManager;
use native_launcher::usage::UsageTracker;
use std::time::{Duration, Instant};

fn should_skip_perf() -> bool {
    cfg!(debug_assertions) && std::env::var("NL_STRICT_PERF").is_err()
}

/// Performance test: Measure input lag during typing simulation
#[test]
fn test_typing_performance_target() {
    if should_skip_perf() {
        eprintln!(
            "Skipping typing performance test in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
        );
        return;
    }
    // Target: Each keystroke should complete search in <16ms (60fps)
    // Stretch goal: <8ms (120fps)
    const MAX_KEYSTROKE_LATENCY_MS: u128 = 16;
    const STRETCH_GOAL_MS: u128 = 8;

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

    // Simulate typing "config.txt" (triggers file index at char 3)
    let query = "config.txt";
    let mut violations = Vec::new();
    let mut cumulative = String::new();

    for (i, ch) in query.chars().enumerate() {
        cumulative.push(ch);

        let start = Instant::now();
        let _results = plugin_manager.search(&cumulative, 10);
        let elapsed = start.elapsed();

        let elapsed_ms = elapsed.as_millis();

        println!(
            "Keystroke {}: '{}' → {}ms ({})",
            i + 1,
            cumulative,
            elapsed_ms,
            if elapsed_ms > MAX_KEYSTROKE_LATENCY_MS {
                "❌ TOO SLOW"
            } else if elapsed_ms > STRETCH_GOAL_MS {
                "⚠️  ACCEPTABLE"
            } else {
                "✅ FAST"
            }
        );

        if elapsed_ms > MAX_KEYSTROKE_LATENCY_MS {
            violations.push((cumulative.clone(), elapsed_ms));
        }
    }

    // Report results
    if violations.is_empty() {
        println!("\n✅ All keystrokes meet <16ms latency target!");
    } else {
        println!("\n❌ {} keystrokes exceeded 16ms target:", violations.len());
        for (query, latency) in &violations {
            println!("  - '{}': {}ms", query, latency);
        }

        panic!(
            "Input lag detected: {} keystrokes > {}ms (max: {}ms)",
            violations.len(),
            MAX_KEYSTROKE_LATENCY_MS,
            violations.iter().map(|(_, l)| l).max().unwrap()
        );
    }
}

/// Test: File index search should be fast when cached
#[test]
fn test_file_index_cache_performance() {
    if should_skip_perf() {
        eprintln!(
            "Skipping cache performance test in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
        );
        return;
    }
    const MAX_CACHED_SEARCH_MS: u128 = 5;

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

    // First search (cache miss, can be slow)
    let query = "config";
    let start = Instant::now();
    let _results = plugin_manager.search(query, 10);
    let first_search = start.elapsed();

    println!("First search (cache miss): {}ms", first_search.as_millis());

    // Wait briefly to ensure cache doesn't expire (TTL is 120s)
    std::thread::sleep(Duration::from_millis(100));

    // Second search (should hit cache)
    let start = Instant::now();
    let _results = plugin_manager.search(query, 10);
    let cached_search = start.elapsed();

    println!("Cached search (hit): {}ms", cached_search.as_millis());

    assert!(
        cached_search.as_millis() < MAX_CACHED_SEARCH_MS,
        "Cached search took {}ms, expected < {}ms",
        cached_search.as_millis(),
        MAX_CACHED_SEARCH_MS
    );
}

/// Test: Short queries (< 3 chars) should be very fast (no file index)
#[test]
fn test_short_query_performance() {
    if should_skip_perf() {
        eprintln!(
            "Skipping short query performance test in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
        );
        return;
    }
    const MAX_SHORT_QUERY_MS: u128 = 5;

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

    let short_queries = vec!["f", "fi", "fir"];

    for query in short_queries {
        let start = Instant::now();
        let _results = plugin_manager.search(query, 10);
        let elapsed = start.elapsed();

        println!("Query '{}': {}ms", query, elapsed.as_millis());

        assert!(
            elapsed.as_millis() < MAX_SHORT_QUERY_MS,
            "Short query '{}' took {}ms, expected < {}ms",
            query,
            elapsed.as_millis(),
            MAX_SHORT_QUERY_MS
        );
    }
}

/// Test: App-only searches should be fast (no file index trigger)
#[test]
fn test_app_search_performance() {
    if should_skip_perf() {
        eprintln!(
            "Skipping app search performance test in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
        );
        return;
    }
    const MAX_APP_SEARCH_MS: u128 = 10;

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

    // Queries that should match apps, not trigger file search
    let app_queries = vec!["firefox", "chrome", "terminal"];

    for query in app_queries {
        let start = Instant::now();
        let _results = plugin_manager.search(query, 10);
        let elapsed = start.elapsed();

        println!("App query '{}': {}ms", query, elapsed.as_millis());

        assert!(
            elapsed.as_millis() < MAX_APP_SEARCH_MS,
            "App search '{}' took {}ms, expected < {}ms",
            query,
            elapsed.as_millis(),
            MAX_APP_SEARCH_MS
        );
    }
}

/// Test: Measure worst-case file index latency (cache miss)
#[test]
#[ignore] // Slow test, run with: cargo test --test performance_tests -- --ignored --nocapture
fn test_file_index_worst_case() {
    const MAX_FILE_INDEX_MS: u128 = 500; // Generous limit for find fallback

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();

    // Test multiple unique queries to avoid caching
    let file_queries = vec!["config123", "bashrc456", "document789", "readme012"];

    let mut latencies = Vec::new();

    for query in file_queries {
        // Fresh plugin manager to avoid cache
        let plugin_manager = PluginManager::new(
            entry_arena.clone(),
            Some(usage_tracker.clone()),
            None,
            &config,
        );

        let start = Instant::now();
        let _results = plugin_manager.search(query, 10);
        let elapsed = start.elapsed();

        let elapsed_ms = elapsed.as_millis();
        latencies.push(elapsed_ms);

        println!("File search '{}': {}ms", query, elapsed_ms);

        assert!(
            elapsed_ms < MAX_FILE_INDEX_MS,
            "File index search '{}' took {}ms, expected < {}ms",
            query,
            elapsed_ms,
            MAX_FILE_INDEX_MS
        );
    }

    let avg_latency = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let max_latency = latencies.iter().max().unwrap();

    println!("\n📊 File Index Performance:");
    println!("  Average: {}ms", avg_latency);
    println!("  Maximum: {}ms", max_latency);
    println!("  Minimum: {}ms", latencies.iter().min().unwrap());
}

/// Test: Progressive typing latency analysis
#[test]
fn test_progressive_typing_analysis() {
    if should_skip_perf() {
        eprintln!(
            "Skipping progressive typing analysis in debug build. Set NL_STRICT_PERF=1 and run in --release to enable."
        );
        return;
    }
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().expect("Failed to scan desktop entries");
    let entry_arena = DesktopEntryArena::from_vec(entries);
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entry_arena, Some(usage_tracker), None, &config);

    let queries = vec![
        ("firefox", "App name"),
        ("config.txt", "File name"),
        ("~/Documents/test", "Path"),
        ("@recent doc", "Command"),
    ];

    println!("\n📊 Progressive Typing Latency Analysis:\n");

    for (full_query, description) in queries {
        println!("Testing: {} ('{}')", description, full_query);

        let mut cumulative = String::new();
        let mut total_time = 0u128;
        let mut max_time = 0u128;

        for (i, ch) in full_query.chars().enumerate() {
            cumulative.push(ch);

            let start = Instant::now();
            let _results = plugin_manager.search(&cumulative, 10);
            let elapsed = start.elapsed().as_millis();

            total_time += elapsed;
            max_time = max_time.max(elapsed);

            let status = if elapsed > 16 {
                "❌"
            } else if elapsed > 8 {
                "⚠️ "
            } else {
                "✅"
            };

            println!(
                "  [{}] Char {}: '{}' → {}ms",
                status,
                i + 1,
                cumulative,
                elapsed
            );
        }

        let avg_time = total_time / full_query.len() as u128;
        println!(
            "  Summary: avg={}ms, max={}ms, total={}ms\n",
            avg_time, max_time, total_time
        );
    }
}
