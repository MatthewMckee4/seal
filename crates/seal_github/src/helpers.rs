use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub fn get_git_remote_url<P: AsRef<Path>>(current_directory: P) -> Result<String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(current_directory)
        .output()
        .context("Failed to execute git config")?;

    let url = String::from_utf8(output.stdout)
        .context("Git remote URL is not valid UTF-8")?
        .trim()
        .to_string();

    if url.is_empty() {
        anyhow::bail!("No remote URL found");
    }

    Ok(url)
}

pub fn parse_github_repo(repo_url: &str) -> Result<(String, String)> {
    let url = repo_url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string();

    let parts: Vec<&str> = if url.starts_with("https://github.com/") {
        url.trim_start_matches("https://github.com/")
            .split('/')
            .collect()
    } else if url.starts_with("git@github.com:") {
        url.trim_start_matches("git@github.com:")
            .split('/')
            .collect()
    } else {
        anyhow::bail!("Invalid GitHub repository URL: {repo_url}");
    };

    if parts.len() != 2 {
        anyhow::bail!("Invalid GitHub repository URL: {repo_url}");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_parse_github_repo_https() {
        let (owner, repo) = parse_github_repo("https://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_https_with_git() {
        let (owner, repo) = parse_github_repo("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_ssh() {
        let (owner, repo) = parse_github_repo("git@github.com:owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_ssh_with_git() {
        let (owner, repo) = parse_github_repo("git@github.com:owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_invalid() {
        assert!(parse_github_repo("https://example.com/owner/repo").is_err());
        assert!(parse_github_repo("https://github.com/owner/repo/other.git").is_err());
        assert!(parse_github_repo("not-a-url").is_err());
    }

    #[test]
    fn test_get_git_remote_url() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/user/repo.git",
            ])
            .current_dir(repo_path)
            .output()?;

        let url = get_git_remote_url(repo_path)?;
        assert_eq!(url, "https://github.com/user/repo.git");

        Ok(())
    }

    #[test]
    fn test_get_git_remote_url_no_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = get_git_remote_url(repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_git_remote_url_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();

        let result = get_git_remote_url(temp_dir.path());
        assert!(result.is_err());
    }
}
