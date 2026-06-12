use crate::model::*;

/// Serialize the output document to compact JSON with a trailing newline.
pub fn to_json_string(doc: &OutputDocument) -> String {
    let mut json = serde_json::to_string(doc).expect("serialization should not fail");
    json.push('\n');
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_doc() -> OutputDocument {
        OutputDocument {
            schema_version: "0.1.0".into(),
            path: ".".into(),
            project_type: ProjectTypeOutput {
                name: "unknown".into(),
                confidence: 0.0,
                evidence: vec![],
            },
            summary: SummaryOutput {
                total_entries: 0,
                shown_entries: 0,
                important_entries: 0,
                ignored_entries: 0,
            },
            entries: vec![],
            recommended_next_steps: vec![],
            warnings: vec![],
        }
    }

    #[test]
    fn test_compact_json() {
        let json = to_json_string(&empty_doc());
        // Should end with one newline
        assert!(json.ends_with('\n'), "JSON should end with newline");
        // Only one newline at the end
        assert_eq!(json.matches('\n').count(), 1, "only one trailing newline");
    }

    #[test]
    fn test_no_null_values() {
        let json = to_json_string(&empty_doc());
        assert!(
            !json.contains(":null"),
            "JSON should not contain null values"
        );
    }

    #[test]
    fn test_entry_in_json() {
        let doc = OutputDocument {
            schema_version: "0.1.0".into(),
            path: ".".into(),
            project_type: ProjectTypeOutput {
                name: "rust_cli".into(),
                confidence: 0.95,
                evidence: vec!["Cargo.toml".into(), "src/main.rs".into()],
            },
            summary: SummaryOutput {
                total_entries: 1,
                shown_entries: 1,
                important_entries: 1,
                ignored_entries: 0,
            },
            entries: vec![EntryOutput {
                name: "Cargo.toml".into(),
                path: "Cargo.toml".into(),
                entry_type: "file".into(),
                role: "manifest".into(),
                priority: "critical".into(),
                reason_code: "known_manifest".into(),
                reason: "Rust のマニフェストファイル".into(),
                generated: false,
                sensitive: false,
                text: Some(true),
                binary: None,
                size_bytes: Some(1024),
            }],
            recommended_next_steps: vec![RecommendationOutput {
                action: "read".into(),
                path: "Cargo.toml".into(),
                reason_code: "read_manifest_first".into(),
                reason: "プロジェクト構成を理解するため".into(),
            }],
            warnings: vec![],
        };

        let json = to_json_string(&doc);
        assert!(json.contains("schema_version"));
        assert!(json.contains("rust_cli"));
        assert!(json.contains("Cargo.toml"));
        assert!(json.contains("critical"));
    }
}
