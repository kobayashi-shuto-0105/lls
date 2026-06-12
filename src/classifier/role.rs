use crate::config::model::RulesConfig;
use crate::config::pattern::GlobPattern;
use crate::model::{EntryType, Role};

/// A built-in role classification rule.
#[derive(Debug, Clone)]
pub enum BuiltinRoleRule {
    /// Match by exact relative path
    ExactPath(&'static str, Role),
    /// Match by exact basename
    ExactName(&'static str, Role),
    /// Match by directory pattern (the entry is a directory with this name)
    DirectoryName(&'static str, Role),
    /// Match by extension
    Extension(&'static str, Role),
}

/// Built-in role rules in precedence order (spec 10.2):
/// 2. exact relative path
/// 3. exact basename
/// 4. directory pattern
/// 5. extension pattern
fn builtin_role_rules() -> Vec<BuiltinRoleRule> {
    vec![
        // Exact relative paths
        BuiltinRoleRule::ExactPath("README.md", Role::ProjectOverview),
        BuiltinRoleRule::ExactPath("Cargo.toml", Role::Manifest),
        BuiltinRoleRule::ExactPath("package.json", Role::Manifest),
        BuiltinRoleRule::ExactPath("pyproject.toml", Role::Manifest),
        BuiltinRoleRule::ExactPath("setup.py", Role::Manifest),
        BuiltinRoleRule::ExactPath("go.mod", Role::Manifest),
        BuiltinRoleRule::ExactPath("Cargo.lock", Role::Lockfile),
        BuiltinRoleRule::ExactPath("package-lock.json", Role::Lockfile),
        BuiltinRoleRule::ExactPath("yarn.lock", Role::Lockfile),
        BuiltinRoleRule::ExactPath("pnpm-lock.yaml", Role::Lockfile),
        BuiltinRoleRule::ExactPath("Gemfile.lock", Role::Lockfile),
        BuiltinRoleRule::ExactPath("go.sum", Role::Lockfile),
        BuiltinRoleRule::ExactPath("LICENSE", Role::License),
        BuiltinRoleRule::ExactPath("LICENSE.txt", Role::License),
        BuiltinRoleRule::ExactPath("LICENSE.md", Role::License),
        // .env files are config, not sensitive as role
        BuiltinRoleRule::ExactPath(".env", Role::Config),
        BuiltinRoleRule::ExactPath(".gitignore", Role::Config),
        BuiltinRoleRule::ExactPath(".dockerignore", Role::Config),
        // CI config
        BuiltinRoleRule::ExactPath(".github/workflows/", Role::CiConfig),
        BuiltinRoleRule::ExactPath(".gitlab-ci.yml", Role::CiConfig),
        BuiltinRoleRule::ExactPath("Dockerfile", Role::CiConfig),
        BuiltinRoleRule::ExactPath(".github/", Role::CiConfig),
        // Build output
        BuiltinRoleRule::ExactPath("target/", Role::BuildOutput),
        BuiltinRoleRule::ExactPath("dist/", Role::BuildOutput),
        BuiltinRoleRule::ExactPath("build/", Role::BuildOutput),
        // Dependency cache
        BuiltinRoleRule::ExactPath("node_modules/", Role::DependencyCache),
        BuiltinRoleRule::ExactPath(".git/", Role::DependencyCache),
        // Exact basenames
        BuiltinRoleRule::ExactName("Makefile", Role::Config),
        BuiltinRoleRule::ExactName("Cargo.toml", Role::Manifest),
        // Directory names
        BuiltinRoleRule::DirectoryName("src", Role::SourceCode),
        BuiltinRoleRule::DirectoryName("lib", Role::SourceCode),
        BuiltinRoleRule::DirectoryName("tests", Role::TestCode),
        BuiltinRoleRule::DirectoryName("test", Role::TestCode),
        BuiltinRoleRule::DirectoryName("spec", Role::TestCode),
        BuiltinRoleRule::DirectoryName("docs", Role::Documentation),
        BuiltinRoleRule::DirectoryName("doc", Role::Documentation),
        BuiltinRoleRule::DirectoryName("examples", Role::Documentation),
        BuiltinRoleRule::DirectoryName("config", Role::Config),
        BuiltinRoleRule::DirectoryName("ci", Role::CiConfig),
        // Extensions
        BuiltinRoleRule::Extension("rs", Role::SourceCode),
        BuiltinRoleRule::Extension("py", Role::SourceCode),
        BuiltinRoleRule::Extension("js", Role::SourceCode),
        BuiltinRoleRule::Extension("ts", Role::SourceCode),
        BuiltinRoleRule::Extension("tsx", Role::SourceCode),
        BuiltinRoleRule::Extension("jsx", Role::SourceCode),
        BuiltinRoleRule::Extension("go", Role::SourceCode),
        BuiltinRoleRule::Extension("java", Role::SourceCode),
        BuiltinRoleRule::Extension("rb", Role::SourceCode),
        BuiltinRoleRule::Extension("c", Role::SourceCode),
        BuiltinRoleRule::Extension("cpp", Role::SourceCode),
        BuiltinRoleRule::Extension("h", Role::SourceCode),
        BuiltinRoleRule::Extension("hpp", Role::SourceCode),
        BuiltinRoleRule::Extension("toml", Role::Config),
        BuiltinRoleRule::Extension("yaml", Role::Config),
        BuiltinRoleRule::Extension("yml", Role::Config),
        BuiltinRoleRule::Extension("json", Role::Config),
        BuiltinRoleRule::Extension("ini", Role::Config),
        BuiltinRoleRule::Extension("cfg", Role::Config),
        BuiltinRoleRule::Extension("conf", Role::Config),
        BuiltinRoleRule::Extension("md", Role::Documentation),
        BuiltinRoleRule::Extension("txt", Role::Documentation),
        BuiltinRoleRule::Extension("markdown", Role::Documentation),
    ]
}

/// Classify the role of an entry.
///
/// Follows spec section 10.2 precedence:
/// 1. User role overrides (first match)
/// 2. Built-in exact relative path
/// 3. Built-in exact basename
/// 4. Built-in directory pattern
/// 5. Built-in extension pattern
/// 6. Unknown
pub fn classify_role(name: &str, path: &str, entry_type: EntryType, rules: &RulesConfig) -> Role {
    // 1. User role overrides
    for override_rule in &rules.role_overrides {
        if let Some(p) = GlobPattern::compile(&override_rule.pattern)
            && p.matches(path)
        {
            return parse_role_str(&override_rule.role);
        }
    }

    // 2-5. Built-in rules
    for rule in builtin_role_rules() {
        match rule {
            BuiltinRoleRule::ExactPath(p, role) => {
                if path == p.trim_end_matches('/')
                    || path == p
                    || (entry_type == EntryType::Directory && format!("{path}/") == p)
                {
                    return role;
                }
            }
            BuiltinRoleRule::ExactName(n, role) => {
                if name == n {
                    return role;
                }
            }
            BuiltinRoleRule::DirectoryName(n, role) => {
                if entry_type == EntryType::Directory && name == n {
                    return role;
                }
            }
            BuiltinRoleRule::Extension(ext, role) => {
                if entry_type == EntryType::File
                    && let Some(dot) = name.rfind('.')
                    && name[dot + 1..] == *ext
                {
                    return role;
                }
            }
        }
    }

    // 6. Unknown
    Role::Unknown
}

fn parse_role_str(s: &str) -> Role {
    match s {
        "project_overview" => Role::ProjectOverview,
        "manifest" => Role::Manifest,
        "source_code" => Role::SourceCode,
        "test_code" => Role::TestCode,
        "documentation" => Role::Documentation,
        "ci_config" => Role::CiConfig,
        "config" => Role::Config,
        "lockfile" => Role::Lockfile,
        "license" => Role::License,
        "data" => Role::Data,
        "build_output" => Role::BuildOutput,
        "dependency_cache" => Role::DependencyCache,
        _ => Role::Unknown,
    }
}

// Keep this public so other modules can use it
#[allow(dead_code)]
pub fn parse_role(s: &str) -> Role {
    parse_role_str(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::RulesConfig;

    fn empty_rules() -> RulesConfig {
        RulesConfig {
            priority_overrides: vec![],
            role_overrides: vec![],
            ignore_patterns: vec![],
            sensitive_patterns: vec![],
        }
    }

    #[test]
    fn test_readme_is_project_overview() {
        let role = classify_role("README.md", "README.md", EntryType::File, &empty_rules());
        assert_eq!(role, Role::ProjectOverview);
    }

    #[test]
    fn test_cargo_toml_is_manifest() {
        let role = classify_role("Cargo.toml", "Cargo.toml", EntryType::File, &empty_rules());
        assert_eq!(role, Role::Manifest);
    }

    #[test]
    fn test_src_directory_is_source_code() {
        let role = classify_role("src", "src", EntryType::Directory, &empty_rules());
        assert_eq!(role, Role::SourceCode);
    }

    #[test]
    fn test_tests_directory_is_test_code() {
        let role = classify_role("tests", "tests", EntryType::Directory, &empty_rules());
        assert_eq!(role, Role::TestCode);
    }

    #[test]
    fn test_rust_file_is_source_code() {
        let role = classify_role("main.rs", "main.rs", EntryType::File, &empty_rules());
        assert_eq!(role, Role::SourceCode);
    }

    #[test]
    fn test_md_file_is_documentation() {
        let role = classify_role("docs.md", "docs.md", EntryType::File, &empty_rules());
        assert_eq!(role, Role::Documentation);
    }

    #[test]
    fn test_env_is_config() {
        let role = classify_role(".env", ".env", EntryType::File, &empty_rules());
        assert_eq!(role, Role::Config);
    }

    #[test]
    fn test_user_role_override() {
        let mut rules = empty_rules();
        rules
            .role_overrides
            .push(crate::config::model::RoleOverride {
                pattern: "*.md".into(),
                role: "documentation".into(),
            });
        let role = classify_role("CHANGELOG.md", "CHANGELOG.md", EntryType::File, &rules);
        assert_eq!(role, Role::Documentation);
    }

    #[test]
    fn test_unknown_extension() {
        let role = classify_role("data.csv", "data.csv", EntryType::File, &empty_rules());
        assert_eq!(role, Role::Unknown);
    }

    #[test]
    fn test_target_is_build_output() {
        let role = classify_role("target", "target", EntryType::Directory, &empty_rules());
        assert_eq!(role, Role::BuildOutput);
    }

    #[test]
    fn test_node_modules_is_dependency_cache() {
        let role = classify_role(
            "node_modules",
            "node_modules",
            EntryType::Directory,
            &empty_rules(),
        );
        assert_eq!(role, Role::DependencyCache);
    }
}
