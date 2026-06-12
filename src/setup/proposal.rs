use crate::config::model::ProjectConfig;

/// Generate a built-in default config proposal (used with --without-codex).
pub fn generate_proposal() -> ProjectConfig {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_is_deterministic() {
        let p1 = generate_proposal();
        let p2 = generate_proposal();
        assert_eq!(
            serde_json::to_string(&p1).unwrap(),
            serde_json::to_string(&p2).unwrap()
        );
    }

    #[test]
    fn test_proposal_valid_schema() {
        let proposal = generate_proposal();
        let json = serde_json::to_string(&proposal).unwrap();
        // Should pass schema validation
        let result = crate::config::validate_config(&json);
        assert!(result.is_ok());
    }
}
