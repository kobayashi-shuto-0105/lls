use crate::classifier::ClassifiedEntry;
use crate::project_type::ProjectType;
use crate::recommendation::RecommendedStep;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlsOutput {
    pub schema_version: String,
    pub path: String,
    pub project_type: ProjectType,
    pub summary: Summary,
    pub entries: Vec<ClassifiedEntry>,
    pub recommended_next_steps: Vec<RecommendedStep>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Summary {
    pub total_entries: usize,
    pub shown_entries: usize,
    pub important_entries: usize,
    pub ignored_entries: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Warning {
    pub path: String,
    pub message: String,
}

/// Build the full output structure
pub fn build_output(
    path: &str,
    project_type: ProjectType,
    entries: Vec<ClassifiedEntry>,
    recommended_next_steps: Vec<RecommendedStep>,
    scanner_warnings: Vec<(String, String)>,
    compact: bool,
) -> String {
    let total_entries = entries.len();
    let important_entries = entries.iter().filter(|e| e.priority != "ignore").count();
    let ignored_entries = entries.iter().filter(|e| e.priority == "ignore").count();
    let shown_entries = total_entries;

    let mut warnings: Vec<Warning> = scanner_warnings
        .into_iter()
        .map(|(path, msg)| Warning { path, message: msg })
        .collect();

    // Add sensitive file warnings
    for entry in &entries {
        if entry.sensitive {
            warnings.push(Warning {
                path: entry.path.clone(),
                message: "秘密情報候補を検出したため、明示的に必要な場合を除き内容を読まないこと".into(),
            });
        }
    }

    let output = LlsOutput {
        schema_version: "0.1.0".into(),
        path: path.to_string(),
        project_type,
        summary: Summary {
            total_entries,
            shown_entries,
            important_entries,
            ignored_entries,
        },
        entries,
        recommended_next_steps,
        warnings,
    };

    if compact {
        serde_json::to_string(&output).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    } else {
        serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }
}

/// Format human-readable output
pub fn format_human(
    path: &str,
    project_type: &ProjectType,
    entries: &[ClassifiedEntry],
    recommended_next_steps: &[RecommendedStep],
    warnings: &[Warning],
) -> String {
    let mut out = String::new();

    // Project header
    out.push_str(&format!("Project: {} (confidence: {})\n", project_type.name, project_type.confidence));
    out.push_str(&format!("Path: {}\n\n", path));

    // Recommended next steps
    if !recommended_next_steps.is_empty() {
        out.push_str("Recommended next steps:\n");
        for (i, step) in recommended_next_steps.iter().enumerate() {
            out.push_str(&format!("{}. {} {} - {}\n", i + 1, step.action, step.path, step.reason));
        }
        out.push('\n');
    }

    // Entries
    out.push_str("Entries:\n");
    for entry in entries {
        let priority_padded = format!("{:9}", entry.priority);
        let size_str = entry.size_bytes.map(|s| format!("{:>8} B", s)).unwrap_or_default();
        out.push_str(&format!(
            "[{}] {:<20} {:<12} {:<18} {} {} {}\n",
            priority_padded,
            entry.name,
            entry.entry_type,
            entry.role,
            size_str,
            entry.reason,
            if entry.generated { "[generated]" } else { "" }
        ));
    }

    // Warnings
    if !warnings.is_empty() {
        out.push_str("\nWarnings:\n");
        for w in warnings {
            out.push_str(&format!("- [{}] {}\n", w.path, w.message));
        }
    }

    out
}
