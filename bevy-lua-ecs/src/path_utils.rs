/// Centralized path normalization utilities for cross-platform consistency.
///
/// This module provides functions to normalize paths to use forward slashes,
/// which is critical for:
/// - HashMap/HashSet lookups (paths used as keys)
/// - Network transmission (client/server may be on different platforms)
/// - Script module resolution (require() paths must match across platforms)
use std::path::Path;

/// Normalize path separators to forward slashes.
/// This is the primary function for converting paths to a canonical string format
/// for storage, comparison, and network transmission.
///
/// # Examples
/// ```
/// let path = "scripts\\examples\\test.lua";
/// assert_eq!(normalize_path_separators(path), "scripts/examples/test.lua");
/// ```
pub fn normalize_path_separators(path: &str) -> String {
    path.replace('\\', "/")
}

/// Normalize a path by resolving .. and . components AND converting to forward slashes.
/// This provides full path normalization for script module resolution.
///
/// # Examples
/// ```
/// let path = "scripts/examples/../test.lua";
/// assert_eq!(normalize_path(path), "scripts/test.lua");
/// ```
pub fn normalize_path(path: &str) -> String {
    let path = normalize_path_separators(path);
    let mut parts: Vec<&str> = Vec::new();

    for part in path.split('/') {
        match part {
            "" | "." => {} // Skip empty and current directory
            ".." => {
                parts.pop(); // Go up one directory
            }
            _ => parts.push(part),
        }
    }

    parts.join("/")
}

/// Convert a Path/PathBuf to a forward-slash string.
/// This is useful when you have a Path type and need a normalized string.
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// let path = PathBuf::from("scripts\\test.lua");
/// assert_eq!(to_forward_slash(&path), "scripts/test.lua");
/// ```
pub fn to_forward_slash<P: AsRef<Path>>(path: P) -> String {
    normalize_path_separators(&path.as_ref().to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_separators() {
        assert_eq!(
            normalize_path_separators("scripts\\test.lua"),
            "scripts/test.lua"
        );
        assert_eq!(
            normalize_path_separators("scripts/test.lua"),
            "scripts/test.lua"
        );
        assert_eq!(
            normalize_path_separators("scripts\\examples\\test.lua"),
            "scripts/examples/test.lua"
        );
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("scripts/examples/../test.lua"),
            "scripts/test.lua"
        );
        assert_eq!(normalize_path("scripts/./test.lua"), "scripts/test.lua");
        assert_eq!(
            normalize_path("scripts\\examples\\..\\test.lua"),
            "scripts/test.lua"
        );
        assert_eq!(
            normalize_path("scripts/examples/../../test.lua"),
            "test.lua"
        );
    }

    #[test]
    fn test_to_forward_slash() {
        use std::path::PathBuf;
        let path = PathBuf::from("scripts").join("test.lua");
        let normalized = to_forward_slash(&path);
        assert!(normalized.contains('/') || !normalized.contains('\\'));
    }
}
