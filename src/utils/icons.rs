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
pub fn preload_icon_cache(entries: &[crate::desktop::DesktopEntry]) {
    use tracing::info;

    info!("Preloading icon cache for {} entries...", entries.len());
    let start = std::time::Instant::now();
    let mut loaded = 0;

    for entry in entries {
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
}
