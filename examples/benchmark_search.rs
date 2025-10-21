use native_launcher::desktop::{DesktopEntry, DesktopScanner};
use native_launcher::search::SearchEngine;
use std::path::PathBuf;
use std::time::Instant;

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

fn main() {
    println!("=== Native Launcher Search Performance Benchmark ===\n");

    // Test with realistic 500 apps dataset
    let sizes = vec![100, 500, 1000];

    for size in sizes {
        println!("--- Dataset: {} entries ---", size);
        let entries = create_test_entries(size);

        // Measure SearchEngine creation
        let start = Instant::now();
        let engine = SearchEngine::new(entries.clone());
        let creation_time = start.elapsed();
        println!("  SearchEngine creation: {:?}", creation_time);

        // Test various search scenarios
        let test_queries = vec![
            ("empty", ""),
            ("short", "app"),
            ("medium", "application"),
            ("exact", "Application 42"),
            ("keyword", "benchmark"),
            ("typo", "apppp"),
        ];

        for (name, query) in test_queries {
            let start = Instant::now();
            let results = engine.search(query, 10);
            let search_time = start.elapsed();

            println!(
                "  Search '{}' ({}): {:?} ({} results)",
                query,
                name,
                search_time,
                results.len()
            );

            // Check against <10ms target for 500 apps
            if size == 500 && search_time.as_millis() > 10 {
                println!("    ⚠️  WARNING: Exceeds 10ms target!");
            } else if size == 500 {
                println!("    ✓ Within 10ms target");
            }
        }

        println!();
    }

    // Real-world test with actual desktop files
    println!("--- Real Desktop Files ---");
    let scanner = DesktopScanner::new();

    let start = Instant::now();
    let real_entries_result = scanner.scan();
    let scan_time = start.elapsed();

    match real_entries_result {
        Ok(real_entries) => {
            println!(
                "  Scanned {} desktop files in {:?}",
                real_entries.len(),
                scan_time
            );

            let start = Instant::now();
            let engine = SearchEngine::new(real_entries.clone());
            let creation_time = start.elapsed();
            println!("  SearchEngine creation: {:?}", creation_time);

            let test_queries = vec!["fire", "code", "term", "file"];

            for query in test_queries {
                let start = Instant::now();
                let results = engine.search(query, 10);
                let search_time = start.elapsed();

                println!(
                    "  Search '{}': {:?} ({} results)",
                    query,
                    search_time,
                    results.len()
                );

                if results.len() > 0 {
                    println!("    Top result: {}", results[0].name);
                }
            }
        }
        Err(e) => {
            println!("  Error scanning desktop files: {}", e);
        }
    }

    println!("\n=== Performance Summary ===");
    println!("Target: Search <10ms for 500 apps");
    println!("See results above for detailed measurements");
}
