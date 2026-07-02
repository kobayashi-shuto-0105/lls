use crate::model::*;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_BLACK: &str = "\x1b[90m";

/// Format the output document as human-readable text.
pub fn to_human_string(doc: &OutputDocument) -> String {
    let mut out = String::new();

    out.push_str(&format!("{BOLD}🌿 lls — {}{RESET}\n", doc.path));
    out.push_str(&format!(
        "{}{}{} {}{}{}  {}({:.2}){}\n",
        BOLD,
        CYAN,
        "🧭 Project:",
        MAGENTA,
        doc.project_type.name,
        RESET,
        DIM,
        doc.project_type.confidence,
        RESET
    ));
    if !doc.project_type.evidence.is_empty() {
        out.push_str(&format!(
            "{}{}{} {}{}\n",
            BOLD,
            BLUE,
            "🔎 Evidence:",
            format_inline_list(&doc.project_type.evidence),
            RESET
        ));
    }
    out.push('\n');

    out.push_str(&section_title("📊", "Summary", CYAN));
    out.push_str(&format!(
        "  {} {} total  {} {} shown  {} {} important  {} {} ignored\n",
        paint("📦", BLUE),
        doc.summary.total_entries,
        paint("👀", GREEN),
        doc.summary.shown_entries,
        paint("🔥", RED),
        doc.summary.important_entries,
        paint("🧹", YELLOW),
        doc.summary.ignored_entries
    ));
    out.push_str(&format!(
        "  {} {} next steps  {} {} warnings\n",
        paint("🪄", GREEN),
        doc.recommended_next_steps.len(),
        paint("⚠️", YELLOW),
        doc.warnings.len()
    ));
    out.push('\n');

    out.push_str(&section_title("✨", "Top entries", MAGENTA));
    for (index, entry) in doc.entries.iter().take(20).enumerate() {
        out.push_str(&format!(
            "  {}{:>2}.{} {}  {}  {}  {}  {}\n",
            DIM,
            index + 1,
            RESET,
            priority_badge(&entry.priority),
            role_badge(&entry.role),
            entry_type_badge(&entry.entry_type),
            paint(&format_size(entry.size_bytes), BRIGHT_BLACK),
            path_badge(&entry.path)
        ));
        if entry.generated || entry.sensitive {
            out.push_str(&format!(
                "      {} {}\n",
                paint("🏷 flags:", YELLOW),
                format_flags(entry)
            ));
        }
        out.push_str(&format!(
            "      {} {}\n",
            paint("💡 why:", BLUE),
            entry.reason
        ));
    }

    if doc.entries.len() > 20 {
        out.push_str(&format!(
            "  {} ... and {} more entries{}\n",
            DIM,
            doc.entries.len() - 20,
            RESET
        ));
    }

    if !doc.recommended_next_steps.is_empty() {
        out.push_str(&format!("\n{}", section_title("🪜", "Next steps", GREEN)));
        for (index, rec) in doc.recommended_next_steps.iter().enumerate() {
            out.push_str(&format!(
                "  {}{}.{} {} {}{}\n",
                DIM,
                index + 1,
                RESET,
                action_badge(&rec.action),
                BOLD,
                rec.path
            ));
            out.push_str(&format!("     {}{}\n", DIM, rec.reason));
            out.push_str(RESET);
        }
    }

    if !doc.warnings.is_empty() {
        out.push_str(&format!("\n{}", section_title("⚠️", "Warnings", YELLOW)));
        for w in &doc.warnings {
            out.push_str(&format!(
                "  {} {}\n",
                paint("•", YELLOW),
                paint(&w.code, BOLD)
            ));
            out.push_str(&format!("    {}\n", w.message));
            if let Some(ref path) = w.path {
                out.push_str(&format!(
                    "    {} {}\n",
                    paint("📍 path:", BRIGHT_BLACK),
                    path
                ));
            }
        }
    }

    out
}

fn humanize(value: &str) -> String {
    value.replace('_', " ")
}

fn paint(text: &str, color: &str) -> String {
    format!("{color}{text}{RESET}")
}

fn section_title(icon: &str, title: &str, color: &str) -> String {
    format!("{BOLD}{color} {icon} {title}{RESET}\n")
}

fn priority_badge(priority: &str) -> String {
    let (icon, color) = match priority {
        "critical" => ("🟥", RED),
        "high" => ("🟧", YELLOW),
        "medium" => ("🟦", BLUE),
        "low" => ("🟪", MAGENTA),
        "ignore" => ("⬜", BRIGHT_BLACK),
        _ => ("▫️", BRIGHT_BLACK),
    };
    paint(&format!("{icon} {}", priority.to_uppercase()), color)
}

fn role_badge(role: &str) -> String {
    let icon = match role {
        "project_overview" => "📖",
        "manifest" => "📦",
        "source_code" => "🧠",
        "test_code" => "🧪",
        "documentation" => "📝",
        "ci_config" => "⚙️",
        "config" => "🔧",
        "lockfile" => "🔒",
        "license" => "⚖️",
        "data" => "🗂️",
        "build_output" => "🏗️",
        "dependency_cache" => "📚",
        _ => "❔",
    };
    format!("{icon} {}", humanize(role))
}

fn entry_type_badge(entry_type: &str) -> String {
    let icon = match entry_type {
        "file" => "📄",
        "directory" => "📁",
        "symlink" => "🔗",
        _ => "📦",
    };
    format!("{icon} {entry_type}")
}

fn path_badge(path: &str) -> String {
    format!("{}{}{RESET}", BOLD, path)
}

fn action_badge(action: &str) -> String {
    let icon = match action {
        "read" => "📖",
        "inspect" => "🔍",
        _ => "➡️",
    };
    paint(&format!("{icon} {action}"), GREEN)
}

fn format_inline_list(items: &[String]) -> String {
    items.join(&format!(" {}•{} ", BRIGHT_BLACK, RESET))
}

fn format_flags(entry: &EntryOutput) -> String {
    match (entry.generated, entry.sensitive) {
        (true, true) => "generated, sensitive".to_string(),
        (true, false) => "generated".to_string(),
        (false, true) => "sensitive".to_string(),
        (false, false) => String::new(),
    }
}

fn format_size(size_bytes: Option<u64>) -> String {
    match size_bytes {
        Some(bytes) => human_size(bytes),
        None => "-".to_string(),
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> OutputDocument {
        OutputDocument {
            schema_version: "0.1.0".into(),
            path: ".".into(),
            project_type: ProjectTypeOutput {
                name: "rust_cli".into(),
                confidence: 0.95,
                evidence: vec!["Cargo.toml".into(), "src/main.rs".into()],
            },
            summary: SummaryOutput {
                total_entries: 3,
                shown_entries: 3,
                important_entries: 2,
                ignored_entries: 1,
            },
            entries: vec![
                EntryOutput {
                    name: "Cargo.toml".into(),
                    path: "Cargo.toml".into(),
                    entry_type: "file".into(),
                    role: "manifest".into(),
                    priority: "critical".into(),
                    reason_code: "test".into(),
                    reason: "test".into(),
                    generated: false,
                    sensitive: false,
                    text: None,
                    binary: None,
                    size_bytes: None,
                },
                EntryOutput {
                    name: "target".into(),
                    path: "target".into(),
                    entry_type: "directory".into(),
                    role: "build_output".into(),
                    priority: "ignore".into(),
                    reason_code: "test".into(),
                    reason: "test".into(),
                    generated: true,
                    sensitive: false,
                    text: None,
                    binary: None,
                    size_bytes: None,
                },
            ],
            recommended_next_steps: vec![RecommendationOutput {
                action: "read".into(),
                path: "Cargo.toml".into(),
                reason_code: "test".into(),
                reason: "プロジェクト構成を理解するため".into(),
            }],
            warnings: vec![],
        }
    }

    #[test]
    fn test_human_output_contains_path() {
        let output = to_human_string(&sample_doc());
        assert!(output.contains("lls — ."));
        assert!(output.contains("rust_cli"));
        assert!(output.contains("Evidence:"));
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("📊 Summary"));
        assert!(output.contains("Top entries"));
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("Next steps"));
    }

    #[test]
    fn test_human_no_json() {
        let output = to_human_string(&sample_doc());
        assert!(!output.starts_with('{'));
        assert!(!output.starts_with('['));
    }

    #[test]
    fn test_human_generated_marker() {
        let output = to_human_string(&sample_doc());
        assert!(output.contains("flags:"));
        assert!(output.contains("generated"));
    }

    #[test]
    fn test_human_size_formatting() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1024), "1.0 KB");
    }

    #[test]
    fn test_priority_badge_includes_emoji() {
        let badge = priority_badge("critical");
        assert!(badge.contains("🟥"));
        assert!(badge.contains("CRITICAL"));
    }
}
