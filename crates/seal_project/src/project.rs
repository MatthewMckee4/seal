use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{Config, ProjectError, ProjectName, WorkspaceMember};

#[derive(Debug, Clone)]
pub struct ProjectWorkspace {
    /// Root directory of the workspace
    root: PathBuf,

    /// Path to the workspace configuration file
    config_file: PathBuf,

    /// Configuration of the workspace
    config: Config,

    /// Members of the workspace
    members: BTreeMap<ProjectName, WorkspaceMember>,
}

impl ProjectWorkspace {
    /// Discover workspace from current directory
    pub fn discover() -> Result<Self, ProjectError> {
        let current_dir =
            std::env::current_dir().map_err(|e| ProjectError::ConfigFileNotReadable {
                path: PathBuf::from("."),
                source: e,
            })?;
        Self::from_project_path(&current_dir)
    }

    /// Load workspace from a specific config file path
    pub fn from_config_file(config_path: &Path) -> Result<Self, ProjectError> {
        let config = Config::from_file(config_path)?;
        let root = config_path
            .parent()
            .ok_or_else(|| ProjectError::ConfigFileNotReadable {
                path: config_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "config file has no parent directory",
                ),
            })?
            .to_path_buf();

        let members = Self::load_members(&root, &config)?;

        Ok(Self {
            root,
            config_file: config_path.to_path_buf(),
            config,
            members,
        })
    }

    /// Load workspace from a project directory path
    pub fn from_project_path(project_path: &Path) -> Result<Self, ProjectError> {
        let seal_toml_path = project_path.join("seal.toml");
        let config = Config::from_file(&seal_toml_path)?;

        let members = Self::load_members(project_path, &config)?;

        Ok(Self {
            root: project_path.to_path_buf(),
            config_file: seal_toml_path.clone(),
            config,
            members,
        })
    }

    fn load_members(
        root: &Path,
        config: &Config,
    ) -> Result<BTreeMap<ProjectName, WorkspaceMember>, ProjectError> {
        let mut members = BTreeMap::new();

        if let Some(config_members) = &config.members {
            for (name, relative_path) in config_members {
                let member_path = root.join(relative_path);

                if !member_path.exists() {
                    return Err(ProjectError::MemberPathNotFound {
                        member: name.to_string(),
                        path: member_path,
                    });
                }

                let member_config_path = member_path.join("seal.toml");
                if !member_config_path.exists() {
                    return Err(ProjectError::MemberMissingSealToml {
                        member: name.to_string(),
                        path: member_config_path,
                    });
                }

                let member_config = Config::from_file(&member_config_path)?;
                members.insert(
                    name.clone(),
                    WorkspaceMember::new(member_path, member_config),
                );
            }
        }

        Ok(members)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn members(&self) -> &BTreeMap<ProjectName, WorkspaceMember> {
        &self.members
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_json_snapshot};
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

        let workspace = ProjectWorkspace::from_config_file(&config_path).unwrap();
        assert_json_snapshot!(workspace.config, @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.0.0",
            "version-files": [
              "test.txt"
            ],
            "commit-message": null,
            "branch-name": null,
            "push": false,
            "create-pr": false,
            "confirm": true,
            "changelog": null
          }
        }
        "#);
        assert!(workspace.members.is_empty());
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

        let workspace = ProjectWorkspace::from_project_path(project_dir).unwrap();
        assert_json_snapshot!(workspace.config, @r#"
        {
          "members": null,
          "release": {
            "current-version": "2.0.0",
            "version-files": [
              "Cargo.toml"
            ],
            "commit-message": null,
            "branch-name": null,
            "push": false,
            "create-pr": false,
            "confirm": true,
            "changelog": null
          }
        }
        "#);
        assert!(workspace.members.is_empty());
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
        let workspace = ProjectWorkspace::discover().unwrap();
        assert_json_snapshot!(workspace.config, @r#"
        {
          "members": null,
          "release": {
            "current-version": "3.0.0",
            "commit-message": null,
            "branch-name": null,
            "push": false,
            "create-pr": false,
            "confirm": true,
            "changelog": null
          }
        }
        "#);
        assert!(workspace.members.is_empty());
    }

    #[test]
    fn test_discover_config_file_not_found() {
        let temp = TempDir::new().unwrap();
        let missing_path = temp.path().join("missing.toml");

        let result = ProjectWorkspace::from_config_file(&missing_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_invalid_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("invalid.toml");

        fs::write(&config_path, "invalid toml content [[[").unwrap();

        let result = ProjectWorkspace::from_config_file(&config_path);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "key with no value, expected `=`",
                input: Some(
                    "invalid toml content [[[",
                ),
                keys: [],
                span: Some(
                    8..8,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_workspace_with_members() {
        let temp = TempDir::new().unwrap();
        let root_dir = temp.path();

        let pkg1_dir = root_dir.join("packages/pkg1");
        let pkg2_dir = root_dir.join("packages/pkg2");
        fs::create_dir_all(&pkg1_dir).unwrap();
        fs::create_dir_all(&pkg2_dir).unwrap();

        fs::write(
            root_dir.join("seal.toml"),
            r#"
[members]
pkg1 = "packages/pkg1"
pkg2 = "packages/pkg2"

[release]
current-version = "1.0.0"
"#,
        )
        .unwrap();

        fs::write(
            pkg1_dir.join("seal.toml"),
            r#"
[release]
current-version = "0.1.0"
"#,
        )
        .unwrap();

        fs::write(
            pkg2_dir.join("seal.toml"),
            r#"
[release]
current-version = "0.2.0"
"#,
        )
        .unwrap();

        let workspace = ProjectWorkspace::from_project_path(root_dir).unwrap();
        assert_eq!(workspace.members.len(), 2);
        assert!(
            workspace
                .members
                .contains_key(&ProjectName::new("pkg1".to_string()).unwrap())
        );
        assert!(
            workspace
                .members
                .contains_key(&ProjectName::new("pkg2".to_string()).unwrap())
        );
    }

    #[test]
    fn test_workspace_member_missing_seal_toml() {
        let temp = TempDir::new().unwrap();
        let root_dir = temp.path();
        let pkg_dir = root_dir.join("packages/pkg1");
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            root_dir.join("seal.toml"),
            r#"
[members]
pkg1 = "packages/pkg1"

[release]
current-version = "1.0.0"
"#,
        )
        .unwrap();

        let result = ProjectWorkspace::from_project_path(root_dir);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ProjectError::MemberMissingSealToml { .. }));
    }

    #[test]
    fn test_workspace_member_path_not_found() {
        let temp = TempDir::new().unwrap();
        let root_dir = temp.path();

        fs::write(
            root_dir.join("seal.toml"),
            r#"
[members]
pkg1 = "packages/pkg1"

[release]
current-version = "1.0.0"
"#,
        )
        .unwrap();

        let result = ProjectWorkspace::from_project_path(root_dir);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ProjectError::MemberPathNotFound { .. }));
    }
}
