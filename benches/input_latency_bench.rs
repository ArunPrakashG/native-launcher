use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use native_launcher::config::Config;
use native_launcher::desktop::DesktopScanner;
use native_launcher::plugins::PluginManager;
use native_launcher::usage::UsageTracker;
use std::time::Instant;

/// Simulates typing a query character by character and measures cumulative latency
fn simulate_typing_latency(
    plugin_manager: &PluginManager,
    full_query: &str,
    max_results: usize,
) -> Vec<(usize, u128)> {
    let mut latencies = Vec::new();
    let mut cumulative = String::new();

    for (i, ch) in full_query.chars().enumerate() {
        cumulative.push(ch);

        let start = Instant::now();
        let _ = plugin_manager.search(&cumulative, max_results);
        let elapsed = start.elapsed().as_micros();

        latencies.push((i + 1, elapsed));
    }

    latencies
}

/// Benchmark typing latency for common queries
fn typing_latency_benchmark(c: &mut Criterion) {
    // Setup plugin manager (expensive, do once)
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().unwrap_or_default();
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entries, Some(usage_tracker), &config);

    let mut group = c.benchmark_group("typing_latency");
    group.sample_size(20); // Reduce samples since this is expensive

    // Test queries that trigger different search paths
    let test_queries = vec![
        ("firefox", "App search"),
        ("config.txt", "File search (indexed)"),
        ("~/Documents/test.pdf", "Path search"),
        ("@recent document", "Command search"),
    ];

    for (query, description) in test_queries {
        group.bench_with_input(BenchmarkId::new(description, query), &query, |b, &query| {
            b.iter(|| {
                let latencies = simulate_typing_latency(
                    black_box(&plugin_manager),
                    black_box(query),
                    black_box(10),
                );

                // Return total time and worst keystroke
                let total: u128 = latencies.iter().map(|(_, lat)| lat).sum();
                let worst = *latencies.iter().map(|(_, lat)| lat).max().unwrap_or(&0);

                black_box((total, worst))
            })
        });
    }

    group.finish();
}

/// Benchmark single keystroke latency (what user actually feels)
fn keystroke_latency_benchmark(c: &mut Criterion) {
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().unwrap_or_default();
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entries, Some(usage_tracker), &config);

    let mut group = c.benchmark_group("keystroke_latency");

    // Test different query lengths (simulates progressive typing)
    let queries = vec![
        ("f", "1 char (skipped)"),
        ("fi", "2 chars (min search)"),
        ("fir", "3 chars (file index triggers)"),
        ("fire", "4 chars"),
        ("firef", "5 chars"),
        ("firefo", "6 chars"),
        ("firefox", "7 chars (full app name)"),
    ];

    for (query, description) in queries {
        group.bench_with_input(BenchmarkId::new(description, query), &query, |b, &query| {
            b.iter(|| plugin_manager.search(black_box(query), black_box(10)))
        });
    }

    group.finish();
}

/// Benchmark file index search in isolation
fn file_index_benchmark(c: &mut Criterion) {
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().unwrap_or_default();
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entries, Some(usage_tracker), &config);

    let mut group = c.benchmark_group("file_index_search");

    // Test file searches that definitely trigger system-wide index
    let file_queries = vec!["config", "bashrc", "document", "readme"];

    for query in file_queries {
        group.bench_with_input(
            BenchmarkId::new("indexed_file_search", query),
            &query,
            |b, &query| b.iter(|| plugin_manager.search(black_box(query), black_box(10))),
        );
    }

    group.finish();
}

/// Benchmark app search (should be fast, no file index)
fn app_search_benchmark(c: &mut Criterion) {
    let scanner = DesktopScanner::new();
    let entries = scanner.scan().unwrap_or_default();
    let usage_tracker = UsageTracker::new();
    let config = Config::default();
    let plugin_manager = PluginManager::new(entries, Some(usage_tracker), &config);

    let mut group = c.benchmark_group("app_search");

    // App names (should not trigger file index)
    let app_queries = vec!["firefox", "chrome", "code", "terminal"];

    for query in app_queries {
        group.bench_with_input(
            BenchmarkId::new("app_only_search", query),
            &query,
            |b, &query| b.iter(|| plugin_manager.search(black_box(query), black_box(10))),
        );
    }

    group.finish();
}

/// Benchmark worst-case: cache miss on system file search
fn cache_miss_benchmark(c: &mut Criterion) {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let scanner = DesktopScanner::new();
    let entries = scanner.scan().unwrap_or_default();
    let usage_tracker = UsageTracker::new();
    let config = Config::default();

    let counter = AtomicUsize::new(0);

    c.bench_function("file_index_cache_miss", |b| {
        b.iter(|| {
            // Create fresh plugin manager each time to simulate cache miss
            let plugin_manager =
                PluginManager::new(entries.clone(), Some(usage_tracker.clone()), &config);

            // Use different query each time to avoid cache hits
            let n = counter.fetch_add(1, Ordering::Relaxed);
            let query = format!("config{}", n);

            plugin_manager.search(black_box(&query), black_box(10))
        })
    });
}

criterion_group!(
    benches,
    typing_latency_benchmark,
    keystroke_latency_benchmark,
    file_index_benchmark,
    app_search_benchmark,
    cache_miss_benchmark,
);

criterion_main!(benches);
