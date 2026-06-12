use lls::app::{self, CommandRequest, OutputMode};
use lls::classifier::{classify_attributes, classify_role};
use lls::cli::CliArgs;
use lls::config::{ConfigDiscovery, validate_config};
use lls::error::AppError;
use lls::model::*;
use lls::output;
use lls::priority::assign_priority;
use lls::project_probe::probe_project;
use lls::recommendation::generate_recommendations;
use lls::scanner::Scanner;
use lls::setup;
use lls::sorting::{SortField, sort_entries};
use std::path::Path;
use std::process;
fn main() {
    let args = match try_parse_cli() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    match run(args) {
        Ok(()) => process::exit(0),
        Err(err) => {
            match &err {
                AppError::SetupRequired => {
                    eprintln!("lls: project configuration was not found");
                    eprintln!(
                        "Run `lls setup` to create .lls/config.json, or use `--no-config` to run with built-in defaults."
                    );
                }
                AppError::InvalidConfig(msg) => {
                    eprintln!("lls: invalid configuration: {msg}");
                }
                AppError::Cli(msg) => {
                    eprintln!("lls: {msg}");
                }
                AppError::TargetNotFound { path } => {
                    eprintln!("lls: target not found: {}", path.display());
                }
                AppError::PermissionDenied { path } => {
                    eprintln!("lls: permission denied: {}", path.display());
                }
                AppError::Codex(msg) => {
                    eprintln!("lls: Codex setup failed: {msg}");
                }
                AppError::Runtime(msg) => {
                    eprintln!("lls: unexpected error: {msg}");
                }
            }
            process::exit(err.exit_code());
        }
    }
}
fn run(args: CliArgs) -> Result<(), AppError> {
    let request = app::parse_request(args)?;
    match request {
        CommandRequest::List(list_req) => run_list(list_req),
        CommandRequest::Setup(setup_req) => run_setup(setup_req),
    }
}
fn run_list(req: app::ListRequest) -> Result<(), AppError> {
    let target_path = Path::new(&req.path);
    // Resolve target
    if !target_path.exists() {
        return Err(AppError::TargetNotFound {
            path: target_path.to_path_buf(),
        });
    }
    // Resolve project root
    let project_root = resolve_project_root(target_path)?;
    // Discover config
    let (config, output_mode) = match &req.config_source {
        app::ConfigSource::Explicit(config_path) => {
            let path = Path::new(config_path);
            let content = std::fs::read_to_string(path)
                .map_err(|e| AppError::InvalidConfig(format!("cannot read config: {e}")))?;
            let validated = validate_config(&content)?;
            (Some(validated.config), req.output_mode)
        }
        app::ConfigSource::Auto => {
            match ConfigDiscovery::resolve(None, &project_root, false) {
                Ok(Some(config_path)) => {
                    let content = std::fs::read_to_string(&config_path)
                        .map_err(|e| AppError::InvalidConfig(format!("cannot read config: {e}")))?;
                    let validated = validate_config(&content)?;
                    (Some(validated.config), req.output_mode)
                }
                Ok(None) => {
                    // --no-config case (should not happen via Auto, but handle gracefully)
                    (None, req.output_mode)
                }
                Err(_) => {
                    return Err(AppError::SetupRequired);
                }
            }
        }
        app::ConfigSource::None => {
            // --no-config: use built-in defaults
            (None, req.output_mode)
        }
    };
    // Determine effective settings
    let depth = config.as_ref().map(|c| c.scan.depth).unwrap_or(req.depth);
    let include_ignored = config
        .as_ref()
        .map(|c| c.scan.include_ignored)
        .unwrap_or(false);
    let rules =
        config
            .as_ref()
            .map(|c| c.rules.clone())
            .unwrap_or(lls::config::model::RulesConfig {
                priority_overrides: vec![],
                role_overrides: vec![],
                ignore_patterns: vec![],
                sensitive_patterns: vec![],
            });
    // Scan
    let scanner = Scanner::new(&rules.ignore_patterns, include_ignored);
    let scan_result = scanner.scan(target_path, depth);
    // Classify each entry
    let mut entries: Vec<Entry> = scan_result
        .entries
        .into_iter()
        .map(|raw| {
            let path_str = raw.relative_path.to_string_lossy().replace('\\', "/");
            let attrs = classify_attributes(&raw.name, &path_str, raw.entry_type, &rules);
            let role = classify_role(&raw.name, &path_str, raw.entry_type, &rules);
            let _reason_code = if attrs.sensitive {
                "sensitive_name_pattern".to_string()
            } else if attrs.generated {
                "generated_name_pattern".to_string()
            } else {
                format!("role_{}", role.as_str())
            };
            let reason = match role {
                Role::ProjectOverview => "プロジェクト概要".into(),
                Role::Manifest => "マニフェストファイル".into(),
                Role::SourceCode => "ソースコード".into(),
                Role::TestCode => "テストコード".into(),
                Role::Documentation => "ドキュメント".into(),
                Role::CiConfig => "CI/CD設定".into(),
                Role::Config => "設定ファイル".into(),
                Role::Lockfile => "ロックファイル".into(),
                Role::License => "ライセンス".into(),
                Role::Data => "データファイル".into(),
                Role::BuildOutput => "ビルド出力".into(),
                Role::DependencyCache => "依存キャッシュ".into(),
                Role::Unknown => "分類不明".into(),
            };
            let priority_result = assign_priority(
                &raw.name,
                &path_str,
                raw.entry_type,
                role,
                attrs.generated,
                attrs.sensitive,
                &rules,
            );
            let text = attrs.text;
            let binary = attrs.binary;
            Entry {
                name: raw.name,
                path: path_str,
                entry_type: raw.entry_type,
                role,
                priority: priority_result.priority,
                reason_code: priority_result.reason_code,
                reason,
                generated: attrs.generated,
                sensitive: attrs.sensitive,
                text,
                binary,
                size_bytes: raw.size_bytes,
                modified_at: raw.modified_at,
            }
        })
        .collect();
    // Sort entries
    let sort_field = match req.sort {
        Some(lls::cli::SortBy::Name) => SortField::Name,
        Some(lls::cli::SortBy::Mtime) => SortField::Mtime,
        Some(lls::cli::SortBy::Size) => SortField::Size,
        Some(lls::cli::SortBy::Priority) | None => SortField::Canonical,
    };
    sort_entries(&mut entries, &sort_field);
    // Project probe
    let probe_result = probe_project(&project_root);
    // Recommendations
    let recommendations = generate_recommendations(&entries);
    // Build summary
    let total_entries = entries.len();
    let shown_entries = entries.len();
    let important_entries = entries
        .iter()
        .filter(|e| matches!(e.priority, Priority::Critical | Priority::High))
        .count();
    let ignored_entries = entries
        .iter()
        .filter(|e| matches!(e.priority, Priority::Ignore))
        .count();
    // Build output entries
    let output_entries: Vec<EntryOutput> = entries
        .iter()
        .map(|e| EntryOutput {
            name: e.name.clone(),
            path: e.path.clone(),
            entry_type: e.entry_type.as_str().to_string(),
            role: e.role.as_str().to_string(),
            priority: e.priority.as_str().to_string(),
            reason_code: e.reason_code.clone(),
            reason: e.reason.clone(),
            generated: e.generated,
            sensitive: e.sensitive,
            text: e.text,
            binary: e.binary,
            size_bytes: e.size_bytes,
        })
        .collect();
    // Build warnings from scan + probe
    let mut warnings: Vec<WarningOutput> = scan_result
        .warnings
        .into_iter()
        .map(|w| WarningOutput {
            code: w.code,
            path: w.path,
            message: w.message,
        })
        .collect();
    for w in probe_result.warnings {
        warnings.push(WarningOutput {
            code: w.code,
            path: w.path,
            message: w.message,
        });
    }
    // Sensitive candidate warning
    for entry in &output_entries {
        if entry.sensitive {
            warnings.push(WarningOutput {
                code: "sensitive_candidate_detected".into(),
                path: Some(entry.path.clone()),
                message: "秘密情報候補を検出した".into(),
            });
        }
    }
    let path_str = req.path;
    let doc = OutputDocument {
        schema_version: "0.1.0".into(),
        path: path_str.clone(),
        project_type: ProjectTypeOutput {
            name: probe_result.project_type.name,
            confidence: probe_result.project_type.confidence,
            evidence: probe_result.project_type.evidence,
        },
        summary: SummaryOutput {
            total_entries,
            shown_entries,
            important_entries,
            ignored_entries,
        },
        entries: output_entries,
        recommended_next_steps: recommendations
            .into_iter()
            .map(|r| RecommendationOutput {
                action: r.action,
                path: r.path,
                reason_code: r.reason_code,
                reason: r.reason,
            })
            .collect(),
        warnings,
    };
    // Output
    match output_mode {
        OutputMode::Json => {
            let json = output::to_json_string(&doc);
            print!("{json}");
        }
        OutputMode::Human => {
            let text = output::to_human_string(&doc);
            print!("{text}");
        }
        OutputMode::Long => {
            let text = output::to_long_string(&doc);
            print!("{text}");
        }
    }
    Ok(())
}
fn run_setup(req: app::SetupRequest) -> Result<(), AppError> {
    let target_path = Path::new(&req.path);
    let project_root = resolve_project_root(target_path)?;
    // Check existing config
    let config_path = project_root.join(".lls").join("config.json");
    if config_path.exists() && !req.force {
        return Err(AppError::Cli(
            "config already exists at .lls/config.json, use --force to overwrite".into(),
        ));
    }
    // Generate proposal
    let proposal = setup::generate_proposal();
    // Validate proposal
    let proposal_json = serde_json::to_string_pretty(&proposal)
        .map_err(|e| AppError::Runtime(format!("serialization error: {e}")))?;
    let validated = validate_config(&proposal_json)?;
    setup::safety_check(&validated.config)?;
    // Confirm with user (skip if --yes)
    if !req.yes {
        eprintln!("lls: config proposal for {}:\n", project_root.display());
        eprintln!("{proposal_json}\n");
        eprint!("Write this config to .lls/config.json? [y/N] ");
        use std::io::Write;
        std::io::stderr().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            eprintln!("lls: setup cancelled");
            return Ok(());
        }
    }
    // Write
    setup::write_config(&project_root, &proposal_json, req.force)?;
    eprintln!("lls: config written to .lls/config.json");
    Ok(())
}
/// Resolve the project root by looking for `.git` in ancestor directories.
fn resolve_project_root(target: &Path) -> Result<std::path::PathBuf, AppError> {
    let canonical = target
        .canonicalize()
        .map_err(|_| AppError::TargetNotFound {
            path: target.to_path_buf(),
        })?;
    // If target is a file, start from its parent
    let start = if canonical.is_dir() {
        &canonical
    } else {
        canonical.parent().unwrap_or(&canonical)
    };
    // Walk up looking for .git
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }
    // No .git found: use target itself (or parent of file)
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Ok(canonical.parent().unwrap_or(&canonical).to_path_buf())
    }
}
fn try_parse_cli() -> Result<CliArgs, String> {
    use clap::Parser;
    match CliArgs::try_parse_from(std::env::args()) {
        Ok(args) => Ok(args),
        Err(e) => {
            // For help/version, print the message and exit with 0
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                print!("{e}");
                std::process::exit(0);
            }
            // For other errors, propagate as string
            Err(e.to_string())
        }
    }
}
