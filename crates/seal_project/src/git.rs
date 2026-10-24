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

pub fn ensure_clean_worktree(current_directory: &Path) -> anyhow::Result<()> {
    const COMMAND: &str = "git --no-optional-locks status --porcelain=v1 --untracked-files=all --ignore-submodules=none";

    let output = Command::new("git")
        .args([
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ])
        .current_dir(current_directory)
        .output()
        .context("Failed to inspect Git state")?;

    if !output.status.success() {
        return Err(ProjectError::GitCommandFailed {
            command: COMMAND.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
        .into());
    }

    let status = String::from_utf8(output.stdout).context("Git status is not valid UTF-8")?;
    if !status.is_empty() {
        let changes = status
            .trim_end()
            .lines()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(ProjectError::DirtyGitState { changes }.into());
    }

    Ok(())
}

pub fn ensure_release_branch_available(
    current_directory: &Path,
    branch: &str,
    remote: &str,
    require_remote: bool,
) -> anyhow::Result<bool> {
    let validated_branch = Command::new("git")
        .args(["check-ref-format", "--branch", branch])
        .current_dir(current_directory)
        .output()
        .context("Failed to validate release branch name")?;
    let validated_branch_name = String::from_utf8_lossy(&validated_branch.stdout);

    if !validated_branch.status.success()
        || validated_branch_name.trim() != branch
        || branch == "HEAD"
    {
        return Err(ProjectError::InvalidGitBranch {
            branch: branch.to_string(),
        }
        .into());
    }

    let local_ref = format!("refs/heads/{branch}");
    let local = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &local_ref])
        .current_dir(current_directory)
        .output()
        .context("Failed to inspect local Git branches")?;

    match local.status.code() {
        Some(0) => {
            return Err(ProjectError::LocalGitBranchExists {
                branch: branch.to_string(),
            }
            .into());
        }
        Some(1) => {}
        _ => {
            return Err(ProjectError::GitCommandFailed {
                command: format!("git show-ref --verify --quiet {local_ref}"),
                stderr: String::from_utf8_lossy(&local.stderr).trim().to_string(),
            }
            .into());
        }
    }

    let remote_urls = Command::new("git")
        .args(["remote", "get-url", "--push", "--all", remote])
        .current_dir(current_directory)
        .output()
        .context("Failed to inspect Git remotes")?;

    if !remote_urls.status.success() || remote_urls.stdout.is_empty() {
        if require_remote {
            return Err(ProjectError::MissingGitRemote {
                remote: remote.to_string(),
            }
            .into());
        }

        return Ok(false);
    }

    let remote_ref = format!("refs/heads/{branch}");
    let remote_urls =
        String::from_utf8(remote_urls.stdout).context("Git remote URLs are not valid UTF-8")?;

    for remote_url in remote_urls.lines() {
        let remote_branch = Command::new("git")
            .args([
                "ls-remote",
                "--exit-code",
                "--heads",
                remote_url,
                &remote_ref,
            ])
            .current_dir(current_directory)
            .output()
            .context("Failed to inspect remote Git branches")?;

        match remote_branch.status.code() {
            Some(0) => {
                return Err(ProjectError::RemoteGitBranchExists {
                    branch: branch.to_string(),
                    remote: remote.to_string(),
                }
                .into());
            }
            Some(2) => {}
            _ => {
                return Err(ProjectError::GitCommandFailed {
                    command: format!("git ls-remote --exit-code --heads {remote} {remote_ref}"),
                    stderr: String::from_utf8_lossy(&remote_branch.stderr)
                        .trim()
                        .to_string(),
                }
                .into());
            }
        }
    }

    Ok(true)
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

    fn commit_file(dir: &Path) {
        fs::write(dir.join("README.md"), "# Test").unwrap();

        let add = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(add.status.success());

        let commit = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(commit.status.success());
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

    #[test]
    fn test_ensure_clean_worktree_rejects_untracked_file() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        assert!(ensure_clean_worktree(temp.path()).is_ok());

        fs::write(temp.path().join("untracked.txt"), "change").unwrap();
        let error = ensure_clean_worktree(temp.path()).unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::DirtyGitState { changes }) if changes.contains("?? untracked.txt")
        ));
    }

    #[test]
    fn test_ensure_clean_worktree_rejects_staged_file() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        fs::write(temp.path().join("staged.txt"), "change").unwrap();
        let add = Command::new("git")
            .args(["add", "staged.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(add.status.success());

        let error = ensure_clean_worktree(temp.path()).unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::DirtyGitState { changes }) if changes.contains("A  staged.txt")
        ));
    }

    #[test]
    fn test_ensure_release_branch_available_rejects_invalid_branch() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());

        let error =
            ensure_release_branch_available(temp.path(), "release/invalid branch", "origin", false)
                .unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::InvalidGitBranch { branch }) if branch == "release/invalid branch"
        ));
    }

    #[test]
    fn test_ensure_release_branch_available_rejects_local_branch() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        commit_file(temp.path());

        let error =
            ensure_release_branch_available(temp.path(), "main", "origin", false).unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::LocalGitBranchExists { branch }) if branch == "main"
        ));
        let remote_checked =
            ensure_release_branch_available(temp.path(), "release/v1.2.3", "origin", false)
                .unwrap();
        assert!(!remote_checked);
    }

    #[test]
    fn test_ensure_release_branch_available_requires_remote() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        commit_file(temp.path());

        let error = ensure_release_branch_available(temp.path(), "release/v1.2.3", "origin", true)
            .unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::MissingGitRemote { remote }) if remote == "origin"
        ));
    }

    #[test]
    fn test_ensure_release_branch_available_rejects_remote_branch() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        commit_file(temp.path());
        let remote = temp.path().join("remote.git");
        let init_remote = Command::new("git")
            .args(["init", "--bare"])
            .arg(&remote)
            .output()
            .unwrap();
        assert!(init_remote.status.success());
        let add_remote = Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&remote)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(add_remote.status.success());
        let push = Command::new("git")
            .args(["push", "origin", "HEAD:refs/heads/release/v1.2.3"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(push.status.success());

        let error = ensure_release_branch_available(temp.path(), "release/v1.2.3", "origin", false)
            .unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::RemoteGitBranchExists { branch, remote })
                if branch == "release/v1.2.3" && remote == "origin"
        ));
        assert!(
            ensure_release_branch_available(temp.path(), "release/v1.2.4", "origin", true).is_ok()
        );
    }

    #[test]
    fn test_ensure_release_branch_available_checks_push_url() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());
        commit_file(temp.path());
        let fetch_remote = temp.path().join("fetch.git");
        let push_remote = temp.path().join("push.git");

        for remote in [&fetch_remote, &push_remote] {
            let init_remote = Command::new("git")
                .args(["init", "--bare"])
                .arg(remote)
                .output()
                .unwrap();
            assert!(init_remote.status.success());
        }

        let add_remote = Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&fetch_remote)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(add_remote.status.success());

        let set_push_url = Command::new("git")
            .args(["remote", "set-url", "--push", "origin"])
            .arg(&push_remote)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(set_push_url.status.success());

        let push = Command::new("git")
            .arg("push")
            .arg(&push_remote)
            .args(["HEAD:refs/heads/release/v1.2.3"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(push.status.success());

        let error = ensure_release_branch_available(temp.path(), "release/v1.2.3", "origin", true)
            .unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ProjectError>(),
            Some(ProjectError::RemoteGitBranchExists { branch, remote })
                if branch == "release/v1.2.3" && remote == "origin"
        ));
    }
}
