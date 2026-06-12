use crate::model::*;

/// Format entries as a long listing (like `-l` mode).
///
/// Columns: priority, role, type, size, path, generated/sensitive marker.
pub fn to_long_string(doc: &OutputDocument) -> String {
    let mut out = String::new();

    // Header (commented out for parsing simplicity)
    out.push_str(&format!(
        "lls -l — {} ({} entries, type: {})\n",
        doc.path, doc.summary.shown_entries, doc.project_type.name
    ));
    out.push('\n');

    for entry in &doc.entries {
        let marker = if entry.generated && entry.sensitive {
            " [GS]"
        } else if entry.generated {
            " [G]"
        } else if entry.sensitive {
            " [S]"
        } else {
            ""
        };

        let size_str = match entry.size_bytes {
            Some(s) => human_size(s),
            None => "       -".to_string(),
        };

        out.push_str(&format!(
            "{:<8} {:<15} {:<5} {:>8} {}{}\n",
            entry.priority, entry.role, entry.entry_type, size_str, entry.path, marker
        ));
    }

    out
}

/// Format bytes in human-readable form.
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:>4} B", bytes)
    } else {
        format!("{:>4.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> OutputDocument {
        OutputDocument {
            schema_version: "0.1.0".into(),
            path: ".".into(),
            project_type: ProjectTypeOutput {
                name: "rust_cli".into(),
                confidence: 0.95,
                evidence: vec![],
            },
            summary: SummaryOutput {
                total_entries: 2,
                shown_entries: 2,
                important_entries: 1,
                ignored_entries: 1,
            },
            entries: vec![
                EntryOutput {
                    name: "Cargo.toml".into(),
                    path: "Cargo.toml".into(),
                    entry_type: "file".into(),
                    role: "manifest".into(),
                    priority: "critical".into(),
                    reason_code: "test".into(),
                    reason: "test".into(),
                    generated: false,
                    sensitive: false,
                    text: None,
                    binary: None,
                    size_bytes: Some(1024),
                },
                EntryOutput {
                    name: "target".into(),
                    path: "target".into(),
                    entry_type: "directory".into(),
                    role: "build_output".into(),
                    priority: "ignore".into(),
                    reason_code: "test".into(),
                    reason: "test".into(),
                    generated: true,
                    sensitive: false,
                    text: None,
                    binary: None,
                    size_bytes: None,
                },
            ],
            recommended_next_steps: vec![],
            warnings: vec![],
        }
    }

    #[test]
    fn test_long_listing_contains_columns() {
        let output = to_long_string(&sample_doc());
        assert!(output.contains("critical"));
        assert!(output.contains("manifest"));
        assert!(output.contains("file"));
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains(" [G]"));
    }

    #[test]
    fn test_human_size() {
        assert_eq!(human_size(0), "   0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1024), " 1.0 KB");
        assert_eq!(human_size(1536), " 1.5 KB");
        assert_eq!(human_size(1048576), " 1.0 MB");
    }
}
