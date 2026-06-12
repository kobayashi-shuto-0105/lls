use serde::{Deserialize, Serialize};

/// The built-in default configuration used when `--no-config` is specified.
/// These values match the spec's "組み込み既定値" (built-in defaults).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltInDefaults {
    pub output: String,
    pub depth: u8,
    pub include_hidden: bool,
    pub include_ignored: bool,
    pub long_sort: String,
}

pub fn built_in_defaults() -> BuiltInDefaults {
    BuiltInDefaults {
        output: "json".into(),
        depth: 1,
        include_hidden: true,
        include_ignored: false,
        long_sort: "priority".into(),
    }
}
