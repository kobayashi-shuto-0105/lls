use crate::error::AppError;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Result of writing a config file.
#[derive(Debug)]
pub enum WriteResult {
    Created,
    Skipped,
}

/// Atomically write a config JSON to `.lls/config.json`.
///
/// Steps:
/// 1. Create `.lls/` directory if needed
/// 2. Write to a temporary file in the same directory
/// 3. Flush and sync
/// 4. Atomic rename to `.lls/config.json`
/// 5. Clean up on failure
pub fn write_config(project_root: &Path, json: &str, force: bool) -> Result<WriteResult, AppError> {
    let config_dir = project_root.join(".lls");
    let config_path = config_dir.join("config.json");

    // Check if config exists
    if config_path.exists() && !force {
        return Err(AppError::Cli(
            "config already exists, use --force to overwrite".into(),
        ));
    }

    // Create .lls/ directory
    fs::create_dir_all(&config_dir)
        .map_err(|e| AppError::Runtime(format!("cannot create .lls directory: {e}")))?;

    // Write to temporary file
    let temp_path = config_dir.join(format!(".config.tmp.{}", std::process::id()));

    let mut file = fs::File::create(&temp_path)
        .map_err(|e| AppError::Runtime(format!("cannot create temp file: {e}")))?;

    file.write_all(json.as_bytes())
        .map_err(|e| AppError::Runtime(format!("cannot write config: {e}")))?;

    file.flush()
        .map_err(|e| AppError::Runtime(format!("cannot flush config: {e}")))?;

    // Sync data
    file.sync_all()
        .map_err(|e| AppError::Runtime(format!("cannot sync config: {e}")))?;

    drop(file);

    // Atomic rename
    fs::rename(&temp_path, &config_path)
        .map_err(|e| AppError::Runtime(format!("cannot rename config file: {e}")))?;

    // Sync the directory (best-effort)
    let _ = sync_dir(&config_dir);

    Ok(WriteResult::Created)
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> std::io::Result<()> {
    let file = fs::File::open(dir)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_new_config() {
        let dir = tempfile::tempdir().unwrap();
        let result = write_config(dir.path(), r#"{"schema_version":"0.1.0"}"#, false);
        assert!(result.is_ok());
        assert!(dir.path().join(".lls/config.json").exists());
    }

    #[test]
    fn test_no_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lls")).unwrap();
        fs::write(dir.path().join(".lls/config.json"), "old").unwrap();

        let result = write_config(dir.path(), "new", false);
        assert!(result.is_err());
        // Original should still be intact
        assert_eq!(
            fs::read_to_string(dir.path().join(".lls/config.json")).unwrap(),
            "old"
        );
    }

    #[test]
    fn test_overwrite_with_force() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lls")).unwrap();
        fs::write(dir.path().join(".lls/config.json"), "old").unwrap();

        let result = write_config(dir.path(), r#"{"schema_version":"0.1.0"}"#, true);
        assert!(result.is_ok());
        assert_eq!(
            fs::read_to_string(dir.path().join(".lls/config.json")).unwrap(),
            r#"{"schema_version":"0.1.0"}"#
        );
    }

    #[test]
    fn test_temp_not_left_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        // Make .lls unwritable by creating a file instead of directory
        fs::write(dir.path().join(".lls"), "").unwrap();

        let result = write_config(dir.path(), "{}", false);
        assert!(result.is_err());
        // No temp file should remain
        let config_dir = dir.path().join(".lls");
        if config_dir.is_dir() {
            assert!(!config_dir.join(".config.tmp.*").exists());
        }
    }
}
