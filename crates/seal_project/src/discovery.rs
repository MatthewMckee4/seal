use std::path::{Path, PathBuf};

use crate::git::find_git_root;
use crate::{Config, ProjectError};

const CONFIG_FILE_NAME: &str = "seal.toml";

pub fn find_project_config(start_dir: Option<&Path>) -> Result<Option<Config>, ProjectError> {
    let start_dir = start_dir.map_or_else(
        || {
            std::env::current_dir().map_err(|e| ProjectError::ConfigFileNotReadable {
                path: PathBuf::from("."),
                source: e,
            })
        },
        |p| Ok(p.to_path_buf()),
    )?;

    if let Some(config_path) = find_config_file(&start_dir)? {
        let config = Config::from_file(&config_path)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}

fn find_config_file(start_dir: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let git_root = find_git_root(start_dir).map_err(|_| ProjectError::NotInGitRepository {
        path: start_dir.to_path_buf(),
    })?;

    let mut current = start_dir;
    loop {
        let config_path = current.join(CONFIG_FILE_NAME);
        if config_path.is_file() {
            return Ok(Some(config_path));
        }

        if current == git_root {
            break;
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }

        if current == git_root {
            let config_path = current.join(CONFIG_FILE_NAME);
            if config_path.is_file() {
                return Ok(Some(config_path));
            }
            break;
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_find_config_in_current_dir() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let config_path = repo_dir.join("seal.toml");
        fs::write(
            &config_path,
            "[release]\ncurrent-version = \"1.0.0\"\nversion-files = [\"test.txt\"]",
        )
        .unwrap();

        let config = find_project_config(Some(repo_dir)).unwrap();
        assert!(config.is_some());
        assert_json_snapshot!(config.unwrap(), @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.0.0",
            "version-files": [
              "test.txt"
            ],
            "commit-message": "Release v{version}",
            "branch-name": "release/v{version}",
            "tag-format": "v{version}"
          }
        }
        "#);
    }

    #[test]
    fn test_find_config_in_parent_dir() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let config_path = repo_dir.join("seal.toml");
        fs::write(
            &config_path,
            "[release]\ncurrent-version = \"1.0.0\"\nversion-files = [\"parent.txt\"]",
        )
        .unwrap();

        let subdir = repo_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let config = find_project_config(Some(&subdir)).unwrap();
        assert!(config.is_some());
        assert_json_snapshot!(config.unwrap(), @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.0.0",
            "version-files": [
              "parent.txt"
            ],
            "commit-message": "Release v{version}",
            "branch-name": "release/v{version}",
            "tag-format": "v{version}"
          }
        }
        "#);
    }

    #[test]
    fn test_no_config_file() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let config = find_project_config(Some(repo_dir)).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_not_in_git_repository() {
        let temp = TempDir::new().unwrap();
        let non_git_dir = temp.path();

        let result = find_project_config(Some(non_git_dir));
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ProjectError::NotInGitRepository { .. }));
    }

    #[test]
    fn test_stops_at_git_root() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let deep_dir = repo_dir.join("a").join("b").join("c");
        fs::create_dir_all(&deep_dir).unwrap();

        let config_path = repo_dir.join("seal.toml");
        fs::write(
            &config_path,
            "[release]\ncurrent-version = \"1.0.0\"\nversion-files = [\"root.txt\"]",
        )
        .unwrap();

        let config = find_project_config(Some(&deep_dir)).unwrap();
        assert!(config.is_some());
        assert_json_snapshot!(config.unwrap(), @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.0.0",
            "version-files": [
              "root.txt"
            ],
            "commit-message": "Release v{version}",
            "branch-name": "release/v{version}",
            "tag-format": "v{version}"
          }
        }
        "#);
    }
}
