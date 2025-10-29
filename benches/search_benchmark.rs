use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use native_launcher::desktop::{DesktopEntry, DesktopEntryArena, DesktopScanner};
use native_launcher::search::SearchEngine;
use std::path::PathBuf;
use std::time::Duration;

// Helper to create test entries
fn create_test_entries(count: usize) -> Vec<DesktopEntry> {
    (0..count)
        .map(|i| DesktopEntry {
            name: format!("Application {}", i),
            generic_name: Some(format!("Test App {}", i)),
            exec: format!("app{}", i),
            icon: Some(format!("icon{}", i)),
            categories: vec!["Utility".to_string(), "Development".to_string()],
            keywords: vec!["test".to_string(), "benchmark".to_string()],
            terminal: false,
            path: PathBuf::from(format!("/test/app{}.desktop", i)),
            no_display: false,
            actions: vec![],
        })
        .collect()
}

// Create realistic test entries mimicking real applications
fn create_realistic_entries() -> Vec<DesktopEntry> {
    vec![
        DesktopEntry {
            name: "Firefox".to_string(),
            generic_name: Some("Web Browser".to_string()),
            exec: "firefox".to_string(),
            icon: Some("firefox".to_string()),
            categories: vec!["Network".to_string(), "WebBrowser".to_string()],
            keywords: vec![
                "browser".to_string(),
                "web".to_string(),
                "internet".to_string(),
            ],
            terminal: false,
            path: PathBuf::from("/usr/share/applications/firefox.desktop"),
            no_display: false,
            actions: vec![],
        },
        DesktopEntry {
            name: "Visual Studio Code".to_string(),
            generic_name: Some("Text Editor".to_string()),
            exec: "code".to_string(),
            icon: Some("vscode".to_string()),
            categories: vec!["Development".to_string(), "IDE".to_string()],
            keywords: vec![
                "editor".to_string(),
                "programming".to_string(),
                "coding".to_string(),
            ],
            terminal: false,
            path: PathBuf::from("/usr/share/applications/code.desktop"),
            no_display: false,
            actions: vec![],
        },
        DesktopEntry {
            name: "Files".to_string(),
            generic_name: Some("File Manager".to_string()),
            exec: "nautilus".to_string(),
            icon: Some("system-file-manager".to_string()),
            categories: vec!["System".to_string(), "FileManager".to_string()],
            keywords: vec![
                "folder".to_string(),
                "manager".to_string(),
                "explore".to_string(),
            ],
            terminal: false,
            path: PathBuf::from("/usr/share/applications/org.gnome.Nautilus.desktop"),
            no_display: false,
            actions: vec![],
        },
    ]
}

fn bench_desktop_scanner(c: &mut Criterion) {
    c.bench_function("desktop_scanner_new", |b| {
        b.iter(|| {
            let scanner = DesktopScanner::new();
            black_box(scanner);
        });
    });
}

fn bench_search_engine_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_engine_creation");

    for size in [10, 50, 100, 500, 1000].iter() {
        let entries = create_test_entries(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let arena = DesktopEntryArena::from_vec(entries.clone());
                let engine = SearchEngine::new(arena, false);
                black_box(engine);
            });
        });
    }

    group.finish();
}

fn bench_fuzzy_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search_performance");

    // Set stricter time limits for critical path
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(3));

    // Test with different dataset sizes - focus on 500 apps (realistic)
    for size in [100, 500, 1000].iter() {
        let entries = create_test_entries(*size);
        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Critical: Fuzzy search with common patterns
        group.bench_with_input(BenchmarkId::new("fuzzy_short_query", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("app"), 10);
                black_box(results);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("fuzzy_medium_query", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = engine.search(black_box("application"), 10);
                    black_box(results);
                });
            },
        );

        // Typo tolerance test
        group.bench_with_input(BenchmarkId::new("fuzzy_typo", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("appp"), 10);
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_realistic_search_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_search");

    // Create a realistic dataset
    let mut entries = create_realistic_entries();

    // Expand to ~500 apps
    for i in 0..500 {
        entries.push(DesktopEntry {
            name: format!("App {}", i),
            generic_name: Some(format!("Generic App {}", i)),
            exec: format!("app{}", i),
            icon: Some(format!("icon{}", i)),
            categories: vec!["Utility".to_string()],
            keywords: vec!["tool".to_string()],
            terminal: false,
            path: PathBuf::from(format!("/test/app{}.desktop", i)),
            no_display: false,
            actions: vec![],
        });
    }

    let arena = DesktopEntryArena::from_vec(entries);
    let engine = SearchEngine::new(arena, false);

    // Real-world search patterns
    group.bench_function("search_firefox", |b| {
        b.iter(|| {
            let results = engine.search(black_box("firefox"), 10);
            black_box(results);
        });
    });

    group.bench_function("search_fire_prefix", |b| {
        b.iter(|| {
            let results = engine.search(black_box("fire"), 10);
            black_box(results);
        });
    });

    group.bench_function("search_browser_generic", |b| {
        b.iter(|| {
            let results = engine.search(black_box("browser"), 10);
            black_box(results);
        });
    });

    group.bench_function("search_code_typo", |b| {
        b.iter(|| {
            let results = engine.search(black_box("cod"), 10);
            black_box(results);
        });
    });

    group.bench_function("search_single_char", |b| {
        b.iter(|| {
            let results = engine.search(black_box("f"), 10);
            black_box(results);
        });
    });

    group.finish();
}

fn bench_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance");

    // Test with different dataset sizes
    for size in [10, 50, 100, 500, 1000].iter() {
        let entries = create_test_entries(*size);
        let arena = DesktopEntryArena::from_vec(entries);
        let engine = SearchEngine::new(arena, false);

        // Benchmark empty query (return all)
        group.bench_with_input(BenchmarkId::new("empty_query", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box(""), 10);
                black_box(results);
            });
        });

        // Benchmark exact match
        group.bench_with_input(BenchmarkId::new("exact_match", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("Application 42"), 10);
                black_box(results);
            });
        });

        // Benchmark partial match
        group.bench_with_input(BenchmarkId::new("partial_match", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("app"), 10);
                black_box(results);
            });
        });

        // Benchmark keyword match
        group.bench_with_input(BenchmarkId::new("keyword_match", size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("benchmark"), 10);
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_entry_matching(c: &mut Criterion) {
    let entry = DesktopEntry {
        name: "Firefox Web Browser".to_string(),
        generic_name: Some("Web Browser".to_string()),
        exec: "firefox".to_string(),
        icon: Some("firefox".to_string()),
        categories: vec!["Network".to_string(), "WebBrowser".to_string()],
        keywords: vec![
            "browser".to_string(),
            "web".to_string(),
            "internet".to_string(),
        ],
        terminal: false,
        path: PathBuf::from("/usr/share/applications/firefox.desktop"),
        no_display: false,
        actions: vec![],
    };

    let mut group = c.benchmark_group("entry_matching");

    group.bench_function("exact_name", |b| {
        b.iter(|| {
            let matches = entry.matches(black_box("Firefox Web Browser"));
            black_box(matches);
        });
    });

    group.bench_function("partial_name", |b| {
        b.iter(|| {
            let matches = entry.matches(black_box("fire"));
            black_box(matches);
        });
    });

    group.bench_function("keyword_match", |b| {
        b.iter(|| {
            let matches = entry.matches(black_box("browser"));
            black_box(matches);
        });
    });

    group.bench_function("no_match", |b| {
        b.iter(|| {
            let matches = entry.matches(black_box("xyz123"));
            black_box(matches);
        });
    });

    group.finish();
}

fn bench_entry_scoring(c: &mut Criterion) {
    let entry = DesktopEntry {
        name: "Firefox Web Browser".to_string(),
        generic_name: Some("Web Browser".to_string()),
        exec: "firefox".to_string(),
        icon: Some("firefox".to_string()),
        categories: vec!["Network".to_string()],
        keywords: vec!["browser".to_string(), "web".to_string()],
        terminal: false,
        path: PathBuf::from("/usr/share/applications/firefox.desktop"),
        no_display: false,
        actions: vec![],
    };

    let mut group = c.benchmark_group("entry_scoring");

    group.bench_function("score_exact", |b| {
        b.iter(|| {
            let score = entry.match_score(black_box("firefox"));
            black_box(score);
        });
    });

    group.bench_function("score_partial", |b| {
        b.iter(|| {
            let score = entry.match_score(black_box("fire"));
            black_box(score);
        });
    });

    group.bench_function("score_keyword", |b| {
        b.iter(|| {
            let score = entry.match_score(black_box("browser"));
            black_box(score);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_desktop_scanner,
    bench_search_engine_creation,
    bench_search_performance,
    bench_fuzzy_search_performance,
    bench_realistic_search_scenarios,
    bench_entry_matching,
    bench_entry_scoring
);

criterion_main!(benches);
