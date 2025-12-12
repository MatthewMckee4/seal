use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use octocrab::Octocrab;
use thiserror::Error;

mod helpers;

pub use helpers::{create_pull_request, get_git_remote_url, parse_github_repo, push_branch};

pub trait GitHubService: Send + Sync {
    fn get_latest_release(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitHubRelease>> + Send + '_>>;

    /// Get all releases for a repository.
    ///
    /// Sorted by creation date in ascending order.
    fn get_all_releases(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<GitHubRelease>>> + Send + '_>>;

    fn get_prs_between(
        &self,
        since: Option<&DateTime<Utc>>,
        until: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    >;

    fn get_prs(
        &self,
        max: Option<usize>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    >;

    fn push_branch(&self, current_directory: &Path, branch_name: &str) -> Result<()>;

    fn create_pull_request(&self, current_directory: &Path, version: &str) -> Result<()>;
}

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("No releases found for {owner}/{repo}")]
    NoReleasesFound { owner: String, repo: String },
}

#[derive(Debug, Clone)]
pub struct GitHubRelease {
    pub created_at: DateTime<Utc>,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GitHubPullRequest {
    pub title: String,
    pub number: u64,
    pub url: Option<String>,
    pub labels: Vec<String>,
    pub author: Option<String>,
    pub merged_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct GitHubClient {
    octocrab: Octocrab,
    owner: String,
    repo: String,
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
                .and_then(|r| {
                    r.created_at.map(|dt| GitHubRelease {
                        created_at: dt,
                        name: r.name.clone(),
                    })
                })
                .ok_or(GitHubError::NoReleasesFound {
                    owner: self.owner.clone(),
                    repo: self.repo.clone(),
                })?)
        })
    }

    fn get_all_releases(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<GitHubRelease>>> + Send + '_>>
    {
        Box::pin(async move {
            let mut page = 1u32;
            let mut all_releases = Vec::new();

            loop {
                let releases = self
                    .octocrab
                    .repos(&self.owner, &self.repo)
                    .releases()
                    .list()
                    .per_page(100)
                    .page(page)
                    .send()
                    .await?;

                if releases.items.is_empty() {
                    break;
                }

                for release in releases.items {
                    if let Some(created_at) = release.created_at {
                        all_releases.push(GitHubRelease {
                            created_at,
                            name: release.name.clone(),
                        });
                    }
                }

                page += 1;
            }

            all_releases.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            Ok(all_releases)
        })
    }

    fn get_prs_between(
        &self,
        since: Option<&DateTime<Utc>>,
        until: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        let since = since.copied();
        let until = until.copied();
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
                    .await?
                    .into_iter()
                    .filter_map(|pr| {
                        pr.merged_at.map(|merged_at| GitHubPullRequest {
                            title: pr.title.unwrap_or_default(),
                            number: pr.number,
                            url: pr.html_url.map(|u| u.to_string()),
                            labels: pr
                                .labels
                                .map(|labels| labels.iter().map(|l| l.name.clone()).collect())
                                .unwrap_or_default(),
                            author: pr.user.map(|u| u.login),
                            merged_at,
                        })
                    })
                    .collect::<Vec<_>>();

                if prs.is_empty() {
                    break;
                }

                for pr in prs {
                    if let Some(since) = since {
                        if pr.merged_at <= since {
                            return Ok(all_prs);
                        }
                    }

                    if let Some(until) = until {
                        if pr.merged_at > until {
                            continue;
                        }
                    }

                    all_prs.push(pr);
                }

                page += 1;
            }

            Ok(all_prs)
        })
    }

    fn get_prs(
        &self,
        max: Option<usize>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        Box::pin(async move {
            let mut all_prs = Vec::new();
            let per_page = 100u8;
            let max_prs = max.unwrap_or(usize::MAX);
            let mut page = 1u32;

            loop {
                let response = self
                    .octocrab
                    .pulls(&self.owner, &self.repo)
                    .list()
                    .state(octocrab::params::State::Closed)
                    .per_page(per_page)
                    .page(page)
                    .send()
                    .await?;

                let merged_prs: Vec<_> = response
                    .into_iter()
                    .filter_map(|pr| {
                        pr.merged_at.map(|merged_at| GitHubPullRequest {
                            title: pr.title.unwrap_or_default(),
                            number: pr.number,
                            url: pr.html_url.map(|u| u.to_string()),
                            labels: pr
                                .labels
                                .map(|labels| labels.iter().map(|l| l.name.clone()).collect())
                                .unwrap_or_default(),
                            author: pr.user.map(|u| u.login),
                            merged_at,
                        })
                    })
                    .collect();

                let is_empty = merged_prs.is_empty();
                all_prs.extend(merged_prs);

                // Stop if we've hit our max or if the page was empty
                if all_prs.len() >= max_prs || is_empty {
                    break;
                }

                page += 1;
            }

            all_prs.truncate(max_prs);
            Ok(all_prs)
        })
    }

    fn push_branch(&self, current_directory: &Path, branch_name: &str) -> Result<()> {
        push_branch(current_directory, branch_name)
    }

    fn create_pull_request(&self, current_directory: &Path, version: &str) -> Result<()> {
        create_pull_request(current_directory, version)
    }
}

#[derive(Default, Clone)]
pub struct MockGithubClient {
    prs: Vec<GitHubPullRequest>,
}

impl MockGithubClient {
    pub fn new() -> Self {
        let prs = vec![
            GitHubPullRequest {
                title: "Add new feature X".to_string(),
                number: 5,
                url: Some("https://github.com/owner/repo/pull/5".to_string()),
                labels: vec!["feature".to_string(), "enhancement".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 8, 10, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Fix critical bug in module Y".to_string(),
                number: 4,
                url: Some("https://github.com/owner/repo/pull/4".to_string()),
                labels: vec!["bug".to_string()],
                author: Some("bob".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 5, 0, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Update documentation".to_string(),
                number: 3,
                url: Some("https://github.com/owner/repo/pull/3".to_string()),
                labels: vec!["documentation".to_string()],
                author: Some("joe".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 3, 0, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Update documentation".to_string(),
                number: 2,
                url: Some("https://github.com/owner/repo/pull/2".to_string()),
                labels: vec!["documentation".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 11, 25, 0, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Update documentation".to_string(),
                number: 1,
                url: Some("https://github.com/owner/repo/pull/1".to_string()),
                labels: vec!["documentation".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 11, 10, 0, 0, 0).unwrap(),
            },
        ];
        Self { prs }
    }
}

impl GitHubService for MockGithubClient {
    fn get_latest_release(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GitHubRelease>> + Send + '_>>
    {
        Box::pin(async {
            use chrono::TimeZone;

            Ok(GitHubRelease {
                created_at: Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap(),
                name: Some("v1.0.0".to_string()),
            })
        })
    }

    fn get_all_releases(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<GitHubRelease>>> + Send + '_>>
    {
        Box::pin(async {
            use chrono::TimeZone;

            Ok(vec![
                GitHubRelease {
                    created_at: Utc.with_ymd_and_hms(2025, 11, 15, 0, 0, 0).unwrap(),
                    name: Some("v0.2.0".to_string()),
                },
                GitHubRelease {
                    created_at: Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap(),
                    name: Some("v1.0.0".to_string()),
                },
            ])
        })
    }

    fn get_prs_between(
        &self,
        since: Option<&DateTime<Utc>>,
        until: Option<&DateTime<Utc>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        let since = since.copied();
        let until = until.copied();
        Box::pin(async move {
            let filtered_prs = filter_prs_by_date_range(&self.prs, since.as_ref(), until.as_ref());

            Ok(filtered_prs)
        })
    }

    fn get_prs(
        &self,
        max: Option<usize>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<GitHubPullRequest>>> + Send + '_>,
    > {
        Box::pin(async move {
            let mut prs = self.prs.clone();
            if let Some(max) = max {
                prs.truncate(max);
            }
            Ok(prs)
        })
    }

    fn push_branch(&self, _current_directory: &Path, _branch_name: &str) -> Result<()> {
        Ok(())
    }

    fn create_pull_request(&self, _current_directory: &Path, _version: &str) -> Result<()> {
        Ok(())
    }
}

pub fn filter_prs_by_date_range(
    prs: &[GitHubPullRequest],
    since: Option<&DateTime<Utc>>,
    until: Option<&DateTime<Utc>>,
) -> Vec<GitHubPullRequest> {
    prs.iter()
        .filter(|pr| {
            let after_since = if let Some(since_date) = since {
                pr.merged_at > *since_date
            } else {
                true
            };

            let before_until = if let Some(until_date) = until {
                pr.merged_at <= *until_date
            } else {
                true
            };

            after_since && before_until
        })
        .cloned()
        .collect()
}
