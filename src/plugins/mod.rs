pub mod advanced_calc;
pub mod applications;
pub mod calculator;
pub mod dynamic;
pub mod editors;
pub mod file_index;
pub mod files;
pub mod launcher;
pub mod manager;
#[allow(dead_code)] // Complete but not yet integrated - see docs/SCRIPT_PLUGIN_SYSTEM.md
pub mod script_plugin;
pub mod shell;
pub mod ssh;
pub mod theme_switcher;
pub mod traits;
pub mod web_search;

pub use advanced_calc::AdvancedCalculatorPlugin;
pub use applications::ApplicationsPlugin;
pub use calculator::CalculatorPlugin;
pub use dynamic::{load_plugins, PluginMetrics};
pub use editors::EditorsPlugin;
pub use files::FileBrowserPlugin;
pub use launcher::LauncherPlugin;
pub use manager::PluginManager;
// Script plugin system is complete but not integrated yet - uncomment when ready to use
// pub use script_plugin::{ScriptPlugin, ScriptPluginManager};
pub use shell::ShellPlugin;
pub use ssh::SshPlugin;
pub use theme_switcher::ThemeSwitcherPlugin;
pub use traits::{KeyboardAction, KeyboardEvent, PluginResult};
pub use web_search::WebSearchPlugin;
