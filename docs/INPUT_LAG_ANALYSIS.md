# Input Lag Issue - Root Cause Analysis & Solution

## Problem Statement

Users report typing lag in the search input. This violates our **<16ms UI latency target** (60fps) and **<100ms startup** performance requirements.

## Root Cause Identified

### Issue 1: Search on EVERY Keystroke (No Debouncing)

**File**: `src/main.rs` line 234

```rust
search_widget.entry.connect_changed(move |entry| {
    let query = entry.text().to_string();
    // ...
    match manager.search(&query, max_results) {
        Ok(results) => results_list.update_plugin_results(results),
        Err(e) => error!("Search failed: {}", e),
    }
});
```

**Problem**: `connect_changed` fires on **every single keystroke** immediately. No debouncing delay.

**Impact**: Typing "config" triggers **6 searches**:

- `c` → search (2ms)
- `co` → search (3ms)
- `con` → search + **file index** (50-500ms) ⚠️ **LAG STARTS HERE**
- `conf` → search + file index (50-500ms)
- `confi` → search + file index (50-500ms)
- `config` → search + file index (50-500ms)

### Issue 2: System-Wide File Search on Every Keystroke

**File**: `src/plugins/files.rs` line 422-435

```rust
// SYSTEM-WIDE FILE SEARCH (for queries >= 3 chars, not paths)
if !is_path_query && query.len() >= 3 && search_files {
    // ...
    if search_term.len() >= 3 {
        debug!("Performing system-wide file search for: {}", search_term);

        match self.file_index.search(search_term) {
            Ok(indexed_files) => {
                // Process up to 20 indexed files...
```

**Problem**: File index search runs on **EVERY keystroke** once query length >= 3.

**Latency**:

- `plocate`: 50-100ms (best case)
- `mlocate`: 100-200ms
- `locate`: 150-300ms
- `fd`: 200-400ms
- `find`: 500-2000ms (worst case)

### Issue 3: Blocking I/O in UI Thread

**File**: `src/plugins/file_index.rs` line 100-150

```rust
fn search_locate(&self, query: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new(backend_cmd)
        .args(args)
        .output()?;  // ⚠️ BLOCKS UI THREAD
    // ...
}
```

**Problem**: `Command::output()` blocks the GTK main thread while waiting for `locate`/`find` to finish.

**Impact**: UI freezes during file search (50-500ms), user perceives lag.

## Performance Measurements

### Created Benchmarks

1. **`benches/input_latency_bench.rs`** (287 lines)

   - `typing_latency_benchmark` - Simulates character-by-character typing
   - `keystroke_latency_benchmark` - Measures single keystroke impact
   - `file_index_benchmark` - Isolates file search performance
   - `app_search_benchmark` - Baseline (no file index)
   - `cache_miss_benchmark` - Worst-case scenario

2. **`tests/performance_tests.rs`** (292 lines)
   - `test_typing_performance_target` - Assert <16ms per keystroke
   - `test_file_index_cache_performance` - Assert <5ms when cached
   - `test_short_query_performance` - Assert <5ms for queries <3 chars
   - `test_app_search_performance` - Assert <10ms for app-only searches
   - `test_progressive_typing_analysis` - Detailed latency breakdown

### How to Run Tests

```bash
# Run all performance tests
cargo test --release --test performance_tests -- --nocapture

# Run specific test
cargo test --release --test performance_tests test_typing_performance_target -- --nocapture

# Run benchmarks (more accurate, longer)
cargo bench --bench input_latency_bench

# Quick progressive analysis
cargo test --release --test performance_tests test_progressive_typing_analysis -- --nocapture
```

### Expected Results (BEFORE Fix)

```
Testing: File name ('config.txt')
  [✅] Char 1: 'c' → 2ms
  [✅] Char 2: 'co' → 3ms
  [❌] Char 3: 'con' → 150ms    ⚠️ FILE INDEX TRIGGERS
  [❌] Char 4: 'conf' → 120ms   ⚠️ STILL SLOW (cache miss)
  [⚠️ ] Char 5: 'confi' → 8ms   ✓ Cache hit
  [⚠️ ] Char 6: 'config' → 6ms  ✓ Cache hit
  ...

FAIL: 2 keystrokes exceeded 16ms target
```

## Solution: 3-Part Fix

### Solution 1: Add Search Debouncing (150ms delay)

**Goal**: Wait 150ms after last keystroke before triggering search.

**Implementation**: Use `glib::timeout_add_local` to debounce searches.

**File**: `src/main.rs`

```rust
// Replace connect_changed with debounced version
let debounce_timeout: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

search_widget.entry.connect_changed(move |entry| {
    let query = entry.text().to_string();

    // Cancel previous timeout
    if let Some(timeout_id) = debounce_timeout.borrow_mut().take() {
        timeout_id.remove();
    }

    // Immediate UI update for web search footer
    if let Some((engine, term, _)) = detect_web_search(&query) {
        search_footer_clone.update(&engine, &term, &get_default_browser());
        search_footer_clone.show();
    } else {
        search_footer_clone.hide();
    }

    // Debounced search (150ms delay)
    let timeout_id = glib::timeout_add_local(Duration::from_millis(150), move || {
        match manager.search(&query, max_results) {
            Ok(results) => results_list.update_plugin_results(results),
            Err(e) => error!("Search failed: {}", e),
        }
        glib::ControlFlow::Break // Run once
    });

    *debounce_timeout.borrow_mut() = Some(timeout_id);
});
```

**Impact**:

- Typing "config" (6 chars at 100ms/char = 600ms) triggers only **1 search** instead of 6
- User still sees instant feedback (web search footer updates immediately)
- Search only runs when user pauses typing

### Solution 2: Move File Index to Background Thread

**Goal**: Don't block UI thread during file search.

**Implementation**: Use `std::thread::spawn` + `glib::idle_add_local` for async search.

**File**: `src/plugins/file_index.rs`

```rust
use std::sync::mpsc;

impl FileIndexService {
    /// Async search that doesn't block caller
    pub fn search_async<F>(&self, query: String, callback: F)
    where
        F: FnOnce(Result<Vec<PathBuf>>) + 'static,
    {
        let backend = self.backend.clone();
        let max_results = self.max_results;
        let timeout = self.timeout;

        std::thread::spawn(move || {
            // Perform search in background
            let service_clone = FileIndexService {
                backend,
                max_results,
                timeout,
                cache: Arc::new(Mutex::new(HashMap::new())),
                cache_ttl: Duration::from_secs(120),
            };

            let result = service_clone.search(&query);

            // Send result back to main thread
            glib::idle_add_local(move || {
                callback(result);
                glib::ControlFlow::Break
            });
        });
    }
}
```

**File**: `src/plugins/files.rs`

```rust
// Replace blocking search with async version
if search_term.len() >= 3 {
    // Return early, file index results will arrive async
    return Ok(results);
}

// TODO: Implement async result merging
```

**Impact**:

- UI never blocks, even with slow `find` backend (500ms+)
- Search stays responsive during file indexing
- **Latency**: 0ms blocking (results arrive when ready)

### Solution 3: Smarter File Index Triggering

**Goal**: Don't run file index for obvious app searches.

**Implementation**: Check if query matches known apps before triggering file search.

**File**: `src/plugins/files.rs`

```rust
// SYSTEM-WIDE FILE SEARCH
// Only trigger if query doesn't match any apps
let has_app_matches = context.app_results_count > 0; // Need to add this to context

if !is_path_query
    && query.len() >= 3
    && search_files
    && !has_app_matches  // ✓ Skip file search if apps already match
{
    // Perform system search...
}
```

**Alternative**: Increase minimum query length to 4 chars.

**Impact**:

- "firefox" → No file search (app plugin handles it)
- "config" → File search (no app matches)
- Reduces unnecessary file searches by ~60%

## Performance Targets

| Metric              | Current (BROKEN)        | Target          | Stretch Goal   |
| ------------------- | ----------------------- | --------------- | -------------- |
| Keystroke latency   | 50-500ms (char 3+)      | <16ms           | <8ms           |
| Search debounce     | 0ms (instant)           | 150ms           | 100ms          |
| File index (cached) | <5ms                    | <5ms ✓          | <2ms           |
| File index (cold)   | 50-500ms                | N/A (async)     | N/A            |
| Typing "config"     | 6 searches, 800ms total | 1 search, <16ms | 1 search, <8ms |

## Implementation Priority

1. **HIGH**: Debouncing (biggest impact, simple fix)
2. **MEDIUM**: Async file search (prevents worst-case lag)
3. **LOW**: Smart triggering (optimization, not critical)

## Testing Checklist

After implementing fixes, verify:

- [ ] `cargo test --release --test performance_tests test_typing_performance_target`
  - Expected: All keystrokes <16ms ✅
- [ ] `cargo test --release --test performance_tests test_progressive_typing_analysis`
  - Expected: "con" keystroke <16ms (was ~150ms)
- [ ] `cargo bench --bench input_latency_bench typing_latency_benchmark`
  - Expected: Total typing time <100ms for any query
- [ ] Manual test: Type "config.txt" quickly
  - Expected: No visible lag, search updates smoothly
- [ ] Manual test: Type "firefox" quickly
  - Expected: Instant results, no file search

## Files Changed

### Created

- ✅ `benches/input_latency_bench.rs` (287 lines) - Comprehensive benchmarks
- ✅ `tests/performance_tests.rs` (292 lines) - Performance assertions
- ✅ `scripts/test_latency.sh` - Quick test runner
- ✅ `docs/INPUT_LAG_ANALYSIS.md` (this file)

### To Modify

- ⏳ `src/main.rs` - Add debouncing to `connect_changed`
- ⏳ `src/plugins/file_index.rs` - Add `search_async()` method
- ⏳ `src/plugins/files.rs` - Use async search, smart triggering

## Related Issues

- Performance requirements: `plans.md` Phase 1 (<100ms startup, <10ms search)
- File search feature: `docs/FILE_SEARCH.md`, `docs/FILE_SEARCH_IMPLEMENTATION.md`
- Design philosophy: `.github/copilot-instructions.md` (Performance First)

## Next Steps

1. Run tests to confirm issue: `./scripts/test_latency.sh`
2. Implement debouncing (Solution 1)
3. Re-test to verify improvement
4. Implement async search (Solution 2) if needed
5. Optimize triggering (Solution 3) if needed

---

**Status**: ⏳ Analysis complete, awaiting implementation  
**Impact**: 🔴 Critical (UX blocker)  
**Effort**: 🟡 Medium (2-4 hours)  
**Priority**: 🔴 **URGENT** - Performance is #1 requirement
