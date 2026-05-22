use std::process::Command;

fn run_lls(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_lls"))
        .args(args)
        .output()
        .expect("Failed to run lls");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

fn fixture_path(name: &str) -> String {
    format!("tests/fixtures/{name}")
}

#[test]
fn test_rust_cli_project_human_output() {
    let path = fixture_path("rust_cli");
    let (stdout, stderr, code) = run_lls(&["--depth", "2", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("rust_cli"), "Should detect rust_cli project type");
    assert!(stdout.contains("README.md"), "Should list README.md");
    assert!(stdout.contains("Cargo.toml"), "Should list Cargo.toml");
    assert!(stdout.contains("src"), "Should list src directory");
    assert!(stdout.contains("tests"), "Should list tests directory");
    assert!(stdout.contains("target"), "Should list target directory");
    assert!(stdout.contains("critical"), "Should have critical entries");
    assert!(stdout.contains("ignore"), "Should have ignore entries");
}

#[test]
fn test_rust_cli_project_json_output() {
    let path = fixture_path("rust_cli");
    let (stdout, stderr, code) = run_lls(&["--json", "--depth", "2", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");

    // Parse JSON
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert_eq!(v["project_type"]["name"], "rust_cli");
    assert!(v["project_type"]["confidence"].as_f64().unwrap_or(0.0) > 0.0);
    assert!(v["entries"].as_array().unwrap().len() > 0);
    assert!(v["recommended_next_steps"].as_array().unwrap().len() > 0);
}

#[test]
fn test_node_project_output() {
    let path = fixture_path("node_project");
    let (stdout, stderr, code) = run_lls(&[&path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("node_project"), "Should detect node_project");
    assert!(stdout.contains("node_modules"), "Should list node_modules");
    assert!(stdout.contains("dist"), "Should list dist");
}

#[test]
fn test_node_project_json() {
    let path = fixture_path("node_project");
    let (stdout, stderr, code) = run_lls(&["--json", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert_eq!(v["project_type"]["name"], "node_project");

    // Check node_modules and dist are ignore
    let entries = v["entries"].as_array().unwrap();
    let node_modules = entries.iter().find(|e| e["name"] == "node_modules").unwrap();
    assert_eq!(node_modules["priority"], "ignore");
    let dist = entries.iter().find(|e| e["name"] == "dist").unwrap();
    assert_eq!(dist["priority"], "ignore");

    // Recommended should not include node_modules or dist
    let steps = v["recommended_next_steps"].as_array().unwrap();
    let step_paths: Vec<&str> = steps.iter().map(|s| s["path"].as_str().unwrap()).collect();
    assert!(!step_paths.contains(&"node_modules"), "node_modules should not be recommended");
    assert!(!step_paths.contains(&"dist"), "dist should not be recommended");
}

#[test]
fn test_secret_candidate_file() {
    let path = fixture_path("secret_candidate");
    let (stdout, stderr, code) = run_lls(&["--json", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    let entries = v["entries"].as_array().unwrap();
    let env_entry = entries.iter().find(|e| e["name"] == ".env").unwrap();
    assert_eq!(env_entry["sensitive"], true, ".env should be sensitive");
    assert_eq!(env_entry["role"], "secret_candidate", ".env should be secret_candidate");

    // Recommended should not include .env
    let steps = v["recommended_next_steps"].as_array().unwrap();
    let step_paths: Vec<&str> = steps.iter().map(|s| s["path"].as_str().unwrap()).collect();
    assert!(!step_paths.contains(&".env"), ".env should not be recommended");

    // Should have a warning about secret
    let warnings = v["warnings"].as_array().unwrap();
    let env_warning = warnings.iter().find(|w| w["path"] == ".env");
    assert!(env_warning.is_some(), "Should have warning about .env");
}

#[test]
fn test_unknown_project_type() {
    let path = fixture_path("unknown_project");
    let (stdout, stderr, code) = run_lls(&["--json", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert_eq!(v["project_type"]["name"], "unknown");
    assert_eq!(v["project_type"]["confidence"], 0.0);
    assert!(v["entries"].as_array().unwrap().len() > 0);
}

#[test]
fn test_depth_control() {
    let path = fixture_path("deep_scan");
    let (stdout, stderr, code) = run_lls(&["--json", "--depth", "1", &path]);

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    let entries = v["entries"].as_array().unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"src"), "depth 1 should include src");
    // depth 1 should not include files inside src
    assert!(!names.contains(&"a.rs"), "depth 1 should not include a.rs");

    let (stdout2, stderr2, code2) = run_lls(&["--json", "--depth", "2", &path]);
    assert_eq!(code2, 0, "stderr: {stderr2}");
    let v2: serde_json::Value = serde_json::from_str(&stdout2).expect("Should be valid JSON");
    let entries2 = v2["entries"].as_array().unwrap();
    let names2: Vec<&str> = entries2.iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert!(names2.contains(&"a.rs"), "depth 2 should include a.rs");
}

#[test]
fn test_compact_json() {
    let path = fixture_path("rust_cli");
    let (pretty, _, _) = run_lls(&["--json", "--depth", "2", &path]);
    let (compact, _, code) = run_lls(&["--json", "--compact", "--depth", "2", &path]);

    assert_eq!(code, 0);
    // Compact should have no newlines (strip trailing newline from println!)
    let compact_trimmed = compact.trim_end();
    assert!(!compact_trimmed.contains('\n'), "Compact JSON should be single line");
    // Both should parse as valid JSON
    let _pv: serde_json::Value = serde_json::from_str(&pretty).expect("Pretty JSON should be valid");
    let _cv: serde_json::Value = serde_json::from_str(compact_trimmed).expect("Compact JSON should be valid");
}

#[test]
fn test_file_input() {
    let path = fixture_path("rust_cli");
    let (stdout, stderr, code) = run_lls(&["--json", &format!("{path}/Cargo.toml")]);

    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "Should have exactly 1 entry for file input");
    assert_eq!(entries[0]["name"], "Cargo.toml");
    assert_eq!(entries[0]["role"], "manifest");
}

#[test]
fn test_nonexistent_path() {
    let (_stdout, stderr, code) = run_lls(&["/nonexistent/path"]);

    assert_ne!(code, 0, "Should fail with non-zero exit code");
    assert_eq!(code, 2, "Should exit with code 2 for not found");
    assert!(stderr.contains("not found"), "Should mention not found");
}
