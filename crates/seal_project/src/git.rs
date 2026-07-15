use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;

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

pub fn get_current_branch(current_directory: &Path) -> anyhow::Result<String> {
    const COMMAND: &str = "git symbolic-ref --short HEAD";

    let output = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(current_directory)
        .output()
        .context("Failed to determine current Git branch")?;

    if !output.status.success() {
        return Err(ProjectError::GitCommandFailed {
            command: COMMAND.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
        .into());
    }

    let branch = String::from_utf8(output.stdout)
        .context("Current Git branch is not valid UTF-8")?
        .trim()
        .to_string();

    if branch.is_empty() {
        return Err(ProjectError::GitCommandFailed {
            command: COMMAND.to_string(),
            stderr: "Git returned an empty branch name".to_string(),
        }
        .into());
    }

    Ok(branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
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

    #[test]
    fn test_get_current_branch() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let branch = get_current_branch(repo_dir).unwrap();

        assert_eq!(branch, "main");
    }

    #[test]
    fn test_get_current_branch_detached_head() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);
        fs::write(repo_dir.join("README.md"), "# Test").unwrap();

        let add = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(repo_dir)
            .output()
            .unwrap();
        assert!(add.status.success());

        let commit = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_dir)
            .output()
            .unwrap();
        assert!(commit.status.success());

        let checkout = Command::new("git")
            .args(["checkout", "--detach"])
            .current_dir(repo_dir)
            .output()
            .unwrap();
        assert!(checkout.status.success());

        let error = get_current_branch(repo_dir).unwrap_err();

        assert!(error.to_string().contains("git symbolic-ref --short HEAD"));
    }
}
