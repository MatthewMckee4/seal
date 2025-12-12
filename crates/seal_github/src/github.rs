use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use thiserror::Error;

mod client;
mod mock;

pub use client::GitHubClient;
pub use mock::MockGithubClient;

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
    pub url: String,
    pub labels: Vec<String>,
    pub author: Option<String>,
    pub merged_at: DateTime<Utc>,
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
