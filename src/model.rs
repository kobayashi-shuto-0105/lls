use std::path::PathBuf;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Domain enums
// ---------------------------------------------------------------------------

/// Filesystem entry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

impl EntryType {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryType::File => "file",
            EntryType::Directory => "directory",
            EntryType::Symlink => "symlink",
            EntryType::Other => "other",
        }
    }
}

/// Semantic role of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    ProjectOverview,
    Manifest,
    SourceCode,
    TestCode,
    Documentation,
    CiConfig,
    Config,
    Lockfile,
    License,
    Data,
    BuildOutput,
    DependencyCache,
    Unknown,
}

impl Role {
    /// Returns the spec-defined rank for role-based sorting.
    /// Lower values sort first.
    pub fn sort_rank(self) -> u8 {
        match self {
            Self::ProjectOverview => 0,
            Self::Manifest => 1,
            Self::SourceCode => 2,
            Self::TestCode => 3,
            Self::Documentation => 4,
            Self::CiConfig => 5,
            Self::Config => 6,
            Self::Lockfile => 7,
            Self::License => 8,
            Self::Data => 9,
            Self::BuildOutput => 10,
            Self::DependencyCache => 11,
            Self::Unknown => 12,
        }
    }

    /// Returns the serialized string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProjectOverview => "project_overview",
            Self::Manifest => "manifest",
            Self::SourceCode => "source_code",
            Self::TestCode => "test_code",
            Self::Documentation => "documentation",
            Self::CiConfig => "ci_config",
            Self::Config => "config",
            Self::Lockfile => "lockfile",
            Self::License => "license",
            Self::Data => "data",
            Self::BuildOutput => "build_output",
            Self::DependencyCache => "dependency_cache",
            Self::Unknown => "unknown",
        }
    }
}

/// Priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Ignore,
}

impl Priority {
    /// Returns the spec-defined rank for sorting. Lower = higher priority.
    pub fn sort_rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
            Self::Ignore => 4,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Ignore => "ignore",
        }
    }
}

// ---------------------------------------------------------------------------
// Project type detection
// ---------------------------------------------------------------------------

/// Detected project type.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectType {
    pub name: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

// ---------------------------------------------------------------------------
// Warning
// ---------------------------------------------------------------------------

/// A non-fatal warning attached to the output.
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Recommendation
// ---------------------------------------------------------------------------

/// A recommended next action for the user.
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub action: String,
    pub path: String,
    pub reason_code: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub total_entries: usize,
    pub shown_entries: usize,
    pub important_entries: usize,
    pub ignored_entries: usize,
}

// ---------------------------------------------------------------------------
// Raw / classified entries
// ---------------------------------------------------------------------------

/// Filesystem metadata before classification.
#[derive(Debug, Clone)]
pub struct RawEntry {
    pub name: String,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub entry_type: EntryType,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<SystemTime>,
}

/// Fully classified entry with role, priority, and attributes.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub role: Role,
    pub priority: Priority,
    pub reason_code: String,
    pub reason: String,
    pub generated: bool,
    pub sensitive: bool,
    pub text: Option<bool>,
    pub binary: Option<bool>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<SystemTime>,
}

// ---------------------------------------------------------------------------
// JSON output model
// ---------------------------------------------------------------------------

use serde::Serialize;

/// Top-level output document.
#[derive(Debug, Clone, Serialize)]
pub struct OutputDocument {
    pub schema_version: String,
    pub path: String,
    pub project_type: ProjectTypeOutput,
    pub summary: SummaryOutput,
    pub entries: Vec<EntryOutput>,
    pub recommended_next_steps: Vec<RecommendationOutput>,
    pub warnings: Vec<WarningOutput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectTypeOutput {
    pub name: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryOutput {
    pub total_entries: usize,
    pub shown_entries: usize,
    pub important_entries: usize,
    pub ignored_entries: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryOutput {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub role: String,
    pub priority: String,
    pub reason_code: String,
    pub reason: String,
    pub generated: bool,
    pub sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationOutput {
    pub action: String,
    pub path: String,
    pub reason_code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WarningOutput {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::ProjectOverview.as_str(), "project_overview");
        assert_eq!(Role::Manifest.as_str(), "manifest");
        assert_eq!(Role::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::Critical.as_str(), "critical");
        assert_eq!(Priority::Ignore.as_str(), "ignore");
    }

    #[test]
    fn test_sort_ranks() {
        assert!(Priority::Critical.sort_rank() < Priority::High.sort_rank());
        assert!(Priority::High.sort_rank() < Priority::Medium.sort_rank());
        assert!(Priority::Medium.sort_rank() < Priority::Low.sort_rank());
        assert!(Priority::Low.sort_rank() < Priority::Ignore.sort_rank());

        assert!(Role::ProjectOverview.sort_rank() < Role::Manifest.sort_rank());
        assert!(Role::Manifest.sort_rank() < Role::SourceCode.sort_rank());
        assert!(Role::DependencyCache.sort_rank() < Role::Unknown.sort_rank());
    }

    #[test]
    fn test_optional_field_omitted() {
        let entry = EntryOutput {
            name: "f.txt".into(),
            path: "f.txt".into(),
            entry_type: "file".into(),
            role: "data".into(),
            priority: "low".into(),
            reason_code: "fallback".into(),
            reason: "test".into(),
            generated: false,
            sensitive: false,
            text: None,
            binary: None,
            size_bytes: None,
        };
        let json = serde_json::to_value(&entry).unwrap();
        // Optional fields should be absent, not null
        assert!(json.get("text").is_none());
        assert!(json.get("binary").is_none());
        assert!(json.get("size_bytes").is_none());
        // Required fields must be present
        assert!(json.get("name").is_some());
        assert!(json.get("type").is_some());
    }

    #[test]
    fn test_output_document_serialization() {
        let doc = OutputDocument {
            schema_version: "0.1.0".into(),
            path: ".".into(),
            project_type: ProjectTypeOutput {
                name: "unknown".into(),
                confidence: 0.0,
                evidence: vec![],
            },
            summary: SummaryOutput {
                total_entries: 0,
                shown_entries: 0,
                important_entries: 0,
                ignored_entries: 0,
            },
            entries: vec![],
            recommended_next_steps: vec![],
            warnings: vec![],
        };
        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json["schema_version"], "0.1.0");
        assert_eq!(json["summary"]["total_entries"], 0);
        // No null values
        assert!(!json.to_string().contains("null"));
    }
}
