use std::fs;
use std::path::Path;

use crate::error::LlsError;

/// Result from scanning: entries and warnings [(path, message)]
pub type ScanResult = Result<(Vec<EntryMeta>, Vec<(String, String)>), LlsError>;

#[derive(Debug, Clone)]
pub struct EntryMeta {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub size_bytes: Option<u64>,
    pub is_text: bool,
    pub is_binary: bool,
    pub is_broken_symlink: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::File => "file",
            EntryType::Directory => "directory",
            EntryType::Symlink => "symlink",
            EntryType::Other => "other",
        }
    }
}

/// Scan a target path. Returns entries and warnings.
pub fn scan(
    target: &Path,
    max_depth: usize,
) -> ScanResult {
    if !target.exists() {
        return Err(LlsError::NotFound(target.display().to_string()));
    }

    let metadata = target.metadata().map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            LlsError::PermissionDenied(target.display().to_string())
        } else {
            LlsError::Io(e)
        }
    })?;

    if metadata.is_file() {
        let meta = entry_from_path(target, target)?;
        return Ok((vec![meta], vec![]));
    }

    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    // Walk directory from depth 0
    walk_dir(target, target, 0, max_depth, &mut entries, &mut warnings);

    Ok((entries, warnings))
}

fn walk_dir(
    base: &Path,
    dir: &Path,
    current_depth: usize,
    max_depth: usize,
    entries: &mut Vec<EntryMeta>,
    warnings: &mut Vec<(String, String)>,
) {
    if current_depth >= max_depth {
        return;
    }

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            warnings.push((
                dir.display().to_string(),
                format!("エントリのメタデータ取得時に権限エラーが発生した: {e}"),
            ));
            return;
        }
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push((
                    format!("{}", dir.display()),
                    format!("エントリの読み取りに失敗: {e}"),
                ));
                continue;
            }
        };

        let path = entry.path();
        match entry_from_path(base, &path) {
            Ok(meta) => {
                if meta.is_broken_symlink {
                    warnings.push((
                        meta.path.clone(),
                        format!("壊れたシンボリックリンク: {}", meta.name),
                    ));
                }
                if meta.entry_type == EntryType::Directory {
                    entries.push(meta);
                    walk_dir(base, &path, current_depth + 1, max_depth, entries, warnings);
                } else {
                    entries.push(meta);
                }
            }
            Err(e) => {
                warnings.push((
                    path.display().to_string(),
                    format!("メタデータ取得エラー: {e}"),
                ));
            }
        }
    }
}

fn entry_from_path(base: &Path, path: &Path) -> Result<EntryMeta, LlsError> {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let rel = if base == path {
        name.clone()
    } else {
        path
            .strip_prefix(base)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| name.clone())
    };

    // Check for broken symlink: use symlink_metadata to detect symlinks
    let sym_meta = path.symlink_metadata().map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            LlsError::PermissionDenied(path.display().to_string())
        } else {
            LlsError::Io(e)
        }
    })?;
    let is_symlink = sym_meta.is_symlink();
    let is_broken_symlink = is_symlink && path.metadata().is_err();

    let metadata = if is_symlink {
        // For symlinks, try to get target metadata
        path.metadata().unwrap_or(sym_meta)
    } else {
        sym_meta
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

    let (is_text, is_binary) = if metadata.is_file() {
        detect_text_binary(path)
    } else {
        (false, false)
    };

    Ok(EntryMeta {
        name,
        path: rel,
        entry_type,
        size_bytes,
        is_text,
        is_binary,
        is_broken_symlink,
    })
}

/// Simple text/binary detection based on extension and content sniff
fn detect_text_binary(path: &Path) -> (bool, bool) {
    // Check extension-based heuristics first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        let binary_exts = [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg",
            "woff", "woff2", "ttf", "eot",
            "zip", "tar", "gz", "bz2", "xz", "zst",
            "pdf", "doc", "docx", "xls", "xlsx",
            "mp3", "mp4", "avi", "mov", "mkv",
            "wasm", "o", "so", "dylib", "dll", "exe",
            "class", "jar",
            "pyc", "pyo",
            "bin", "dat",
        ];
        if binary_exts.contains(&ext_lower.as_str()) {
            return (false, true);
        }
        return (true, false);
    }

    // For files without extension, try to sniff content
    if let Ok(content) = std::fs::read(path) {
        let sample = if content.len() > 512 { &content[..512] } else { &content };
        let has_null = sample.contains(&0x00);
        return (!has_null, has_null);
    }

    (false, false)
}
