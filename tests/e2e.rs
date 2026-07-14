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

#[test]
fn test_completions_subcommand_generates_all_shell_files() {
    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("generated-completions");
    let output = Command::new(lls_binary())
        .args([
            "completions",
            "--out-dir",
            output_dir.to_str().expect("temporary path should be UTF-8"),
        ])
        .output()
        .expect("failed to run lls");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("completion files written"));

    for relative_path in [
        "bash/lls",
        "zsh/_lls",
        "fish/lls",
        "powershell/lls.ps1",
        "elvish/lls",
    ] {
        assert!(
            output_dir.join(relative_path).is_file(),
            "missing completion file: {relative_path}"
        );
    }
}
