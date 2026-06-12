use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the `lls` binary, discovered via Cargo's integration test env var.
pub fn lls_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lls"))
}

/// Run `lls` with given args in the given directory, returning output.
pub fn run_lls(args: &[&str], dir: &Path) -> std::process::Output {
    Command::new(lls_binary())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run lls")
}

/// Run `lls` with --json and parse the output.
pub fn run_lls_json(args: &[&str], dir: &Path) -> serde_json::Value {
    let mut all_args = vec!["--json"];
    all_args.extend_from_slice(args);
    let output = run_lls(&all_args, dir);
    assert!(
        output.status.success(),
        "lls failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).expect("invalid JSON output")
}

/// Create a Rust CLI project fixture.
pub fn fixture_rust_cli(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "# Test\n").unwrap();
    std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
    std::fs::write(dir.join(".gitignore"), "target/\n").unwrap();
}

/// Create a Rust library project fixture.
pub fn fixture_rust_library(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn test() {}\n").unwrap();
}

/// Create a Node project fixture.
pub fn fixture_node(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("node_modules")).unwrap();
    std::fs::create_dir_all(dir.join("dist")).unwrap();
    std::fs::write(dir.join("package.json"), "{\"name\": \"test\"}\n").unwrap();
    std::fs::write(dir.join("README.md"), "# Node Test\n").unwrap();
    std::fs::write(dir.join("src/index.js"), "console.log('hello');\n").unwrap();
}

/// Create a Python project fixture.
pub fn fixture_python(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
    std::fs::write(dir.join("README.md"), "# Python Test\n").unwrap();
    std::fs::write(dir.join("src/main.py"), "print('hello')\n").unwrap();
}

/// Create a Go project fixture.
pub fn fixture_go(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("go.mod"), "module test\n").unwrap();
    std::fs::write(dir.join("README.md"), "# Go Test\n").unwrap();
    std::fs::write(dir.join("src/main.go"), "package main\n").unwrap();
}

/// Create a polyglot fixture (multiple project types).
pub fn fixture_polyglot(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    std::fs::write(dir.join("package.json"), "{\"name\": \"test\"}\n").unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
    std::fs::write(dir.join("go.mod"), "module test\n").unwrap();
}

/// Create a sensitive content fixture.
pub fn fixture_sensitive(dir: &Path) {
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::write(dir.join(".env"), "SECRET=value\n").unwrap();
    std::fs::write(
        dir.join("config/credentials.json"),
        "{\"password\": \"secret\"}\n",
    )
    .unwrap();
}

/// Create a fixture with broken symlinks.
#[cfg(unix)]
#[allow(dead_code)]
pub fn fixture_broken_symlink(dir: &Path) {
    std::fs::write(dir.join("real.txt"), "real\n").unwrap();
    std::os::unix::fs::symlink("nonexistent.txt", dir.join("broken.lnk")).unwrap();
}

/// Create a fixture with invalid config.
pub fn fixture_invalid_config(dir: &Path) {
    std::fs::create_dir_all(dir.join(".lls")).unwrap();
    std::fs::write(dir.join(".lls/config.json"), "not valid json\n").unwrap();
}

/// Create a fixture with a valid .lls/config.json.
#[allow(dead_code)]
pub fn fixture_with_config(dir: &Path, config_json: &str) {
    std::fs::create_dir_all(dir.join(".lls")).unwrap();
    std::fs::write(dir.join(".lls/config.json"), config_json).unwrap();
}

/// Create an unknown project (no manifest files).
pub fn fixture_unknown(dir: &Path) {
    std::fs::write(dir.join("readme.txt"), "Hello\n").unwrap();
    std::fs::write(dir.join("data.csv"), "a,b,c\n").unwrap();
}
