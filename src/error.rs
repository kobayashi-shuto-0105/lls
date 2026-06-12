use std::path::PathBuf;
use thiserror::Error;

/// Application-level errors with exit code mapping.
///
/// Each variant maps to a single exit code defined in the spec.
/// No module returns arbitrary integer exit codes.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("CLI argument error: {0}")]
    Cli(String),

    #[error("target path does not exist: {path}")]
    TargetNotFound { path: PathBuf },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("unexpected runtime error: {0}")]
    Runtime(String),

    #[error("project configuration was not found")]
    SetupRequired,

    #[error("Codex setup failed: {0}")]
    Codex(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

impl AppError {
    /// Returns the exit code for this error as defined in the spec.
    ///
    /// | code | meaning                     |
    /// |-----:|-----------------------------|
    /// |   1  | CLI argument error          |
    /// |   2  | target missing              |
    /// |   3  | permission denied           |
    /// |   4  | unexpected runtime error    |
    /// |   5  | setup required              |
    /// |   6  | Codex/setup failure         |
    /// |   7  | invalid config              |
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Cli(_) => 1,
            Self::TargetNotFound { .. } => 2,
            Self::PermissionDenied { .. } => 3,
            Self::Runtime(_) => 4,
            Self::SetupRequired => 5,
            Self::Codex(_) => 6,
            Self::InvalidConfig(_) => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_exit_code_mapping() {
        assert_eq!(AppError::Cli("bad arg".into()).exit_code(), 1);
        assert_eq!(
            AppError::TargetNotFound {
                path: PathBuf::from("x")
            }
            .exit_code(),
            2
        );
        assert_eq!(
            AppError::PermissionDenied {
                path: PathBuf::from("x")
            }
            .exit_code(),
            3
        );
        assert_eq!(AppError::Runtime("oops".into()).exit_code(), 4);
        assert_eq!(AppError::SetupRequired.exit_code(), 5);
        assert_eq!(AppError::Codex("fail".into()).exit_code(), 6);
        assert_eq!(AppError::InvalidConfig("bad".into()).exit_code(), 7);
    }
}
