use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LlsError {
    Cli(String),
    NotFound(String),
    PermissionDenied(String),
    Io(io::Error),
    Unknown(String),
}

impl fmt::Display for LlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlsError::Cli(msg) => write!(f, "CLI error: {msg}"),
            LlsError::NotFound(path) => write!(f, "path not found: {path}"),
            LlsError::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            LlsError::Io(e) => write!(f, "I/O error: {e}"),
            LlsError::Unknown(msg) => write!(f, "error: {msg}"),
        }
    }
}

impl From<io::Error> for LlsError {
    fn from(e: io::Error) -> Self {
        LlsError::Io(e)
    }
}

impl LlsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            LlsError::Cli(_) => 1,
            LlsError::NotFound(_) => 2,
            LlsError::PermissionDenied(_) => 3,
            LlsError::Io(_) | LlsError::Unknown(_) => 4,
        }
    }
}
