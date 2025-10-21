//! Example Plugin Template for Native Launcher
//!
//! This is a minimal example showing how to create an external plugin
//! that can be loaded dynamically by Native Launcher.
//!
//! ## Building
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! ## Installing
//!
//! ```bash
//! mkdir -p ~/.config/native-launcher/plugins
//! cp target/release/libexample_plugin.so ~/.config/native-launcher/plugins/
//! ```
//!
//! ## Testing
//!
//! Restart Native Launcher. The plugin will be loaded automatically.
//! Type "example:" to trigger this plugin.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;

/// Plugin ABI version - must match launcher's version
const PLUGIN_ABI_VERSION: u32 = 1;

/// C-compatible string slice
#[repr(C)]
pub struct CStringSlice {
    pub ptr: *const c_char,
    pub len: usize,
}

impl CStringSlice {
    /// Create from Rust string (creates new CString, must be freed)
    fn from_string(s: &str) -> Self {
        let cstr = CString::new(s).unwrap();
        let len = cstr.as_bytes().len();
        let ptr = cstr.into_raw();
        Self { ptr, len }
    }

    /// Create empty string
    fn empty() -> Self {
        Self {
            ptr: std::ptr::null(),
            len: 0,
        }
    }

    /// Convert to Rust string (unsafe - doesn't take ownership)
    unsafe fn to_string(&self) -> String {
        if self.ptr.is_null() {
            return String::new();
        }
        let slice = slice::from_raw_parts(self.ptr as *const u8, self.len);
        String::from_utf8_lossy(slice).to_string()
    }
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

/// C-compatible result array
#[repr(C)]
pub struct CResultArray {
    pub ptr: *mut CPluginResult,
    pub len: usize,
    pub capacity: usize,
}

impl CResultArray {
    fn from_vec(results: Vec<CPluginResult>) -> Self {
        let mut results = results.into_boxed_slice();
        let len = results.len();
        let capacity = len;
        let ptr = results.as_mut_ptr();
        std::mem::forget(results); // Don't drop, caller will free
        Self { ptr, len, capacity }
    }
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

//
// Plugin Implementation - Customize these functions
//

/// Get plugin ABI version
#[no_mangle]
pub extern "C" fn plugin_get_abi_version() -> u32 {
    PLUGIN_ABI_VERSION
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn plugin_get_name() -> CStringSlice {
    CStringSlice::from_string("Example Plugin")
}

/// Get plugin description
#[no_mangle]
pub extern "C" fn plugin_get_description() -> CStringSlice {
    CStringSlice::from_string("An example plugin demonstrating the plugin API")
}

/// Get plugin priority (higher = searched first)
/// Applications: 1000, Calculator: 500, default: 100
#[no_mangle]
pub extern "C" fn plugin_get_priority() -> c_int {
    200 // Medium priority
}

/// Check if plugin should handle this query
/// Return true if your plugin can provide results
#[no_mangle]
pub extern "C" fn plugin_should_handle(query: CStringSlice) -> bool {
    unsafe {
        let query_str = query.to_string();
        // This plugin handles queries starting with "example:"
        query_str.starts_with("example:")
    }
}

/// Search for results matching the query
#[no_mangle]
pub extern "C" fn plugin_search(query: CStringSlice, context: CPluginContext) -> CResultArray {
    unsafe {
        let query_str = query.to_string();

        // Remove "example:" prefix if present
        let search_term = query_str
            .strip_prefix("example:")
            .unwrap_or(&query_str)
            .trim();

        let mut results = Vec::new();

        // Example: Create some sample results
        if search_term.is_empty() {
            // Show help when no search term
            results.push(CPluginResult {
                title: CStringSlice::from_string("Example Plugin"),
                subtitle: CStringSlice::from_string("Type 'example: <query>' to search"),
                icon: CStringSlice::from_string("dialog-information"),
                command: CStringSlice::from_string("echo 'Example plugin'"),
                terminal: true,
                score: 1000,
            });
        } else {
            // Create results based on search term
            for i in 1..=context.max_results.min(3) {
                results.push(CPluginResult {
                    title: CStringSlice::from_string(&format!(
                        "Result {} for '{}'",
                        i, search_term
                    )),
                    subtitle: CStringSlice::from_string("Click to execute example command"),
                    icon: CStringSlice::from_string("emblem-default"),
                    command: CStringSlice::from_string(&format!(
                        "echo 'Executed: {} - result {}'",
                        search_term, i
                    )),
                    terminal: true,
                    score: 1000 - (i as i64 * 10),
                });
            }
        }

        CResultArray::from_vec(results)
    }
}

/// Handle keyboard events (optional)
/// Return CKeyboardAction::None if you don't handle this event
#[no_mangle]
pub extern "C" fn plugin_handle_keyboard_event(event: CKeyboardEvent) -> CKeyboardActionData {
    unsafe {
        let query = event.query.to_string();

        // Example: Handle Ctrl+E to execute special command
        if event.modifiers & 0x04 != 0 && event.key_val == 'e' as u32 {
            if query.starts_with("example:") {
                return CKeyboardActionData {
                    action: CKeyboardAction::Execute,
                    data: CStringSlice::from_string("echo 'Ctrl+E pressed!'"),
                    terminal: true,
                };
            }
        }

        // Don't handle this event
        CKeyboardActionData {
            action: CKeyboardAction::None,
            data: CStringSlice::empty(),
            terminal: false,
        }
    }
}

/// Free result array (must match allocation method)
#[no_mangle]
pub extern "C" fn plugin_free_results(results: CResultArray) {
    if !results.ptr.is_null() {
        unsafe {
            // Reconstruct the Vec to drop it properly
            let results_vec = Vec::from_raw_parts(results.ptr, results.len, results.capacity);

            // Free all strings in results
            for result in results_vec {
                plugin_free_string(result.title);
                plugin_free_string(result.subtitle);
                plugin_free_string(result.icon);
                plugin_free_string(result.command);
            }
        }
    }
}

/// Free string data (must match allocation method)
#[no_mangle]
pub extern "C" fn plugin_free_string(data: CStringSlice) {
    if !data.ptr.is_null() {
        unsafe {
            // Reconstruct CString and drop it
            let _ = CString::from_raw(data.ptr as *mut c_char);
        }
    }
}
