use crate::classifier::ClassifiedEntry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectType {
    pub name: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Detect project type from a list of classified entries
pub fn detect(entries: &[ClassifiedEntry]) -> ProjectType {
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    let has = |name: &str| names.contains(&name) || paths.contains(&name);

    // Check for manifest files
    let has_cargo = has("Cargo.toml");
    let has_src_dir = names.contains(&"src") || paths.contains(&"src");
    let has_package_json = has("package.json");
    let has_pyproject = has("pyproject.toml");
    let has_setup_py = has("setup.py");
    let has_go_mod = has("go.mod");

    if has_cargo {
        if has_src_dir {
            return ProjectType {
                name: "rust_cli".into(),
                confidence: 0.9,
                evidence: vec!["Cargo.toml".into(), "src/".into()],
            };
        }
        return ProjectType {
            name: "rust_package".into(),
            confidence: 0.8,
            evidence: vec!["Cargo.toml".into()],
        };
    }

    if has_package_json {
        // Check for monorepo indicators
        if entries.iter().any(|e| e.name == "pnpm-workspace.yaml" || e.name == "turbo.json" || e.name == "nx.json") {
            return ProjectType {
                name: "monorepo".into(),
                confidence: 0.85,
                evidence: vec!["package.json".into(), "workspace config".into()],
            };
        }
        return ProjectType {
            name: "node_project".into(),
            confidence: 0.9,
            evidence: vec!["package.json".into()],
        };
    }

    if has_pyproject || has_setup_py {
        let mut evidence = Vec::new();
        if has_pyproject { evidence.push("pyproject.toml".into()); }
        if has_setup_py { evidence.push("setup.py".into()); }
        return ProjectType {
            name: "python_package".into(),
            confidence: 0.9,
            evidence,
        };
    }

    if has_go_mod {
        return ProjectType {
            name: "go_module".into(),
            confidence: 0.9,
            evidence: vec!["go.mod".into()],
        };
    }

    // Check for monorepo workspace configs without package.json
    if has("pnpm-workspace.yaml") || has("turbo.json") || has("nx.json") {
        return ProjectType {
            name: "monorepo".into(),
            confidence: 0.7,
            evidence: vec!["workspace config".into()],
        };
    }

    ProjectType {
        name: "unknown".into(),
        confidence: 0.0,
        evidence: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::ClassifiedEntry;

    fn entry(name: &str) -> ClassifiedEntry {
        ClassifiedEntry {
            name: name.to_string(),
            path: name.to_string(),
            entry_type: "file".into(),
            role: String::new(),
            priority: String::new(),
            reason: String::new(),
            generated: false,
            sensitive: false,
            text: Some(true),
            binary: Some(false),
            size_bytes: Some(100),
        }
    }

    #[test]
    fn test_rust_cli_detection() {
        let entries = vec![entry("Cargo.toml"), entry("src")];
        let pt = detect(&entries);
        assert_eq!(pt.name, "rust_cli");
        assert!(pt.confidence > 0.8);
    }

    #[test]
    fn test_node_project_detection() {
        let entries = vec![entry("package.json"), entry("src/index.js")];
        let pt = detect(&entries);
        assert_eq!(pt.name, "node_project");
    }

    #[test]
    fn test_unknown_detection() {
        let entries = vec![entry("notes.txt"), entry("data.csv")];
        let pt = detect(&entries);
        assert_eq!(pt.name, "unknown");
        assert_eq!(pt.confidence, 0.0);
    }
}
