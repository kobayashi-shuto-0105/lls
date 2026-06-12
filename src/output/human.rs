use crate::model::*;

/// Format the output document as human-readable text.
pub fn to_human_string(doc: &OutputDocument) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("lls — {}\n", doc.path));
    out.push_str(&format!(
        "Project type: {} (confidence: {:.2})\n",
        doc.project_type.name, doc.project_type.confidence
    ));
    out.push('\n');

    // Summary
    out.push_str(&format!(
        "{} entries ({} shown, {} important, {} ignored)\n\n",
        doc.summary.total_entries,
        doc.summary.shown_entries,
        doc.summary.important_entries,
        doc.summary.ignored_entries,
    ));

    // Entries (first 20)
    for entry in doc.entries.iter().take(20) {
        let marker = if entry.generated && entry.sensitive {
            " [generated, sensitive]"
        } else if entry.generated {
            " [generated]"
        } else if entry.sensitive {
            " [sensitive]"
        } else {
            ""
        };
        out.push_str(&format!(
            "  [{:<8}] [{:<15}] {}{}\n",
            entry.priority, entry.role, entry.path, marker
        ));
    }

    if doc.entries.len() > 20 {
        out.push_str(&format!(
            "  ... and {} more entries\n",
            doc.entries.len() - 20
        ));
    }

    // Recommendations
    if !doc.recommended_next_steps.is_empty() {
        out.push_str("\nRecommended next steps:\n");
        for rec in &doc.recommended_next_steps {
            out.push_str(&format!("  {} {} — {}\n", rec.action, rec.path, rec.reason));
        }
    }

    // Warnings
    if !doc.warnings.is_empty() {
        out.push_str("\nWarnings:\n");
        for w in &doc.warnings {
            if let Some(ref path) = w.path {
                out.push_str(&format!("  {}: {} ({})\n", w.code, w.message, path));
            } else {
                out.push_str(&format!("  {}: {}\n", w.code, w.message));
            }
        }
    }

    out
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
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("Recommended next steps"));
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
        assert!(output.contains("[generated]"));
    }
}
