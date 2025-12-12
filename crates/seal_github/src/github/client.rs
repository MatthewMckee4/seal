use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use octocrab::{Octocrab, models::pulls::PullRequest};

use crate::{
    create_pull_request,
    github::{GitHubError, GitHubPullRequest, GitHubRelease, GitHubService},
    push_branch,
};

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
                    .filter_map(gh_pr_to_github_pull_request)
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
                    .filter_map(gh_pr_to_github_pull_request)
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

fn gh_pr_to_github_pull_request(pr: PullRequest) -> Option<GitHubPullRequest> {
    pr.merged_at.and_then(|merged_at| {
        pr.html_url.map(|url| GitHubPullRequest {
            title: pr.title.unwrap_or_default(),
            number: pr.number,
            url: url.to_string(),
            labels: pr
                .labels
                .map(|labels| labels.iter().map(|l| l.name.clone()).collect())
                .unwrap_or_default(),
            author: pr.user.map(|u| u.login),
            merged_at,
        })
    })
}
