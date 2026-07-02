use crate::config::pattern::{GlobPattern, GlobSet};
use crate::model::{EntryType, RawEntry, Warning};
use std::fs;
use std::path::Path;

/// Result of scanning a directory tree.
#[derive(Debug)]
pub struct ScanResult {
    pub entries: Vec<RawEntry>,
    pub warnings: Vec<Warning>,
}

/// The filesystem scanner.
///
/// Performs depth-limited directory traversal and collects raw entries.
/// Does NOT perform classification, priority assignment, or sorting.
pub struct Scanner {
    /// Built-in ignore directory patterns (e.g., .git, target, node_modules)
    builtin_ignore: GlobSet,
    /// User-configured ignore patterns
    user_ignore: GlobSet,
    /// Whether to include ignored directories' children
    include_ignored: bool,
}

fn builtin_ignore_patterns() -> GlobSet {
    let mut set = GlobSet::new();
    for pattern in &[
        ".git",
        "target",
        "node_modules",
        ".cache",
        ".next",
        ".DS_Store",
        "__pycache__",
        ".venv",
        "venv",
        ".tox",
    ] {
        if let Some(p) = GlobPattern::compile(pattern) {
            set.add(p);
        }
    }
    set
}

impl Scanner {
    pub fn new(user_ignore_patterns: &[String], include_ignored: bool) -> Self {
        let mut user_ignore = GlobSet::new();
        for pat in user_ignore_patterns {
            if let Some(p) = GlobPattern::compile(pat) {
                user_ignore.add(p);
            }
        }

        Self {
            builtin_ignore: builtin_ignore_patterns(),
            user_ignore,
            include_ignored,
        }
    }

    /// Scan the given path up to the specified depth.
    ///
    /// `depth` of 0 means just the target itself.
    /// `depth` of 1 means direct children.
    pub fn scan(&self, target: &Path, depth: u8) -> ScanResult {
        let mut entries = Vec::new();
        let mut warnings = Vec::new();

        let target_metadata = match fs::symlink_metadata(target) {
            Ok(metadata) => metadata,
            Err(e) => {
                warnings.push(Warning {
                    code: "metadata_error".into(),
                    path: Some(target.to_string_lossy().into()),
                    message: format!("cannot read metadata: {e}"),
                });
                return ScanResult { entries, warnings };
            }
        };

        if let Some(raw) =
            self.build_raw_entry(target, target, &target_metadata, true, &mut warnings)
        {
            let target_is_dir = raw.entry_type == EntryType::Directory;
            entries.push(raw);

            if target_is_dir && depth > 0 {
                self.scan_internal(target, target, depth, 1, &mut entries, &mut warnings);
            }
        }

        ScanResult { entries, warnings }
    }

    fn scan_internal(
        &self,
        root: &Path,
        current: &Path,
        max_depth: u8,
        current_depth: u8,
        entries: &mut Vec<RawEntry>,
        warnings: &mut Vec<Warning>,
    ) {
        if current_depth > max_depth {
            return;
        }

        // Collect directory entries
        let dir_entries = match fs::read_dir(current) {
            Ok(reader) => reader,
            Err(e) => {
                // Permission error for child — add warning
                warnings.push(Warning {
                    code: "permission_denied".into(),
                    path: Some(current.to_string_lossy().into()),
                    message: format!("cannot read directory: {e}"),
                });
                return;
            }
        };

        for entry in dir_entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warnings.push(Warning {
                        code: "entry_read_error".into(),
                        path: None,
                        message: format!("failed to read entry: {e}"),
                    });
                    continue;
                }
            };

            let absolute_path = entry.path();

            let metadata = match fs::symlink_metadata(&absolute_path) {
                Ok(m) => m,
                Err(e) => {
                    let relative_path = absolute_path
                        .strip_prefix(root)
                        .unwrap_or(&absolute_path)
                        .to_path_buf();
                    warnings.push(Warning {
                        code: "metadata_error".into(),
                        path: Some(relative_path.to_string_lossy().into()),
                        message: format!("cannot read metadata: {e}"),
                    });
                    continue;
                }
            };

            let Some(raw) = self.build_raw_entry(root, &absolute_path, &metadata, false, warnings)
            else {
                continue;
            };

            let rel_path_str = raw.relative_path.to_string_lossy().replace('\\', "/");
            let entry_type = raw.entry_type;
            entries.push(raw);

            // Recurse into directories (unless ignored and pruning)
            if entry_type == EntryType::Directory && current_depth < max_depth {
                let is_ignored = self.is_ignore(&rel_path_str);
                if is_ignored && !self.include_ignored {
                    // Prune: don't descend into ignored directories
                    continue;
                }
                self.scan_internal(
                    root,
                    &absolute_path,
                    max_depth,
                    current_depth + 1,
                    entries,
                    warnings,
                );
            }
        }
    }

    fn build_raw_entry(
        &self,
        root: &Path,
        absolute_path: &Path,
        metadata: &fs::Metadata,
        is_root: bool,
        warnings: &mut Vec<Warning>,
    ) -> Option<RawEntry> {
        let relative_path = if is_root {
            Path::new(".").to_path_buf()
        } else {
            absolute_path
                .strip_prefix(root)
                .unwrap_or(absolute_path)
                .to_path_buf()
        };

        let name = if is_root {
            absolute_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| ".".to_string())
        } else {
            match absolute_path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_string(),
                None => {
                    warnings.push(Warning {
                        code: "non_utf8_path_skipped".into(),
                        path: None,
                        message: "skipped non-UTF-8 path".into(),
                    });
                    return None;
                }
            }
        };

        let entry_type = if metadata.is_symlink() {
            EntryType::Symlink
        } else if metadata.is_dir() {
            EntryType::Directory
        } else if metadata.is_file() {
            EntryType::File
        } else {
            EntryType::Other
        };

        let size_bytes = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };

        let modified_at = metadata.modified().ok();

        Some(RawEntry {
            name,
            absolute_path: absolute_path.to_path_buf(),
            relative_path,
            entry_type,
            size_bytes,
            modified_at,
        })
    }

    /// Check if a path matches any ignore pattern (built-in or user).
    fn is_ignore(&self, path: &str) -> bool {
        // Check built-in patterns first
        if self.builtin_ignore.is_match(path) || self.builtin_ignore.is_match_dir_only(path) {
            return true;
        }
        // Check user patterns
        if self.user_ignore.is_match(path) || self.user_ignore.is_match_dir_only(path) {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_depth_0() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();

        let scanner = Scanner::new(&[], false);
        let result = scanner.scan(dir.path(), 0);
        // Depth 0: only the target itself.
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].relative_path.to_string_lossy(), ".");
    }

    #[test]
    fn test_scan_depth_1() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();

        let scanner = Scanner::new(&[], false);
        let result = scanner.scan(dir.path(), 1);
        let paths: Vec<_> = result
            .entries
            .iter()
            .map(|e| e.relative_path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&".".to_string()));
        assert!(paths.contains(&"a.txt".to_string()));
        assert!(paths.contains(&"sub".to_string()));
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_scan_prune_ignored() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target/out.txt"), "").unwrap();
        fs::write(dir.path().join("src.rs"), "").unwrap();

        let scanner = Scanner::new(&[], false);
        let result = scanner.scan(dir.path(), 2);
        // target/ is in result but target/out.txt should be pruned
        let paths: Vec<_> = result
            .entries
            .iter()
            .map(|e| e.relative_path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"target".to_string()));
        assert!(!paths.contains(&"target/out.txt".to_string()));
        assert!(paths.contains(&"src.rs".to_string()));
    }

    #[test]
    fn test_scan_include_ignored() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("node_modules")).unwrap();
        fs::write(dir.path().join("node_modules/pkg.js"), "").unwrap();

        let scanner = Scanner::new(&[], true);
        let result = scanner.scan(dir.path(), 2);
        let paths: Vec<_> = result
            .entries
            .iter()
            .map(|e| e.relative_path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"node_modules".to_string()));
        assert!(paths.contains(&"node_modules/pkg.js".to_string()));
    }

    #[test]
    fn test_symlink_metadata() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("real.txt"), "hello").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("real.txt", dir.path().join("link.txt")).unwrap();

        let scanner = Scanner::new(&[], false);
        let result = scanner.scan(dir.path(), 1);
        #[cfg(unix)]
        assert_eq!(
            result
                .entries
                .iter()
                .find(|e| e.name == "link.txt")
                .unwrap()
                .entry_type,
            EntryType::Symlink
        );
    }
}
