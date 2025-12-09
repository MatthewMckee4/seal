use anyhow::Result;
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use thiserror::Error;

mod helpers;

pub use helpers::{create_pull_request, get_git_remote_url, parse_github_repo, push_branch};

pub trait GitHubService: Send + Sync {
    fn get_latest_release(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitHubRelease>> + Send + '_>>;

    fn get_prs_since_release(
        &self,
        since: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    >;

    fn push_branch(&self, branch_name: &str) -> Result<()>;

    fn create_pull_request(&self, version: &str) -> Result<()>;
}

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("No releases found for {owner}/{repo}")]
    NoReleasesFound { owner: String, repo: String },
}

pub struct GitHubRelease {
    pub created_at: DateTime<Utc>,
}

pub struct GitHubPullRequest {
    pub title: String,
    pub number: u64,
    pub url: Option<String>,
    pub labels: Vec<String>,
    pub author: Option<String>,
}

#[derive(Debug)]
pub struct GitHubClient {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubService for GitHubClient {
    fn get_latest_release(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitHubRelease>> + Send + '_>>
    {
        Box::pin(async move {
            let releases = self
                .octocrab
                .repos(&self.owner, &self.repo)
                .releases()
                .list()
                .per_page(1)
                .send()
                .await?;

            Ok(releases
                .items
                .first()
                .and_then(|r| r.created_at.map(|dt| GitHubRelease { created_at: dt }))
                .ok_or(GitHubError::NoReleasesFound {
                    owner: self.owner.clone(),
                    repo: self.repo.clone(),
                })?)
        })
    }

    fn get_prs_since_release(
        &self,
        since: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        let since = since.copied();
        Box::pin(async move {
            let mut page = 1u32;
            let mut all_prs = Vec::new();

            loop {
                let prs = self
                    .octocrab
                    .pulls(&self.owner, &self.repo)
                    .list()
                    .state(octocrab::params::State::Closed)
                    .per_page(100)
                    .page(page)
                    .send()
                    .await?;

                if prs.items.is_empty() {
                    break;
                }

                for pr in prs.items {
                    if pr.merged_at.is_none() {
                        continue;
                    }

                    if let Some(since) = since {
                        if let Some(merged_date) = &pr.merged_at {
                            if merged_date <= &since {
                                return Ok(all_prs);
                            }
                        }
                    }

                    all_prs.push(GitHubPullRequest {
                        title: pr.title.unwrap_or_default(),
                        number: pr.number,
                        url: pr.html_url.map(|u| u.to_string()),
                        labels: pr
                            .labels
                            .map(|labels| labels.iter().map(|l| l.name.clone()).collect())
                            .unwrap_or_default(),
                        author: pr.user.map(|u| u.login),
                    });
                }

                page += 1;
            }

            Ok(all_prs)
        })
    }

    fn push_branch(&self, branch_name: &str) -> Result<()> {
        push_branch(branch_name)
    }

    fn create_pull_request(&self, version: &str) -> Result<()> {
        create_pull_request(version)
    }
}

impl GitHubClient {
    pub fn new(owner: String, repo: String) -> Result<Self> {
        let github_token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .ok();

        let mut octocrab = Octocrab::builder();

        if let Some(token) = github_token {
            octocrab = octocrab.personal_token(token);
        }

        let octocrab = octocrab.build()?;

        Ok(Self {
            octocrab,
            owner,
            repo,
        })
    }
}

#[derive(Default, Clone, Copy)]
pub struct MockGithubClient;

impl GitHubService for MockGithubClient {
    fn get_latest_release(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitHubRelease>> + Send + '_>>
    {
        Box::pin(async {
            use chrono::TimeZone;

            Ok(GitHubRelease {
                created_at: Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap(),
            })
        })
    }

    fn get_prs_since_release(
        &self,
        since: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        let since = since.copied();
        Box::pin(async move {
            use chrono::TimeZone;

            let mock_prs = vec![
                (
                    Utc.with_ymd_and_hms(2025, 12, 8, 10, 0, 0).unwrap(),
                    GitHubPullRequest {
                        title: "Add new feature X".to_string(),
                        number: 4,
                        url: Some("https://github.com/owner/repo/pull/123".to_string()),
                        labels: vec!["feature".to_string(), "enhancement".to_string()],
                        author: Some("alice".to_string()),
                    },
                ),
                (
                    Utc.with_ymd_and_hms(2025, 12, 5, 0, 0, 0).unwrap(),
                    GitHubPullRequest {
                        title: "Fix critical bug in module Y".to_string(),
                        number: 3,
                        url: Some("https://github.com/owner/repo/pull/122".to_string()),
                        labels: vec!["bug".to_string()],
                        author: Some("bob".to_string()),
                    },
                ),
                (
                    Utc.with_ymd_and_hms(2025, 12, 3, 0, 0, 0).unwrap(),
                    GitHubPullRequest {
                        title: "Update documentation".to_string(),
                        number: 2,
                        url: Some("https://github.com/owner/repo/pull/121".to_string()),
                        labels: vec!["documentation".to_string()],
                        author: Some("joe".to_string()),
                    },
                ),
                (
                    Utc.with_ymd_and_hms(2025, 11, 25, 0, 0, 0).unwrap(),
                    GitHubPullRequest {
                        title: "Update documentation".to_string(),
                        number: 1,
                        url: Some("https://github.com/owner/repo/pull/121".to_string()),
                        labels: vec!["documentation".to_string()],
                        author: Some("alice".to_string()),
                    },
                ),
            ];

            let filtered_prs: Vec<GitHubPullRequest> = mock_prs
                .into_iter()
                .filter(|(merged_at, _)| {
                    if let Some(since_date) = since {
                        merged_at > &since_date
                    } else {
                        true
                    }
                })
                .map(|(_, pr)| pr)
                .collect();

            Ok(filtered_prs)
        })
    }

    fn push_branch(&self, _branch_name: &str) -> Result<()> {
        Ok(())
    }

    fn create_pull_request(&self, _version: &str) -> Result<()> {
        Ok(())
    }
}
