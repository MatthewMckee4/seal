use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ProjectError;

pub fn find_git_root(start_dir: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()?;

    if !output.status.success() {
        return Err(ProjectError::NotInGitRepository {
            path: start_dir.to_path_buf(),
        }
        .into());
    }

    let path_str = String::from_utf8(output.stdout)?.trim().to_string();

    Ok(PathBuf::from(path_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_find_git_root() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let subdir = repo_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let root = find_git_root(&subdir).unwrap();
        let canonicalised_root = dunce::canonicalize(root).unwrap();
        let canonicalised_repo_dir = dunce::canonicalize(repo_dir).unwrap();
        assert_eq!(canonicalised_root, canonicalised_repo_dir);
    }

    #[test]
    fn test_find_git_root_in_root() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let root = find_git_root(repo_dir).unwrap();

        let canonicalised_root = dunce::canonicalize(root).unwrap();
        let canonicalised_repo_dir = dunce::canonicalize(repo_dir).unwrap();
        assert_eq!(canonicalised_root, canonicalised_repo_dir);
    }

    #[test]
    fn test_not_in_git_repo() {
        let temp = TempDir::new().unwrap();
        let result = find_git_root(temp.path());
        assert!(result.is_err());
    }
}
