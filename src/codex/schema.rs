use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The embedded config schema JSON for Codex output validation.
///
/// This is the same schema as `.github/assets/config.schema.json`.
pub const CONFIG_SCHEMA_JSON: &str = include_str!("../../.github/assets/config.schema.json");

/// Write the embedded schema to a temporary file and return the path.
///
/// The caller is responsible for cleaning up the file after use.
pub fn write_schema_temp_file(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let lls_dir = project_root.join(".lls");

    // Create .lls/ directory if it doesn't exist
    fs::create_dir_all(&lls_dir)?;

    // Create a unique temp file name
    let temp_path = lls_dir.join(format!(".codex-schema.tmp.{}", std::process::id()));

    // Write the schema
    let mut file = fs::File::create(&temp_path)?;
    file.write_all(CONFIG_SCHEMA_JSON.as_bytes())?;
    file.flush()?;
    file.sync_all()?;

    Ok(temp_path)
}

/// Remove the temporary schema file if it exists.
pub fn cleanup_schema_temp_file(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_schema_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(CONFIG_SCHEMA_JSON);
        assert!(result.is_ok(), "embedded schema must be valid JSON");
    }

    #[test]
    fn test_embedded_schema_has_required_fields() {
        let schema: serde_json::Value = serde_json::from_str(CONFIG_SCHEMA_JSON).unwrap();
        assert!(schema.get("$schema").is_some(), "must have $schema");
        assert!(schema.get("type").is_some(), "must have type");
        assert!(schema.get("properties").is_some(), "must have properties");
    }

    #[test]
    fn test_write_schema_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_schema_temp_file(dir.path()).unwrap();

        // Verify the file was created
        assert!(path.exists());

        // Verify the content is valid JSON
        let content = fs::read_to_string(&path).unwrap();
        let result: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(result.is_ok());

        // Verify it's the expected schema
        let schema: serde_json::Value = result.unwrap();
        assert_eq!(
            schema.get("title").and_then(|v| v.as_str()),
            Some("lls project configuration")
        );

        // Clean up
        cleanup_schema_temp_file(&path);
        assert!(!path.exists());
    }

    #[test]
    fn test_write_schema_creates_lls_dir() {
        let dir = tempfile::tempdir().unwrap();
        let lls_dir = dir.path().join(".lls");
        assert!(!lls_dir.exists());

        let path = write_schema_temp_file(dir.path()).unwrap();
        assert!(lls_dir.exists());
        assert!(path.exists());

        cleanup_schema_temp_file(&path);
    }

    #[test]
    fn test_cleanup_nonexistent_file_is_noop() {
        // Should not panic or error
        cleanup_schema_temp_file(Path::new("/nonexistent/path/file.json"));
    }
}
