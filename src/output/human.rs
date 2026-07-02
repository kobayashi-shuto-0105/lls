use crate::model::*;

/// Format the output document as human-readable text.
pub fn to_human_string(doc: &OutputDocument) -> String {
    let mut out = String::new();

    out.push_str(&format!("lls — {}\n", doc.path));
    out.push_str(&format!(
        "Project: {} (confidence {:.2})\n",
        doc.project_type.name, doc.project_type.confidence
    ));
    if !doc.project_type.evidence.is_empty() {
        out.push_str(&format!(
            "Evidence: {}\n",
            doc.project_type.evidence.join(", ")
        ));
    }
    out.push('\n');

    out.push_str("Summary\n");
    out.push_str(&format!(
        "  entries: {} total | {} shown | {} important | {} ignored\n",
        doc.summary.total_entries,
        doc.summary.shown_entries,
        doc.summary.important_entries,
        doc.summary.ignored_entries,
    ));
    out.push_str(&format!(
        "  signals: {} next steps | {} warnings\n",
        doc.recommended_next_steps.len(),
        doc.warnings.len()
    ));
    out.push('\n');

    out.push_str("Top entries\n");
    for (index, entry) in doc.entries.iter().take(20).enumerate() {
        out.push_str(&format!(
            "  {:>2}. {:<8} | {:<16} | {:<9} | {:>8} | {}\n",
            index + 1,
            entry.priority,
            humanize(&entry.role),
            entry.entry_type,
            format_size(entry.size_bytes),
            entry.path
        ));
        if entry.generated || entry.sensitive {
            out.push_str(&format!("      flags: {}\n", format_flags(entry)));
        }
        out.push_str(&format!("      why: {}\n", entry.reason));
    }

    if doc.entries.len() > 20 {
        out.push_str(&format!(
            "  ... and {} more entries\n",
            doc.entries.len() - 20
        ));
    }

    if !doc.recommended_next_steps.is_empty() {
        out.push_str("\nNext steps\n");
        for (index, rec) in doc.recommended_next_steps.iter().enumerate() {
            out.push_str(&format!("  {}. {} {}\n", index + 1, rec.action, rec.path));
            out.push_str(&format!("     {}\n", rec.reason));
        }
    }

    if !doc.warnings.is_empty() {
        out.push_str("\nWarnings\n");
        for w in &doc.warnings {
            out.push_str(&format!("  - {}\n", w.code));
            out.push_str(&format!("    {}\n", w.message));
            if let Some(ref path) = w.path {
                out.push_str(&format!("    path: {}\n", path));
            }
        }
    }

    out
}

fn humanize(value: &str) -> String {
    value.replace('_', " ")
}

fn format_flags(entry: &EntryOutput) -> String {
    match (entry.generated, entry.sensitive) {
        (true, true) => "generated, sensitive".to_string(),
        (true, false) => "generated".to_string(),
        (false, true) => "sensitive".to_string(),
        (false, false) => String::new(),
    }
}

fn format_size(size_bytes: Option<u64>) -> String {
    match size_bytes {
        Some(bytes) => human_size(bytes),
        None => "-".to_string(),
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit_idx])
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
                evidence: vec!["Cargo.toml".into(), "src/main.rs".into()],
            },
            summary: SummaryOutput {
                total_entries: 3,
                shown_entries: 3,
                important_entries: 2,
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
                    size_bytes: None,
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
            recommended_next_steps: vec![RecommendationOutput {
                action: "read".into(),
                path: "Cargo.toml".into(),
                reason_code: "test".into(),
                reason: "プロジェクト構成を理解するため".into(),
            }],
            warnings: vec![],
        }
    }

    #[test]
    fn test_human_output_contains_path() {
        let output = to_human_string(&sample_doc());
        assert!(output.contains("lls — ."));
        assert!(output.contains("rust_cli"));
        assert!(output.contains("Evidence: Cargo.toml, src/main.rs"));
        assert!(output.contains("Top entries"));
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("Next steps"));
    }

    #[test]
    fn test_human_no_json() {
        let output = to_human_string(&sample_doc());
        assert!(!output.starts_with('{'));
        assert!(!output.starts_with('['));
    }

    #[test]
    fn test_human_generated_marker() {
        let output = to_human_string(&sample_doc());
        assert!(output.contains("flags: generated"));
    }

    #[test]
    fn test_human_size_formatting() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1024), "1.0 KB");
    }
}
