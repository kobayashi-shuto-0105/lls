//! Authentication status adapter for Codex CLI.
//!
//! `lls` does not read credential files or API-key environment variables.
//! Instead, it asks the Codex CLI whether the current session is logged in
//! with ChatGPT before running `codex exec` during setup.

use crate::codex::{ProcessError, ProcessRequest, ProcessRunner};
use std::time::Duration;

/// Authentication status result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    /// Authenticated via ChatGPT login.
    LoggedIn,
    /// No Codex session is available.
    NotLoggedIn,
    /// Codex is authenticated, but not with the supported ChatGPT method.
    UnsupportedAuthMethod,
    /// Codex CLI is not installed.
    CodexNotFound,
    /// The status could not be determined safely.
    Unknown { message: String },
}

/// Authentication check error.
#[derive(Debug, Clone)]
pub struct AuthCheckError {
    pub status: AuthStatus,
    pub guidance: String,
}

impl AuthCheckError {
    fn not_logged_in() -> Self {
        Self {
            status: AuthStatus::NotLoggedIn,
            guidance: "Codex CLI is not logged in with ChatGPT. Run `codex login` to sign in. In headless environments, use `codex login --device-auth`.".to_string(),
        }
    }

    fn unsupported_auth_method() -> Self {
        Self {
            status: AuthStatus::UnsupportedAuthMethod,
            guidance: "Codex CLI must be authenticated with ChatGPT for `lls setup`. Run `codex logout`, then `codex login`. In headless environments, use `codex login --device-auth`.".to_string(),
        }
    }

    fn codex_not_found() -> Self {
        Self {
            status: AuthStatus::CodexNotFound,
            guidance: "Codex CLI not found. Install Codex CLI or use `lls setup --without-codex`."
                .to_string(),
        }
    }

    fn unknown(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status: AuthStatus::Unknown {
                message: message.clone(),
            },
            guidance: format!(
                "Could not determine Codex authentication status ({message}). Run `codex login status` manually."
            ),
        }
    }
}

/// Check Codex authentication status using `codex login status`.
pub fn check_auth_status<R: ProcessRunner>(runner: &R) -> Result<AuthStatus, AuthCheckError> {
    let request = ProcessRequest {
        command: "codex".to_string(),
        args: vec!["login".to_string(), "status".to_string()],
        timeout: Duration::from_secs(10),
    };

    match runner.run(request) {
        Ok(result) => parse_auth_status_result(&result.stdout, &result.stderr, result.exit_code),
        Err(ProcessError::NotFound) => Err(AuthCheckError::codex_not_found()),
        Err(ProcessError::Timeout) => {
            Err(AuthCheckError::unknown("`codex login status` timed out"))
        }
        Err(ProcessError::NonZeroExit { code, stderr }) => {
            parse_auth_status_result("", &stderr, code)
        }
        Err(ProcessError::Io(_)) => Err(AuthCheckError::unknown(
            "`codex login status` failed with an I/O error",
        )),
    }
}

fn parse_auth_status_result(
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> Result<AuthStatus, AuthCheckError> {
    let combined = format!("{} {}", stdout.to_lowercase(), stderr.to_lowercase());

    if is_chatgpt_logged_in(&combined) {
        return Ok(AuthStatus::LoggedIn);
    }

    if indicates_unsupported_auth_method(&combined) {
        return Err(AuthCheckError::unsupported_auth_method());
    }

    if indicates_logged_out(&combined) {
        return Err(AuthCheckError::not_logged_in());
    }

    if exit_code == 0 {
        return Err(AuthCheckError::unknown(
            "`codex login status` returned an unrecognized success response",
        ));
    }

    Err(AuthCheckError::unknown(format!(
        "`codex login status` exited with code {exit_code}"
    )))
}

fn is_chatgpt_logged_in(output: &str) -> bool {
    (output.contains("chatgpt") || output.contains("chat gpt"))
        && (output.contains("logged in")
            || output.contains("signed in")
            || output.contains("using")
            || output.contains("authenticated"))
}

fn indicates_unsupported_auth_method(output: &str) -> bool {
    output.contains("api key")
        || output.contains("access token")
        || output.contains("logged in using api")
        || output.contains("logged in using access token")
}

fn indicates_logged_out(output: &str) -> bool {
    output.contains("not logged in")
        || output.contains("not authenticated")
        || output.contains("unauthenticated")
        || output.contains("no session")
        || output.contains("please login")
        || output.contains("please sign in")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::{FakeProcessRunner, ProcessResult};

    struct AssertingRunner {
        result: Result<ProcessResult, ProcessError>,
    }

    impl ProcessRunner for AssertingRunner {
        fn run(&self, request: ProcessRequest) -> Result<ProcessResult, ProcessError> {
            assert_eq!(request.command, "codex");
            assert_eq!(
                request.args,
                vec!["login".to_string(), "status".to_string()]
            );
            assert_eq!(request.timeout, Duration::from_secs(10));
            self.result.clone()
        }
    }

    #[test]
    fn test_check_auth_status_uses_login_status_command() {
        let runner = AssertingRunner {
            result: Ok(ProcessResult {
                exit_code: 0,
                stdout: "Logged in using ChatGPT".to_string(),
                stderr: String::new(),
            }),
        };

        let result = check_auth_status(&runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthStatus::LoggedIn);
    }

    #[test]
    fn test_check_auth_status_logged_in_with_chatgpt() {
        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Logged in using ChatGPT".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthStatus::LoggedIn);
    }

    #[test]
    fn test_check_auth_status_rejects_api_key_login() {
        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Logged in using API key".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, AuthStatus::UnsupportedAuthMethod);
        assert!(err.guidance.contains("codex logout"));
    }

    #[test]
    fn test_check_auth_status_not_logged_in() {
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
        assert!(err.guidance.contains("device-auth"));
    }

    #[test]
    fn test_check_auth_status_not_logged_in_from_stderr() {
        let runner = FakeProcessRunner::new(Err(ProcessError::NonZeroExit {
            code: 1,
            stderr: "Please sign in with ChatGPT".to_string(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, AuthStatus::NotLoggedIn);
    }

    #[test]
    fn test_check_auth_status_codex_not_found() {
        let runner = FakeProcessRunner::new(Err(ProcessError::NotFound));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, AuthStatus::CodexNotFound);
        assert!(err.guidance.contains("without-codex"));
    }

    #[test]
    fn test_check_auth_status_timeout() {
        let runner = FakeProcessRunner::new(Err(ProcessError::Timeout));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.status, AuthStatus::Unknown { .. }));
        assert!(err.guidance.contains("codex login status"));
    }

    #[test]
    fn test_check_auth_status_unknown_success_output_is_error() {
        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "Session active".to_string(),
            stderr: String::new(),
        }));

        let result = check_auth_status(&runner);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().status,
            AuthStatus::Unknown { .. }
        ));
    }
}
