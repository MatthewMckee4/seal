use anyhow::{Context, Result};
use std::process::Command;

pub fn get_git_remote_url() -> Result<String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .context("Failed to execute git config")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git remote URL");
    }

    let url = String::from_utf8(output.stdout)
        .context("Git remote URL is not valid UTF-8")?
        .trim()
        .to_string();

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

pub fn push_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "-u", "origin", branch_name])
        .output()
        .context("Failed to execute git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to push branch: {stderr}");
    }

    Ok(())
}

pub fn create_pull_request(version: &str) -> Result<()> {
    let title = format!("Release v{version}");
    let body = format!("Automated release for version {version}");

    let output = Command::new("gh")
        .args(["pr", "create", "--title", &title, "--body", &body])
        .output()
        .context("Failed to execute gh pr create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create pull request: {stderr}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
        assert!(parse_github_repo("not-a-url").is_err());
    }
}
