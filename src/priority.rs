use crate::classifier::ClassifiedEntry;

/// Assign priority to a classified entry based on its role and path
pub fn assign(entry: &mut ClassifiedEntry) {
    entry.priority = determine_priority(&entry.role, &entry.path, &entry.name);
}

fn determine_priority(role: &str, path: &str, name: &str) -> String {
    // Critical
    match role {
        "project_overview" => return "critical".into(),
        "manifest" => return "critical".into(),
        _ => {}
    }

    // Check for specific critical source files
    if role == "source_code" {
        let name_lower = name.to_lowercase();
        if name_lower == "main.rs" || name_lower == "lib.rs" || name_lower == "mod.rs" {
            return "critical".into();
        }
        if path == "src" || path == "src/" || path.starts_with("src/") {
            return "high".into();
        }
        return "high".into();
    }

    // High
    match role {
        "ci_config" => return "high".into(),
        "source_code" => return "high".into(),
        _ => {}
    }

    // Directories with role source_code
    if role == "source_code" {
        return "high".into();
    }

    // Medium
    match role {
        "test_code" => return "medium".into(),
        "documentation" => return "medium".into(),
        "lockfile" => return "medium".into(),
        "config" => return "medium".into(),
        "secret_candidate" => return "medium".into(),
        "license" => return "low".into(),
        _ => {}
    }

    // Low
    match role {
        "license" => return "low".into(),
        _ => {}
    }

    // Ignore
    match role {
        "build_output" | "dependency_cache" | "generated" => return "ignore".into(),
        _ => {}
    }

    // Unknown files get low priority
    if role == "unknown" {
        return "low".into();
    }

    "medium".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify_with_role(name: &str, path: &str, role: &str) -> ClassifiedEntry {
        let mut entry = ClassifiedEntry {
            name: name.to_string(),
            path: path.to_string(),
            entry_type: "file".into(),
            role: role.to_string(),
            priority: String::new(),
            reason: String::new(),
            generated: false,
            sensitive: false,
            text: Some(true),
            binary: Some(false),
            size_bytes: Some(100),
        };
        assign(&mut entry);
        entry
    }

    #[test]
    fn test_readme_is_critical() {
        let entry = classify_with_role("README.md", "README.md", "project_overview");
        assert_eq!(entry.priority, "critical");
    }

    #[test]
    fn test_manifest_is_critical() {
        let entry = classify_with_role("Cargo.toml", "Cargo.toml", "manifest");
        assert_eq!(entry.priority, "critical");
    }

    #[test]
    fn test_src_main_rs_is_critical() {
        let entry = classify_with_role("main.rs", "src/main.rs", "source_code");
        assert_eq!(entry.priority, "critical");
    }

    #[test]
    fn test_src_dir_is_high() {
        let entry = classify_with_role("src", "src", "source_code");
        assert_eq!(entry.priority, "high");
    }

    #[test]
    fn test_tests_is_medium() {
        let entry = classify_with_role("tests", "tests", "test_code");
        assert_eq!(entry.priority, "medium");
    }

    #[test]
    fn test_target_is_ignore() {
        let entry = classify_with_role("target", "target", "build_output");
        assert_eq!(entry.priority, "ignore");
    }

    #[test]
    fn test_license_is_low() {
        let entry = classify_with_role("LICENSE", "LICENSE", "license");
        assert_eq!(entry.priority, "low");
    }
}
