pub mod browser;
pub mod exec;
pub mod icons;

#[allow(unused_imports)]
pub use browser::get_default_browser;
pub use exec::{build_open_command, execute_command};
