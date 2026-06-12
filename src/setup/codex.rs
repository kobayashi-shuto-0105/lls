use std::time::Duration;

use crate::codex::{ProcessError, ProcessRequest, ProcessRunner};
use crate::config::validate_config;
use crate::error::AppError;

/// Result from a Codex setup run, after validation.
#[derive(Debug)]
pub struct ValidatedCodexOutput {
    pub config: crate::config::model::ProjectConfig,
    pub raw_json: String,
}

/// Run Codex-assisted setup and return the raw JSON output.
pub fn run_codex_setup(project_root: &std::path::Path) -> Result<String, AppError> {
    let runner = RealCodexRunner;

    // Build the exec request
    let schema_path = std::path::PathBuf::from("/dev/null"); // In practice, use embedded schema
    let output_path = project_root.join(".lls").join(".codex-output.tmp");

    let cmd = crate::codex::build_codex_command(
        "codex",
        &schema_path,
        &output_path,
        project_root,
        "Analyze this project and generate an lls configuration file (.lls/config.json) \
         that defines scan rules, priority overrides, role overrides, and ignore patterns.",
    );

    let request = ProcessRequest {
        command: cmd.get_program().to_string_lossy().to_string(),
        args: cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect(),
        timeout: Duration::from_secs(120),
    };

    // Run Codex
    runner.run(request).map_err(map_codex_error)?;

    // Read output from the temporary file
    let output = std::fs::read_to_string(&output_path)
        .map_err(|_| AppError::Codex("Codex did not write output file".into()))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&output_path);

    Ok(output)
}

/// Validate Codex output: parse JSON, schema validate, safety check.
pub fn validate_codex_output(json_str: &str) -> Result<ValidatedCodexOutput, AppError> {
    let validated = validate_config(json_str)?;
    Ok(ValidatedCodexOutput {
        config: validated.config,
        raw_json: json_str.to_string(),
    })
}

/// Map Codex process errors to AppError.
fn map_codex_error(err: ProcessError) -> AppError {
    match err {
        ProcessError::NotFound => {
            AppError::Codex("Codex CLI not found. Install Codex CLI or use --without-codex".into())
        }
        ProcessError::Timeout => AppError::Codex("Codex timeout: generation took too long".into()),
        ProcessError::NonZeroExit { code, stderr } => {
            AppError::Codex(format!("Codex exited with code {code}: {stderr}"))
        }
        ProcessError::Io(msg) => AppError::Codex(format!("Codex I/O error: {msg}")),
    }
}

/// Production Codex runner using `std::process::Command`.
struct RealCodexRunner;

impl ProcessRunner for RealCodexRunner {
    fn run(&self, request: ProcessRequest) -> Result<crate::codex::ProcessResult, ProcessError> {
        let mut cmd = std::process::Command::new(&request.command);
        cmd.args(&request.args);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ProcessError::NotFound
            } else {
                ProcessError::Io(e.to_string())
            }
        })?;

        if !output.status.success() {
            return Err(ProcessError::NonZeroExit {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(crate::codex::ProcessResult {
            exit_code: output.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_codex_output_valid() {
        let json = r#"{
            "schema_version": "0.1.0",
            "default_output": "json",
            "scan": {
                "depth": 1,
                "include_hidden": true,
                "include_ignored": false
            },
            "long_listing": {
                "sort": "priority"
            },
            "rules": {
                "priority_overrides": [],
                "role_overrides": [],
                "ignore_patterns": [],
                "sensitive_patterns": []
            },
            "codex": {
                "enabled": true,
                "auth_method": "chatgpt",
                "use_for_setup": true
            }
        }"#;

        let result = validate_codex_output(json);
        assert!(
            result.is_ok(),
            "valid Codex output should pass: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_validate_codex_output_malformed_json() {
        let result = validate_codex_output("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_codex_output_missing_fields() {
        let json = r#"{"schema_version": "0.1.0"}"#;
        let result = validate_codex_output(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_codex_output_unsafe_override() {
        // Attempt to override .git/ to non-ignore
        let json = r#"{
            "schema_version": "0.1.0",
            "default_output": "json",
            "scan": { "depth": 1, "include_hidden": true, "include_ignored": false },
            "long_listing": { "sort": "priority" },
            "rules": {
                "priority_overrides": [{"pattern": ".git", "priority": "high"}],
                "role_overrides": [],
                "ignore_patterns": [],
                "sensitive_patterns": []
            },
            "codex": { "enabled": true, "auth_method": "chatgpt", "use_for_setup": true }
        }"#;
        // validate_config should reject it via safety rules
        let result = validate_codex_output(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_codex_not_found() {
        let err = map_codex_error(crate::codex::ProcessError::NotFound);
        match err {
            AppError::Codex(msg) => assert!(msg.contains("Codex CLI not found")),
            _ => panic!("expected Codex error"),
        }
    }

    #[test]
    fn test_map_codex_timeout() {
        let err = map_codex_error(crate::codex::ProcessError::Timeout);
        match err {
            AppError::Codex(msg) => assert!(msg.contains("timeout")),
            _ => panic!("expected Codex error"),
        }
    }

    #[test]
    fn test_fake_codex_runner_integration() {
        // Fake runner simulates successful Codex
        let runner = crate::codex::FakeProcessRunner::new(Ok(crate::codex::ProcessResult {
            exit_code: 0,
            stdout: r#"{"schema_version": "0.1.0"}"#.into(),
            stderr: String::new(),
        }));

        let request = ProcessRequest {
            command: "codex".into(),
            args: vec![],
            timeout: Duration::from_secs(30),
        };

        let result = runner.run(request);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("schema_version"));
    }

    #[test]
    fn test_fake_codex_runner_failure() {
        let runner =
            crate::codex::FakeProcessRunner::new(Err(crate::codex::ProcessError::NotFound));
        let request = ProcessRequest {
            command: "codex".into(),
            args: vec![],
            timeout: Duration::from_secs(30),
        };
        let result = runner.run(request);
        assert!(matches!(result, Err(crate::codex::ProcessError::NotFound)));
    }
}
