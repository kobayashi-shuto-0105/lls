use serde::{Deserialize, Serialize};

/// The schema-validated project configuration.
///
/// This mirrors `.github/assets/config.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: String,
    pub default_output: String,
    pub scan: ScanConfig,
    pub long_listing: LongListingConfig,
    pub rules: RulesConfig,
    pub codex: CodexConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub depth: u8,
    pub include_hidden: bool,
    pub include_ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongListingConfig {
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    pub priority_overrides: Vec<OverrideRule>,
    pub role_overrides: Vec<RoleOverride>,
    pub ignore_patterns: Vec<String>,
    pub sensitive_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideRule {
    pub pattern: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleOverride {
    pub pattern: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub enabled: bool,
    pub auth_method: String,
    pub use_for_setup: bool,
}

/// Effective configuration after CLI overrides are merged.
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub output_mode: String,
    pub depth: u8,
    pub include_hidden: bool,
    pub include_ignored: bool,
    pub long_sort: String,
    pub rules: RulesConfig,
    pub codex: CodexConfig,
}
