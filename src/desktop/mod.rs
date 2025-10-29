pub mod cache;
pub mod entry;
pub mod scanner;
pub mod store;
pub mod watcher;

pub use entry::{DesktopAction, DesktopEntry};
pub use scanner::DesktopScanner;
pub use store::{DesktopEntryArena, SharedDesktopEntry};
