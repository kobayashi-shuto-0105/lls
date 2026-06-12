use crate::config::model::ProjectConfig;
use crate::error::AppError;

/// Perform safety checks on a config proposal before writing.
///
/// Returns Ok if the config is safe to write.
pub fn safety_check(config: &ProjectConfig) -> Result<(), AppError> {
    // .git/ must always be ignore
    for rule in &config.rules.priority_overrides {
        if (rule.pattern == ".git"
            || rule.pattern == ".git/"
            || rule.pattern == "/.git"
            || rule.pattern == "/.git/")
            && rule.priority != "ignore"
        {
            return Err(AppError::InvalidConfig(
                ".git/ must have priority 'ignore'".into(),
            ));
        }
    }

    // No absolute paths in patterns
    let all_patterns = config
        .rules
        .priority_overrides
        .iter()
        .map(|r| &r.pattern)
        .chain(config.rules.role_overrides.iter().map(|r| &r.pattern))
        .chain(config.rules.ignore_patterns.iter())
        .chain(config.rules.sensitive_patterns.iter());

    for pattern in all_patterns {
        if pattern.starts_with('/') {
            // Leading / is allowed for anchoring to project root
            // But absolute paths like /etc/... should be rejected
            if pattern.len() > 1 && pattern.as_bytes()[1] == b'/' || pattern.starts_with("//") {
                return Err(AppError::InvalidConfig(format!(
                    "absolute path pattern not allowed: '{pattern}'"
                )));
            }
        }
        if pattern.contains("://") {
            return Err(AppError::InvalidConfig(format!(
                "URL-like pattern not allowed: '{pattern}'"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> ProjectConfig {
        ProjectConfig {
            schema_version: "0.1.0".into(),
            default_output: "json".into(),
            scan: crate::config::model::ScanConfig {
                depth: 1,
                include_hidden: true,
                include_ignored: false,
            },
            long_listing: crate::config::model::LongListingConfig {
                sort: "priority".into(),
            },
            rules: crate::config::model::RulesConfig {
                priority_overrides: vec![],
                role_overrides: vec![],
                ignore_patterns: vec![],
                sensitive_patterns: vec![],
            },
            codex: crate::config::model::CodexConfig {
                enabled: true,
                auth_method: "chatgpt".into(),
                use_for_setup: true,
            },
        }
    }

    #[test]
    fn test_clean_config_passes() {
        let config = valid_config();
        assert!(safety_check(&config).is_ok());
    }

    #[test]
    fn test_git_override_rejected() {
        let mut config = valid_config();
        config
            .rules
            .priority_overrides
            .push(crate::config::model::OverrideRule {
                pattern: ".git".into(),
                priority: "high".into(),
            });
        assert!(safety_check(&config).is_err());
    }
}
