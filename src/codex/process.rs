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
    use std::sync::{Arc, Mutex};

    let mut cmd = std::process::Command::new(command);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::NotFound
        } else {
            ProcessError::Io(e.to_string())
        }
    })?;

    // Start draining stdout/stderr immediately so chatty subprocesses do not
    // block on a full pipe and get misclassified as timeouts.
    let stdout_thread = child.stdout.take().map(spawn_reader_thread);
    let stderr_thread = child.stderr.take().map(spawn_reader_thread);

    // Use Arc<Mutex<>> to share the Child handle with the timeout handler
    let child = Arc::new(Mutex::new(Some(child)));
    let child_for_timeout = Arc::clone(&child);

    // Use a channel to communicate completion from a background thread
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to wait for the child process
    thread::spawn(move || {
        // Take the child from the mutex
        let mut guard = child
            .lock()
            .expect("failed to acquire lock on subprocess handle during wait");
        if let Some(mut c) = guard.take() {
            let status = c.wait();
            // Send result; ignore send error if receiver dropped (timeout case)
            let _ = tx.send(status);
        }
    });

    // Wait for the child with timeout
    match rx.recv_timeout(timeout) {
        Ok(status_result) => {
            let status = status_result.map_err(|e| ProcessError::Io(e.to_string()))?;

            let stdout = join_reader_thread(stdout_thread)?;
            let stderr = join_reader_thread(stderr_thread)?;

            Ok(ProcessResult {
                exit_code: status.code().unwrap_or(-1),
                stdout,
                stderr,
            })
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Timeout reached - kill the child process to prevent resource leaks
            let mut guard = child_for_timeout
                .lock()
                .expect("failed to acquire lock on subprocess handle during timeout cleanup");
            if let Some(mut c) = guard.take() {
                // Best-effort kill; ignore errors (process may have already exited)
                let _ = c.kill();
                // Wait to reap the zombie process
                let _ = c.wait();
            }
            let _ = join_reader_thread(stdout_thread);
            let _ = join_reader_thread(stderr_thread);
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

fn spawn_reader_thread(mut handle: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = handle.read_to_end(&mut buf);
        buf
    })
}

fn join_reader_thread(reader: Option<thread::JoinHandle<Vec<u8>>>) -> Result<String, ProcessError> {
    match reader {
        Some(handle) => handle
            .join()
            .map(|buf| String::from_utf8_lossy(&buf).to_string())
            .map_err(|_| ProcessError::Io("process output reader thread panicked".into())),
        None => Ok(String::new()),
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
    #[cfg(unix)]
    fn test_real_process_timeout() {
        // Unix-only: uses `sleep` command which is standard on Unix systems.
        // Cross-platform timeout tests are tracked in M8-02.
        let result = run_process_with_timeout("sleep", &["10".into()], Duration::from_millis(100));
        assert!(
            matches!(result, Err(ProcessError::Timeout)),
            "expected Timeout, got {:?}",
            result
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_real_process_captures_stderr() {
        // Unix-only: uses `sh` shell which is standard on Unix systems.
        // Cross-platform stderr tests are tracked in M8-02.
        let result = run_process_with_timeout(
            "sh",
            &["-c".into(), "echo error >&2".into()],
            Duration::from_secs(5),
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stderr.contains("error"));
    }

    #[test]
    #[cfg(unix)]
    fn test_real_process_large_stdout_does_not_timeout() {
        let result = run_process_with_timeout(
            "sh",
            &[
                "-c".into(),
                "python3 -c 'import sys; sys.stdout.write(\"x\" * 200000)'".into(),
            ],
            Duration::from_secs(5),
        );
        assert!(result.is_ok(), "expected success, got {:?}", result);
        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.len(), 200000);
    }
}
