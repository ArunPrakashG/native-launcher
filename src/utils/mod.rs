pub mod browser;
pub mod exec;
pub mod icons;

pub use browser::{detect_web_search, get_default_browser};
pub use exec::execute_command;
