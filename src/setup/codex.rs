use once_cell::sync::Lazy;
use regex::Regex;
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
///
/// This function sanitizes error messages to avoid exposing internal paths,
/// credentials, or raw stderr content to users. It categorizes common failure
/// patterns into user-friendly messages.
fn map_codex_error(err: ProcessError) -> AppError {
    match err {
        ProcessError::NotFound => {
            AppError::Codex("Codex CLI not found. Install Codex CLI or use --without-codex".into())
        }
        ProcessError::Timeout => AppError::Codex("Codex timeout: generation took too long".into()),
        ProcessError::NonZeroExit { code, stderr } => {
            let user_message = categorize_codex_failure(code, &stderr);
            AppError::Codex(user_message)
        }
        ProcessError::Io(msg) => {
            // Sanitize I/O error messages to avoid exposing internal paths
            let sanitized = sanitize_io_error(&msg);
            AppError::Codex(format!("Codex I/O error: {sanitized}"))
        }
    }
}

/// Categorize Codex failure based on exit code and stderr patterns.
///
/// Returns a user-friendly message without exposing raw stderr content or
/// internal paths like `~/.codex/state_*.sqlite`.
fn categorize_codex_failure(code: i32, stderr: &str) -> String {
    let stderr_lower = stderr.to_lowercase();

    // Authentication-related failures
    // Note: We use specific phrases to avoid false positives like "author"
    if stderr_lower.contains("not logged in")
        || stderr_lower.contains("authentication required")
        || stderr_lower.contains("authentication failed")
        || stderr_lower.contains("sign in")
        || stderr_lower.contains("please login")
        || stderr_lower.contains("need to login")
        || stderr_lower.contains("unauthenticated")
    {
        return "Codex authentication required. Run `codex login` to sign in with ChatGPT".into();
    }

    // Network-related failures
    if stderr_lower.contains("network")
        || stderr_lower.contains("connection")
        || stderr_lower.contains("timeout")
        || stderr_lower.contains("unreachable")
        || stderr_lower.contains("dns")
    {
        return "Codex network error. Check your internet connection and try again".into();
    }

    // Rate limiting
    if stderr_lower.contains("rate limit") || stderr_lower.contains("too many requests") {
        return "Codex rate limit reached. Please wait a moment and try again".into();
    }

    // Permission or sandbox errors
    if stderr_lower.contains("permission denied") || stderr_lower.contains("sandbox") {
        return "Codex permission error. The sandbox may not have required access".into();
    }

    // Internal state errors (e.g., sqlite database issues)
    if stderr_lower.contains("sqlite")
        || stderr_lower.contains("database")
        || stderr_lower.contains("state")
    {
        return "Codex internal state error. Try running `codex` directly to diagnose".into();
    }

    // Configuration errors
    if stderr_lower.contains("config") || stderr_lower.contains("invalid") {
        return "Codex configuration error. Check your Codex CLI setup".into();
    }

    // Default: provide exit code without raw stderr
    format!("Codex failed with exit code {code}. Run `codex` directly to diagnose")
}

/// Static regex patterns for sanitizing paths.
/// Compiled once using Lazy to avoid recompilation on each call.
static PATH_SANITIZERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    // These patterns are intentionally strict to avoid false positives
    let patterns = [
        // Match paths containing .codex directory (must start with / or ~/)
        r"(?:/[^\s]*)?/\.codex[^\s]*",
        r"~/\.codex[^\s]*",
        // Home directory paths on Unix (must have /home/ prefix)
        r"/home/[^\s]+",
        // Tilde-expanded paths (must start with ~/ to avoid matching ~1.5ms etc.)
        r"~/[^\s]+",
        // Windows user paths
        r"C:\\Users\\[^\s]+",
    ];

    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid path sanitizer regex"))
        .collect()
});

/// Sanitize I/O error messages to avoid exposing internal paths.
fn sanitize_io_error(msg: &str) -> String {
    let mut result = msg.to_string();
    for re in PATH_SANITIZERS.iter() {
        result = re.replace_all(&result, "[internal path]").to_string();
    }
    result
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

    // Tests for error sanitization (issue #12)

    #[test]
    fn test_categorize_auth_failure() {
        let msg = categorize_codex_failure(1, "Error: not logged in");
        assert!(msg.contains("authentication required"));
        assert!(!msg.contains("not logged in")); // raw stderr not exposed

        let msg = categorize_codex_failure(1, "Please sign in to continue");
        assert!(msg.contains("authentication required"));

        // "unauthenticated" should trigger auth message
        let msg = categorize_codex_failure(1, "Request was unauthenticated");
        assert!(msg.contains("authentication required"));
    }

    #[test]
    fn test_categorize_auth_no_false_positives() {
        // "author" should NOT trigger auth message (it was a false positive with old "auth" pattern)
        let msg = categorize_codex_failure(1, "Unknown author field detected");
        assert!(!msg.contains("authentication required"));
        // Should NOT fall through to config error either - "author" shouldn't match anything
        // but "Unknown" doesn't trigger any category, so this becomes an unknown error
        // Let's use a cleaner example that doesn't match any category
        let msg = categorize_codex_failure(1, "File authored by user");
        assert!(
            !msg.contains("authentication required"),
            "'authored' should not trigger auth error"
        );
        assert!(msg.contains("exit code"), "should fall through to unknown");
    }

    #[test]
    fn test_categorize_network_failure() {
        let msg = categorize_codex_failure(1, "Network connection failed");
        assert!(msg.contains("network error"));
        assert!(!msg.contains("failed")); // raw stderr not exposed

        let msg = categorize_codex_failure(1, "DNS resolution timeout");
        assert!(msg.contains("network error"));
    }

    #[test]
    fn test_categorize_rate_limit() {
        let msg = categorize_codex_failure(1, "Rate limit exceeded, too many requests");
        assert!(msg.contains("rate limit"));
    }

    #[test]
    fn test_categorize_permission_error() {
        let msg = categorize_codex_failure(1, "Permission denied accessing sandbox");
        assert!(msg.contains("permission error"));
    }

    #[test]
    fn test_categorize_internal_state_error() {
        // This is the specific case from the issue - sqlite paths should not be exposed
        let raw_stderr = "Error: failed to open /home/user/.codex/state_5.sqlite";
        let msg = categorize_codex_failure(1, raw_stderr);
        assert!(msg.contains("internal state error"));
        // Verify the raw stderr is NOT present in the user-facing message
        assert!(
            !msg.contains(raw_stderr),
            "raw stderr should not be in message"
        );
        assert!(!msg.contains("sqlite"), "sqlite keyword not in message");
        assert!(!msg.contains(".codex"), "internal path not exposed");
        assert!(!msg.contains("/home/user"), "user path not exposed");
    }

    #[test]
    fn test_categorize_config_error() {
        let msg = categorize_codex_failure(1, "Invalid config file format");
        assert!(msg.contains("configuration error"));
    }

    #[test]
    fn test_categorize_unknown_error() {
        // Unknown error should show exit code but not raw stderr
        let raw_stderr = "Some unexpected error with /secret/path";
        let msg = categorize_codex_failure(42, raw_stderr);
        assert!(msg.contains("exit code 42"));
        // Verify the raw stderr is NOT present in the user-facing message
        assert!(
            !msg.contains(raw_stderr),
            "raw stderr should not be in message"
        );
        assert!(!msg.contains("unexpected error"), "raw stderr not exposed");
        assert!(!msg.contains("/secret/path"), "internal path not exposed");
    }

    #[test]
    fn test_sanitize_io_error_removes_codex_paths() {
        let sanitized = sanitize_io_error("Failed to read /home/user/.codex/state_5.sqlite");
        assert!(!sanitized.contains(".codex"));
        assert!(!sanitized.contains("state_5.sqlite"));
        assert!(sanitized.contains("[internal path]"));
    }

    #[test]
    fn test_sanitize_io_error_removes_home_paths() {
        let sanitized = sanitize_io_error("Cannot access /home/user/secret/file.txt");
        assert!(!sanitized.contains("/home/user"));
        assert!(sanitized.contains("[internal path]"));
    }

    #[test]
    fn test_sanitize_io_error_removes_tilde_paths() {
        let sanitized = sanitize_io_error("File not found: ~/.codex/auth.json");
        assert!(!sanitized.contains("~/.codex"));
        assert!(sanitized.contains("[internal path]"));
    }

    #[test]
    fn test_sanitize_io_error_no_false_positives() {
        // Tilde followed by numbers (like "~1.5ms") should NOT be sanitized
        let sanitized = sanitize_io_error("Operation completed in ~1.5ms");
        assert_eq!(sanitized, "Operation completed in ~1.5ms");

        let sanitized = sanitize_io_error("Approximately ~500 requests processed");
        assert_eq!(sanitized, "Approximately ~500 requests processed");

        // Text containing ".codex" but not as a path should be handled correctly
        let sanitized = sanitize_io_error("my.codex-file is missing");
        assert!(
            sanitized.contains("my.codex"),
            "should not match .codex without path prefix"
        );
    }

    #[test]
    fn test_sanitize_io_error_preserves_safe_messages() {
        let sanitized = sanitize_io_error("Operation timed out");
        assert_eq!(sanitized, "Operation timed out");
    }

    #[test]
    fn test_map_codex_non_zero_exit_sanitized() {
        // Verify that NonZeroExit errors are properly sanitized
        let raw_stderr = "Error: failed to connect to /home/user/.codex/state.db";
        let err = map_codex_error(crate::codex::ProcessError::NonZeroExit {
            code: 1,
            stderr: raw_stderr.into(),
        });
        match err {
            AppError::Codex(msg) => {
                // Should NOT contain raw stderr or paths
                assert!(
                    !msg.contains(raw_stderr),
                    "raw stderr should not be in message"
                );
                assert!(!msg.contains("/home/user"));
                assert!(!msg.contains(".codex"));
                assert!(!msg.contains("state.db"));
                // Should contain user-friendly message
                assert!(msg.contains("internal state error") || msg.contains("exit code"));
            }
            _ => panic!("expected Codex error"),
        }
    }

    #[test]
    fn test_map_codex_io_error_sanitized() {
        let err = map_codex_error(crate::codex::ProcessError::Io(
            "Cannot open /home/user/.codex/config.json".into(),
        ));
        match err {
            AppError::Codex(msg) => {
                assert!(!msg.contains("/home/user"));
                assert!(msg.contains("[internal path]") || msg.contains("I/O error"));
            }
            _ => panic!("expected Codex error"),
        }
    }
}
