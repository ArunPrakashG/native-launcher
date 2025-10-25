//! Dynamic plugin loading support
//!
//! Enables loading external plugins compiled as shared libraries (.so files on Linux).
//! Plugins must implement the C FFI interface defined in `PluginFFI`.

use crate::plugins::traits::{KeyboardAction, KeyboardEvent, Plugin, PluginContext, PluginResult};
use anyhow::{anyhow, Context, Result};
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Plugin ABI version - must match between launcher and plugin
/// Increment when breaking changes are made to the FFI interface
const PLUGIN_ABI_VERSION: u32 = 1;

/// Plugin loading metrics
#[derive(Debug, Clone)]
pub struct PluginMetrics {
    /// Time taken to load the plugin (library loading + FFI symbol resolution)
    pub load_time: Duration,
    /// Memory consumed by the plugin library (approximate, based on file size)
    pub memory_bytes: u64,
    /// Path to the plugin file
    pub path: PathBuf,
    /// Whether loading was successful
    pub success: bool,
    /// Error message if loading failed
    #[allow(dead_code)] // Used for error reporting in UI
    pub error: Option<String>,
}

impl PluginMetrics {
    /// Check if this plugin is slow (>10ms load time)
    #[allow(dead_code)] // Part of performance monitoring API
    pub fn is_slow(&self) -> bool {
        self.load_time.as_millis() > 10
    }

    /// Check if this plugin is very slow (>50ms load time)
    pub fn is_very_slow(&self) -> bool {
        self.load_time.as_millis() > 50
    }

    /// Check if this plugin uses a lot of memory (>5MB)
    #[allow(dead_code)] // Part of performance monitoring API
    pub fn is_memory_heavy(&self) -> bool {
        self.memory_bytes > 5 * 1024 * 1024
    }

    /// Get a human-readable size string
    pub fn memory_size_string(&self) -> String {
        let kb = self.memory_bytes as f64 / 1024.0;
        if kb < 1024.0 {
            format!("{:.1} KB", kb)
        } else {
            let mb = kb / 1024.0;
            format!("{:.1} MB", mb)
        }
    }
}

/// C-compatible string slice
#[repr(C)]
pub struct CStringSlice {
    pub ptr: *const c_char,
    pub len: usize,
}

impl CStringSlice {
    /// Convert to Rust String (unsafe - must be valid UTF-8)
    unsafe fn to_string(&self) -> Result<String> {
        if self.ptr.is_null() {
            return Ok(String::new());
        }
        let slice = std::slice::from_raw_parts(self.ptr as *const u8, self.len);
        String::from_utf8(slice.to_vec()).context("Invalid UTF-8 in plugin string")
    }

    /// Create from Rust string (caller must keep CString alive)
    fn from_cstring(s: &CString) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.as_bytes().len(),
        }
    }
}

/// C-compatible result array
#[repr(C)]
pub struct CResultArray {
    pub ptr: *mut CPluginResult,
    pub len: usize,
    pub capacity: usize,
}

/// C-compatible plugin result
#[repr(C)]
pub struct CPluginResult {
    pub title: CStringSlice,
    pub subtitle: CStringSlice,
    pub icon: CStringSlice,
    pub command: CStringSlice,
    pub terminal: bool,
    pub score: i64,
}

/// C-compatible plugin context
#[repr(C)]
pub struct CPluginContext {
    pub max_results: usize,
    pub include_low_scores: bool,
}

/// C-compatible keyboard event
#[repr(C)]
pub struct CKeyboardEvent {
    pub key_val: u32,
    pub modifiers: u32,
    pub query: CStringSlice,
    pub has_selection: bool,
}

/// C-compatible keyboard action
#[repr(C)]
#[allow(dead_code)] // FFI enum - all variants part of plugin API
pub enum CKeyboardAction {
    None,
    Execute,
    OpenUrl,
    Handled,
}

/// C-compatible keyboard action with data
#[repr(C)]
pub struct CKeyboardActionData {
    pub action: CKeyboardAction,
    pub data: CStringSlice,
    pub terminal: bool,
}

/// Plugin FFI interface - all functions must be `extern "C"` with stable ABI
///
/// To create a compatible plugin:
/// 1. Add `crate-type = ["cdylib"]` to Cargo.toml
/// 2. Implement these functions with `#[no_mangle]` and `extern "C"`
/// 3. Compile with `cargo build --release`
/// 4. Copy the .so file to `~/.config/native-launcher/plugins/`
#[allow(dead_code)] // FFI struct - all fields part of plugin API
pub struct PluginFFI {
    /// Get ABI version (must return PLUGIN_ABI_VERSION)
    pub get_abi_version: unsafe extern "C" fn() -> u32,
    /// Get plugin name
    pub get_name: unsafe extern "C" fn() -> CStringSlice,
    /// Get plugin description
    pub get_description: unsafe extern "C" fn() -> CStringSlice,
    /// Get plugin priority
    pub get_priority: unsafe extern "C" fn() -> c_int,
    /// Check if plugin should handle query
    pub should_handle: unsafe extern "C" fn(query: CStringSlice) -> bool,
    /// Search for results
    pub search: unsafe extern "C" fn(query: CStringSlice, context: CPluginContext) -> CResultArray,
    /// Handle keyboard event
    pub handle_keyboard_event: unsafe extern "C" fn(event: CKeyboardEvent) -> CKeyboardActionData,
    /// Free result array (plugin must provide this to free its own memory)
    pub free_results: unsafe extern "C" fn(results: CResultArray),
    /// Free string data (plugin must provide this)
    pub free_string: unsafe extern "C" fn(data: CStringSlice),
}

/// Wrapper for dynamically loaded plugin
pub struct DynamicPlugin {
    name: String,
    description: String,
    priority: i32,
    #[allow(dead_code)]
    library: Library,
    ffi: PluginFFI,
    /// Metrics collected during plugin loading
    pub metrics: PluginMetrics,
}

impl DynamicPlugin {
    /// Load a plugin from a shared library file
    pub fn load(path: &Path) -> Result<Self> {
        let start_time = std::time::Instant::now();
        info!("Loading plugin from: {}", path.display());

        // Get file size for memory tracking
        let memory_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // SAFETY: Loading shared library - plugin must be compiled correctly
        let library = unsafe {
            Library::new(path).context(format!("Failed to load library: {}", path.display()))?
        };

        // Load all required FFI functions
        let get_abi_version: Symbol<unsafe extern "C" fn() -> u32> = unsafe {
            library
                .get(b"plugin_get_abi_version")
                .context("Missing plugin_get_abi_version")?
        };

        let get_name: Symbol<unsafe extern "C" fn() -> CStringSlice> = unsafe {
            library
                .get(b"plugin_get_name")
                .context("Missing plugin_get_name")?
        };

        let get_description: Symbol<unsafe extern "C" fn() -> CStringSlice> = unsafe {
            library
                .get(b"plugin_get_description")
                .context("Missing plugin_get_description")?
        };

        let get_priority: Symbol<unsafe extern "C" fn() -> c_int> = unsafe {
            library
                .get(b"plugin_get_priority")
                .context("Missing plugin_get_priority")?
        };

        let should_handle: Symbol<unsafe extern "C" fn(CStringSlice) -> bool> = unsafe {
            library
                .get(b"plugin_should_handle")
                .context("Missing plugin_should_handle")?
        };

        let search: Symbol<unsafe extern "C" fn(CStringSlice, CPluginContext) -> CResultArray> = unsafe {
            library
                .get(b"plugin_search")
                .context("Missing plugin_search")?
        };

        let handle_keyboard_event: Symbol<
            unsafe extern "C" fn(CKeyboardEvent) -> CKeyboardActionData,
        > = unsafe {
            library
                .get(b"plugin_handle_keyboard_event")
                .context("Missing plugin_handle_keyboard_event")?
        };

        let free_results: Symbol<unsafe extern "C" fn(CResultArray)> = unsafe {
            library
                .get(b"plugin_free_results")
                .context("Missing plugin_free_results")?
        };

        let free_string: Symbol<unsafe extern "C" fn(CStringSlice)> = unsafe {
            library
                .get(b"plugin_free_string")
                .context("Missing plugin_free_string")?
        };

        // Verify ABI version
        let abi_version = unsafe { get_abi_version() };
        if abi_version != PLUGIN_ABI_VERSION {
            return Err(anyhow!(
                "Plugin ABI version mismatch: expected {}, got {}",
                PLUGIN_ABI_VERSION,
                abi_version
            ));
        }

        // Get plugin metadata
        let name = unsafe {
            let name_slice = get_name();
            name_slice
                .to_string()
                .context("Failed to read plugin name")?
        };

        let description = unsafe {
            let desc_slice = get_description();
            desc_slice
                .to_string()
                .context("Failed to read plugin description")?
        };

        let priority = unsafe { get_priority() };

        // Store function pointers (dereferencing Symbols)
        let ffi = PluginFFI {
            get_abi_version: *get_abi_version,
            get_name: *get_name,
            get_description: *get_description,
            get_priority: *get_priority,
            should_handle: *should_handle,
            search: *search,
            handle_keyboard_event: *handle_keyboard_event,
            free_results: *free_results,
            free_string: *free_string,
        };

        let load_time = start_time.elapsed();

        info!(
            "Loaded plugin '{}' (priority: {}, load_time: {:?}, size: {}) from {}",
            name,
            priority,
            load_time,
            if memory_bytes < 1024 {
                format!("{} B", memory_bytes)
            } else if memory_bytes < 1024 * 1024 {
                format!("{:.1} KB", memory_bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", memory_bytes as f64 / 1024.0 / 1024.0)
            },
            path.display()
        );

        // Warn if plugin is slow
        if load_time.as_millis() > 50 {
            warn!(
                "Plugin '{}' took {:?} to load (>50ms threshold)",
                name, load_time
            );
        }

        let metrics = PluginMetrics {
            load_time,
            memory_bytes,
            path: path.to_path_buf(),
            success: true,
            error: None,
        };

        Ok(Self {
            name,
            description,
            priority,
            library,
            ffi,
            metrics,
        })
    }
}

impl std::fmt::Debug for DynamicPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicPlugin")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("priority", &self.priority)
            .finish()
    }
}

impl Plugin for DynamicPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn should_handle(&self, query: &str) -> bool {
        let query_cstr = match CString::new(query) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let query_slice = CStringSlice::from_cstring(&query_cstr);
        unsafe { (self.ffi.should_handle)(query_slice) }
    }

    fn search(&self, query: &str, context: &PluginContext) -> Result<Vec<PluginResult>> {
        let query_cstr = CString::new(query).context("Invalid query string")?;
        let query_slice = CStringSlice::from_cstring(&query_cstr);

        let c_context = CPluginContext {
            max_results: context.max_results,
            include_low_scores: context.include_low_scores,
        };

        let c_results = unsafe { (self.ffi.search)(query_slice, c_context) };

        // Convert C results to Rust
        let mut results = Vec::new();
        if !c_results.ptr.is_null() {
            unsafe {
                let slice = std::slice::from_raw_parts(c_results.ptr, c_results.len);
                for c_result in slice {
                    let title = c_result.title.to_string()?;
                    let subtitle = c_result.subtitle.to_string().ok();
                    let icon = c_result.icon.to_string().ok();
                    let command = c_result.command.to_string()?;

                    let mut result = PluginResult::new(title, command, self.name.clone())
                        .with_score(c_result.score);
                    if let Some(sub) = subtitle.filter(|s| !s.is_empty()) {
                        result = result.with_subtitle(sub);
                    }
                    if let Some(ico) = icon.filter(|s| !s.is_empty()) {
                        result = result.with_icon(ico);
                    }
                    result = result.with_terminal(c_result.terminal);

                    results.push(result);
                }
            }
        }

        // Free C memory
        unsafe {
            (self.ffi.free_results)(c_results);
        }

        Ok(results)
    }

    fn handle_keyboard_event(&self, event: &KeyboardEvent) -> KeyboardAction {
        let query_cstr = match CString::new(event.query.clone()) {
            Ok(s) => s,
            Err(_) => return KeyboardAction::None,
        };
        let query_slice = CStringSlice::from_cstring(&query_cstr);

        let c_event = CKeyboardEvent {
            key_val: event.key.to_unicode().unwrap_or('\0') as u32,
            modifiers: event.modifiers.bits(),
            query: query_slice,
            has_selection: event.has_selection,
        };

        let c_action = unsafe { (self.ffi.handle_keyboard_event)(c_event) };

        let action = match c_action.action {
            CKeyboardAction::None => KeyboardAction::None,
            CKeyboardAction::Execute => {
                let command = unsafe { c_action.data.to_string().unwrap_or_default() };
                KeyboardAction::Execute {
                    command,
                    terminal: c_action.terminal,
                }
            }
            CKeyboardAction::OpenUrl => {
                let url = unsafe { c_action.data.to_string().unwrap_or_default() };
                KeyboardAction::OpenUrl(url)
            }
            CKeyboardAction::Handled => KeyboardAction::Handled,
        };

        // Free string data
        unsafe {
            (self.ffi.free_string)(c_action.data);
        }

        action
    }
}

/// Discover and load all plugins from standard directories
/// Returns tuple of (plugins, all_metrics)
pub fn load_plugins() -> (Vec<Box<dyn Plugin>>, Vec<PluginMetrics>) {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    let mut all_metrics: Vec<PluginMetrics> = Vec::new();

    // Search paths in order of priority
    let search_paths = get_plugin_search_paths();

    for dir in search_paths {
        if !dir.exists() {
            debug!("Plugin directory doesn't exist: {}", dir.display());
            continue;
        }

        info!("Scanning for plugins in: {}", dir.display());

        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // Only load .so files (Linux shared libraries)
                    if path.extension().and_then(|s| s.to_str()) != Some("so") {
                        continue;
                    }

                    match DynamicPlugin::load(&path) {
                        Ok(plugin) => {
                            info!("Successfully loaded plugin: {}", plugin.name());
                            all_metrics.push(plugin.metrics.clone());
                            plugins.push(Box::new(plugin));
                        }
                        Err(e) => {
                            warn!("Failed to load plugin from {}: {}", path.display(), e);
                            // Record failed load attempt
                            let memory_bytes =
                                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            all_metrics.push(PluginMetrics {
                                load_time: std::time::Duration::from_secs(0),
                                memory_bytes,
                                path: path.clone(),
                                success: false,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read plugin directory {}: {}", dir.display(), e);
            }
        }
    }

    info!("Loaded {} dynamic plugins", plugins.len());

    // Log summary statistics
    if !all_metrics.is_empty() {
        let total_load_time: std::time::Duration = all_metrics
            .iter()
            .filter(|m| m.success)
            .map(|m| m.load_time)
            .sum();
        let avg_load_time =
            total_load_time / all_metrics.iter().filter(|m| m.success).count().max(1) as u32;
        let total_memory: u64 = all_metrics.iter().map(|m| m.memory_bytes).sum();

        info!(
            "Plugin metrics: total_load_time={:?}, avg_load_time={:?}, total_memory={}",
            total_load_time,
            avg_load_time,
            if total_memory < 1024 * 1024 {
                format!("{:.1} KB", total_memory as f64 / 1024.0)
            } else {
                format!("{:.1} MB", total_memory as f64 / 1024.0 / 1024.0)
            }
        );

        // Warn about slow plugins
        let slow_plugins: Vec<_> = all_metrics
            .iter()
            .filter(|m| m.success && m.is_very_slow())
            .collect();

        if !slow_plugins.is_empty() {
            warn!(
                "Detected {} slow plugins (>50ms load time):",
                slow_plugins.len()
            );
            for metric in slow_plugins {
                warn!(
                    "  - {} took {:?}",
                    metric
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    metric.load_time
                );
            }
        }
    }

    (plugins, all_metrics)
}

/// Get plugin search paths in order of priority
fn get_plugin_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // User plugins (highest priority)
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("native-launcher/plugins"));
    }

    // System plugins
    paths.push(PathBuf::from("/usr/local/share/native-launcher/plugins"));
    paths.push(PathBuf::from("/usr/share/native-launcher/plugins"));

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_search_paths() {
        let paths = get_plugin_search_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].to_string_lossy().contains("config"));
    }

    #[test]
    fn test_abi_version() {
        assert_eq!(PLUGIN_ABI_VERSION, 1);
    }
}
