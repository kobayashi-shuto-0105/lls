use std::path::Path;

mod common;

/// Test Rust CLI project: project detection, entries, recommendations.
#[test]
fn test_rust_cli_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());

    // Project type should be rust_cli
    assert_eq!(json["project_type"]["name"], "rust_cli");
    assert!(json["project_type"]["confidence"].as_f64().unwrap() > 0.9);
    let evidence: Vec<&str> = json["project_type"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(evidence.contains(&"Cargo.toml"));
    assert!(evidence.contains(&"src/main.rs"));

    // Check summary
    let summary = &json["summary"];
    assert!(summary["total_entries"].as_u64().unwrap() > 0);
    assert!(summary["important_entries"].as_u64().unwrap() > 0);

    // Check specific entries exist
    let entries = json["entries"].as_array().unwrap();
    let names: Vec<&str> = entries
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Cargo.toml"));
    assert!(names.contains(&"README.md"));
    assert!(names.contains(&"src"));

    // Cargo.toml should be critical manifest
    let cargo = entries.iter().find(|e| e["name"] == "Cargo.toml").unwrap();
    assert_eq!(cargo["priority"], "critical");
    assert_eq!(cargo["role"], "manifest");

    // README should be critical project_overview
    let readme = entries.iter().find(|e| e["name"] == "README.md").unwrap();
    assert_eq!(readme["priority"], "critical");
    assert_eq!(readme["role"], "project_overview");

    // src should be high source_code
    let src = entries.iter().find(|e| e["name"] == "src").unwrap();
    assert_eq!(src["priority"], "high");
    assert_eq!(src["role"], "source_code");

    // target should be ignore
    let target = entries.iter().find(|e| e["name"] == "target").unwrap();
    assert_eq!(target["priority"], "ignore");

    // Lockfile should be generated
    let lock = entries.iter().find(|e| e["name"] == "Cargo.lock").unwrap();
    assert_eq!(lock["generated"], true);
}

/// Test Rust library project.
#[test]
fn test_rust_library_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_library(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());

    assert_eq!(json["project_type"]["name"], "rust_library");
    let evidence: Vec<&str> = json["project_type"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(evidence.contains(&"Cargo.toml"));
    assert!(evidence.contains(&"src/lib.rs"));
}

/// Test Node project.
#[test]
fn test_node_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_node(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json["project_type"]["name"], "node_project");
}

/// Test Python project.
#[test]
fn test_python_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_python(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json["project_type"]["name"], "python_package");
}

/// Test Go project.
#[test]
fn test_go_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_go(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json["project_type"]["name"], "go_module");
}

/// Test unknown project (no manifest files).
#[test]
fn test_unknown_project() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_unknown(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json["project_type"]["name"], "unknown");
}

/// Test polyglot project: should detect rust_cli (highest precedence).
#[test]
fn test_polyglot_project_rust_precedence() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_polyglot(dir.path());

    // Without src/main.rs, rust_cli isn't triggered.
    // rust_library requires Cargo.toml + src/lib.rs (neither exists)
    // So it falls through to node_project (package.json is there)
    let json = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json["project_type"]["name"], "node_project");

    // Now add src/main.rs to trigger rust_cli detection
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/main.rs"),
        "fn main() {}
",
    )
    .unwrap();

    let json2 = common::run_lls_json(&["--no-config"], dir.path());
    assert_eq!(json2["project_type"]["name"], "rust_cli");
}

/// Test sensitive entries: .env should have sensitive: true.
#[test]
fn test_sensitive_entries() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_sensitive(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    let entries = json["entries"].as_array().unwrap();

    let env = entries.iter().find(|e| e["name"] == ".env").unwrap();
    assert_eq!(env["sensitive"], true);
    assert_eq!(env["role"], "config");

    // Sensitive entries should NOT appear in recommendations
    let recs = json["recommended_next_steps"].as_array().unwrap();
    assert!(!recs.iter().any(|r| r["path"] == ".env"));
}

/// Test ignored directory pruning.
#[test]
fn test_ignored_directory_pruned() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    // Add some content inside target/ to see if it's pruned
    std::fs::create_dir_all(dir.path().join("target").join("debug")).unwrap();
    std::fs::write(
        dir.path().join("target").join("debug").join("binary"),
        "data\n",
    )
    .unwrap();

    let json = common::run_lls_json(&["--no-config"], dir.path());
    let entries = json["entries"].as_array().unwrap();

    // target itself should appear
    assert!(entries.iter().any(|e| e["name"] == "target"));

    // target/debug should NOT appear (pruned)
    assert!(!entries.iter().any(|e| e["path"] == "target/debug"));
    assert!(!entries.iter().any(|e| e["path"] == "target/debug/binary"));
}

/// Test recommendations include important entries.
#[test]
fn test_recommendations() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let json = common::run_lls_json(&["--no-config"], dir.path());
    let recs = json["recommended_next_steps"].as_array().unwrap();

    assert!(!recs.is_empty(), "should have recommendations");
    assert!(recs.len() <= 5, "max 5 recommendations");

    // README should be first recommendation (project_overview priority)
    assert_eq!(recs[0]["path"], "README.md");
}

/// Test invalid config returns error.
#[test]
fn test_invalid_config() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());
    common::fixture_invalid_config(dir.path());

    let output = common::run_lls(&[], dir.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid configuration"));
    // exit code should be 7
    assert_eq!(output.status.code(), Some(7));
}

/// Test missing config returns setup required.
#[test]
fn test_missing_config_setup_required() {
    let dir = tempfile::tempdir().unwrap();
    // Create some files but no config
    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

    let output = common::run_lls(&[], dir.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("project configuration was not found"));
    // exit code should be 5
    assert_eq!(output.status.code(), Some(5));
    // stdout should be empty
    assert!(output.stdout.is_empty());
}

/// Test depth 0 returns direct children (no recursion).
#[test]
fn test_depth_0_children() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let json = common::run_lls_json(&["--no-config", "--depth", "0"], dir.path());
    let entries = json["entries"].as_array().unwrap();
    // Depth 0: show immediate children of target (no recursion into dirs)
    assert!(!entries.is_empty());
    // src/ directory should be present but its children (src/main.rs) should NOT
    for e in entries {
        assert!(
            !e["path"].as_str().unwrap().contains('/'),
            "depth 0 should not show nested paths: {}",
            e["path"]
        );
    }
}

/// Test depth 2 shows grandchildren.
#[test]
fn test_depth_2_grandchildren() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("a").join("b")).unwrap();
    std::fs::write(dir.path().join("a").join("b").join("c.txt"), "data").unwrap();
    std::fs::write(dir.path().join("a").join("root.txt"), "data").unwrap();

    // Depth 1: shows a, a/b, a/root.txt (recurse 1 level down)
    let json1 = common::run_lls_json(&["--no-config", "--depth", "1"], dir.path());
    let entries1 = json1["entries"].as_array().unwrap();
    let paths1: Vec<&str> = entries1
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert!(paths1.contains(&"a"), "depth 1 should include a");
    assert!(
        paths1.contains(&"a/b"),
        "depth 1 should include a/b (1 level down)"
    );
    assert!(
        !paths1.contains(&"a/b/c.txt"),
        "depth 1 should NOT include a/b/c.txt"
    );

    // Depth 2: shows everything including a/b/c.txt
    let json2 = common::run_lls_json(&["--no-config", "--depth", "2"], dir.path());
    let entries2 = json2["entries"].as_array().unwrap();
    let paths2: Vec<&str> = entries2
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert!(paths2.contains(&"a"));
    assert!(paths2.contains(&"a/b"));
    assert!(paths2.contains(&"a/b/c.txt"));
}

/// Test JSON output format: compact, trailing newline, no null.
#[test]
fn test_json_output_format() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let output = common::run_lls(&["--json", "--no-config"], dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should end with exactly one newline
    assert!(stdout.ends_with('\n'));
    let newline_count = stdout.matches('\n').count();
    assert_eq!(newline_count, 1, "should have exactly one trailing newline");

    // Should be compact (no pretty-print newlines in middle)
    let trimmed = stdout.trim();
    assert!(
        !trimmed.contains('\n'),
        "compact JSON should not have internal newlines"
    );

    // Should not contain null values
    assert!(!stdout.contains(":null"), "should not contain null");
}

/// Test target not found.
#[test]
fn test_target_not_found() {
    let dir = tempfile::tempdir().unwrap();

    // Pass the nonexistent path as an argument
    let output = common::run_lls(&["--no-config", "nonexistent"], dir.path());
    assert!(!output.status.success());
    // Should be exit code 2
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("target not found"));
}

/// Test sorting by name.
#[test]
fn test_sort_by_name() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let json = common::run_lls_json(&["--no-config", "--sort", "name"], dir.path());
    let entries = json["entries"].as_array().unwrap();
    let paths: Vec<&str> = entries
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();

    // Paths should be sorted alphabetically (name sort = path sort)
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "should be sorted by path");
}

/// Test --human output works.
#[test]
fn test_human_output() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let output = common::run_lls(&["--human", "--no-config"], dir.path());
    assert!(output.status.success(), "human output should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lls —"));
    assert!(stdout.contains("Cargo.toml"));
}

/// Test -l long listing output works.
#[test]
fn test_long_output() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let output = common::run_lls(&["-l", "--no-config"], dir.path());
    assert!(output.status.success(), "long output should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lls -l"));
    assert!(stdout.contains("critical"));
    assert!(stdout.contains("manifest"));
}

/// Test with a custom config file (--config flag).
#[test]
fn test_custom_config_flag() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    // Create a custom config with depth 0 (direct children only)
    let config = r#"{
        "schema_version": "0.1.0",
        "default_output": "json",
        "scan": { "depth": 0, "include_hidden": true, "include_ignored": false },
        "long_listing": { "sort": "priority" },
        "rules": {
            "priority_overrides": [],
            "role_overrides": [],
            "ignore_patterns": [],
            "sensitive_patterns": []
        },
        "codex": { "enabled": true, "auth_method": "chatgpt", "use_for_setup": true }
    }"#;
    let config_path = dir.path().join("myconfig.json");
    std::fs::write(&config_path, config).unwrap();

    // Run with --config
    let json = common::run_lls_json(&["--config", "myconfig.json"], dir.path());
    // Depth 0 shows immediate children (no recursion)
    let entries = json["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    // No entry should have a '/' in path (no nested paths at depth 0)
    for e in entries {
        assert!(
            !e["path"].as_str().unwrap().contains('/'),
            "depth 0 should not show nested paths: {}",
            e["path"]
        );
    }
}

/// Test deterministic output: same state gives same JSON.
#[test]
fn test_deterministic_output() {
    let dir = tempfile::tempdir().unwrap();
    common::fixture_rust_cli(dir.path());

    let json1 = common::run_lls_json(&["--no-config"], dir.path());
    let json2 = common::run_lls_json(&["--no-config"], dir.path());

    assert_eq!(json1, json2, "output should be deterministic");
}

/// Test that using --config and --no-config together is an error.
#[test]
fn test_config_and_no_config_conflict() {
    let output = common::run_lls(
        &["--config", "foo.json", "--no-config", "."],
        Path::new("."),
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

/// Test --json and --human together is an error.
#[test]
fn test_json_and_human_conflict() {
    let output = common::run_lls(&["--json", "--human", "."], Path::new("."));
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}
