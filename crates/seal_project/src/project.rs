use std::path::{Path, PathBuf};

use crate::{Config, ProjectError};

#[derive(Debug, Clone)]
pub struct Project {
    pub config: Config,
}

impl Project {
    pub fn discover(
        project_path: Option<&Path>,
        config_file: Option<&Path>,
    ) -> Result<Self, ProjectError> {
        let config = if let Some(config_path) = config_file {
            Config::from_file(config_path)?
        } else {
            let project_dir = project_path.map_or_else(
                || {
                    std::env::current_dir().map_err(|e| ProjectError::ConfigFileNotReadable {
                        path: PathBuf::from("."),
                        source: e,
                    })
                },
                |p| Ok(p.to_path_buf()),
            )?;

            let seal_toml_path = project_dir.join("seal.toml");
            Config::from_file(&seal_toml_path)?
        };

        Ok(Self { config })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_with_explicit_config_path() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("custom.toml");

        fs::write(
            &config_path,
            r#"
[release]
current-version = "1.0.0"
version-files = ["test.txt"]
"#,
        )
        .unwrap();

        let project = Project::discover(None, Some(&config_path)).unwrap();
        assert_eq!(project.config.release.current_version, "1.0.0");
        assert_eq!(project.config.release.version_files, vec!["test.txt"]);
    }

    #[test]
    fn test_discover_from_project_path() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();
        let seal_toml = project_dir.join("seal.toml");

        fs::write(
            &seal_toml,
            r#"
[release]
current-version = "2.0.0"
version-files = ["Cargo.toml"]
"#,
        )
        .unwrap();

        let project = Project::discover(Some(project_dir), None).unwrap();
        assert_eq!(project.config.release.current_version, "2.0.0");
    }

    #[test]
    fn test_discover_from_current_dir() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();
        let seal_toml = project_dir.join("seal.toml");

        fs::write(
            &seal_toml,
            r#"
[release]
current-version = "3.0.0"
"#,
        )
        .unwrap();

        std::env::set_current_dir(project_dir).unwrap();
        let project = Project::discover(None, None).unwrap();
        assert_eq!(project.config.release.current_version, "3.0.0");
    }

    #[test]
    fn test_discover_config_file_not_found() {
        let temp = TempDir::new().unwrap();
        let missing_path = temp.path().join("missing.toml");

        let result = Project::discover(None, Some(&missing_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_invalid_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("invalid.toml");

        fs::write(&config_path, "invalid toml content [[[").unwrap();

        let result = Project::discover(None, Some(&config_path));
        assert!(result.is_err());
    }
}
