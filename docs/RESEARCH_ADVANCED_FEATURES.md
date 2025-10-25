# Research: Advanced Features Implementation

Research document for implementing 6 advanced features in Native Launcher.

**Date**: October 25, 2025  
**Status**: Research Phase  
**Priority**: Performance & UX Improvements

---

## 1. Empty State on Launch (Spotlight-Style)

### Current Behavior

- Shows all apps on launch (`search("")` returns all results)
- Line 223 in `main.rs`: `match plugin_manager.borrow().search("", max_results)`

### Proposed Behavior

- Show empty results list with just search input
- Display placeholder text: "Type to search..."
- Optionally show keyboard hints

### Implementation

**Difficulty**: ⭐ Easy  
**Impact**: 🎯 High (UX improvement)  
**Breaking**: No

#### Code Changes

**1. Modify `main.rs` initial results**:

```rust
// Before (line ~223):
match plugin_manager.borrow().search("", max_results) {
    Ok(initial_results) => results_list.update_plugin_results(initial_results),
    Err(e) => error!("Failed to get initial results: {}", e),
}

// After:
// Don't show initial results - start with empty list
results_list.update_plugin_results(Vec::new());
```

**2. Add placeholder to `ResultsList`** (`src/ui/results_list.rs`):

```rust
impl ResultsList {
    pub fn new() -> Self {
        let list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        // Add placeholder widget
        let placeholder = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(gtk4::Align::Center)
            .spacing(12)
            .build();

        let icon = Image::builder()
            .icon_name("edit-find-symbolic")
            .pixel_size(64)
            .build();
        icon.add_css_class("placeholder-icon");

        let label = Label::builder()
            .label("Type to search applications, files, and more...")
            .build();
        label.add_css_class("placeholder-text");

        placeholder.append(&icon);
        placeholder.append(&label);
        list.set_placeholder(Some(&placeholder));

        // ... rest of constructor
    }
}
```

**3. Add CSS styling** (`src/ui/style.css`):

```css
.placeholder-icon {
  opacity: 0.3;
  margin: 20px;
}

.placeholder-text {
  font-size: 14px;
  color: var(--nl-text-tertiary);
  opacity: 0.6;
}
```

#### Benefits

- ✅ Cleaner, more focused UI (like Spotlight, Raycast)
- ✅ Reduces visual clutter on launch
- ✅ Faster perceived startup (no initial search)
- ✅ Encourages keyboard-driven workflow

#### Considerations

- ❓ Users accustomed to seeing recent apps might be confused
- ❓ Could add config option: `show_initial_results = false` (default)
- ✅ Easy to revert or make configurable

---

## 2. Async Plugin Architecture

### Current Behavior

- Plugin trait is **synchronous**: `fn search(&self, query: &str) -> Result<Vec<PluginResult>>`
- Blocks UI thread during expensive operations (file search, network calls)
- Smart triggering mitigates but doesn't eliminate blocking

### Proposed Behavior

- Plugins return results asynchronously
- UI remains responsive during searches
- Results stream in as they arrive

### Implementation

**Difficulty**: ⭐⭐⭐⭐⭐ Very Hard  
**Impact**: 🎯🎯🎯 Very High (responsiveness)  
**Breaking**: Yes (major API change)

#### Challenges

1. **GTK4 is NOT thread-safe**: All UI updates must happen on main thread
2. **Plugin trait redesign**: Breaking change for all plugins
3. **Result streaming**: Need mechanism to send partial results
4. **Cancellation**: Must cancel in-flight searches when query changes
5. **Ordering**: Async results arrive out-of-order

#### Approach 1: Async Trait with Tokio

**Plugin trait**:

```rust
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait(?Send)] // GTK requires !Send
pub trait Plugin: Debug {
    fn name(&self) -> &str;

    // Async search with result streaming
    async fn search(
        &self,
        query: &str,
        context: &PluginContext,
        tx: mpsc::UnboundedSender<PluginResult>,
    ) -> Result<()>;

    // Optional: Synchronous search for simple plugins
    fn search_sync(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        // Default: not implemented
        Err(anyhow::anyhow!("Synchronous search not supported"))
    }
}
```

**Plugin Manager execution**:

```rust
pub async fn search_async(
    &self,
    query: String,
    callback: impl Fn(Vec<PluginResult>) + 'static,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn search tasks for all plugins
    let handles: Vec<_> = self.plugins.iter()
        .filter(|p| p.should_handle(&query))
        .map(|plugin| {
            let query = query.clone();
            let tx = tx.clone();
            let context = PluginContext::new(); // ... setup

            tokio::spawn(async move {
                let _ = plugin.search(&query, &context, tx).await;
            })
        })
        .collect();

    drop(tx); // Close sender so receiver knows when done

    // Collect results as they arrive
    let mut all_results = Vec::new();
    while let Some(result) = rx.recv().await {
        all_results.push(result);

        // Update UI incrementally (via glib::idle_add)
        let results = all_results.clone();
        glib::idle_add_local_once(move || {
            callback(results);
        });
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }
}
```

**Main.rs integration**:

```rust
// In connect_changed handler
search_widget.entry.connect_changed(move |entry| {
    let query = entry.text().to_string();
    let runtime = tokio_runtime.clone(); // Need to store runtime
    let plugin_manager = plugin_manager.clone();
    let results_list = results_list.clone();

    // Cancel previous search (TODO: implement cancellation)

    runtime.spawn(async move {
        plugin_manager.borrow().search_async(query, move |results| {
            results_list.update_plugin_results(results);
        }).await;
    });
});
```

#### Approach 2: Thread Pool with Channels

**Simpler, no async/await**:

```rust
use std::sync::mpsc;
use std::thread;

impl PluginManager {
    pub fn search_threaded(
        &self,
        query: String,
        callback: Box<dyn Fn(Vec<PluginResult>) + Send>,
    ) {
        let plugins = self.plugins.clone(); // Need Arc<Plugin>

        thread::spawn(move || {
            let (tx, rx) = mpsc::channel();

            // Spawn thread for each plugin
            for plugin in plugins.iter().filter(|p| p.should_handle(&query)) {
                let tx = tx.clone();
                let query = query.clone();

                thread::spawn(move || {
                    if let Ok(results) = plugin.search(&query, &context) {
                        for result in results {
                            let _ = tx.send(result);
                        }
                    }
                });
            }
            drop(tx);

            // Collect and batch results
            let mut all_results = Vec::new();
            for result in rx {
                all_results.push(result);

                // Send to main thread
                let results = all_results.clone();
                glib::idle_add_local_once(move || {
                    callback(results);
                });
            }
        });
    }
}
```

#### Recommendation

**Phase 1**: Keep sync API, improve with:

- ✅ Current smart triggering (already done)
- ✅ Better caching (file index already caches)
- ✅ Incremental search (see #3 below)

**Phase 2** (Future): Async trait when:

- File search consistently >100ms even with caching
- Adding network-based plugins (weather, stocks)
- User feedback indicates UI blocking issues

**Why wait?**:

- Current performance is good (<10ms for apps, <100ms for files)
- Async complexity is HIGH (GTK thread-safety, cancellation, ordering)
- Incremental search (simpler) provides similar UX benefit

---

## 3. Incremental Search

### Current Behavior

- Search runs on every keystroke (with 150ms debouncing)
- Results replace entirely each time
- No streaming or progressive results

### Proposed Behavior

- Results arrive progressively (apps fast, files slower)
- Show fast results immediately, append slower results
- Visual indicator for "still searching"

### Implementation

**Difficulty**: ⭐⭐⭐ Medium  
**Impact**: 🎯🎯 High (perceived performance)  
**Breaking**: No (internal change only)

#### Approach: Two-Phase Display

**Already partially implemented**: Two-pass search (apps first, then files)

**Enhancement**: Visual separation and progressive display

```rust
// In PluginManager
pub fn search_incremental(
    &self,
    query: &str,
    max_results: usize,
    on_fast_results: impl Fn(Vec<PluginResult>),
    on_slow_results: impl Fn(Vec<PluginResult>),
) -> Result<()> {
    // Phase 1: Fast plugins (apps, calculator)
    let fast_plugins = ["Applications", "Calculator", "WebSearch"];
    let mut fast_results = Vec::new();

    for plugin in self.plugins.iter() {
        if !fast_plugins.contains(&plugin.name()) {
            continue;
        }
        if plugin.should_handle(query) {
            fast_results.extend(plugin.search(query, context)?);
        }
    }

    // Show fast results immediately
    on_fast_results(self.rank_results(fast_results, max_results));

    // Phase 2: Slow plugins (files, SSH)
    let mut slow_results = Vec::new();
    for plugin in self.plugins.iter() {
        if fast_plugins.contains(&plugin.name()) {
            continue; // Skip already-searched
        }
        if plugin.should_handle(query) {
            slow_results.extend(plugin.search(query, context)?);
        }
    }

    // Append slow results
    on_slow_results(self.rank_results(slow_results, max_results));

    Ok(())
}
```

**UI Update** (`main.rs`):

```rust
gtk4::glib::timeout_add_local_once(Duration::from_millis(150), move || {
    let manager = plugin_manager_clone.borrow();

    // Show loading indicator
    search_footer_clone.show_loading();

    let _ = manager.search_incremental(
        &query,
        max_results,
        // Fast results callback
        |results| {
            results_list_clone.update_plugin_results(results);
        },
        // Slow results callback
        |results| {
            results_list_clone.append_plugin_results(results);
            search_footer_clone.hide_loading();
        },
    );
});
```

**New method in ResultsList**:

```rust
/// Append results without clearing existing
pub fn append_plugin_results(&self, new_results: Vec<PluginResult>) {
    let mut items = self.items.borrow_mut();

    for result in new_results {
        items.push(ListItem::PluginResult { result });
    }

    // Render only new items (optimization)
    self.render_appended_items();
}
```

#### Benefits

- ✅ Apps appear instantly (<5ms)
- ✅ Files appear shortly after (50-100ms)
- ✅ Better perceived performance
- ✅ Visual feedback ("Searching files...")

#### Alternative: Progressive Plugin Execution

Each plugin reports progress:

```rust
pub trait Plugin {
    fn search_progressive(
        &self,
        query: &str,
        context: &PluginContext,
        progress: &dyn Fn(Vec<PluginResult>),
    ) -> Result<()> {
        // Default: call progress once with all results
        let results = self.search(query, context)?;
        progress(results);
        Ok(())
    }
}
```

File plugin can report in chunks:

```rust
// In FileIndexService
pub fn search_progressive(&self, query: &str, chunk_size: usize, callback: impl Fn(Vec<PathBuf>)) {
    let mut results = Vec::new();

    for path in self.index.search(query) {
        results.push(path);

        if results.len() >= chunk_size {
            callback(results.drain(..).collect());
        }
    }

    if !results.is_empty() {
        callback(results);
    }
}
```

---

## 4. GPU-Accelerated Rendering

### Current Behavior

- GTK4 uses software rendering or basic GPU acceleration
- Animations rendered on CPU
- No explicit GPU optimization

### Proposed Behavior

- Leverage GPU for smooth 120fps animations
- Hardware-accelerated list scrolling
- Faster window compositing

### Implementation

**Difficulty**: ⭐⭐⭐⭐ Hard  
**Impact**: 🎯 Medium (already 60fps, diminishing returns)  
**Breaking**: No

#### GTK4 GPU Capabilities

**Good news**: GTK4 already uses GPU where available via:

- OpenGL backend (default on most systems)
- Vulkan backend (experimental, via `GSK_RENDERER=vulkan`)
- Cairo hardware acceleration

**Check current backend**:

```bash
# Run with debug
GSK_DEBUG=fallback native-launcher
# Or
GTK_DEBUG=interactive native-launcher
```

#### Optimization 1: Force GPU Backend

**Environment variable**:

```bash
# Force OpenGL (usually default)
GDK_BACKEND=wayland GSK_RENDERER=opengl native-launcher

# Try Vulkan (experimental, may crash)
GSK_RENDERER=vulkan native-launcher

# Broadway (web-based, slow - don't use)
GSK_RENDERER=broadway native-launcher
```

**In code** (not recommended, prefer env vars):

```rust
// Before GTK init
std::env::set_var("GSK_RENDERER", "opengl");
```

#### Optimization 2: GPU-Friendly CSS

**Use GPU-accelerated properties**:

```css
/* Good: GPU-accelerated (uses transform matrix) */
.list-row {
  transform: translateY(0);
  opacity: 1;
  will-change: transform, opacity; /* Hint to GPU */
}

/* Bad: CPU layout (triggers reflow) */
.list-row {
  margin-top: 0;
  height: 50px;
}
```

**Current CSS already good**:

- Using `transform: scale()`, `translateX()` ✅
- Using `opacity` for fades ✅
- Avoiding layout properties in animations ✅

#### Optimization 3: Reduce Overdraw

**Layer optimization**:

```css
/* Promote to own GPU layer */
listbox row {
  will-change: transform;
  /* Or force layer: */
  transform: translateZ(0); /* Hack for GPU layer */
}
```

**Caution**: Too many layers = more memory, slower on low-end GPUs

#### Optimization 4: Custom Rendering (Advanced)

**Use GTK Snapshot API for custom drawing**:

```rust
use gtk4::Snapshot;

impl MyWidget {
    fn snapshot(&self, snapshot: &Snapshot) {
        // Custom GPU-optimized rendering
        // Not recommended unless specific need
    }
}
```

#### Benchmarking GPU Performance

**Measure frame times**:

```bash
# GTK Inspector
GTK_DEBUG=interactive native-launcher
# Navigate to "Visual" tab, enable "Show FPS"
```

**Expected**:

- Current: 60fps (16.6ms/frame) - already smooth
- Goal: 120fps (8.3ms/frame) - requires 120Hz display

#### Recommendation

**Current Status**: Already well-optimized

- ✅ Using GPU-accelerated CSS properties
- ✅ GTK4 uses OpenGL by default
- ✅ Animations are smooth (60fps)

**Low Priority**: Only pursue if:

- Targeting 120Hz/144Hz displays specifically
- Seeing frame drops in profiling
- Adding complex particle effects or 3D

**Higher impact**: Focus on #3 (incremental search) and #1 (empty state) first

---

## 5. Preload on Compositor Start (Daemon Mode)

### Current Behavior

- Launcher starts fresh on each hotkey press
- 45-60ms cold start, 30-40ms warm start
- Desktop entries cached, but binary not in memory

### Proposed Behavior

- Background daemon starts with compositor
- Listens for activation signal (D-Bus or socket)
- Instant appearance (<10ms) when hotkey pressed

### Implementation

**Difficulty**: ⭐⭐⭐⭐ Hard  
**Impact**: 🎯🎯🎯 Very High (startup time)  
**Breaking**: No (optional mode)

#### Architecture: D-Bus Activation

**D-Bus service file** (`/usr/share/dbus-1/services/com.github.native-launcher.service`):

```ini
[D-Bus Service]
Name=com.github.native-launcher
Exec=/usr/local/bin/native-launcher --daemon
```

**Daemon mode in main.rs**:

```rust
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--daemon".to_string()) {
        run_daemon_mode()
    } else {
        // Check if daemon is running
        if daemon_is_running() {
            // Send "show" signal to daemon
            send_show_signal()?;
            std::process::exit(0);
        } else {
            // Run normally
            run_normal_mode()
        }
    }
}

fn run_daemon_mode() -> Result<()> {
    info!("Starting in daemon mode");

    // Initialize everything (desktop entries, plugins, etc.)
    // ... same as current startup ...

    // Create GTK app but don't show window
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::IS_SERVICE) // Important!
        .build();

    // Register D-Bus service
    app.connect_activate(|app| {
        // Don't show window on activate in daemon mode
    });

    // Listen for "show" action
    let show_action = gio::SimpleAction::new("show", None);
    show_action.connect_activate(|_, _| {
        // Show launcher window
        show_launcher_window();
    });
    app.add_action(&show_action);

    app.run();
    Ok(())
}

fn send_show_signal() -> Result<()> {
    // Use D-Bus to activate the "show" action
    let connection = gio::DBusConnection::session()?;
    connection.call_sync(
        Some("com.github.native-launcher"),
        "/com/github/NativeLauncher",
        "org.freedesktop.Application",
        "ActivateAction",
        Some(&("show", Vec::<&str>::new()).to_variant()),
        None,
        gio::DBusCallFlags::NONE,
        -1,
        None,
    )?;
    Ok(())
}
```

#### Architecture: Unix Socket (Alternative)

**Simpler, no D-Bus dependency**:

**Server (daemon)**:

```rust
use std::os::unix::net::UnixListener;
use std::path::Path;

fn run_daemon_mode() -> Result<()> {
    let socket_path = "/tmp/native-launcher.sock";

    // Remove old socket if exists
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;

    // Listen for connections in background thread
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(_) => {
                    // Show window on main thread
                    glib::idle_add_local_once(|| {
                        show_launcher_window();
                    });
                }
                Err(e) => error!("Socket error: {}", e),
            }
        }
    });

    // Run GTK main loop
    app.run();
}
```

**Client (hotkey trigger)**:

```rust
use std::os::unix::net::UnixStream;

fn send_show_signal() -> Result<()> {
    let socket_path = "/tmp/native-launcher.sock";

    if Path::new(socket_path).exists() {
        // Daemon running, send signal
        UnixStream::connect(socket_path)?;
        Ok(())
    } else {
        // Daemon not running, start normally
        Err(anyhow::anyhow!("Daemon not running"))
    }
}
```

#### Compositor Integration

**Sway** (`~/.config/sway/config`):

```
# Start daemon on compositor start
exec native-launcher --daemon

# Hotkey triggers client
bindsym $mod+Space exec native-launcher
```

**Hyprland** (`~/.config/hypr/hyprland.conf`):

```
# Start daemon
exec-once = native-launcher --daemon

# Hotkey
bind = SUPER, Space, exec, native-launcher
```

#### Window Management in Daemon Mode

**Challenge**: Hide window vs destroy window

**Option 1: Hide window**:

```rust
// When Esc pressed
window.hide();
// Pros: Instant reappear
// Cons: Memory stays allocated, can't Alt+Tab away
```

**Option 2: Destroy and recreate**:

```rust
// When Esc pressed
window.close();
// On show signal
let window = rebuild_window();
// Pros: Frees memory
// Cons: Slower (~10-20ms to recreate)
```

**Recommendation**: Hide window, recreate if closed

#### Benefits

- ✅ <10ms startup (instant feel)
- ✅ Desktop entries already loaded
- ✅ Icon cache already in memory
- ✅ Plugins initialized

#### Challenges

- ❌ Memory always allocated (~25MB)
- ❌ More complex startup logic
- ❌ Need compositor integration (exec-once)
- ❌ Harder to update/restart

#### Recommendation

**Implement in phases**:

**Phase 1**: Test current startup time consistently

- Measure: Is 45-60ms really a problem?
- User feedback: Do users complain about startup?

**Phase 2**: Optimize cold start first (easier wins)

- Better caching strategies
- Lazy plugin loading
- Profile with `cargo flamegraph`

**Phase 3**: Implement daemon if still needed

- Use Unix socket (simpler than D-Bus)
- Make optional via `--daemon` flag
- Document compositor setup

**Priority**: Medium-Low (current startup already fast)

---

## 6. SIMD Fuzzy Matching

### Current Behavior

- Using `nucleo` crate (already optimized)
- Using `fuzzy-matcher` crate as fallback
- Both are reasonably fast but CPU-based

### Proposed Behavior

- Use SIMD instructions (SSE, AVX) for parallel string matching
- 2-4x faster fuzzy matching
- Better performance on large datasets (>1000 items)

### Implementation

**Difficulty**: ⭐⭐⭐⭐⭐ Very Hard  
**Impact**: 🎯 Low-Medium (current search is already <10ms)  
**Breaking**: No (internal change)

#### Current Performance

**Benchmarks** (from Performance.md):

- Search 10 apps: 2-4ms
- Search 500 apps: 6-9ms
- **Already meeting <10ms target**

#### SIMD-Capable Libraries

**1. `nucleo` (current)**:

- Already uses some SIMD via `memchr` crate
- Optimized for fuzzy matching
- **Status**: Already good enough

**2. `sublime_fuzzy`**:

```toml
sublime_fuzzy = "0.7"
```

- SIMD-optimized (AVX2, SSE4.2)
- Used by Sublime Text, Zed editor
- ~2x faster than `fuzzy-matcher`

**Example**:

```rust
use sublime_fuzzy::best_match;

fn fuzzy_match(query: &str, target: &str) -> Option<i64> {
    best_match(query, target).map(|m| m.score())
}
```

**3. Custom SIMD implementation**:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
unsafe fn simd_fuzzy_match(query: &[u8], target: &[u8]) -> i32 {
    // Load 32 bytes at a time with AVX2
    let query_vec = _mm256_loadu_si256(query.as_ptr() as *const __m256i);
    let target_vec = _mm256_loadu_si256(target.as_ptr() as *const __m256i);

    // Compare (parallel byte comparison)
    let cmp = _mm256_cmpeq_epi8(query_vec, target_vec);

    // Count matches
    let mask = _mm256_movemask_epi8(cmp);
    mask.count_ones() as i32
}
```

**Complexity**: Very high, requires:

- CPU feature detection
- Fallback for non-SIMD CPUs
- Careful memory alignment
- Testing on multiple architectures

#### Recommendation

**Status**: NOT RECOMMENDED

**Reasons**:

1. ✅ Current performance already excellent (<10ms)
2. ❌ SIMD complexity very high
3. ❌ Minimal user-visible benefit (6ms → 3ms?)
4. ❌ Risk of bugs (segfaults, alignment issues)
5. ❌ Limited to x86_64 (ARM needs different SIMD)

**Better alternatives**:

1. **Stick with `nucleo`** - already well-optimized
2. **Try `sublime_fuzzy`** - simple drop-in, 2x faster (low effort)
3. **Better scoring** - improve ranking algorithm (higher impact)

**If pursuing**:

**Easy test**: Swap `nucleo` for `sublime_fuzzy`:

```rust
// In Cargo.toml
sublime_fuzzy = "0.7"

// In search code
use sublime_fuzzy::best_match;

let score = best_match(query, &app.name)
    .map(|m| m.score())
    .unwrap_or(0);
```

**Benchmark before/after**:

```bash
cargo bench --bench search_benchmark
```

**Only adopt if**: >30% improvement measured

---

## Summary & Recommendations

### Priority Ranking

| Feature               | Difficulty           | Impact      | Recommend    | Timeline   |
| --------------------- | -------------------- | ----------- | ------------ | ---------- |
| 1. Empty State        | ⭐ Easy              | 🎯🎯🎯 High | ✅ **YES**   | 1-2 days   |
| 3. Incremental Search | ⭐⭐⭐ Medium        | 🎯🎯 High   | ✅ **YES**   | 1 week     |
| 5. Daemon Mode        | ⭐⭐⭐⭐ Hard        | 🎯🎯 High   | ⚠️ Maybe     | 2-3 weeks  |
| 2. Async Plugins      | ⭐⭐⭐⭐⭐ Very Hard | 🎯🎯 Medium | ❌ **Later** | 1-2 months |
| 4. GPU Rendering      | ⭐⭐⭐⭐ Hard        | 🎯 Low      | ❌ **No**    | N/A        |
| 6. SIMD Matching      | ⭐⭐⭐⭐⭐ Very Hard | 🎯 Low      | ❌ **No**    | N/A        |

### Recommended Implementation Order

**Phase 1: Quick Wins** (1-2 weeks)

1. ✅ Empty state on launch (Spotlight-style)
2. ✅ Incremental search (fast/slow plugin separation)
3. ✅ Test `sublime_fuzzy` vs `nucleo` (1 hour benchmark)

**Phase 2: Major Features** (1-2 months) 4. ⚠️ Daemon mode (optional, if startup time feedback warrants) 5. ⚠️ Progressive plugin results (enhanced incremental)

**Phase 3: Future** (3+ months) 6. ❓ Async plugin architecture (only if adding network plugins) 7. ❌ GPU optimization (only if targeting 120Hz specifically) 8. ❌ Custom SIMD (only if search >50ms on benchmarks)

### Quick Wins to Start Today

**1. Empty State** (1 hour):

```rust
// src/main.rs line 223
// Comment out initial search:
// results_list.update_plugin_results(Vec::new());
```

**2. Placeholder Text** (30 min):

```rust
// src/ui/results_list.rs
list.set_placeholder(Some(&placeholder_widget));
```

**3. Test sublime_fuzzy** (1 hour):

```toml
[dependencies]
sublime_fuzzy = "0.7"
```

```bash
cargo bench
```

### Performance vs Complexity Tradeoff

**Current state**:

- Startup: 45-60ms ✅
- Search: 6-9ms ✅
- Memory: 18-25MB ✅

**All targets already met!**

**Focus on**:

- ✅ UX improvements (empty state, incremental)
- ✅ Code quality (better tests, docs)
- ✅ Features (more plugins, better scoring)

**Avoid**:

- ❌ Premature optimization (SIMD, GPU)
- ❌ Over-engineering (full async when not needed)

---

## Next Steps

1. **Validate assumptions**:

   - Get user feedback: Is startup time a problem?
   - Profile: Where is actual time spent?
   - Measure: Current FPS with GTK Inspector

2. **Prototype #1 (Empty State)**:

   - Implement in feature branch
   - Test with users
   - Make configurable if controversial

3. **Prototype #3 (Incremental)**:

   - Separate fast/slow plugins
   - Add visual loading indicator
   - Measure perceived performance improvement

4. **Decide on #5 (Daemon)**:

   - Collect startup time complaints
   - Profile cold start with flamegraph
   - Implement only if clear user demand

5. **Skip for now**:
   - #2 (Async) - too complex, low benefit
   - #4 (GPU) - already smooth enough
   - #6 (SIMD) - already fast enough

---

**Conclusion**: Focus on **UX wins** (#1, #3) first. Current performance is excellent; premature optimization (#4, #6) not needed. Daemon mode (#5) is "nice to have" but not critical. Async plugins (#2) should wait for actual blocking use cases.
