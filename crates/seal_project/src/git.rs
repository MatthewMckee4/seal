use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ProjectError;

const DEFAULT_REMOTE: &str = "origin";

pub fn find_git_root(start_dir: &Path) -> Result<PathBuf, ProjectError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()
        .map_err(|e| ProjectError::GitCommandFailed {
            command: "git rev-parse --show-toplevel".to_string(),
            stderr: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ProjectError::NotInGitRepository {
            path: start_dir.to_path_buf(),
        });
    }

    let path_str = String::from_utf8(output.stdout)
        .map_err(|_| ProjectError::GitCommandFailed {
            command: "git rev-parse --show-toplevel".to_string(),
            stderr: "Git output is not valid UTF-8".to_string(),
        })?
        .trim()
        .to_string();

    Ok(PathBuf::from(path_str))
}

pub fn get_base_branch() -> Result<String, ProjectError> {
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .map_err(|e| ProjectError::GitCommandFailed {
            command: "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            stderr: e.to_string(),
        })?;

    if !output.status.success() {
        return Ok("main".to_string());
    }

    let full_ref = String::from_utf8(output.stdout)
        .map_err(|_| ProjectError::GitCommandFailed {
            command: "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            stderr: "Git output is not valid UTF-8".to_string(),
        })?
        .trim()
        .to_string();

    let branch = full_ref
        .strip_prefix("refs/remotes/origin/")
        .unwrap_or(&full_ref)
        .to_string();

    Ok(branch)
}

pub fn get_remote() -> Result<String, ProjectError> {
    let output = Command::new("git").args(["remote"]).output().map_err(|e| {
        ProjectError::GitCommandFailed {
            command: "git remote".to_string(),
            stderr: e.to_string(),
        }
    })?;

    if !output.status.success() {
        return Ok(DEFAULT_REMOTE.to_string());
    }

    let remotes = String::from_utf8(output.stdout)
        .map_err(|_| ProjectError::GitCommandFailed {
            command: "git remote".to_string(),
            stderr: "Git output is not valid UTF-8".to_string(),
        })?
        .trim()
        .to_string();

    if remotes.is_empty() {
        return Ok(DEFAULT_REMOTE.to_string());
    }

    for line in remotes.lines() {
        if line == DEFAULT_REMOTE {
            return Ok(DEFAULT_REMOTE.to_string());
        }
    }

    Ok(remotes.lines().next().unwrap_or(DEFAULT_REMOTE).to_string())
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
        assert_eq!(root, repo_dir.canonicalize().unwrap());
    }

    #[test]
    fn test_find_git_root_in_root() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        let root = find_git_root(repo_dir).unwrap();
        assert_eq!(root, repo_dir.canonicalize().unwrap());
    }

    #[test]
    fn test_not_in_git_repo() {
        let temp = TempDir::new().unwrap();
        let result = find_git_root(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_remote_defaults_to_origin() {
        let result = get_remote();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_base_branch_defaults_to_main() {
        let result = get_base_branch();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_remote_with_git_repo() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/test/test.git",
            ])
            .current_dir(repo_dir)
            .output()
            .unwrap();

        std::env::set_current_dir(repo_dir).unwrap();
        let remote = get_remote().unwrap();
        assert_eq!(remote, "origin");
    }

    #[test]
    fn test_get_remote_prefers_origin_over_others() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        Command::new("git")
            .args([
                "remote",
                "add",
                "upstream",
                "https://github.com/upstream/test.git",
            ])
            .current_dir(repo_dir)
            .output()
            .unwrap();

        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/test/test.git",
            ])
            .current_dir(repo_dir)
            .output()
            .unwrap();

        std::env::set_current_dir(repo_dir).unwrap();
        let remote = get_remote().unwrap();
        assert_eq!(remote, "origin");
    }

    #[test]
    fn test_get_remote_fallback_when_no_remotes() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path();
        setup_git_repo(repo_dir);

        std::env::set_current_dir(repo_dir).unwrap();
        let remote = get_remote().unwrap();
        assert_eq!(remote, "origin");
    }
}
