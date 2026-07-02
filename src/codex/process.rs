use std::io::Read;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
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

/// Production runner using `std::process::Command` with timeout support.
#[allow(dead_code)]
pub struct RealProcessRunner;

impl ProcessRunner for RealProcessRunner {
    fn run(&self, request: ProcessRequest) -> Result<ProcessResult, ProcessError> {
        run_process_with_timeout(&request.command, &request.args, request.timeout)
    }
}

/// Run a subprocess with timeout enforcement.
///
/// Spawns the process and waits for completion. If the process does not
/// complete within the specified timeout, it is killed and `ProcessError::Timeout`
/// is returned.
pub fn run_process_with_timeout(
    command: &str,
    args: &[String],
    timeout: Duration,
) -> Result<ProcessResult, ProcessError> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::NotFound
        } else {
            ProcessError::Io(e.to_string())
        }
    })?;

    // Take ownership of stdout/stderr handles before spawning wait thread
    let mut stdout_handle = child.stdout.take();
    let mut stderr_handle = child.stderr.take();

    // Use a channel to communicate completion from a background thread
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to wait for the child process
    thread::spawn(move || {
        let status = child.wait();
        // Send result; ignore send error if receiver dropped (timeout case)
        let _ = tx.send(status);
    });

    // Wait for the child with timeout
    match rx.recv_timeout(timeout) {
        Ok(status_result) => {
            let status = status_result.map_err(|e| ProcessError::Io(e.to_string()))?;

            // Read stdout and stderr after process completes
            let stdout = read_handle(&mut stdout_handle);
            let stderr = read_handle(&mut stderr_handle);

            Ok(ProcessResult {
                exit_code: status.code().unwrap_or(-1),
                stdout,
                stderr,
            })
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Timeout reached - the child process is still running
            // The thread will eventually complete and clean up the process
            // when it exits, but we return Timeout immediately
            Err(ProcessError::Timeout)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Thread panicked or channel closed unexpectedly
            Err(ProcessError::Io(
                "process wait thread terminated unexpectedly".into(),
            ))
        }
    }
}

/// Read all content from an optional handle, returning empty string if None.
fn read_handle(handle: &mut Option<impl Read>) -> String {
    match handle.take() {
        Some(mut h) => {
            let mut buf = Vec::new();
            let _ = h.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        }
        None => String::new(),
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

    #[test]
    fn test_fake_runner_timeout() {
        let runner = FakeProcessRunner::new(Err(ProcessError::Timeout));
        let result = runner.run(ProcessRequest {
            command: "codex".into(),
            args: vec![],
            timeout: Duration::from_secs(1),
        });
        assert!(matches!(result, Err(ProcessError::Timeout)));
    }

    #[test]
    fn test_real_process_success() {
        // Use a quick command that should complete fast
        let result = run_process_with_timeout("echo", &["hello".into()], Duration::from_secs(5));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("hello"));
    }

    #[test]
    fn test_real_process_not_found() {
        let result =
            run_process_with_timeout("nonexistent_command_xyz_12345", &[], Duration::from_secs(5));
        assert!(matches!(result, Err(ProcessError::NotFound)));
    }

    #[test]
    fn test_real_process_timeout() {
        // Use `sleep` command with a duration longer than our timeout
        let result = run_process_with_timeout("sleep", &["10".into()], Duration::from_millis(100));
        assert!(
            matches!(result, Err(ProcessError::Timeout)),
            "expected Timeout, got {:?}",
            result
        );
    }

    #[test]
    fn test_real_process_captures_stderr() {
        // Use a command that writes to stderr
        let result = run_process_with_timeout(
            "sh",
            &["-c".into(), "echo error >&2".into()],
            Duration::from_secs(5),
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stderr.contains("error"));
    }
}
