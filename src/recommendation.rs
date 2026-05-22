use crate::classifier::ClassifiedEntry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecommendedStep {
    pub action: String,
    pub path: String,
    pub reason: String,
}

/// Generate recommended next steps from classified entries.
///
/// Rules:
/// 1. Critical entries first
/// 2. README.md etc. before source
/// 3. Manifest files before source directories
/// 4. Skip priority: "ignore" entries
/// 5. Skip sensitive: true entries
/// 6. Skip binary files
/// 7. Max 5 entries
pub fn generate(entries: &[ClassifiedEntry]) -> Vec<RecommendedStep> {
    let mut candidates: Vec<&ClassifiedEntry> = entries
        .iter()
        .filter(|e| e.priority != "ignore")
        .filter(|e| !e.sensitive)
        .filter(|e| e.binary != Some(true))
        .collect();

    // Sort: critical > high > medium > low, then by priority order within same level
    let priority_order = |p: &str| -> usize {
        match p {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        }
    };

    // Within same priority, prefer README > manifest > source > test > config
    let role_order = |r: &str| -> usize {
        match r {
            "project_overview" => 0,
            "manifest" => 1,
            "source_code" => 2,
            "test_code" => 3,
            "config" => 4,
            "ci_config" => 5,
            "documentation" => 6,
            "license" => 7,
            _ => 8,
        }
    };

    candidates.sort_by(|a, b| {
        let pa = priority_order(&a.priority);
        let pb = priority_order(&b.priority);
        pa.cmp(&pb).then_with(|| {
            let ra = role_order(&a.role);
            let rb = role_order(&b.role);
            ra.cmp(&rb)
        })
    });

    candidates
        .into_iter()
        .take(5)
        .map(|e| {
            let action = if e.entry_type == "directory" {
                "inspect".to_string()
            } else {
                "read".to_string()
            };
            let reason = recommendation_reason(&e.role, &e.name);
            RecommendedStep {
                action,
                path: e.path.clone(),
                reason,
            }
        })
        .collect()
}

fn recommendation_reason(role: &str, _name: &str) -> String {
    match role {
        "project_overview" => "プロジェクト概要を把握するため最初に読むべき".into(),
        "manifest" => "パッケージのメタデータと依存関係を確認するため".into(),
        "source_code" => "メインのソースコードが含まれる可能性が高いため".into(),
        "test_code" => "テストコードを確認するため".into(),
        "config" => "プロジェクトの設定を確認するため".into(),
        "ci_config" => "CI/CD の設定を確認するため".into(),
        "documentation" => "ドキュメントを確認するため".into(),
        "license" => "ライセンス情報を確認するため".into(),
        "lockfile" => "依存関係の固定情報を確認するため".into(),
        _ => "確認する必要がある可能性があるため".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, role: &str, priority: &str, sensitive: bool, entry_type: &str) -> ClassifiedEntry {
        ClassifiedEntry {
            name: path.split('/').last().unwrap_or(path).to_string(),
            path: path.to_string(),
            entry_type: entry_type.to_string(),
            role: role.to_string(),
            priority: priority.to_string(),
            reason: String::new(),
            generated: false,
            sensitive,
            text: Some(true),
            binary: Some(false),
            size_bytes: Some(100),
        }
    }

    #[test]
    fn test_readme_is_first_recommendation() {
        let entries = vec![
            entry("src", "source_code", "high", false, "directory"),
            entry("Cargo.toml", "manifest", "critical", false, "file"),
            entry("README.md", "project_overview", "critical", false, "file"),
        ];
        let steps = generate(&entries);
        assert_eq!(steps[0].path, "README.md");
        assert_eq!(steps[1].path, "Cargo.toml");
        assert_eq!(steps[2].path, "src");
    }

    #[test]
    fn test_ignored_entries_excluded() {
        let entries = vec![
            entry("README.md", "project_overview", "critical", false, "file"),
            entry("target", "build_output", "ignore", false, "directory"),
            entry("node_modules", "dependency_cache", "ignore", false, "directory"),
        ];
        let steps = generate(&entries);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].path, "README.md");
    }

    #[test]
    fn test_sensitive_entries_excluded() {
        let entries = vec![
            entry(".env", "secret_candidate", "medium", true, "file"),
            entry("README.md", "project_overview", "critical", false, "file"),
        ];
        let steps = generate(&entries);
        assert_eq!(steps.len(), 1);
        assert!(!steps.iter().any(|s| s.path == ".env"));
    }

    #[test]
    fn test_max_five_recommendations() {
        let entries = (0..10).map(|i| {
            entry(&format!("file{i}.rs"), "source_code", "high", false, "file")
        }).collect::<Vec<_>>();
        let steps = generate(&entries);
        assert!(steps.len() <= 5);
    }
}
