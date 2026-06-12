use crate::config::model::RulesConfig;
use crate::config::pattern::GlobPattern;
use crate::model::{EntryType, Priority, Role};

/// Result of priority assignment.
#[derive(Debug, Clone)]
pub struct PriorityResult {
    pub priority: Priority,
    pub reason_code: String,
}

/// Assign priority following spec section 10.3:
///
/// 1. Ignore patterns (user) -> ignore
/// 2. User priority overrides (first match)
/// 3. Built-in priority
/// 4. Role default priority
/// 5. Low
///
/// Safety rules (not overridable):
/// - .git/ is always ignore
/// - sensitive: true entries are excluded from recommendations (handled at recommendation level)
/// - binary entries are excluded from recommendations (handled at recommendation level)
pub fn assign_priority(
    name: &str,
    path: &str,
    entry_type: EntryType,
    role: Role,
    generated: bool,
    _sensitive: bool,
    rules: &RulesConfig,
) -> PriorityResult {
    // Safety rule: .git/ is always ignore (checked by path)
    if path == ".git" || path.starts_with(".git/") || path == ".git/" {
        return PriorityResult {
            priority: Priority::Ignore,
            reason_code: "safety_rule_git_ignore".into(),
        };
    }

    // 1. User ignore patterns
    for pattern in &rules.ignore_patterns {
        if let Some(p) = GlobPattern::compile(pattern)
            && p.matches(path)
        {
            return PriorityResult {
                priority: Priority::Ignore,
                reason_code: "matched_ignore_pattern".into(),
            };
        }
    }

    // 2. User priority overrides
    for override_rule in &rules.priority_overrides {
        if let Some(p) = GlobPattern::compile(&override_rule.pattern)
            && p.matches(path)
        {
            let priority = parse_priority_str(&override_rule.priority);
            return PriorityResult {
                priority,
                reason_code: "matched_priority_override".into(),
            };
        }
    }

    // 3. Built-in priority (based on role)
    let (priority, reason_code) = builtin_priority(name, path, role, entry_type, generated);

    PriorityResult {
        priority,
        reason_code,
    }
}

/// Built-in priority rules.
fn builtin_priority(
    name: &str,
    path: &str,
    role: Role,
    entry_type: EntryType,
    _generated: bool,
) -> (Priority, String) {
    // Priority for specific known paths
    match path {
        "README.md" => {
            return (Priority::Critical, "known_project_overview".into());
        }
        "Cargo.toml" | "package.json" | "pyproject.toml" | "setup.py" | "go.mod" => {
            return (Priority::Critical, "known_manifest".into());
        }
        _ => {}
    }

    // Priority for specific names (not full path)
    match name {
        "Makefile" | "Cargo.toml" => {
            return (Priority::Critical, "known_manifest".into());
        }
        _ => {}
    }

    // Built-in build output directories
    if entry_type == EntryType::Directory {
        match name {
            "target" | "dist" | "build" | ".next" | ".nuxt" => {
                return (Priority::Ignore, "known_build_output".into());
            }
            "node_modules" | ".git" => {
                return (Priority::Ignore, "known_ignored_directory".into());
            }
            _ => {}
        }
    }

    // 4. Role default priority
    match role {
        Role::ProjectOverview => (Priority::Critical, "role_default".into()),
        Role::Manifest => (Priority::Critical, "role_default".into()),
        Role::SourceCode => (Priority::High, "role_default".into()),
        Role::CiConfig => (Priority::High, "role_default".into()),
        Role::TestCode => (Priority::Medium, "role_default".into()),
        Role::Documentation => (Priority::Medium, "role_default".into()),
        Role::Config => (Priority::Medium, "role_default".into()),
        Role::Lockfile => (Priority::Medium, "role_default".into()),
        Role::License => (Priority::Medium, "role_default".into()),
        Role::Data => (Priority::Low, "role_default".into()),
        Role::BuildOutput => (Priority::Ignore, "role_default".into()),
        Role::DependencyCache => (Priority::Ignore, "role_default".into()),
        Role::Unknown => (Priority::Low, "fallback_unknown".into()),
    }
}

fn parse_priority_str(s: &str) -> Priority {
    match s {
        "critical" => Priority::Critical,
        "high" => Priority::High,
        "medium" => Priority::Medium,
        "low" => Priority::Low,
        "ignore" => Priority::Ignore,
        _ => Priority::Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_rules() -> RulesConfig {
        RulesConfig {
            priority_overrides: vec![],
            role_overrides: vec![],
            ignore_patterns: vec![],
            sensitive_patterns: vec![],
        }
    }

    #[test]
    fn test_readme_critical() {
        let result = assign_priority(
            "README.md",
            "README.md",
            EntryType::File,
            Role::ProjectOverview,
            false,
            false,
            &empty_rules(),
        );
        assert_eq!(result.priority, Priority::Critical);
    }

    #[test]
    fn test_manifest_critical() {
        let result = assign_priority(
            "Cargo.toml",
            "Cargo.toml",
            EntryType::File,
            Role::Manifest,
            false,
            false,
            &empty_rules(),
        );
        assert_eq!(result.priority, Priority::Critical);
    }

    #[test]
    fn test_src_high() {
        let result = assign_priority(
            "src",
            "src",
            EntryType::Directory,
            Role::SourceCode,
            false,
            false,
            &empty_rules(),
        );
        assert_eq!(result.priority, Priority::High);
    }

    #[test]
    fn test_build_output_ignore() {
        let result = assign_priority(
            "target",
            "target",
            EntryType::Directory,
            Role::BuildOutput,
            true,
            false,
            &empty_rules(),
        );
        assert_eq!(result.priority, Priority::Ignore);
    }

    #[test]
    fn test_git_safety_rule() {
        let result = assign_priority(
            ".git",
            ".git",
            EntryType::Directory,
            Role::DependencyCache,
            false,
            false,
            &empty_rules(),
        );
        assert_eq!(result.priority, Priority::Ignore);
    }

    #[test]
    fn test_priority_override() {
        let mut rules = empty_rules();
        rules
            .priority_overrides
            .push(crate::config::model::OverrideRule {
                pattern: "docs/**".into(),
                priority: "high".into(),
            });
        let result = assign_priority(
            "readme.md",
            "docs/readme.md",
            EntryType::File,
            Role::Documentation,
            false,
            false,
            &rules,
        );
        assert_eq!(result.priority, Priority::High);
    }

    #[test]
    fn test_ignore_pattern() {
        let mut rules = empty_rules();
        rules.ignore_patterns.push("tmp/**".into());
        let result = assign_priority(
            "data.txt",
            "tmp/data.txt",
            EntryType::File,
            Role::Unknown,
            false,
            false,
            &rules,
        );
        assert_eq!(result.priority, Priority::Ignore);
    }
}
