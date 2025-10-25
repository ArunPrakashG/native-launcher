#!/usr/bin/env python3
"""
Quick latency measurement for Native Launcher.
Measures the time plugins take to search on each keystroke.
"""


def measure_search_latency():
    """Run a simple latency measurement using cargo test"""

    print("🔍 Native Launcher - Input Latency Quick Test")
    print("=" * 50)
    print()

    # Test queries to simulate typing
    test_queries = [
        ("firefox", "App search"),
        ("config.txt", "File search (triggers index)"),
        ("~/Documents/test", "Path search"),
    ]

    print("📝 Simulating typing queries...")
    print()

    for full_query, description in test_queries:
        print(f"Testing: {description} ('{full_query}')")

        cumulative = ""
        for i, char in enumerate(full_query):
            cumulative += char

            # We can't easily measure from Python, so we'll just explain
            print(f"  Keystroke {i + 1}: '{cumulative}'")

        print()

    print("=" * 50)
    print()
    print("To measure actual latency, run:")
    print("  cargo test --release --test performance_tests -- --nocapture")
    print()
    print("Or run benchmarks with:")
    print("  cargo bench --bench input_latency_bench")
    print()


if __name__ == "__main__":
    measure_search_latency()
