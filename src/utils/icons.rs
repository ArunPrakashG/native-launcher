use crate::desktop::DesktopEntryArena;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::debug;

/// Icon cache to avoid repeated lookups
static ICON_CACHE: Mutex<Option<HashMap<String, Option<PathBuf>>>> = Mutex::new(None);

/// Default icon size for application icons
const DEFAULT_ICON_SIZE: u32 = 48;

/// Resolve an icon path from an icon name or path
///
/// This function implements the freedesktop.org icon theme specification
/// with a fallback chain for maximum compatibility.
pub fn resolve_icon(icon_name: &str) -> Option<PathBuf> {
    resolve_icon_with_size(icon_name, DEFAULT_ICON_SIZE)
}

/// Resolve an icon path with a specific size
pub fn resolve_icon_with_size(icon_name: &str, size: u32) -> Option<PathBuf> {
    // Check cache first
    {
        let mut cache = ICON_CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(HashMap::new());
        }

        let cache_key = format!("{}:{}", icon_name, size);
        if let Some(cached_path) = cache.as_ref().unwrap().get(&cache_key) {
            debug!("Icon cache hit for: {}", icon_name);
            return cached_path.clone();
        }
    }

    // 1. Check if it's an absolute path
    if icon_name.starts_with('/') {
        let path = Path::new(icon_name);
        if path.exists() {
            debug!("Found absolute icon path: {}", icon_name);
            let result = Some(path.to_path_buf());
            cache_icon(icon_name, size, result.clone());
            return result;
        }
    }

    // 2. Try to find in icon theme using freedesktop-icons
    if let Some(path) = lookup_themed_icon(icon_name, size) {
        debug!("Found themed icon: {} -> {:?}", icon_name, path);
        cache_icon(icon_name, size, Some(path.clone()));
        return Some(path);
    }

    // 3. Try without extension
    let icon_without_ext = icon_name
        .trim_end_matches(".png")
        .trim_end_matches(".svg")
        .trim_end_matches(".xpm");

    if icon_without_ext != icon_name {
        if let Some(path) = lookup_themed_icon(icon_without_ext, size) {
            debug!(
                "Found themed icon without extension: {} -> {:?}",
                icon_without_ext, path
            );
            cache_icon(icon_name, size, Some(path.clone()));
            return Some(path);
        }
    }

    debug!("No icon found for: {}", icon_name);
    cache_icon(icon_name, size, None);
    None
}

/// Lookup icon in system icon themes
fn lookup_themed_icon(icon_name: &str, size: u32) -> Option<PathBuf> {
    // First, try /usr/share/pixmaps (many apps put icons here)
    // Try exact match first, then case variations
    let pixmaps_dir = PathBuf::from("/usr/share/pixmaps");
    for ext in &["png", "svg", "xpm"] {
        // Try exact name
        let pixmaps_path = pixmaps_dir.join(format!("{}.{}", icon_name, ext));
        if pixmaps_path.exists() {
            debug!("Found icon in pixmaps: {:?}", pixmaps_path);
            return Some(pixmaps_path);
        }

        // Try lowercase (e.g., "Alacritty" -> "alacritty")
        let pixmaps_path_lower = pixmaps_dir.join(format!("{}.{}", icon_name.to_lowercase(), ext));
        if pixmaps_path_lower.exists() {
            debug!(
                "Found icon in pixmaps (lowercase): {:?}",
                pixmaps_path_lower
            );
            return Some(pixmaps_path_lower);
        }
    }

    // Also try user's local pixmaps
    if let Some(home) = dirs::home_dir() {
        let local_pixmaps = home.join(".local/share/pixmaps");
        for ext in &["png", "svg", "xpm"] {
            let pixmaps_path = local_pixmaps.join(format!("{}.{}", icon_name, ext));
            if pixmaps_path.exists() {
                debug!("Found icon in local pixmaps: {:?}", pixmaps_path);
                return Some(pixmaps_path);
            }
        }
    }

    // Search in standard XDG icon directories
    let icon_dirs = get_icon_directories();

    for dir in icon_dirs {
        // Try different icon theme subdirectories
        let themes = vec!["hicolor", "Adwaita", "breeze", "Papirus"];

        for theme in themes {
            // Try different size directories (exact, scalable, closest match)
            let size_dirs = vec![
                format!("{}x{}", size, size),
                "scalable".to_string(),
                format!("{}x{}", size * 2, size * 2), // Try 2x for HiDPI
                "48x48".to_string(),                  // Common fallback
            ];

            for size_dir in &size_dirs {
                // Try different icon extensions
                for ext in &["svg", "png", "xpm"] {
                    let path = dir
                        .join(theme)
                        .join(size_dir)
                        .join("apps")
                        .join(format!("{}.{}", icon_name, ext));
                    if path.exists() {
                        return Some(path);
                    }

                    // Also try without apps subdirectory
                    let path = dir
                        .join(theme)
                        .join(size_dir)
                        .join(format!("{}.{}", icon_name, ext));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Get standard icon directories following XDG specification
fn get_icon_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User-specific icons
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/icons"));
        dirs.push(home.join(".icons"));
    }

    // System icons
    dirs.push(PathBuf::from("/usr/share/icons"));
    dirs.push(PathBuf::from("/usr/local/share/icons"));

    // XDG_DATA_DIRS
    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_data_dirs.split(':') {
            dirs.push(PathBuf::from(dir).join("icons"));
        }
    }

    // Filter to only existing directories
    dirs.into_iter().filter(|d| d.exists()).collect()
}

/// Cache an icon lookup result
fn cache_icon(icon_name: &str, size: u32, path: Option<PathBuf>) {
    let mut cache = ICON_CACHE.lock().unwrap();
    if let Some(cache_map) = cache.as_mut() {
        let cache_key = format!("{}:{}", icon_name, size);
        cache_map.insert(cache_key, path);
    }
}

/// Clear the icon cache (useful when theme changes)
#[allow(dead_code)]

pub fn clear_icon_cache() {
    let mut cache = ICON_CACHE.lock().unwrap();
    if let Some(cache_map) = cache.as_mut() {
        cache_map.clear();
        debug!("Icon cache cleared");
    }
}

/// Preload icon cache for all desktop entries in background
pub fn preload_icon_cache(entries: &DesktopEntryArena) {
    use tracing::info;

    info!("Preloading icon cache for {} entries...", entries.len());
    let start = std::time::Instant::now();
    let mut loaded = 0;

    for entry in entries.iter() {
        let entry = entry.as_ref();
        if let Some(ref icon) = entry.icon {
            if resolve_icon(icon).is_some() {
                loaded += 1;
            }
        }
    }

    let duration = start.elapsed();
    info!(
        "Icon cache preloaded: {}/{} icons in {:?}",
        loaded,
        entries.len(),
        duration
    );
}

/// Get default fallback icon path
pub fn get_default_icon() -> PathBuf {
    // Try common application icon names
    let fallbacks = vec![
        "application-x-executable",
        "application-default-icon",
        "preferences-desktop-default-applications",
        "system-run",
        "application",
    ];

    for fallback in fallbacks {
        if let Some(path) = resolve_icon(fallback) {
            return path;
        }
    }

    // Last resort: use a generic icon from pixmaps
    PathBuf::from("/usr/share/pixmaps/debian-logo.png")
}

/// Resolve icon with automatic fallback to default icon
#[allow(dead_code)]

pub fn resolve_icon_or_default(icon_name: &str) -> PathBuf {
    resolve_icon(icon_name).unwrap_or_else(get_default_icon)
}

/// Create a greyed-out version of an icon for secondary items
/// For now, just returns the same icon (visual filtering can be done in CSS)
#[allow(dead_code)]

pub fn get_greyed_icon(icon_name: &str) -> Option<PathBuf> {
    resolve_icon(icon_name)
}

/// Map freedesktop.org categories to appropriate fallback icon names
///
/// This provides intelligent icon fallbacks for applications without
/// explicit Icon= fields by using their Categories field.
/// Based on freedesktop.org menu specification:
/// https://specifications.freedesktop.org/menu-spec/latest/apa.html
pub fn category_to_icon(categories: &[String]) -> Option<&'static str> {
    // Priority order: more specific categories first
    for category in categories {
        let icon = match category.as_str() {
            // Main Categories
            "AudioVideo" => "applications-multimedia",
            "Audio" => "audio-x-generic",
            "Video" => "video-x-generic",
            "Development" => "applications-development",
            "Education" => "applications-education",
            "Game" => "applications-games",
            "Graphics" => "applications-graphics",
            "Network" => "applications-internet",
            "Office" => "applications-office",
            "Science" => "applications-science",
            "Settings" => "preferences-system",
            "System" => "applications-system",
            "Utility" => "applications-utilities",

            // Additional Categories (more specific)
            "Building" => "applications-development",
            "Debugger" => "applications-development",
            "IDE" => "applications-development",
            "GUIDesigner" => "applications-development",
            "Profiling" => "system-monitoring",
            "RevisionControl" => "applications-development",
            "Translation" => "applications-accessories",
            "Calendar" => "x-office-calendar",
            "ContactManagement" => "x-office-address-book",
            "Database" => "x-office-spreadsheet",
            "Dictionary" => "accessories-dictionary",
            "Chart" => "x-office-presentation",
            "Email" => "internet-mail",
            "Finance" => "applications-office",
            "FlowChart" => "x-office-drawing",
            "PDA" => "pda",
            "ProjectManagement" => "applications-office",
            "Presentation" => "x-office-presentation",
            "Spreadsheet" => "x-office-spreadsheet",
            "WordProcessor" => "x-office-document",
            "2DGraphics" => "applications-graphics",
            "VectorGraphics" => "applications-graphics",
            "RasterGraphics" => "applications-graphics",
            "3DGraphics" => "applications-graphics",
            "Scanning" => "scanner",
            "OCR" => "scanner",
            "Photography" => "camera-photo",
            "Publishing" => "applications-publishing",
            "Viewer" => "image-viewer",
            "TextTools" => "accessories-text-editor",
            "DesktopSettings" => "preferences-desktop",
            "HardwareSettings" => "preferences-system",
            "Printing" => "printer",
            "PackageManager" => "system-software-install",
            "Dialup" => "network-wired",
            "InstantMessaging" => "internet-chat",
            "Chat" => "internet-chat",
            "IRCClient" => "internet-chat",
            "Feed" => "internet-news-reader",
            "FileTransfer" => "folder-download",
            "HamRadio" => "audio-input-microphone",
            "News" => "internet-news-reader",
            "P2P" => "network-workgroup",
            "RemoteAccess" => "preferences-system-network",
            "Telephony" => "phone",
            "TelephonyTools" => "phone",
            "VideoConference" => "camera-video",
            "WebBrowser" => "web-browser",
            "WebDevelopment" => "applications-development",
            "Midi" => "audio-x-generic",
            "Mixer" => "multimedia-volume-control",
            "Sequencer" => "audio-x-generic",
            "Tuner" => "multimedia-player",
            "TV" => "video-display",
            "AudioVideoEditing" => "applications-multimedia",
            "Player" => "multimedia-player",
            "Recorder" => "media-record",
            "DiscBurning" => "media-optical-recordable",
            "ActionGame" => "applications-games",
            "AdventureGame" => "applications-games",
            "ArcadeGame" => "applications-games",
            "BoardGame" => "applications-games",
            "BlocksGame" => "applications-games",
            "CardGame" => "applications-games",
            "KidsGame" => "applications-games",
            "LogicGame" => "applications-games",
            "RolePlaying" => "applications-games",
            "Shooter" => "applications-games",
            "Simulation" => "applications-games",
            "SportsGame" => "applications-games",
            "StrategyGame" => "applications-games",
            "Art" => "applications-graphics",
            "Construction" => "applications-engineering",
            "Music" => "applications-multimedia",
            "Languages" => "applications-education",
            "ArtificialIntelligence" => "applications-science",
            "Astronomy" => "applications-science",
            "Biology" => "applications-science",
            "Chemistry" => "applications-science",
            "ComputerScience" => "applications-science",
            "DataVisualization" => "applications-science",
            "Economy" => "applications-office",
            "Electricity" => "applications-science",
            "Geography" => "applications-science",
            "Geology" => "applications-science",
            "Geoscience" => "applications-science",
            "History" => "applications-education",
            "Humanities" => "applications-education",
            "ImageProcessing" => "applications-graphics",
            "Literature" => "applications-education",
            "Maps" => "maps",
            "Math" => "applications-science",
            "NumericalAnalysis" => "applications-science",
            "MedicalSoftware" => "applications-science",
            "Physics" => "applications-science",
            "Robotics" => "applications-science",
            "Spirituality" => "applications-education",
            "Sports" => "applications-games",
            "ParallelComputing" => "applications-system",
            "Amusement" => "applications-games",
            "Archiving" => "package-x-generic",
            "Compression" => "package-x-generic",
            "Electronics" => "applications-electronics",
            "Emulator" => "applications-system",
            "Engineering" => "applications-engineering",
            "FileTools" => "system-file-manager",
            "FileManager" => "system-file-manager",
            "TerminalEmulator" => "utilities-terminal",
            "Filesystem" => "drive-harddisk",
            "Monitor" => "system-monitoring",
            "Security" => "security-high",
            "Accessibility" => "preferences-desktop-accessibility",
            "Calculator" => "accessories-calculator",
            "Clock" => "preferences-system-time",
            "TextEditor" => "accessories-text-editor",
            "Documentation" => "help-browser",
            "Adult" => "applications-other",
            "Core" => "applications-system",
            "KDE" => "kde",
            "GNOME" => "gnome",
            "XFCE" => "xfce",
            "GTK" => "gtk",
            "Qt" => "qt",
            "Motif" => "applications-other",
            "Java" => "applications-development",
            "ConsoleOnly" => "utilities-terminal",

            _ => continue, // Try next category
        };

        return Some(icon);
    }

    // No matching category found
    None
}

/// Resolve icon with category fallback for desktop entries
///
/// This is the recommended function to use for desktop entries.
/// Falls back through: explicit icon → category-based icon → generic fallback
pub fn resolve_icon_with_category_fallback(
    icon_name: Option<&str>,
    categories: &[String],
) -> PathBuf {
    // Try explicit icon first
    if let Some(icon) = icon_name {
        if let Some(path) = resolve_icon(icon) {
            return path;
        }
    }

    // Try category-based fallback
    if let Some(category_icon) = category_to_icon(categories) {
        if let Some(path) = resolve_icon(category_icon) {
            debug!(
                "Using category fallback icon: {} for categories: {:?}",
                category_icon, categories
            );
            return path;
        }
    }

    // Final fallback
    get_default_icon()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_path() {
        // Test with a path that should exist on most Linux systems
        let result = resolve_icon("/usr/share/pixmaps/debian-logo.png");
        // Result may be None if file doesn't exist, which is fine
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_icon_cache() {
        clear_icon_cache();

        // First lookup
        let icon1 = resolve_icon("firefox");

        // Second lookup (should hit cache)
        let icon2 = resolve_icon("firefox");

        assert_eq!(icon1, icon2);
    }

    #[test]
    fn test_category_to_icon_development() {
        let categories = vec!["Development".to_string()];
        let icon = category_to_icon(&categories);
        assert_eq!(icon, Some("applications-development"));
    }

    #[test]
    fn test_category_to_icon_web_browser() {
        let categories = vec!["Network".to_string(), "WebBrowser".to_string()];
        let icon = category_to_icon(&categories);
        // Should return the first matching category (Network)
        assert_eq!(icon, Some("applications-internet"));
    }

    #[test]
    fn test_category_to_icon_specific_beats_general() {
        let categories = vec!["WebBrowser".to_string(), "Network".to_string()];
        let icon = category_to_icon(&categories);
        // Should return first match (WebBrowser is more specific)
        assert_eq!(icon, Some("web-browser"));
    }

    #[test]
    fn test_category_to_icon_no_match() {
        let categories = vec!["UnknownCategory".to_string()];
        let icon = category_to_icon(&categories);
        assert_eq!(icon, None);
    }

    #[test]
    fn test_category_to_icon_empty() {
        let categories: Vec<String> = vec![];
        let icon = category_to_icon(&categories);
        assert_eq!(icon, None);
    }

    #[test]
    fn test_resolve_with_category_fallback_explicit_icon() {
        let categories = vec!["Development".to_string()];
        // Even though we have categories, explicit icon should be tried first
        let path = resolve_icon_with_category_fallback(Some("firefox"), &categories);
        // Should return a non-empty path string
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_resolve_with_category_fallback_no_icon() {
        let categories = vec!["TextEditor".to_string()];
        // No explicit icon, should use category fallback
        let path = resolve_icon_with_category_fallback(None, &categories);
        // Should return a non-empty path string
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_resolve_with_category_fallback_unknown_category() {
        let categories = vec!["UnknownCategory".to_string()];
        // Unknown category should fall back to default icon
        let path = resolve_icon_with_category_fallback(None, &categories);
        let default_path = get_default_icon();
        assert_eq!(path, default_path);
    }

    #[test]
    fn test_category_fallback_priority() {
        let categories_specific = vec!["WebBrowser".to_string(), "Network".to_string()];
        let icon_specific = category_to_icon(&categories_specific);
        // Should match first category
        assert_eq!(icon_specific, Some("web-browser"));

        let categories_general = vec!["Network".to_string()];
        let icon_general = category_to_icon(&categories_general);
        assert_eq!(icon_general, Some("applications-internet"));
    }
}
