use crate::model::{ProjectType, Warning};
use std::path::Path;

/// Result of probing a project.
#[derive(Debug)]
pub struct ProbeResult {
    pub project_type: ProjectType,
    pub warnings: Vec<Warning>,
}

/// Probe descriptors with their required files.
const PROBES: &[(&str, f64, &[&str])] = &[
    ("rust_cli", 0.95, &["Cargo.toml", "src/main.rs"]),
    ("rust_library", 0.95, &["Cargo.toml", "src/lib.rs"]),
    ("node_project", 0.90, &["package.json"]),
    ("python_package", 0.85, &["pyproject.toml"]),
    ("python_package", 0.80, &["setup.py"]),
    ("go_module", 0.85, &["go.mod"]),
];

/// Precedence order for project types (spec 8.1).
const PRECEDENCE: &[&str] = &[
    "rust_cli",
    "rust_library",
    "node_project",
    "python_package",
    "go_module",
];

/// Probe for project type by checking fixed paths from project root.
///
/// Does NOT perform recursive scanning.
pub fn probe_project(project_root: &Path) -> ProbeResult {
    let mut candidates: Vec<(&str, f64, Vec<String>)> = Vec::new();

    for (name, confidence, required_files) in PROBES {
        let mut evidence = Vec::new();
        let mut all_found = true;

        for file in *required_files {
            let path = project_root.join(file);
            if path.exists() {
                evidence.push(file.to_string());
            } else {
                all_found = false;
                break;
            }
        }

        if all_found && !evidence.is_empty() {
            candidates.push((name, *confidence, evidence));
        }
    }

    let mut warnings = Vec::new();

    // Apply precedence to find the best match
    let (name, confidence, evidence) = if candidates.is_empty() {
        ("unknown".to_string(), 0.0, Vec::new())
    } else if candidates.len() == 1 {
        let (n, c, e) = candidates.remove(0);
        (n.to_string(), c, e)
    } else {
        // Multiple candidates: log warning and pick by precedence
        warnings.push(Warning {
            code: "multiple_project_types_detected".into(),
            path: Some(project_root.to_string_lossy().into()),
            message: format!(
                "multiple project types detected: {}",
                candidates
                    .iter()
                    .map(|(n, _, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });

        // Pick the first match in precedence order
        let mut best = candidates[0].clone();
        for pref in PRECEDENCE {
            if let Some(c) = candidates.iter().find(|(n, _, _)| *n == *pref) {
                best = c.clone();
                break;
            }
        }
        (best.0.to_string(), best.1, best.2)
    };

    ProbeResult {
        project_type: ProjectType {
            name,
            confidence,
            evidence,
        },
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rust_cli() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "").unwrap();

        let result = probe_project(dir.path());
        assert_eq!(result.project_type.name, "rust_cli");
        assert!(result.project_type.confidence > 0.9);
        assert!(result.project_type.evidence.contains(&"Cargo.toml".into()));
        assert!(result.project_type.evidence.contains(&"src/main.rs".into()));
    }

    #[test]
    fn test_rust_library() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "").unwrap();

        let result = probe_project(dir.path());
        assert_eq!(result.project_type.name, "rust_library");
    }

    #[test]
    fn test_node_project() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();

        let result = probe_project(dir.path());
        assert_eq!(result.project_type.name, "node_project");
    }

    #[test]
    fn test_unknown() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("random.txt"), "").unwrap();

        let result = probe_project(dir.path());
        assert_eq!(result.project_type.name, "unknown");
        assert_eq!(result.project_type.confidence, 0.0);
    }

    #[test]
    fn test_polyglot_warning() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();

        let result = probe_project(dir.path());
        // Rust CLI wins precedence
        assert_eq!(result.project_type.name, "rust_cli");
        // Should have a polyglot warning
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.code == "multiple_project_types_detected")
        );
    }

    #[test]
    fn test_depth_independent() {
        // Project probe works at depth 1 - doesn't need recursive scan
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "").unwrap();

        // Even though src/main.rs is 2 levels deep in a file tree,
        // the probe just checks if the file exists
        let result = probe_project(dir.path());
        assert_eq!(result.project_type.name, "rust_cli");
    }
}
