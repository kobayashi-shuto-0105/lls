use std::time::Duration;

/// Request to run a Codex subprocess.
#[derive(Debug, Clone)]
pub struct ProcessRequest {
    pub command: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}

/// Result of a Codex subprocess execution.
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Error from running a process.
#[derive(Debug, Clone)]
pub enum ProcessError {
    NotFound,
    Timeout,
    NonZeroExit { code: i32, stderr: String },
    Io(String),
}

/// Abstraction for running a Codex subprocess.
///
/// This allows tests to use a fake runner instead of the real Codex CLI.
pub trait ProcessRunner {
    fn run(&self, request: ProcessRequest) -> Result<ProcessResult, ProcessError>;
}

/// Production runner using `std::process::Command`.
#[allow(dead_code)]
pub struct RealProcessRunner;

impl ProcessRunner for RealProcessRunner {
    fn run(&self, request: ProcessRequest) -> Result<ProcessResult, ProcessError> {
        let mut cmd = std::process::Command::new(&request.command);
        cmd.args(&request.args);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ProcessError::NotFound
            } else {
                ProcessError::Io(e.to_string())
            }
        })?;

        Ok(ProcessResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

/// Fake runner for testing.
pub struct FakeProcessRunner {
    pub result: Result<ProcessResult, ProcessError>,
}

impl FakeProcessRunner {
    pub fn new(result: Result<ProcessResult, ProcessError>) -> Self {
        Self { result }
    }
}

impl ProcessRunner for FakeProcessRunner {
    fn run(&self, _request: ProcessRequest) -> Result<ProcessResult, ProcessError> {
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_runner_success() {
        let runner = FakeProcessRunner::new(Ok(ProcessResult {
            exit_code: 0,
            stdout: "{}".into(),
            stderr: String::new(),
        }));

        let result = runner.run(ProcessRequest {
            command: "codex".into(),
            args: vec![],
            timeout: Duration::from_secs(30),
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().stdout, "{}");
    }

    #[test]
    fn test_fake_runner_failure() {
        let runner = FakeProcessRunner::new(Err(ProcessError::NotFound));
        let result = runner.run(ProcessRequest {
            command: "codex".into(),
            args: vec![],
            timeout: Duration::from_secs(30),
        });
        assert!(matches!(result, Err(ProcessError::NotFound)));
    }
}
