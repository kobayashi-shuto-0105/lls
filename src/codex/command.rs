use std::path::Path;

/// Build the argument vector for a safe Codex exec invocation.
///
/// This follows the spec's concept:
/// ```sh
/// codex exec \
///   --ephemeral \
///   --sandbox read-only \
///   --ignore-user-config \
///   --ignore-rules \
///   --output-schema <schema-path> \
///   --output-last-message <output-path> \
///   --cd <project-root> \
///   "<setup-prompt>"
/// ```
pub fn build_codex_command(
    codex_path: &str,
    schema_path: &Path,
    output_path: &Path,
    project_root: &Path,
    prompt: &str,
) -> std::process::Command {
    let mut cmd = std::process::Command::new(codex_path);
    cmd.arg("exec")
        .arg("--ephemeral")
        .arg("--sandbox")
        .arg("read-only")
        .arg("--ignore-user-config")
        .arg("--ignore-rules")
        .arg("--output-schema")
        .arg(schema_path)
        .arg("--output-last-message")
        .arg(output_path)
        .arg("--cd")
        .arg(project_root)
        .arg(prompt);

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_args() {
        let schema = Path::new("/tmp/schema.json");
        let output = Path::new("/tmp/output.json");
        let root = Path::new("/project");

        let cmd = build_codex_command("codex", schema, output, root, "analyze");
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        assert_eq!(args[0], "exec");
        assert!(args.contains(&"--ephemeral".to_string()));
        assert!(args.contains(&"read-only".to_string()));
        assert!(args.contains(&"--output-schema".to_string()));
        assert!(args.contains(&schema.to_string_lossy().into_owned()));
        assert!(args.contains(&output.to_string_lossy().into_owned()));
        assert!(args.contains(&root.to_string_lossy().into_owned()));
        assert!(args.contains(&"analyze".to_string()));

        // Should NOT contain dangerous flags
        assert!(!args.contains(&"--danger-full-access".to_string()));
        assert!(!args.contains(&"--writable".to_string()));
    }
}
