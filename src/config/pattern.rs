/// A compiled glob pattern that matches relative paths.
///
/// Supports:
/// - `*` — matches any chars except `/`
/// - `?` — matches one char except `/`
/// - `**` — matches across `/` (zero or more path elements)
/// - Leading `/` — anchors to project root
/// - Trailing `/` — matches directories only
#[derive(Debug, Clone)]
pub struct GlobPattern {
    raw: String,
    #[allow(dead_code)]
    anchored: bool, // starts with /
    dir_only: bool, // ends with /
    regex: String,  // compiled regex pattern string
}

impl GlobPattern {
    /// Compile a glob pattern string.
    ///
    /// Returns `None` if the pattern contains invalid syntax.
    pub fn compile(raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        // Normalize: convert backslash to forward slash
        let normalized = raw.replace('\\', "/");

        let anchored = normalized.starts_with('/');
        let dir_only = normalized.ends_with('/');

        // Work on a substring that excludes the leading / and trailing /
        let body = if anchored && dir_only {
            &normalized[1..normalized.len() - 1]
        } else if anchored {
            &normalized[1..]
        } else if dir_only {
            &normalized[..normalized.len() - 1]
        } else {
            &normalized
        };

        let mut regex = String::new();
        regex.push('^');

        if !anchored {
            // Not anchored, allow match anywhere in path
            regex.push_str("(?:.*/)?");
        }

        let mut literal = String::new();
        let chars: Vec<char> = body.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '\\' && i + 1 < chars.len() {
                literal.push(chars[i + 1]);
                i += 2;
                continue;
            }

            if ch == '*' {
                // Flush literal buffer
                regex.push_str(&regex_escape(&literal));
                literal.clear();
                // Check for **
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    i += 2; // consume both *
                    regex.push_str("(?:.*/)?.*");
                    continue;
                } else {
                    regex.push_str("[^/]*");
                }
            } else if ch == '?' {
                regex.push_str(&regex_escape(&literal));
                literal.clear();
                regex.push_str("[^/]");
            } else {
                literal.push(ch);
            }
            i += 1;
        }
        regex.push_str(&regex_escape(&literal));

        if dir_only {
            // Match directory itself or any path beneath it
            regex.push_str("(?:/.*)?$");
        } else {
            regex.push('$');
        }

        regex.shrink_to_fit();

        Some(Self {
            raw: raw.to_string(),
            anchored,
            dir_only,
            regex,
        })
    }

    /// Returns true if this pattern matches the given path.
    ///
    /// The path must be a `/`-separated relative path.
    pub fn matches(&self, path: &str) -> bool {
        let re = match regex::Regex::new(&self.regex) {
            Ok(re) => re,
            Err(_) => return false,
        };
        re.is_match(path)
    }

    /// Whether this pattern only matches directories.
    pub fn is_dir_only(&self) -> bool {
        self.dir_only
    }

    /// The original raw pattern string.
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

fn regex_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                result.push('\\');
                result.push(ch);
            }
            _ => result.push(ch),
        }
    }
    result
}

/// A collection of compiled glob patterns.
#[derive(Debug, Clone, Default)]
pub struct GlobSet {
    patterns: Vec<GlobPattern>,
}

impl GlobSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compiled pattern.
    pub fn add(&mut self, pattern: GlobPattern) {
        self.patterns.push(pattern);
    }

    /// Returns true if any pattern in the set matches the given path.
    pub fn is_match(&self, path: &str) -> bool {
        self.patterns.iter().any(|p| p.matches(path))
    }

    /// Returns true if the path matches a dir-only pattern.
    pub fn is_match_dir_only(&self, path: &str) -> bool {
        self.patterns
            .iter()
            .any(|p| p.is_dir_only() && p.matches(path))
    }

    pub fn patterns(&self) -> &[GlobPattern] {
        &self.patterns
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        assert!(GlobPattern::compile("").is_none());
    }

    #[test]
    fn test_star_no_slash() {
        let p = GlobPattern::compile("*.rs").unwrap();
        assert!(p.matches("main.rs"));
        assert!(p.matches("lib.rs"));
        // * doesn't cross /, but unanchored pattern matches at any depth
    }

    #[test]
    fn test_question_mark() {
        let p = GlobPattern::compile("?.txt").unwrap();
        assert!(p.matches("a.txt"));
        assert!(!p.matches("ab.txt"));
    }

    #[test]
    fn test_globstar() {
        let p = GlobPattern::compile("**/*.rs").unwrap();
        assert!(p.matches("src/main.rs"));
        assert!(p.matches("src/sub/mod.rs"));
    }

    #[test]
    fn test_leading_slash() {
        let p = GlobPattern::compile("/src/**").unwrap();
        assert!(p.matches("src/main.rs"));
        assert!(!p.matches("other/src/main.rs"));
    }

    #[test]
    fn test_trailing_slash() {
        let p = GlobPattern::compile("build/").unwrap();
        assert!(p.matches("build"));
        assert!(p.matches("build/"));
        assert!(p.matches("build/sub"));
        assert!(!p.matches("build.rs"));
    }

    #[test]
    fn test_no_anchored_match_anywhere() {
        let p = GlobPattern::compile("Cargo.toml").unwrap();
        assert!(p.matches("Cargo.toml"));
        assert!(p.matches("sub/Cargo.toml"));
    }

    #[test]
    fn test_globset() {
        let mut set = GlobSet::new();
        set.add(GlobPattern::compile("*.rs").unwrap());
        set.add(GlobPattern::compile("*.md").unwrap());
        assert!(set.is_match("main.rs"));
        assert!(set.is_match("README.md"));
        assert!(!set.is_match("Cargo.toml"));
    }
}
