use std::process::Command;

/// Path to the `lls` binary, discovered via Cargo's integration test env var.
fn lls_binary() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_lls"))
}

#[test]
fn test_no_config_flag() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(lls_binary())
        .arg("--no-config")
        .current_dir(dir.path())
        .output()
        .expect("failed to run lls");
    assert!(output.status.success());
}

#[test]
fn test_conflicting_modes() {
    let output = Command::new(lls_binary())
        .args(["--json", "--human"])
        .output()
        .expect("failed to run lls");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_help() {
    let output = Command::new(lls_binary())
        .arg("--help")
        .output()
        .expect("failed to run lls");
    assert!(output.status.success());
}
