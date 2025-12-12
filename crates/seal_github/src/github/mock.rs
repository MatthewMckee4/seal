use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};

use crate::github::{GitHubPullRequest, GitHubRelease, GitHubService, filter_prs_by_date_range};

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
