use crate::model::Entry;
use std::cmp::Ordering;

/// Sort entries in canonical order:
/// 1. Priority rank (critical > high > medium > low > ignore)
/// 2. Role rank (as defined in spec)
/// 3. Normalized path byte-wise ascending
fn canonical_cmp(a: &Entry, b: &Entry) -> Ordering {
    a.priority
        .sort_rank()
        .cmp(&b.priority.sort_rank())
        .then_with(|| a.role.sort_rank().cmp(&b.role.sort_rank()))
        .then_with(|| a.path.cmp(&b.path))
}

/// Sort entries by name (normalized path ascending).
fn name_cmp(a: &Entry, b: &Entry) -> Ordering {
    a.path.cmp(&b.path)
}

/// Sort entries by mtime (descending), with tie-break on path.
fn mtime_cmp(a: &Entry, b: &Entry) -> Ordering {
    match (&a.modified_at, &b.modified_at) {
        (Some(a_mt), Some(b_mt)) => b_mt.cmp(a_mt).then_with(|| a.path.cmp(&b.path)),
        (Some(_), None) => Ordering::Less, // a has mtime, b doesn't -> a first
        (None, Some(_)) => Ordering::Greater, // b has mtime -> b first
        (None, None) => a.path.cmp(&b.path),
    }
}

/// Sort entries by size (descending), directories and unknown last.
fn size_cmp(a: &Entry, b: &Entry) -> Ordering {
    match (a.size_bytes, b.size_bytes) {
        (Some(a_sz), Some(b_sz)) => b_sz.cmp(&a_sz).then_with(|| a.path.cmp(&b.path)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.path.cmp(&b.path),
    }
}

/// Sort entries using the given comparator.
///
/// All sorts are stable (preserve original order for equal items).
pub fn sort_entries(entries: &mut [Entry], sort_by: &SortField) {
    match sort_by {
        SortField::Canonical | SortField::Priority => {
            entries.sort_by(|a, b| canonical_cmp(a, b).then_with(|| a.path.cmp(&b.path)));
        }
        SortField::Name => {
            entries.sort_by(name_cmp);
        }
        SortField::Mtime => {
            entries.sort_by(mtime_cmp);
        }
        SortField::Size => {
            entries.sort_by(size_cmp);
        }
    }
}

/// Sort field selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Canonical,
    Priority,
    Name,
    Mtime,
    Size,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EntryType, Priority, Role};
    use std::time::{Duration, SystemTime};

    fn make_entry(
        path: &str,
        priority: Priority,
        role: Role,
        size: Option<u64>,
        mtime: Option<SystemTime>,
    ) -> Entry {
        let name = path.rsplit('/').next().unwrap_or(path);
        Entry {
            name: name.into(),
            path: path.into(),
            entry_type: EntryType::File,
            role,
            priority,
            reason_code: "test".into(),
            reason: "test".into(),
            generated: false,
            sensitive: false,
            text: None,
            binary: None,
            size_bytes: size,
            modified_at: mtime,
        }
    }

    #[test]
    fn test_canonical_order_priority() {
        let mut entries = vec![
            make_entry("medium.txt", Priority::Medium, Role::Unknown, None, None),
            make_entry(
                "critical.txt",
                Priority::Critical,
                Role::Manifest,
                None,
                None,
            ),
            make_entry("high.txt", Priority::High, Role::SourceCode, None, None),
        ];

        sort_entries(&mut entries, &SortField::Canonical);
        assert_eq!(entries[0].name, "critical.txt");
        assert_eq!(entries[1].name, "high.txt");
        assert_eq!(entries[2].name, "medium.txt");
    }

    #[test]
    fn test_canonical_order_role() {
        let mut entries = vec![
            make_entry("b.txt", Priority::High, Role::SourceCode, None, None),
            make_entry("a.txt", Priority::High, Role::ProjectOverview, None, None),
        ];

        sort_entries(&mut entries, &SortField::Canonical);
        assert_eq!(entries[0].name, "a.txt"); // ProjectOverview first
        assert_eq!(entries[1].name, "b.txt");
    }

    #[test]
    fn test_canonical_order_path_tiebreak() {
        let mut entries = vec![
            make_entry("z.txt", Priority::Medium, Role::Data, None, None),
            make_entry("a.txt", Priority::Medium, Role::Data, None, None),
        ];

        sort_entries(&mut entries, &SortField::Canonical);
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[1].name, "z.txt");
    }

    #[test]
    fn test_name_sort() {
        let mut entries = vec![
            make_entry("z.txt", Priority::Low, Role::Unknown, None, None),
            make_entry("a.txt", Priority::Critical, Role::Manifest, None, None),
        ];

        sort_entries(&mut entries, &SortField::Name);
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[1].name, "z.txt");
    }

    #[test]
    fn test_mtime_sort() {
        let now = SystemTime::now();
        let old = now.checked_sub(Duration::from_secs(1000)).unwrap();
        let mut entries = vec![
            make_entry("old.txt", Priority::Low, Role::Unknown, None, Some(old)),
            make_entry("new.txt", Priority::Low, Role::Unknown, None, Some(now)),
        ];

        sort_entries(&mut entries, &SortField::Mtime);
        assert_eq!(entries[0].name, "new.txt"); // newest first
        assert_eq!(entries[1].name, "old.txt");
    }

    #[test]
    fn test_mtime_missing_last() {
        let now = SystemTime::now();
        let mut entries = vec![
            make_entry("no_mtime.txt", Priority::Low, Role::Unknown, None, None),
            make_entry(
                "has_mtime.txt",
                Priority::Low,
                Role::Unknown,
                None,
                Some(now),
            ),
        ];

        sort_entries(&mut entries, &SortField::Mtime);
        assert_eq!(entries[0].name, "has_mtime.txt");
        assert_eq!(entries[1].name, "no_mtime.txt");
    }

    #[test]
    fn test_size_sort() {
        let mut entries = vec![
            make_entry("small.txt", Priority::Low, Role::Unknown, Some(100), None),
            make_entry("large.txt", Priority::Low, Role::Unknown, Some(10000), None),
            make_entry("medium.txt", Priority::Low, Role::Unknown, Some(1000), None),
        ];

        sort_entries(&mut entries, &SortField::Size);
        assert_eq!(entries[0].name, "large.txt");
        assert_eq!(entries[1].name, "medium.txt");
        assert_eq!(entries[2].name, "small.txt");
    }

    #[test]
    fn test_size_missing_last() {
        let mut entries = vec![
            make_entry("no_size.txt", Priority::Low, Role::Unknown, None, None),
            make_entry(
                "has_size.txt",
                Priority::Low,
                Role::Unknown,
                Some(500),
                None,
            ),
        ];

        sort_entries(&mut entries, &SortField::Size);
        assert_eq!(entries[0].name, "has_size.txt");
        assert_eq!(entries[1].name, "no_size.txt");
    }

    #[test]
    fn test_stable_sort() {
        // Equal-key items preserve original order
        let mut entries = vec![
            make_entry("b.txt", Priority::Medium, Role::Data, Some(100), None),
            make_entry("a.txt", Priority::Medium, Role::Data, Some(100), None),
        ];

        sort_entries(&mut entries, &SortField::Size);
        // Both have same size, tie-break by path
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[1].name, "b.txt");
    }
}
