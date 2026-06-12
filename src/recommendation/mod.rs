use crate::model::{Entry, Priority, Recommendation, Role};

/// Generate recommended next steps from the classified entries.
///
/// Rules (spec section 12):
/// 1. Critical priority first
/// 2. Project overview is highest priority
/// 3. Manifest before source
/// 4. Ignore entries excluded
/// 5. Sensitive entries excluded
/// 6. Binary entries excluded
/// 7. Symlinks excluded
/// 8. Maximum 5 entries
pub fn generate_recommendations(entries: &[Entry]) -> Vec<Recommendation> {
    let mut eligible: Vec<&Entry> = entries
        .iter()
        .filter(|e| {
            // Exclude ignore, sensitive, binary, and symlinks
            e.priority != Priority::Ignore
                && !e.sensitive
                && e.binary != Some(true)
                && e.entry_type != crate::model::EntryType::Symlink
        })
        .collect();

    // Sort by priority (critical first), then role (project overview first, then manifest)
    eligible.sort_by(|a, b| {
        a.priority
            .sort_rank()
            .cmp(&b.priority.sort_rank())
            .then_with(|| a.role.sort_rank().cmp(&b.role.sort_rank()))
            .then_with(|| a.path.cmp(&b.path))
    });

    let mut recommendations = Vec::new();
    for entry in eligible.iter().take(5) {
        let action = match entry.role {
            Role::ProjectOverview => "read",
            Role::Manifest => "read",
            _ => "inspect",
        };

        let reason = match entry.role {
            Role::ProjectOverview => "プロジェクト概要を把握するため",
            Role::Manifest => "プロジェクト構成を理解するため",
            Role::SourceCode => "ソースコードの構造を確認するため",
            Role::CiConfig => "CI/CD設定を確認するため",
            Role::Config => "設定内容を確認するため",
            Role::Documentation => "ドキュメントを参照するため",
            _ => "内容を確認するため",
        };

        recommendations.push(Recommendation {
            action: action.to_string(),
            path: entry.path.clone(),
            reason_code: format!("role_{}_first", entry.role.as_str()),
            reason: reason.to_string(),
        });
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Entry, EntryType, Priority, Role};

    fn make_entry(
        name: &str,
        path: &str,
        role: Role,
        priority: Priority,
        sensitive: bool,
        binary: Option<bool>,
        entry_type: EntryType,
    ) -> Entry {
        Entry {
            name: name.into(),
            path: path.into(),
            entry_type,
            role,
            priority,
            reason_code: "test".into(),
            reason: "test".into(),
            generated: false,
            sensitive,
            text: None,
            binary,
            size_bytes: None,
            modified_at: None,
        }
    }

    #[test]
    fn test_basic_recommendations() {
        let entries = vec![
            make_entry(
                "README.md",
                "README.md",
                Role::ProjectOverview,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
            make_entry(
                "Cargo.toml",
                "Cargo.toml",
                Role::Manifest,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
            make_entry(
                "src",
                "src",
                Role::SourceCode,
                Priority::High,
                false,
                None,
                EntryType::Directory,
            ),
        ];

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].path, "README.md");
        assert_eq!(recs[0].action, "read");
        assert_eq!(recs[1].path, "Cargo.toml");
        assert_eq!(recs[1].action, "read");
    }

    #[test]
    fn test_sensitive_excluded() {
        let entries = vec![
            make_entry(
                "README.md",
                "README.md",
                Role::ProjectOverview,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
            make_entry(
                ".env",
                ".env",
                Role::Config,
                Priority::Medium,
                true,
                None,
                EntryType::File,
            ),
        ];

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].path, "README.md");
    }

    #[test]
    fn test_binary_excluded() {
        let entries = vec![
            make_entry(
                "image.png",
                "image.png",
                Role::Unknown,
                Priority::Low,
                false,
                Some(true),
                EntryType::File,
            ),
            make_entry(
                "README.md",
                "README.md",
                Role::ProjectOverview,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
        ];

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn test_symlink_excluded() {
        let entries = vec![
            make_entry(
                "link",
                "link",
                Role::Unknown,
                Priority::Medium,
                false,
                None,
                EntryType::Symlink,
            ),
            make_entry(
                "README.md",
                "README.md",
                Role::ProjectOverview,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
        ];

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn test_ignore_excluded() {
        let entries = vec![
            make_entry(
                "target",
                "target",
                Role::BuildOutput,
                Priority::Ignore,
                false,
                None,
                EntryType::Directory,
            ),
            make_entry(
                "README.md",
                "README.md",
                Role::ProjectOverview,
                Priority::Critical,
                false,
                None,
                EntryType::File,
            ),
        ];

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn test_max_five() {
        let mut entries = Vec::new();
        for i in 0..10 {
            entries.push(make_entry(
                &format!("file{i}.rs"),
                &format!("file{i}.rs"),
                Role::SourceCode,
                Priority::High,
                false,
                None,
                EntryType::File,
            ));
        }

        let recs = generate_recommendations(&entries);
        assert_eq!(recs.len(), 5);
    }

    #[test]
    fn test_deterministic_order() {
        let entries = vec![
            make_entry(
                "z.txt",
                "z.txt",
                Role::Data,
                Priority::Low,
                false,
                None,
                EntryType::File,
            ),
            make_entry(
                "a.txt",
                "a.txt",
                Role::Data,
                Priority::Low,
                false,
                None,
                EntryType::File,
            ),
            make_entry(
                "m.txt",
                "m.txt",
                Role::Data,
                Priority::Low,
                false,
                None,
                EntryType::File,
            ),
        ];

        let recs1 = generate_recommendations(&entries);
        let recs2 = generate_recommendations(&entries);
        let paths1: Vec<_> = recs1.iter().map(|r| r.path.clone()).collect();
        let paths2: Vec<_> = recs2.iter().map(|r| r.path.clone()).collect();
        assert_eq!(paths1, paths2);
    }
}
