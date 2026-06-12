use crate::config::model::ProjectConfig;
use crate::config::pattern::GlobPattern;
use crate::error::AppError;

/// A config file that has been parsed and schema-validated.
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub config: ProjectConfig,
}

/// A fully validated config after semantic checks.
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub config: ProjectConfig,
}

/// Validate a JSON string against the config schema and perform semantic validation.
pub fn validate_config(json_str: &str) -> Result<ValidatedConfig, AppError> {
    // Parse JSON
    let config: ProjectConfig = serde_json::from_str(json_str)
        .map_err(|e| AppError::InvalidConfig(format!("JSON parse error: {e}")))?;

    // Schema version must match
    if config.schema_version != "0.1.0" {
        return Err(AppError::InvalidConfig(format!(
            "unsupported schema_version: {}",
            config.schema_version
        )));
    }

    // Validate scan depth
    if config.scan.depth > 8 {
        return Err(AppError::InvalidConfig("scan.depth must be 0..=8".into()));
    }

    // Validate auth_method
    if config.codex.auth_method != "chatgpt" {
        return Err(AppError::InvalidConfig(format!(
            "codex.auth_method must be 'chatgpt', got '{}'",
            config.codex.auth_method
        )));
    }

    // Validate output enum
    match config.default_output.as_str() {
        "json" | "human" | "long" => {}
        other => {
            return Err(AppError::InvalidConfig(format!(
                "invalid default_output: {other}"
            )));
        }
    }

    // Validate long_listing.sort
    match config.long_listing.sort.as_str() {
        "priority" | "name" | "mtime" | "size" => {}
        other => {
            return Err(AppError::InvalidConfig(format!(
                "invalid long_listing.sort: {other}"
            )));
        }
    }

    // Validate glob patterns and semantic rules
    validate_rules(&config)?;

    Ok(ValidatedConfig { config })
}

fn validate_rules(config: &ProjectConfig) -> Result<(), AppError> {
    // Check priority_overrides patterns
    for rule in &config.rules.priority_overrides {
        GlobPattern::compile(&rule.pattern).ok_or_else(|| {
            AppError::InvalidConfig(format!("invalid glob pattern: '{}'", rule.pattern))
        })?;

        // Validate priority enum
        match rule.priority.as_str() {
            "critical" | "high" | "medium" | "low" | "ignore" => {}
            other => {
                return Err(AppError::InvalidConfig(format!(
                    "invalid priority value: {other}"
                )));
            }
        }
    }

    // Check role_overrides patterns
    for rule in &config.rules.role_overrides {
        GlobPattern::compile(&rule.pattern).ok_or_else(|| {
            AppError::InvalidConfig(format!("invalid glob pattern: '{}'", rule.pattern))
        })?;

        // Validate role enum
        match rule.role.as_str() {
            "project_overview" | "manifest" | "lockfile" | "source_code" | "test_code"
            | "documentation" | "config" | "ci_config" | "build_output" | "dependency_cache"
            | "license" | "data" | "unknown" => {}
            other => {
                return Err(AppError::InvalidConfig(format!(
                    "invalid role value: {other}"
                )));
            }
        }
    }

    // Check ignore_patterns
    for pattern in &config.rules.ignore_patterns {
        GlobPattern::compile(pattern)
            .ok_or_else(|| AppError::InvalidConfig(format!("invalid glob pattern: '{pattern}'")))?;

        // Safety rule: .git/ cannot be overridden to non-ignore
        // (We check this in the safety validation)
    }

    // Check sensitive_patterns
    for pattern in &config.rules.sensitive_patterns {
        GlobPattern::compile(pattern)
            .ok_or_else(|| AppError::InvalidConfig(format!("invalid glob pattern: '{pattern}'")))?;
    }

    // Safety rule: .git/ must always be ignore
    for rule in &config.rules.priority_overrides {
        if (rule.pattern == ".git"
            || rule.pattern == ".git/"
            || rule.pattern == "/.git"
            || rule.pattern == "/.git/")
            && rule.priority != "ignore"
        {
            return Err(AppError::InvalidConfig(
                ".git/ must always have priority 'ignore'".into(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> String {
        r#"{
            "schema_version": "0.1.0",
            "default_output": "json",
            "scan": {
                "depth": 1,
                "include_hidden": true,
                "include_ignored": false
            },
            "long_listing": {
                "sort": "priority"
            },
            "rules": {
                "priority_overrides": [],
                "role_overrides": [],
                "ignore_patterns": [],
                "sensitive_patterns": []
            },
            "codex": {
                "enabled": true,
                "auth_method": "chatgpt",
                "use_for_setup": true
            }
        }"#
        .to_string()
    }

    #[test]
    fn test_valid_config() {
        let result = validate_config(&valid_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_schema_version() {
        let mut cfg = valid_config();
        cfg = cfg.replace("\"0.1.0\"", "\"0.2.0\"");
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_invalid_auth_method() {
        let cfg = valid_config().replace("\"chatgpt\"", "\"api_key\"");
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_invalid_depth() {
        let cfg = valid_config().replace("\"depth\": 1", "\"depth\": 9");
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_invalid_priority_enum() {
        let cfg = valid_config().replace(
            "\"priority_overrides\": []",
            r#""priority_overrides": [{"pattern": "*.rs", "priority": "super"}]"#,
        );
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_invalid_role_enum() {
        let cfg = valid_config().replace(
            "\"role_overrides\": []",
            r#""role_overrides": [{"pattern": "*.rs", "role": "super_code"}]"#,
        );
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_empty_glob_pattern_rejected() {
        // Empty pattern is invalid
        let cfg = valid_config().replace("\"ignore_patterns\": []", r#""ignore_patterns": [""]"#);
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_git_safety_rule() {
        let cfg = valid_config().replace(
            "\"priority_overrides\": []",
            r#""priority_overrides": [{"pattern": ".git", "priority": "high"}]"#,
        );
        assert!(validate_config(&cfg).is_err());
    }
}
