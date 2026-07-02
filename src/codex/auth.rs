//! Authentication status adapter for Codex CLI.
//!
//! This module provides functionality to check Codex authentication status
//! and enforce the ChatGPT-only authentication policy defined in the spec.
//!
//! The MVP supports only "Sign in with ChatGPT" authentication.
//! API key-based authentication (OPENAI_API_KEY, CODEX_API_KEY) is explicitly rejected.

use crate::codex::{ProcessError, ProcessRequest, ProcessRunner};
use std::time::Duration;

/// Authentication status result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    /// Authenticated via ChatGPT login.
    LoggedIn,
    /// Not logged in (no valid session).
    NotLoggedIn,
    /// API key environment variable detected (not supported).
    ApiKeyDetected { var_name: String },
    /// Codex CLI not found.
    CodexNotFound,
    /// Unknown status (could not determine).
    Unknown { message: String },
}

/// Authentication check error.
#[derive(Debug, Clone)]
pub struct AuthCheckError {
    pub status: AuthStatus,
    pub guidance: String,
}

impl AuthCheckError {
    fn api_key(var_name: &str) -> Self {
        Self {
            status: AuthStatus::ApiKeyDetected {
                var_name: var_name.to_string(),
            },
            guidance: format!(
                "API key authentication is not supported. \
                 Please unset {var_name} and use `codex login` to sign in with ChatGPT."
            ),
        }
    }

    fn not_logged_in(is_headless: bool) -> Self {
        let guidance = if is_headless {
            "Codex CLI is not logged in. \
             Run `codex login --device-auth` to authenticate in a headless environment."
                .to_string()
        } else {
            "Codex CLI is not logged in. \
             Run `codex login` to sign in with ChatGPT."
                .to_string()
        };
        Self {
            status: AuthStatus::NotLoggedIn,
            guidance,
        }
    }

    fn codex_not_found() -> Self {
        Self {
            status: AuthStatus::CodexNotFound,
            guidance: "Codex CLI not found. Install Codex CLI or use `lls setup --without-codex`."
                .to_string(),
        }
    }

    fn unknown(message: &str) -> Self {
        Self {
            status: AuthStatus::Unknown {
                message: message.to_string(),
            },
            guidance: format!(
                "Could not determine Codex authentication status: {message}. \
                 Try running `codex auth status` manually."
            ),
        }
    }
}

/// Environment variable names that indicate API key authentication.
const REJECTED_ENV_VARS: &[&str] = &["OPENAI_API_KEY", "CODEX_API_KEY"];

/// Check for rejected API key environment variables.
///
/// Returns `Err` if any API key environment variable is set.
pub fn check_api_key_env() -> Result<(), AuthCheckError> {
    for var_name in REJECTED_ENV_VARS {
        if std::env::var(var_name).is_ok() {
            return Err(AuthCheckError::api_key(var_name));
        }
    }
    Ok(())
}

/// Check Codex authentication status using the Codex CLI.
///
/// This function:
/// 1. Rejects API key environment variables
/// 2. Runs `codex auth status` to check if logged in
/// 3. Returns `Ok(AuthStatus::LoggedIn)` only if ChatGPT login is active
pub fn check_auth_status<R: ProcessRunner>(runner: &R) -> Result<AuthStatus, AuthCheckError> {
    // First, check for rejected environment variables
    check_api_key_env()?;

    // Run `codex auth status` to check login status
    let request = ProcessRequest {
        command: "codex".to_string(),
        args: vec!["auth".to_string(), "status".to_string()],
        timeout: Duration::from_secs(10),
    };

    match runner.run(request) {
        Ok(result) => {
            // Parse the output to determine authentication status
            // Expected output patterns:
            // - "Logged in" / "authenticated" / "chatgpt" -> LoggedIn
            // - "Not logged in" / "unauthenticated" -> NotLoggedIn
            let stdout_lower = result.stdout.to_lowercase();
            let stderr_lower = result.stderr.to_lowercase();
            let combined = format!("{stdout_lower} {stderr_lower}");

            // Check for NOT logged in patterns FIRST (before positive patterns)
            // because "not logged in" contains "logged in"
            if combined.contains("not logged in")
                || combined.contains("unauthenticated")
                || combined.contains("not authenticated")
                || combined.contains("no session")
                || combined.contains("please login")
                || combined.contains("please sign in")
            {
                let is_headless = is_headless_environment();
                Err(AuthCheckError::not_logged_in(is_headless))
            } else if combined.contains("logged in")
                || combined.contains("authenticated")
                || combined.contains("chatgpt")
                || combined.contains("signed in")
            {
                Ok(AuthStatus::LoggedIn)
            } else if result.exit_code == 0 {
                // Exit code 0 but unclear output - assume logged in
                Ok(AuthStatus::LoggedIn)
            } else {
                // Non-zero exit with unclear output
                Err(AuthCheckError::unknown(&format!(
                    "codex auth status returned exit code {}",
                    result.exit_code
                )))
            }
        }
        Err(ProcessError::NotFound) => Err(AuthCheckError::codex_not_found()),
        Err(ProcessError::Timeout) => Err(AuthCheckError::unknown("codex auth status timed out")),
        Err(ProcessError::NonZeroExit { code, stderr }) => {
            let stderr_lower = stderr.to_lowercase();
            if stderr_lower.contains("not logged in")
                || stderr_lower.contains("unauthenticated")
                || stderr_lower.contains("not authenticated")
                || stderr_lower.contains("please login")
                || stderr_lower.contains("please sign in")
            {
                let is_headless = is_headless_environment();
                Err(AuthCheckError::not_logged_in(is_headless))
            } else {
                Err(AuthCheckError::unknown(&format!(
                    "codex auth status exited with code {code}"
                )))
            }
        }
        Err(ProcessError::Io(msg)) => Err(AuthCheckError::unknown(&format!("I/O error: {msg}"))),
    }
}

/// Check if the current environment appears to be headless (no display).
fn is_headless_environment() -> bool {
    // Check common environment variables that indicate a display is available
    std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::{FakeProcessRunner, ProcessResult};

    #[test]
    fn test_api_key_env_not_set() {
        // Temporarily ensure env vars are not set
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }
        assert!(check_api_key_env().is_ok());
    }

    #[test]
    fn test_api_key_env_openai_set() {
        // Set the env var for this test
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "sk-test-key");
        }
        let result = check_api_key_env();
        // SAFETY: Clean up
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err.status,
            AuthStatus::ApiKeyDetected { var_name } if var_name == "OPENAI_API_KEY"
        ));
        assert!(err.guidance.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_api_key_env_codex_set() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::set_var("CODEX_API_KEY", "test-key");
        }
        let result = check_api_key_env();
        // SAFETY: Clean up
        unsafe {
            std::env::remove_var("CODEX_API_KEY");
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err.status,
            AuthStatus::ApiKeyDetected { var_name } if var_name == "CODEX_API_KEY"
        ));
    }

    #[test]
    fn test_check_auth_status_logged_in() {
        // Ensure no API key env vars
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Logged in as user@example.com via ChatGPT".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthStatus::LoggedIn);
    }

    #[test]
    fn test_check_auth_status_authenticated() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Authenticated".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthStatus::LoggedIn);
    }

    #[test]
    fn test_check_auth_status_not_logged_in() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 1,
            stdout: "Not logged in".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, AuthStatus::NotLoggedIn);
        assert!(err.guidance.contains("codex login"));
    }

    #[test]
    fn test_check_auth_status_please_login() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Err(ProcessError::NonZeroExit {
            code: 1,
            stderr: "Please login first".to_string(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, AuthStatus::NotLoggedIn);
    }

    #[test]
    fn test_check_auth_status_codex_not_found() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Err(ProcessError::NotFound));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, AuthStatus::CodexNotFound);
        assert!(err.guidance.contains("Codex CLI not found"));
    }

    #[test]
    fn test_check_auth_status_timeout() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        let runner = FakeProcessRunner::new(Err(ProcessError::Timeout));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.status, AuthStatus::Unknown { .. }));
    }

    #[test]
    fn test_check_auth_status_with_api_key_env() {
        // API key env var should be rejected before running codex
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "sk-test");
        }

        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Logged in".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        // SAFETY: Clean up
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.status, AuthStatus::ApiKeyDetected { .. }));
    }

    #[test]
    fn test_check_auth_status_exit_code_0_unclear_output() {
        // SAFETY: Tests run in isolation; we clean up after ourselves.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("CODEX_API_KEY");
        }

        // Exit code 0 with unclear output should assume logged in
        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "session active".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthStatus::LoggedIn);
    }

    #[test]
    fn test_rejected_env_vars_list() {
        assert!(REJECTED_ENV_VARS.contains(&"OPENAI_API_KEY"));
        assert!(REJECTED_ENV_VARS.contains(&"CODEX_API_KEY"));
    }
}
