use criterion::{black_box, criterion_group, criterion_main, Criterion};
use native_launcher::config::ConfigLoader;
use native_launcher::desktop::DesktopScanner;
use native_launcher::plugins::PluginManager;
use native_launcher::usage::UsageTracker;
use std::time::Duration;

/// Benchmark the complete startup sequence
fn bench_full_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    
    // Set realistic timing expectations
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50); // Fewer samples since this is expensive
    
    group.bench_function("config_load", |b| {
        b.iter(|| {
            let config = ConfigLoader::load().unwrap_or_else(|_| ConfigLoader::new());
            black_box(config);
        });
    });
    
    group.bench_function("usage_tracker_load", |b| {
        b.iter(|| {
            let tracker = UsageTracker::load().unwrap_or_else(|_| UsageTracker::new());
            black_box(tracker);
        });
    });
    
    group.bench_function("desktop_scanner_scan_cached", |b| {
        b.iter(|| {
            let scanner = DesktopScanner::new();
            let entries = scanner.scan_cached().expect("Failed to scan");
            black_box(entries);
        });
    });
    
    group.bench_function("full_startup_sequence", |b| {
        b.iter(|| {
            // Config loading
            let config_loader = ConfigLoader::load().unwrap_or_else(|_| ConfigLoader::new());
            let config = config_loader.config().clone();
            
            // Usage tracking
            let usage_tracker = UsageTracker::load().unwrap_or_else(|_| UsageTracker::new());
            
            // Desktop scanning
            let scanner = DesktopScanner::new();
            let entries = scanner.scan_cached().expect("Failed to scan");
            
            // Plugin manager creation
            let plugin_manager = PluginManager::new(
                entries.clone(),
                Some(usage_tracker.clone()),
                &config
            );
            
            // Load dynamic plugins
            let (dynamic_plugins, _metrics) = native_launcher::plugins::load_plugins();
            
            black_box((config, usage_tracker, entries, plugin_manager, dynamic_plugins));
        });
    });
    
    group.finish();
}

/// Benchmark plugin system initialization
fn bench_plugin_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_init");
    
    // Load test data once
    let scanner = DesktopScanner::new();
    let entries = scanner.scan_cached().expect("Failed to scan");
    let usage_tracker = UsageTracker::load().unwrap_or_else(|_| UsageTracker::new());
    let config_loader = ConfigLoader::load().unwrap_or_else(|_| ConfigLoader::new());
    let config = config_loader.config().clone();
    
    group.bench_function("plugin_manager_creation", |b| {
        b.iter(|| {
            let plugin_manager = PluginManager::new(
                entries.clone(),
                Some(usage_tracker.clone()),
                &config
            );
            black_box(plugin_manager);
        });
    });
    
    group.bench_function("dynamic_plugin_loading", |b| {
        b.iter(|| {
            let (plugins, metrics) = native_launcher::plugins::load_plugins();
            black_box((plugins, metrics));
        });
    });
    
    group.finish();
}

/// Benchmark cache operations
fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");
    
    group.bench_function("desktop_cache_cold_scan", |b| {
        b.iter(|| {
            let scanner = DesktopScanner::new();
            // Force full scan by not using cache
            let entries = scanner.scan().expect("Failed to scan");
            black_box(entries);
        });
    });
    
    group.bench_function("desktop_cache_warm_scan", |b| {
        // Pre-warm the cache
        let scanner = DesktopScanner::new();
        let _ = scanner.scan_cached().expect("Failed to scan");
        
        b.iter(|| {
            let scanner = DesktopScanner::new();
            let entries = scanner.scan_cached().expect("Failed to scan");
            black_box(entries);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_full_startup,
    bench_plugin_initialization,
    bench_cache_operations
);

criterion_main!(benches);
