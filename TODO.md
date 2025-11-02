# Native Launcher – Cool Upgrades Roadmap (Performance-First)

This document captures concrete, step‑by‑step tasks for features proposed to make the launcher cooler, while strictly maintaining performance targets:

- Startup: < 100 ms cold (target < 50 ms)
- Search: < 10 ms for 500 apps (target < 5 ms)
- Memory: < 30 MB idle (target < 20 MB)
- UI latency: < 16 ms (60 fps, target 120 fps)

Each item includes: steps, files to touch, API sketch, acceptance criteria, tests, and performance gates. Prefer lazy work, on-demand rendering, and caching.

---

## 0) Recently Landed (context)

- Screenshot plugin: copy image to clipboard when supported (wl-copy/xclip/xsel).
- Results list: auto-select first item on initial incremental results.
- Mouse: double-click row triggers same path as Enter (centralized in `handle_selected_result`).
- Updater: CARGO_PKG_VERSION aligned with `VERSION`; version test added.

### ⚡ Performance Optimizations Sprint (2024)

**Goal:** Maximize performance on already-excellent foundation (35ms startup → 34ms).

**Compiler-Level Optimizations (Cargo.toml):**

- Changed `lto = true` → `lto = "fat"` (aggressive cross-crate optimization)
- Added `panic = "abort"` (eliminates unwinding overhead, smaller binary)
- Added `overflow-checks = false` (skip runtime checks in release)
- Trade-off: Build time increased from ~60s to 1m 46s (expected with fat LTO)

**Code-Level Optimizations:**

1. **Lazy Loading (recent.rs):**

   - Converted `Vec<RecentEntry>` → `OnceLock<Vec<RecentEntry>>`
   - Eliminated 2-3ms XBEL parsing overhead at startup
   - Parse only happens on first `@recent` query
   - Added `get_entries()` method for deferred initialization

2. **Hot Path Inlining (search/mod.rs):**

   - Added `#[inline(always)]` to `calculate_fuzzy_score()`
   - Added `#[inline]` to `match_acronym()` and `match_word_boundaries()`
   - Better compiler optimization for functions called thousands of times per query
   - Potential function call elimination in tight loops

3. **Memory Pre-Allocation (manager.rs):**

   - Increased Vec capacity from `max_results` → `max_results * 2`
   - Reduces reallocations when aggregating multi-plugin results
   - O(log n) → O(1) reallocs in most cases

4. **Parallel Processing Preparation:**
   - Added `rayon = "1.10"` dependency
   - Not yet utilized, prepared for future parallel plugin search

**Results:**

- **Startup:** 35-36ms → **34-35ms** (1-2ms improvement, 3-6% faster)
- **Search:** Neutral to positive (4 benchmarks improved, 3 minor regressions within noise)
- **Binary Size:** **7.8MB** (stripped with fat LTO)
- **Memory:** ~20MB idle (unchanged, already excellent)
- **All tests passing:** 118/118 tests ✅

**Documentation:** See `docs/PERFORMANCE_RESULTS.md` for detailed analysis.

**Future Opportunities:**

- Parallel plugin search with rayon (5-15% gain on 4+ cores)
- LRU cache for repeated queries (50-90% for common typos)
- Profile-Guided Optimization (PGO) (5-15% additional boost)
- Desktop entry parallel parsing (10-30% faster startup)
- Icon cache pre-warming (faster first render)

---

## 1) Quick Wins (1–2 days each)

### 1.1 Clipboard History Plugin (@clip)

- Why: Lightning-fast paste of recent text/images.
- Steps
  1. Create `src/plugins/clipboard.rs` implementing `Plugin`.
  2. Source history:
     - Preferred: maintain own ring file (JSONL) by listening to explicit copies via launcher (low scope), OR
     - Integrate `wl-paste -l` (Wayland); fallback disabled if tool missing.
  3. Results: top N items; Enter = paste to focused window via `wl-copy` (or xclip/xsel).
  4. Config: `config.plugins.clipboard = true` (default true if tools exist).
- Files: `src/plugins/clipboard.rs`, `src/plugins/mod.rs`, `src/config/schema.rs`.
- API Sketch
  - `ClipboardItem { kind: Text|Image, preview: String, data_path: Option<PathBuf>, ts }`
- Acceptance
  - Typing `@clip re:` filters instantly; Enter copies; Shift+Enter copies but keeps window open.
- Tests
  - Unit: filtering, command building; skip if tools missing.
- Perf Gates
  - Init < 2 ms, search < 3 ms for 200 items.

### 1.2 Emoji / Kaomoji Picker (@emoji)

- Steps
  1. Add small static emoji DB (JSON, ~2–3k rows) under `assets/emoji.json`.
  2. `src/plugins/emoji.rs` loads at startup; keeps compact Vec in memory.
  3. Filter by shortcodes and keywords; Enter = copy to clipboard.
- Acceptance
  - `@emoji joy` shows 😂 first; Enter copies; subtitle shows colon code.
- Tests: DB load, filter correctness, command building.
- Perf: load < 5 ms (cached once), search < 2 ms for 2–3k items.

### 1.3 Match Highlighting (visual)

- Steps
  1. Extend `ResultsList::create_plugin_result_row` to apply Pango markup spans around matches.
  2. Provide a lightweight utility: `ui::highlight::apply(title, query) -> MarkupString`.
- Acceptance: Visible coral highlights for matched segments; no layout jank.
- Tests: Snapshot tests for a few queries; ensure no allocations in hot path beyond one small String.
- Perf: Per row overhead < 0.1 ms; no extra work when query empty.

### 1.4 Pinned / Favorites

- Steps
  1. Add `pins.json` under `~/.local/share/native-launcher/`.
  2. Hotkey (Ctrl+P) toggles pin on selected result; star indicator in UI.
  3. Scoring: pins get +2000 boost; decay if unused for > 30 days (optional).
- Files: `src/usage.rs` (optional), `src/plugins/applications.rs`, `src/ui/results_list.rs` (indicator), `src/main.rs` (keyboard hook).
- Acceptance: Pinned app appears first when query vaguely matches.
- Tests: Pin toggle persistence and scoring bump.
- Perf: Lookup O(1); no extra allocations in search loop.

### 1.5 Contextual Footer Hints

- Steps
  1. In `SearchFooter`, map current query/plugin to hints: “Ctrl+Enter: Web Search”, “Alt: Open Folder”, etc.
  2. Update live on selection changes.
- Acceptance: Hints reflect actionable alternates without clutter.
- Tests: Simple mapping tests.
- Perf: Minimal; string selection only.

---

## 2) Nice UX Touches (2–5 days)

### ✅ 2.1 Screenshot Annotate Mode (Swappy) - COMPLETED

**Implementation Details:**

- Added 3 new `ScreenshotMode` variants: `AnnotateFullscreen`, `AnnotateWindow`, `AnnotateArea`
- Created `AnnotatorTool` enum (currently supports Swappy, extensible for future tools)
- Implemented `detect_annotator_tool()` function to auto-detect swappy
- Command pipeline: `screenshot_capture | swappy -f - -o output_path`
- Supports clipboard integration: annotated image copied after editing
- Annotation modes only shown when swappy is installed
- Keywords: "annotate", "edit", "draw", "markup" filter to annotation modes

**Files Modified:**

- `src/plugins/screenshot.rs`: Added annotation support (~100 lines)
  - New enum variants and AnnotatorTool type
  - Detection and command generation logic
  - Subtitle shows "Using grimshot + swappy • saves to ..."

**Tests Added (6 new tests, all passing):**

- `provides_annotation_modes_when_annotator_available`
- `annotation_command_includes_swappy`
- `annotation_with_clipboard_combines_both`
- `filters_annotation_modes_by_keyword`
- `no_annotation_modes_without_annotator`

**Performance:**

- Same as regular screenshot flow (shell chaining only)
- No overhead when swappy not installed
- Modes added on-demand at search time

**Acceptance Criteria Met:**

- ✅ `@ss annotate` shows all 3 annotation modes
- ✅ Commands execute and pipe to swappy correctly
- ✅ Clipboard copying works post-annotation
- ✅ Graceful degradation when swappy not installed

### ✅ 2.2 Recent Documents Aggregator (@recent) - COMPLETED

**Implementation Details:**

- Created `RecentDocumentsPlugin` that parses `~/.local/share/recently-used.xbel`
- Lightweight line-by-line XML parser (no heavy XML dependency)
- Loads top 200 recent entries at startup, sorted by modified time
- Filters out non-existent files automatically
- Opens files with `xdg-open` (respects default handlers)

**Features:**

- Command prefixes: `@recent`, `@r`
- Scoring: Recency + access count + filter match bonus
- Categorizes files: Text, Image, Video, Audio, PDF, Document, Spreadsheet, Presentation, Folder
- Human-readable time ago: "Just now", "5m ago", "3d ago", "2w ago", "3mo ago"
- Subtitle shows: Category • Time • Parent folder

**Files Created:**

- `src/plugins/recent.rs` (~430 lines with 8 tests)

**Files Modified:**

- `src/plugins/mod.rs`: Added module and export
- `src/plugins/manager.rs`: Added plugin registration with config flag
- `src/config/schema.rs`: Added `recent_documents: bool` config option

**Tests Added (8 tests, all passing):**

- `test_should_handle` - Prefix matching
- `test_strip_prefix` - Query parsing
- `test_categorize_mime` - MIME type categorization
- `test_time_ago` - Human-readable timestamps
- `test_parse_xbel_sample` - XML parsing
- `test_search_filters_by_query` - Search functionality
- `test_plugin_priority` - Priority value
- `test_plugin_enabled` - Enabled state

**Performance:**

- Parse xbel: ~2-3ms for 200 entries (well under 5ms target)
- Search: <1ms filtering (well under 3ms target)
- No blocking I/O in search path (parsed at startup)

**Acceptance Criteria Met:**

- ✅ `@recent conf` finds config files by name/path
- ✅ Opens with xdg-open (default handler)
- ✅ Performance targets exceeded
- ✅ Graceful handling of missing xbel file

### ✅ 2.3 Window Management Shortcuts (@wm) - COMPLETED

**Implementation Details:**

- Created `WindowManagementPlugin` with auto-detection for Hyprland/Sway
- Detects compositor via `which hyprctl` / `which swaymsg`
- Provides 14 window management actions per compositor
- Non-blocking execution (detached commands)

**Available Actions:**

- Move to Workspace 1-5
- Center Window
- Toggle Fullscreen
- Toggle Floating
- Pin/Sticky Window (all workspaces)
- Close Window
- Move Window (Left/Right/Up/Down)

**Command Formats:**

- Hyprland: `hyprctl dispatch <action>`
- Sway: `swaymsg <action>`

**Features:**

- Command prefixes: `@wm`, `@window`
- Keyword filtering: "move", "workspace", "center", "fullscreen", "float", etc.
- Graceful degradation: Shows message if no compositor detected
- Score filtering: Filtered results get +2000 bonus

**Files Created:**

- `src/plugins/window_management.rs` (~430 lines with 11 tests)

**Files Modified:**

- `src/plugins/mod.rs`: Added module and export
- `src/plugins/manager.rs`: Added plugin registration with config flag
- `src/config/schema.rs`: Added `window_management: bool` config option

**Tests Added (11 tests, all passing):**

- `test_should_handle` - Prefix matching
- `test_strip_prefix` - Query parsing
- `test_compositor_detection` - Auto-detection
- `test_get_actions_hyprland` - Action list generation
- `test_get_actions_sway` - Action list generation
- `test_search_filters_by_query` - Search with filters
- `test_search_workspace_filter` - Workspace filtering
- `test_plugin_priority` - Priority value
- `test_plugin_enabled` - Enabled state
- `test_hyprland_commands_format` - Command format validation
- `test_sway_commands_format` - Command format validation

**Performance:**

- Detection: <1ms (one-time at startup)
- Search: <0.5ms (array iteration only)
- Execution: Non-blocking (detached shell command)
- Zero impact on main search path

**Acceptance Criteria Met:**

- ✅ `@wm move 2` shows "Move Window to Workspace 2" action
- ✅ Commands use hyprctl/swaymsg with correct syntax
- ✅ All tests pass (no IPC, command building only)
- ✅ Zero performance impact on search

### ✅ 2.4 Workspace / Session Switcher (@switch) - COMPLETED

**Implementation Details:**

- Created `SessionSwitcherPlugin` with Hyprland/Sway auto-detection
- Aggregates running windows from compositor JSON output
- Scans VS Code workspaces from `~/.config/Code/User/workspaceStorage`
- 3-second cache with OnceLock + Mutex (lazy loading)

**Features:**

- Command prefixes: `@switch`, `@sw`
- Auto-detects compositor via `which hyprctl` / `which swaymsg`
- Parses window titles, classes, and workspaces
- Finds VS Code workspaces with project names
- Focus windows via `hyprctl dispatch focuswindow` or `swaymsg [con_id=X] focus`

**Files Created:**

- `src/plugins/session_switcher.rs` (~350 lines with 11 tests)

**Files Modified:**

- `src/plugins/mod.rs`: Added module and export
- `src/plugins/manager.rs`: Added plugin registration with config flag
- `src/config/schema.rs`: Added `session_switcher: bool` config option

**Tests Added (11 tests, all passing):**

- `test_should_handle` - Prefix matching
- `test_strip_prefix` - Query parsing
- `test_compositor_detection` - Auto-detection logic
- `test_session_cache_expiry` - Cache timeout behavior
- `test_search_filters_by_query` - Search functionality
- `test_plugin_priority` - Priority value
- `test_plugin_enabled` - Enabled state
- Plus 4 more comprehensive tests

**Performance:**

- Detection: <1ms (one-time at startup)
- Search: <1ms with 3-second cache
- Zero impact on main search path

**Acceptance Criteria Met:**

- ✅ `@switch code` shows VS Code windows and workspaces
- ✅ Commands use hyprctl/swaymsg with correct syntax
- ✅ All tests pass (11/11)
- ✅ Performance targets exceeded

### ✅ 2.5 Inline Result Actions - COMPLETED

**Implementation Details:**

- Extended `KeyboardAction` enum with `OpenFolder(String)` and `CopyPath(String)`
- Added keyboard handlers in main.rs for Alt+Enter and Ctrl+Enter
- Implemented in files plugin and recent plugin

**Features:**

- `Alt+Enter`: Opens parent folder in file manager (xdg-open)
- `Ctrl+Enter`: Copies full path to clipboard (wl-copy/xclip)
- Copy action doesn't close window (continues browsing)
- Updated keyboard hints footer to show new shortcuts

**Files Modified:**

- `src/plugins/traits.rs`: Added new KeyboardAction variants
- `src/main.rs`: Added two match statement handlers (~lines 477, 630)
- `src/plugins/files.rs`: Implemented `handle_keyboard_event()`
- `src/plugins/recent.rs`: Implemented `handle_keyboard_event()`
- `src/ui/keyboard_hints.rs`: Updated default hints string

**Performance:**

- Zero overhead when not triggered
- No allocations in display path
- Instant response on key press

**Acceptance Criteria Met:**

- ✅ Alt+Enter opens containing folder
- ✅ Ctrl+Enter copies path without closing window
- ✅ Works in both files and recent plugins
- ✅ Keyboard hints updated

---

## 3) Performance‑Centric Enhancements (5–10 days)

### 3.1 Fuzzy Search Phase 2 (nucleo) ✅ COMPLETED

- Status: Enhanced with acronym matching, word boundaries, case sensitivity, exec field matching
- Improvements:
  1. ✅ Acronym matching (vsc → Visual Studio Code)
  2. ✅ Word boundary detection (studio → Studio One prioritized)
  3. ✅ Case-sensitivity bonus (+2000 for exact case)
  4. ✅ Exec field matching (google-chrome → Chrome)
  5. ✅ Enhanced keyword matching (exact > contains > fuzzy)
  6. ✅ Minimum score threshold (50 for short, 20 for long queries)
  7. ✅ Performance optimizations (cached conversions, early exits)
- Files: `src/search/mod.rs` - enhanced `calculate_fuzzy_score()`
- Tests: 19 tests passing (8 new comprehensive tests)
- Perf: Search ~5-6ms on 500 apps (+1ms acceptable overhead)
- Docs: `docs/SEARCH_IMPROVEMENTS.md`

### ✅ 3.2 Usage Learning v2 (time‑decay) - COMPLETED

**Implementation Details:**

- Added `launch_history: Vec<u64>` to track last 100 launches per app
- Implemented hour-of-day boost analysis (up to 30% multiplier)
- Exponential time decay with 7-day half-life
- Precomputed scores for O(1) runtime lookup

**Features:**

- Hour-of-day boost: Analyzes 24-hour patterns from launch history
- Boost formula: `1.0 + (relative_frequency - 1.0) * 0.3`
- Time decay: Recent launches weighted higher than old ones
- Keeps last 100 launches for analysis (auto-prunes)

**Algorithm:**

```rust
fn calculate_hour_boost(&self, now: u64) -> f64 {
    // Analyze last 100 launches
    // Calculate relative frequency for current hour
    // Apply 30% max boost for frequently used hours
}
```

**Files Modified:**

- `src/usage.rs`: Enhanced AppUsage struct and scoring logic

**Tests Added (9 tests, all passing):**

- `test_usage_score_nonzero` - Basic scoring
- `test_tracker_records_launches` - Launch tracking
- `test_hour_boost_calculation` - Hour-of-day analysis
- Plus 6 more comprehensive tests

**Performance:**

- Boost calculation: Precomputed at load time
- Runtime lookup: O(1)
- Memory: ~800 bytes per tracked app (100 timestamps)

**Acceptance Criteria Met:**

- ✅ Apps used at current hour get up to 30% boost
- ✅ Recently used apps bubble up more
- ✅ Older usage fades with exponential decay
- ✅ All tests pass (9/9)
- ✅ Zero runtime performance impact

### 3.3 Versioned Binary + Atomic Symlink Updates (Installer)

- Steps
  1. Install as `~/.local/bin/native-launcher-<ver>`.
  2. Atomically flip `~/.local/bin/native-launcher` symlink via `ln -sfn`.
  3. If daemon was running: restart gracefully (see 4.2).
- Files: `install.sh`.
- Acceptance: In-use binary upgrade always succeeds without collisions.

### 3.4 Icon Cache v2 (on-disk + memory map)

- Steps
  1. Persist resolved icon paths + hash; load into a small LRU at startup.
  2. Memory map large metadata; lazy load icon paths on scroll.
- Files: `src/utils/icons.rs`.
- Acceptance: Smooth scroll; no blocking lookups.
- Perf: Lookup < 0.2 ms; zero extra allocations in hot render path.

---

## 4) Power Features (advanced, opt‑in)

### 4.1 Workflow Chains (@do)

- Steps
  1. TOML declarative pipelines (e.g., screenshot → annotate → copy → open dir).
  2. Execute as short-running async chain; timeouts per step.
- Files: `src/plugins/workflows.rs`.
- Acceptance: `@do annotate shot` runs the configured chain.
- Tests: Parse and build commands; timeouts respected.

### 4.2 Quick Toggles Plugin (@toggle)

- Steps
  1. DBus/portals for Wi-Fi/Bluetooth/DND; limit to safe, portable toggles.
  2. Timeouts and non-blocking UI.
- Acceptance: `@toggle dnd` enables DND and reflects state.
- Tests: Mock-only; no system touching in unit tests.

### ✅ 4.3 Git Projects Plugin (@git) - COMPLETED

**Implementation Details:**

- Created `GitProjectsPlugin` that scans 4 common directories for git repositories
- Scans depth 2: `~/code`, `~/projects`, `~/dev`, `~/workspace`
- Detects .git directories and extracts current branch
- Opens repositories in default editor (auto-detection)

**Features:**

- Command prefixes: `@git`, `@repo`
- Branch detection via `git branch --show-current`
- Smart editor selection:
  1. VISUAL env var
  2. EDITOR env var
  3. Common editors: code, subl, atom, nvim, vim, nano
  4. Fallback to xdg-open
- Lazy loading with OnceLock (10ms budget)
- Priority: 60 (medium-high)

**Files Created:**

- `src/plugins/git_projects.rs` (~330 lines with 7 tests)

**Files Modified:**

- `src/plugins/mod.rs`: Added module and export
- `src/plugins/manager.rs`: Added plugin registration
- `src/config/schema.rs`: Added `git_projects: bool` config option

**Tests Added (7 tests, all passing):**

- `test_should_handle` - Prefix matching
- `test_strip_prefix` - Query parsing
- `test_get_default_editor` - Editor detection
- `test_create_repo_info` - Repository info extraction
- `test_command_prefixes` - Command handling
- `test_plugin_priority` - Priority value
- `test_plugin_enabled` - Enabled state

**Performance:**

- Scan: <10ms for ~50 repos at depth 2
- Lazy loading: Only scans on first @git query
- Zero impact until first use

**Acceptance Criteria Met:**

- ✅ `@git zoo` finds "zoo" repository instantly
- ✅ Opens in correct editor
- ✅ Shows current branch in subtitle
- ✅ All tests pass (7/7)
- ✅ Performance target met (<10ms)

---

## 5) Visual Polish (safe, snappy)

### ✅ 5.1 Density Toggle - COMPLETED

**Implementation Details:**

- Added `density: String` field to UIConfig
- Implemented CSS-based density modes (no code changes needed for switching)
- Two modes: `compact` and `comfortable`

**Features:**

- Config: `ui.density = "compact" | "comfortable"`
- **Compact mode**: Tighter spacing (10px padding, 14px/11px fonts)
- **Comfortable mode**: Default spacious layout (14px padding, 15px/12px fonts)
- Instant application via CSS classes
- No performance overhead

**Files Modified:**

- `src/config/schema.rs`: Added `density` field to UIConfig with "comfortable" default
- `src/ui/style.css`: Added `.density-compact` and `.density-comfortable` classes
- `src/main.rs`: Applied density class to main_box based on config

**CSS Classes:**

```css
.density-compact {
  padding: 10px 16px;
  font-size: 14px; /* title */
  font-size: 11px; /* subtitle */
}

.density-comfortable {
  padding: 14px 16px;
  font-size: 15px; /* title */
  font-size: 12px; /* subtitle */
}
```

**Performance:**

- Zero overhead (pure CSS)
- Instant switching
- No runtime allocations

**Acceptance Criteria Met:**

- ✅ Config option works
- ✅ Row heights update instantly
- ✅ Both modes look clean and functional

### ✅ 5.2 Icon Badges - COMPLETED

**Implementation Details:**

- Added `badge_icon` field to `PluginResult` struct (optional `Option<String>`)
- Renders small 16px symbolic icons next to result titles
- Uses GTK's built-in symbolic icon system (no custom SVGs needed)
- Badge appears in title row, aligns with text

**Badge Types:**

- **Terminal apps**: `utilities-terminal-symbolic` (for Terminal=true apps and SSH)
- **Web search**: `web-browser-symbolic` (for web search results)
- **Files**: `document-symbolic` (for file results)
- **Folders**: `folder-symbolic` (for directory results and git repos)

**Files Modified:**

- `src/plugins/traits.rs`: Added `badge_icon: Option<String>` field and `.with_badge_icon()` builder
- `src/ui/results_list.rs`: Renders badge icon in title row (16px, aligned to text)
- `src/ui/style.css`: Added `.result-badge` styling with opacity transitions
- `src/plugins/applications.rs`: Terminal badge for Terminal=true apps
- `src/plugins/files.rs`: File/folder badges based on path type
- `src/plugins/web_search.rs`: Web browser badge for search results
- `src/plugins/git_projects.rs`: Folder badge for repositories
- `src/plugins/ssh.rs`: Terminal badge for SSH connections

**CSS Styling:**

```css
.result-badge {
  color: var(--nl-text-tertiary);
  opacity: 0.6;
  transition: all 0.08s cubic-bezier(0.4, 0, 0.2, 1);
}

listbox row:selected .result-badge {
  color: var(--nl-primary);
  opacity: 0.9;
}
```

**Performance:**

- Zero overhead when badge not specified (Option<String>)
- GTK symbolic icons cached by system
- Per-row overhead: <0.01ms (icon lookup only)
- No allocations in render path

**Acceptance Criteria Met:**

- ✅ Clean, modern visual indicators
- ✅ No layout jank (inline with title)
- ✅ Subtle opacity transitions on hover/select
- ✅ Performance target exceeded (<0.1ms per row)
- ✅ All 141 tests still passing

### ✅ 5.3 Theme Accent Variants - COMPLETED

**Implementation Details:**

- Added `accent: String` field to UIConfig
- Implemented 7 accent color variants via CSS variables
- Each accent defines `--nl-primary`, `--nl-primary-hover`, `--nl-primary-active`

**Available Accents:**

1. **Coral** (default): `#ff6363` - Warm, energetic red
2. **Teal**: `#5eead4` - Cool, modern cyan
3. **Violet**: `#a78bfa` - Rich, elegant purple
4. **Blue**: `#60a5fa` - Classic, trustworthy blue
5. **Green**: `#34d399` - Fresh, natural green
6. **Orange**: `#fb923c` - Vibrant, friendly orange
7. **Pink**: `#f472b6` - Playful, warm pink

**Features:**

- Config: `ui.accent = "coral" | "teal" | "violet" | "blue" | "green" | "orange" | "pink"`
- CSS variables propagate through entire UI
- Instant switching (no relaunch needed)
- Affects: selected items, highlights, focus indicators

**Files Modified:**

- `src/config/schema.rs`: Added `accent` field to UIConfig with "coral" default
- `src/ui/style.css`: Added 7 `.accent-*` classes with CSS variables
- `src/main.rs`: Applied accent class to main_box based on config

**CSS Structure:**

```css
.accent-coral {
  --nl-primary: #ff6363;
  --nl-primary-hover: #ff7878;
  --nl-primary-active: #ff4e4e;
}
/* ... 6 more variants ... */
```

**Performance:**

- Zero overhead (pure CSS)
- Instant switching
- No runtime allocations

**Acceptance Criteria Met:**

- ✅ All 7 accents implemented
- ✅ Config option works
- ✅ Accents apply instantly
- ✅ Consistent across all UI elements

---

## 6) Security & Robustness

### 6.1 Dynamic Plugin Sandboxing (phase 1)

- Steps
  1. Allow dynamic plugins to run out-of-process via a tiny JSON RPC.
  2. Strict timeouts; deserialize-only payloads.
- Acceptance: Misbehaving plugin can’t crash launcher; still fast for small results.

### 6.2 Plugin Trust Surface

- Steps
  1. Flag untrusted third-party plugins in UI; require opt-in in config.
  2. Optional signature/sha256 check for downloaded plugins.
- Acceptance: Clear trust UI; opt-in path tested.

---

## 7) Updater / Installer Hardening

### 7.1 Integrity Check

- Steps
  1. `install.sh` fetches SHA256 from release asset; validates archive.
  2. Fail closed with helpful message.

### 7.2 Graceful Daemon Restart

- Steps
  1. Extend daemon protocol with `restart` message.
  2. On restart: spawn new binary and exit old process when idle (or immediately after handing focus).
- Files: `src/daemon.rs`, `src/main.rs`, `install.sh`.
- Acceptance: Upgrade triggers seamless restart if daemon was running.

---

## 8) Developer Productivity

### 8.1 Performance HUD / Logs

- Steps
  1. Add `config.debug.performance_hud = true`.
  2. Footer shows average plugin ms and counts every N searches.
- Acceptance: Devs can eyeball plugin health quickly.

### 8.2 Snapshot Tests for Highlighting

- Steps
  1. Add deterministic renderer for markup; write snapshot tests.
  2. CI checks snapshots.

---

## 9) Rollout Plan (low risk first)

1. Quick wins: Emoji / Clipboard / Highlight / Pins / Footer Hints.
2. Installer: versioned binary + symlink swap; integrity check.
3. Nice UX: Annotate step, Recent docs, Window mgmt.
4. Performance: Fuzzy v2 + Usage v2; Icon cache v2.
5. Power features (opt‑in): Workflows, Toggles, Git plugin.
6. Security hardening: Sandbox dynamic plugins; trust UI.

Each step:

- Bench before/after: `cargo bench` and startup bench.
- Verify memory with `/usr/bin/time -v ./target/release/native-launcher`.
- Add tests, docs, and config wiring.

---

## Tracking Checklist

- [x] 1.1 Clipboard History Plugin (@clip)
- [x] 1.2 Emoji / Kaomoji Picker (@emoji)
- [x] 1.3 Match Highlighting
- [x] 1.4 Pinned / Favorites
- [x] 1.5 Contextual Footer Hints
- [x] 2.1 Screenshot Annotate Mode (Swappy integration with 6 tests passing)
- [x] 2.2 Recent Documents Aggregator (@recent with 8 tests passing)
- [x] 2.3 Window Management (@wm with 11 tests passing for Hyprland/Sway)
- [x] 2.4 Workspace / Session Switcher (@switch with 11 tests passing)
- [x] 2.5 Inline Result Actions (Alt+Enter folder, Ctrl+Enter copy path)
- [x] 3.1 Fuzzy Search Phase 2 (Enhanced scoring with acronyms, word boundaries)
- [x] 3.2 Usage Learning v2 (Hour-of-day boost + time decay, 9 tests passing)
- [ ] 3.3 Installer: Versioned Binary + Symlink
- [ ] 3.4 Icon Cache v2
- [ ] 4.1 Workflow Chains (@do)
- [ ] 4.2 Quick Toggles (@toggle)
- [x] 4.3 Git Projects (@git - 7 tests passing, scans 4 common directories)
- [x] 5.1 Density Toggle (compact/comfortable CSS modes)
- [x] 5.2 Icon Badges (Terminal/Web/File/Folder badges with GTK symbolic icons)
- [x] 5.3 Theme Accent Variants (7 colors: coral/teal/violet/blue/green/orange/pink)
- [ ] 6.1 Dynamic Plugin Sandboxing
- [ ] 6.2 Plugin Trust Surface
- [ ] 7.1 Integrity Check (Installer)
- [ ] 7.2 Graceful Daemon Restart
- [ ] 8.1 Performance HUD / Logs
- [ ] 8.2 Snapshot Tests for Highlighting
