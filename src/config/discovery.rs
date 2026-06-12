use std::path::{Path, PathBuf};

/// Discovers the project configuration file.
pub struct ConfigDiscovery;

impl ConfigDiscovery {
    /// Resolve the config path based on the spec's resolution order:
    ///
    /// 1. `--config <path>` (explicit)
    /// 2. `<project_root>/.lls/config.json`
    /// 3. `--no-config` (None, use built-in defaults)
    /// 4. Missing — setup required
    pub fn resolve(
        explicit_path: Option<&Path>,
        project_root: &Path,
        no_config: bool,
    ) -> Result<Option<PathBuf>, ConfigDiscoveryError> {
        if let Some(path) = explicit_path {
            return Ok(Some(path.to_path_buf()));
        }

        if no_config {
            return Ok(None);
        }

        let config_path = project_root.join(".lls").join("config.json");
        if config_path.exists() {
            Ok(Some(config_path))
        } else {
            Err(ConfigDiscoveryError::NotFound {
                root: project_root.to_path_buf(),
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigDiscoveryError {
    #[error("project configuration was not found in {root}")]
    NotFound { root: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_explicit_path() {
        let result = ConfigDiscovery::resolve(
            Some(Path::new("/tmp/myconfig.json")),
            Path::new("/tmp/project"),
            false,
        );
        assert_eq!(result.unwrap(), Some(PathBuf::from("/tmp/myconfig.json")));
    }

    #[test]
    fn test_no_config_flag() {
        let result = ConfigDiscovery::resolve(None, Path::new("/tmp/project"), true);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_found_in_project_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        fs::create_dir_all(root.join(".lls")).unwrap();
        fs::write(root.join(".lls/config.json"), "{}").unwrap();

        let result = ConfigDiscovery::resolve(None, &root, false);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = ConfigDiscovery::resolve(None, dir.path(), false);
        assert!(matches!(result, Err(ConfigDiscoveryError::NotFound { .. })));
    }
}
