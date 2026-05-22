use crate::scanner::{EntryMeta, EntryType};

/// Classified entry with role, priority, and other semantic attributes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassifiedEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub role: String,
    pub priority: String,
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

/// Classify a single entry: determine role, generated, sensitive, text/binary
pub fn classify(meta: &EntryMeta) -> ClassifiedEntry {
    let (role, reason) = classify_role(meta);
    let generated = is_generated(&role, meta);
    let sensitive = is_sensitive(&role, meta);

    ClassifiedEntry {
        name: meta.name.clone(),
        path: meta.path.clone(),
        entry_type: meta.entry_type.as_str().to_string(),
        role,
        priority: String::new(), // will be filled by priority module
        reason,
        generated,
        sensitive,
        text: if meta.entry_type == EntryType::File {
            Some(meta.is_text)
        } else {
            None
        },
        binary: if meta.entry_type == EntryType::File {
            Some(meta.is_binary)
        } else {
            None
        },
        size_bytes: meta.size_bytes,
    }
}

fn classify_role(meta: &EntryMeta) -> (String, String) {
    let name_lower = meta.name.to_lowercase();
    let path_lower = meta.path.to_lowercase();

    // Directories
    if meta.entry_type == EntryType::Directory {
        return match path_lower.as_str() {
            "src" | "src/" => ("source_code".into(), "メインのソースコードディレクトリ".into()),
            "app" | "app/" => ("source_code".into(), "アプリケーションコードディレクトリ".into()),
            "lib" | "lib/" => ("source_code".into(), "ライブラリコードディレクトリ".into()),
            "packages" | "packages/" => ("source_code".into(), "パッケージディレクトリ".into()),
            "tests" | "tests/" => ("test_code".into(), "テストコードを含むディレクトリ".into()),
            "__tests__" | "__tests__/" => ("test_code".into(), "テストコードを含むディレクトリ".into()),
            "test" | "test/" => ("test_code".into(), "テストコードを含むディレクトリ".into()),
            "docs" | "docs/" => ("documentation".into(), "ドキュメントディレクトリ".into()),
            "examples" | "examples/" => ("documentation".into(), "サンプルコードディレクトリ".into()),
            _ => {
                // Check for CI/CD config dirs
                if path_lower.starts_with(".github/")
                    || path_lower == ".github"
                    || path_lower.starts_with(".gitlab/")
                {
                    return ("ci_config".into(), "CI/CD 設定ディレクトリ".into());
                }
                if path_lower.starts_with("node_modules") || path_lower == "node_modules" {
                    return ("dependency_cache".into(), "npm の依存パッケージディレクトリ".into());
                }
                if path_lower.starts_with("target") || path_lower == "target" {
                    return ("build_output".into(), "Rust のビルド成果物ディレクトリ".into());
                }
                if path_lower.starts_with("dist") || path_lower == "dist" {
                    return ("build_output".into(), "ビルド成果物ディレクトリ".into());
                }
                if path_lower.starts_with("build") || path_lower == "build" {
                    return ("build_output".into(), "ビルド成果物ディレクトリ".into());
                }
                if path_lower.starts_with("coverage") || path_lower == "coverage" {
                    return ("build_output".into(), "カバレッジレポートディレクトリ".into());
                }
                if path_lower.starts_with(".next") || path_lower == ".next" {
                    return ("build_output".into(), "Next.js のビルド成果物ディレクトリ".into());
                }
                if path_lower.starts_with(".turbo") || path_lower == ".turbo" {
                    return ("build_output".into(), "Turborepo のキャッシュディレクトリ".into());
                }
                if path_lower.starts_with("vendor") || path_lower == "vendor" {
                    return ("dependency_cache".into(), "ベンダーディレクトリ".into());
                }
                if path_lower.starts_with(".git") && path_lower != ".github" {
                    return ("dependency_cache".into(), "Git の内部ディレクトリ".into());
                }
                if path_lower.starts_with("__pycache__") || path_lower == "__pycache__" {
                    return ("build_output".into(), "Python のキャッシュディレクトリ".into());
                }
                if path_lower.starts_with(".cache") || path_lower == ".cache" {
                    return ("dependency_cache".into(), "キャッシュディレクトリ".into());
                }

                ("unknown".into(), "役割が特定できないディレクトリ".to_string())
            }
        };
    }

    // Files
    let ext = std::path::Path::new(&meta.name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // Check by exact filename
    match name_lower.as_str() {
        "readme.md" | "readme" | "readme.txt" => {
            return ("project_overview".into(), "プロジェクト概要や使い方が書かれている可能性が高い".into());
        }
        "cargo.toml" => {
            return ("manifest".into(), "Rust のマニフェストファイル。パッケージ情報や依存関係の把握に重要".into());
        }
        "package.json" => {
            return ("manifest".into(), "Node.js パッケージのマニフェストファイル".into());
        }
        "pyproject.toml" => {
            return ("manifest".into(), "Python プロジェクトのマニフェストファイル".into());
        }
        "setup.py" => {
            return ("manifest".into(), "Python パッケージのセットアップスクリプト".into());
        }
        "go.mod" => {
            return ("manifest".into(), "Go モジュールのマニフェストファイル".into());
        }
        "cargo.lock" => {
            return ("lockfile".into(), "Rust の依存関係固定ファイル".into());
        }
        "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "pnpm-lock.yml" => {
            return ("lockfile".into(), "依存関係の固定ファイル".into());
        }
        "license" | "license.md" | "license.txt" | "licence" => {
            return ("license".into(), "ライセンスファイル".into());
        }
        ".gitignore" => {
            return ("config".into(), "Git の除外設定ファイル".into());
        }
        ".env" | ".env.local" | ".env.production" | ".env.development" => {
            return ("secret_candidate".into(), "環境変数や秘密情報を含む可能性がある".into());
        }
        "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => {
            return ("config".into(), "Docker 設定ファイル".into());
        }
        ".github" => {
            // Actually a directory, already handled above
            return ("ci_config".into(), "GitHub 設定ディレクトリ".into());
        }
        _ => {}
    }

    // Check by extension
    if let Some(ext) = ext {
        match ext.as_str() {
            "rs" | "c" | "cpp" | "h" | "hpp" | "cs" => {
                if name_lower == "main.rs" || name_lower == "lib.rs" || name_lower == "mod.rs" {
                    return ("source_code".into(), "Rust のエントリポイントファイル".into());
                }
                if ext.as_str() == "rs" {
                    return ("source_code".into(), "Rust ソースファイル".into());
                }
                return ("source_code".into(), "ソースファイル".into());
            }
            "ts" | "tsx" => {
                return ("source_code".into(), "TypeScript ソースファイル".into());
            }
            "js" | "jsx" => {
                return ("source_code".into(), "JavaScript ソースファイル".into());
            }
            "py" => {
                return ("source_code".into(), "Python ソースファイル".into());
            }
            "go" => {
                return ("source_code".into(), "Go ソースファイル".into());
            }
            "java" => {
                return ("source_code".into(), "Java ソースファイル".into());
            }
            "rb" => {
                return ("source_code".into(), "Ruby ソースファイル".into());
            }
            "md" | "markdown" | "rst" | "txt" => {
                return ("documentation".into(), "ドキュメントファイル".to_string());
            }
            "yml" | "yaml" => {
                if path_lower.contains(".github/workflows")
                    || path_lower.contains(".gitlab-ci")
                {
                    return ("ci_config".into(), "CI/CD 設定ファイル".to_string());
                }
                return ("config".into(), "YAML 設定ファイル".to_string());
            }
            "json" => {
                if name_lower == "tsconfig.json"
                    || name_lower == "eslintrc.json"
                    || name_lower == ".prettierrc"
                {
                    return ("config".into(), "設定ファイル".to_string());
                }
                return ("config".into(), "JSON 設定ファイル".to_string());
            }
            "toml" => {
                return ("config".into(), "TOML 設定ファイル".to_string());
            }
            "cfg" | "conf" | "ini" => {
                return ("config".into(), "設定ファイル".to_string());
            }
            "pem" | "key" => {
                return ("secret_candidate".into(), "秘密鍵や証明書の可能性がある".into());
            }
            "lock" => {
                return ("lockfile".into(), "依存関係の固定ファイル".into());
            }
            "css" | "scss" | "less" => {
                return ("source_code".into(), "スタイルシートファイル".to_string());
            }
            "html" | "htm" | "vue" | "svelte" => {
                return ("source_code".into(), "テンプレート/マークアップファイル".to_string());
            }
            "svg" => {
                return ("generated".into(), "SVG 画像ファイル".to_string());
            }
            "min.js" | "min.css" => {
                return ("generated".into(), "Minified ファイル。内容の読み取りには向かない".into());
            }
            _ => {}
        }
    }

    // Fallback for specific well-known filenames
    if name_lower.starts_with("id_rsa") || name_lower.starts_with("id_ed25519") || name_lower == "id_ecdsa" {
        return ("secret_candidate".into(), "SSH 秘密鍵の可能性がある".into());
    }

    if name_lower == ".gitattributes" || name_lower == ".editorconfig" || name_lower == ".prettierrc" || name_lower == "eslint.config.js" {
        return ("config".into(), "プロジェクト設定ファイル".into());
    }

    if name_lower == "changelog.md" || name_lower == "changelog" {
        return ("documentation".into(), "変更履歴ファイル".into());
    }

    if name_lower.ends_with(".test.rs") || name_lower.ends_with("_test.rs") || name_lower.ends_with("_spec.rs")
        || name_lower.ends_with(".test.ts") || name_lower.ends_with("_test.ts") || name_lower.ends_with(".spec.ts")
        || name_lower.ends_with(".test.js") || name_lower.ends_with("_test.js") || name_lower.ends_with(".spec.js")
        || name_lower.ends_with("_test.py") || name_lower.ends_with("test_.py")
    {
        return ("test_code".into(), "テストファイル".to_string());
    }

    // If binary, mark as generated
    if meta.is_binary && !meta.is_text {
        return ("generated".into(), "バイナリファイル。内容の読み取りには向かない".into());
    }

    ("unknown".into(), "役割が特定できなかったファイル".into())
}

fn is_generated(role: &str, meta: &EntryMeta) -> bool {
    match role {
        "build_output" | "dependency_cache" | "lockfile" | "generated" => true,
        _ => {
            // Specific generated patterns
            let name_lower = meta.name.to_lowercase();
            name_lower.ends_with(".min.js") || name_lower.ends_with(".min.css")
        }
    }
}

fn is_sensitive(role: &str, _meta: &EntryMeta) -> bool {
    role == "secret_candidate"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{EntryMeta, EntryType};

    fn make_meta(name: &str, path: &str, entry_type: EntryType, is_text: bool, is_binary: bool) -> EntryMeta {
        EntryMeta {
            name: name.to_string(),
            path: path.to_string(),
            entry_type,
            size_bytes: Some(100),
            is_text,
            is_binary,
            is_broken_symlink: false,
        }
    }

    #[test]
    fn test_readme_is_project_overview() {
        let meta = make_meta("README.md", "README.md", EntryType::File, true, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "project_overview");
    }

    #[test]
    fn test_cargo_toml_is_manifest() {
        let meta = make_meta("Cargo.toml", "Cargo.toml", EntryType::File, true, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "manifest");
    }

    #[test]
    fn test_env_is_sensitive() {
        let meta = make_meta(".env", ".env", EntryType::File, true, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "secret_candidate");
        assert!(entry.sensitive);
    }

    #[test]
    fn test_src_dir_is_source_code() {
        let meta = make_meta("src", "src", EntryType::Directory, false, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "source_code");
    }

    #[test]
    fn test_node_modules_is_dependency_cache() {
        let meta = make_meta("node_modules", "node_modules", EntryType::Directory, false, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "dependency_cache");
    }

    #[test]
    fn test_target_is_build_output() {
        let meta = make_meta("target", "target", EntryType::Directory, false, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "build_output");
    }

    #[test]
    fn test_main_rs_is_source_code() {
        let meta = make_meta("main.rs", "main.rs", EntryType::File, true, false);
        let entry = classify(&meta);
        assert_eq!(entry.role, "source_code");
    }

    #[test]
    fn test_svg_is_generated() {
        let meta = make_meta("logo.svg", "logo.svg", EntryType::File, false, true);
        let entry = classify(&meta);
        assert_eq!(entry.role, "generated");
    }
}
