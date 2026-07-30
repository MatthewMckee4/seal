use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use octocrab::{
    Octocrab,
    models::pulls::PullRequest,
    params::{Direction, pulls::Sort},
};

use crate::github::{
    GitHubError, GitHubPullRequest, GitHubPullRequestOptions, GitHubPullRequestReference,
    GitHubRelease, GitHubService,
};

const CONVERT_PULL_REQUEST_TO_DRAFT: &str = "mutation ConvertPullRequestToDraft($pullRequestId: ID!) {\
        convertPullRequestToDraft(input: { pullRequestId: $pullRequestId }) {\
            pullRequest { isDraft }\
        }\
    }";
const MARK_PULL_REQUEST_READY_FOR_REVIEW: &str = "mutation MarkPullRequestReadyForReview($pullRequestId: ID!) {\
        markPullRequestReadyForReview(input: { pullRequestId: $pullRequestId }) {\
            pullRequest { isDraft }\
        }\
    }";

#[derive(Debug)]
pub struct GitHubClient {
    octocrab: Octocrab,
    owner: String,
    repo: String,
    authenticated: bool,
}

impl GitHubClient {
    pub fn new(owner: String, repo: String) -> Result<Self> {
        let github_token = ["GITHUB_TOKEN", "GH_TOKEN"].into_iter().find_map(|name| {
            std::env::var(name)
                .ok()
                .filter(|token| !token.trim().is_empty())
        });
        let authenticated = github_token.is_some();

        let mut octocrab = Octocrab::builder();

        if let Some(token) = github_token {
            octocrab = octocrab.personal_token(token);
        }

        let octocrab = octocrab.build()?;

        Ok(Self {
            octocrab,
            owner,
            repo,
            authenticated,
        })
    }

    async fn update_pull_request_draft_state(&self, node_id: &str, draft: bool) -> Result<()> {
        let (action, query) = if draft {
            (
                "convert GitHub pull request to draft",
                CONVERT_PULL_REQUEST_TO_DRAFT,
            )
        } else {
            (
                "mark GitHub pull request ready for review",
                MARK_PULL_REQUEST_READY_FOR_REVIEW,
            )
        };

        let response: serde_json::Value = self
            .octocrab
            .graphql(&serde_json::json!({
                "query": query,
                "variables": { "pullRequestId": node_id },
            }))
            .await
            .with_context(|| format!("Failed to {action}"))?;

        if let Some(errors) = response.get("errors").and_then(serde_json::Value::as_array)
            && !errors.is_empty()
        {
            return Err(GitHubError::GraphQlErrors {
                action,
                errors: serde_json::Value::Array(errors.clone()).to_string(),
            }
            .into());
        }

        Ok(())
    }
}

impl GitHubService for GitHubClient {
    fn ensure_authenticated(&self) -> Result<()> {
        if !self.authenticated {
            return Err(GitHubError::AuthenticationRequired.into());
        }

        Ok(())
    }

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
                .ok_or_else(|| GitHubError::NoReleasesFound {
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

            all_releases.sort_by_key(|release| release.created_at);

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
                let response = self
                    .octocrab
                    .pulls(&self.owner, &self.repo)
                    .list()
                    .state(octocrab::params::State::Closed)
                    .sort(Sort::Updated)
                    .direction(Direction::Descending)
                    .per_page(100)
                    .page(page)
                    .send()
                    .await?;

                if response.items.is_empty() {
                    break;
                }

                for pr in response {
                    if let Some(since) = since
                        && let Some(updated_at) = pr.updated_at.as_ref()
                        && *updated_at <= since
                    {
                        return Ok(all_prs);
                    }

                    let Some(pr) = gh_pr_to_github_pull_request(pr) else {
                        continue;
                    };

                    if let Some(since) = since
                        && pr.merged_at <= since
                    {
                        continue;
                    }

                    if let Some(until) = until
                        && pr.merged_at > until
                    {
                        continue;
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

    fn create_or_update_pull_request(
        &self,
        options: GitHubPullRequestOptions,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GitHubPullRequestReference>> + Send + '_>,
    > {
        Box::pin(async move {
            self.ensure_authenticated()?;

            let pull_requests = self.octocrab.pulls(&self.owner, &self.repo);
            let existing = pull_requests
                .list()
                .state(octocrab::params::State::Open)
                .head(format!("{}:{}", self.owner, options.head))
                .base(options.base.as_str())
                .per_page(1)
                .send()
                .await
                .context("Failed to find an existing GitHub pull request")?
                .items
                .into_iter()
                .next();

            let pull_request = if let Some(existing) = existing {
                let number = existing.number;
                let node_id = existing.node_id;
                let draft_state_changed = existing.draft != Some(options.draft);

                let pull_request = pull_requests
                    .update(existing.number)
                    .title(options.title.as_str())
                    .body(options.body.as_str())
                    .send()
                    .await
                    .context("Failed to update GitHub pull request")?;

                if draft_state_changed {
                    let node_id =
                        node_id.ok_or(GitHubError::MissingPullRequestNodeId { number })?;
                    self.update_pull_request_draft_state(&node_id, options.draft)
                        .await?;
                }

                pull_request
            } else {
                pull_requests
                    .create(
                        options.title.as_str(),
                        options.head.as_str(),
                        options.base.as_str(),
                    )
                    .body(options.body.as_str())
                    .draft(options.draft)
                    .send()
                    .await
                    .context("Failed to create GitHub pull request")?
            };

            let number = pull_request.number;
            let url = pull_request
                .html_url
                .ok_or(GitHubError::MissingPullRequestUrl { number })?;

            Ok(GitHubPullRequestReference {
                url: url.to_string(),
            })
        })
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

#[cfg(test)]
mod tests {
    use anyhow::{Context as _, Result};
    use chrono::{DateTime, Utc};
    use octocrab::Octocrab;
    use serde_json::{Value, json};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, method, path, query_param},
    };

    use super::{CONVERT_PULL_REQUEST_TO_DRAFT, GitHubClient, MARK_PULL_REQUEST_READY_FOR_REVIEW};
    use crate::github::{GitHubError, GitHubPullRequestOptions, GitHubService};

    const OWNER: &str = "owner";
    const REPO: &str = "repo";
    const TITLE: &str = "Release v1.2.4";
    const BODY: &str = "Release notes";
    const HEAD: &str = "release/v1.2.4";
    const BASE: &str = "main";
    const PULL_NUMBER: u64 = 8;
    const NODE_ID: &str = "PR_8";

    fn test_client(server: &MockServer, authenticated: bool) -> Result<GitHubClient> {
        let octocrab = Octocrab::builder().base_uri(server.uri())?.build()?;

        Ok(GitHubClient {
            octocrab,
            owner: OWNER.to_string(),
            repo: REPO.to_string(),
            authenticated,
        })
    }

    fn options(draft: bool) -> GitHubPullRequestOptions {
        GitHubPullRequestOptions {
            title: TITLE.to_string(),
            body: BODY.to_string(),
            head: HEAD.to_string(),
            base: BASE.to_string(),
            draft,
        }
    }

    fn pull_request(number: u64, draft: bool, node_id: Option<&str>) -> Value {
        json!({
            "url": format!("https://api.github.com/repos/{OWNER}/{REPO}/pulls/{number}"),
            "id": number,
            "node_id": node_id,
            "html_url": format!("https://github.com/{OWNER}/{REPO}/pull/{number}"),
            "number": number,
            "locked": false,
            "maintainer_can_modify": true,
            "head": {
                "ref": HEAD,
                "sha": "head-sha",
            },
            "base": {
                "ref": BASE,
                "sha": "base-sha",
            },
            "draft": draft,
        })
    }

    fn closed_pull_request(
        number: u64,
        created_at: &str,
        updated_at: &str,
        merged_at: Option<&str>,
    ) -> Value {
        let mut pull_request = pull_request(number, false, None);
        pull_request["created_at"] = json!(created_at);
        pull_request["updated_at"] = json!(updated_at);
        pull_request["closed_at"] = json!(updated_at);
        pull_request["merged_at"] = json!(merged_at);
        pull_request
    }

    async fn mount_closed_pull_requests_page(server: &MockServer, page: u32, response: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path(format!("/repos/{OWNER}/{REPO}/pulls")))
            .and(query_param("state", "closed"))
            .and(query_param("sort", "updated"))
            .and(query_param("direction", "desc"))
            .and(query_param("per_page", "100"))
            .and(query_param("page", page.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_lookup(server: &MockServer, response: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path(format!("/repos/{OWNER}/{REPO}/pulls")))
            .and(query_param("state", "open"))
            .and(query_param("head", format!("{OWNER}:{HEAD}")))
            .and(query_param("base", BASE))
            .and(query_param("per_page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_update(server: &MockServer, draft: bool) {
        Mock::given(method("PATCH"))
            .and(path(format!("/repos/{OWNER}/{REPO}/pulls/{PULL_NUMBER}")))
            .and(body_json(json!({
                "pull_number": PULL_NUMBER,
                "title": TITLE,
                "body": BODY,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(pull_request(
                PULL_NUMBER,
                draft,
                Some(NODE_ID),
            )))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_graphql(server: &MockServer, query: &str, response: Value) {
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_json(json!({
                "query": query,
                "variables": { "pullRequestId": NODE_ID },
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn assert_received_paths(server: &MockServer, expected: &[&str]) -> Result<()> {
        let actual = server
            .received_requests()
            .await
            .context("Wiremock request recording is disabled")?
            .iter()
            .map(|request| request.url.path().to_string())
            .collect::<Vec<_>>();
        let expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(actual, expected);
        Ok(())
    }

    #[tokio::test]
    async fn unauthenticated_preflight_and_write_fail_without_requests() -> Result<()> {
        let server = MockServer::start().await;
        let client = test_client(&server, false)?;

        let preflight_error = client
            .ensure_authenticated()
            .expect_err("authentication preflight should fail");
        assert!(matches!(
            preflight_error.downcast_ref::<GitHubError>(),
            Some(GitHubError::AuthenticationRequired)
        ));

        let write_error = client
            .create_or_update_pull_request(options(false))
            .await
            .expect_err("unauthenticated write should fail");
        assert!(matches!(
            write_error.downcast_ref::<GitHubError>(),
            Some(GitHubError::AuthenticationRequired)
        ));
        assert_received_paths(&server, &[]).await?;

        Ok(())
    }

    #[tokio::test]
    async fn gets_pr_created_before_release_when_merged_after_release() -> Result<()> {
        let server = MockServer::start().await;
        mount_closed_pull_requests_page(
            &server,
            1,
            vec![closed_pull_request(
                4,
                "2026-01-20T00:00:00Z",
                "2026-02-02T00:00:00Z",
                None,
            )],
        )
        .await;
        mount_closed_pull_requests_page(
            &server,
            2,
            vec![
                closed_pull_request(
                    1,
                    "2025-12-20T00:00:00Z",
                    "2026-02-01T00:00:00Z",
                    Some("2025-12-25T00:00:00Z"),
                ),
                closed_pull_request(
                    2,
                    "2025-01-01T00:00:00Z",
                    "2026-01-15T00:00:00Z",
                    Some("2026-01-15T00:00:00Z"),
                ),
                closed_pull_request(3, "2025-12-01T00:00:00Z", "2025-12-31T00:00:00Z", None),
            ],
        )
        .await;

        let since = "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
        let pull_requests = test_client(&server, true)?
            .get_prs_between(Some(&since), None)
            .await?;

        assert_eq!(
            pull_requests
                .iter()
                .map(|pull_request| pull_request.number)
                .collect::<Vec<_>>(),
            vec![2]
        );

        Ok(())
    }

    #[tokio::test]
    async fn creates_pull_request_when_none_exists() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(&server, Vec::new()).await;
        Mock::given(method("POST"))
            .and(path(format!("/repos/{OWNER}/{REPO}/pulls")))
            .and(body_json(json!({
                "title": TITLE,
                "head": HEAD,
                "base": BASE,
                "body": BODY,
                "draft": true,
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(pull_request(
                9,
                true,
                Some("PR_9"),
            )))
            .expect(1)
            .mount(&server)
            .await;

        let pull_request = test_client(&server, true)?
            .create_or_update_pull_request(options(true))
            .await?;

        assert_eq!(pull_request.url, "https://github.com/owner/repo/pull/9");
        assert_received_paths(
            &server,
            &["/repos/owner/repo/pulls", "/repos/owner/repo/pulls"],
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_failure_has_stage_specific_context() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(&server, Vec::new()).await;
        Mock::given(method("POST"))
            .and(path(format!("/repos/{OWNER}/{REPO}/pulls")))
            .and(body_json(json!({
                "title": TITLE,
                "head": HEAD,
                "base": BASE,
                "body": BODY,
                "draft": false,
            })))
            .respond_with(
                ResponseTemplate::new(422).set_body_json(json!({ "message": "Invalid request" })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let error = test_client(&server, true)?
            .create_or_update_pull_request(options(false))
            .await
            .expect_err("failed create request should return an error");
        let message = format!("{error:#}");

        assert!(message.contains("Failed to create GitHub pull request"));
        assert!(message.contains("Invalid request"));

        Ok(())
    }

    #[tokio::test]
    async fn converts_existing_ready_pull_request_to_draft() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(
            &server,
            vec![pull_request(PULL_NUMBER, false, Some(NODE_ID))],
        )
        .await;
        mount_update(&server, false).await;
        mount_graphql(
            &server,
            CONVERT_PULL_REQUEST_TO_DRAFT,
            json!({
                "data": {
                    "convertPullRequestToDraft": {
                        "pullRequest": { "isDraft": true }
                    }
                }
            }),
        )
        .await;

        test_client(&server, true)?
            .create_or_update_pull_request(options(true))
            .await?;

        assert_received_paths(
            &server,
            &[
                "/repos/owner/repo/pulls",
                "/repos/owner/repo/pulls/8",
                "/graphql",
            ],
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn marks_existing_draft_pull_request_ready_for_review() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(
            &server,
            vec![pull_request(PULL_NUMBER, true, Some(NODE_ID))],
        )
        .await;
        mount_update(&server, true).await;
        mount_graphql(
            &server,
            MARK_PULL_REQUEST_READY_FOR_REVIEW,
            json!({
                "data": {
                    "markPullRequestReadyForReview": {
                        "pullRequest": { "isDraft": false }
                    }
                }
            }),
        )
        .await;

        test_client(&server, true)?
            .create_or_update_pull_request(options(false))
            .await?;

        assert_received_paths(
            &server,
            &[
                "/repos/owner/repo/pulls",
                "/repos/owner/repo/pulls/8",
                "/graphql",
            ],
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn matching_draft_state_only_updates_title_and_body() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(
            &server,
            vec![pull_request(PULL_NUMBER, false, Some(NODE_ID))],
        )
        .await;
        mount_update(&server, false).await;

        test_client(&server, true)?
            .create_or_update_pull_request(options(false))
            .await?;

        assert_received_paths(
            &server,
            &["/repos/owner/repo/pulls", "/repos/owner/repo/pulls/8"],
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn draft_transition_requires_node_id_after_update() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(&server, vec![pull_request(PULL_NUMBER, false, None)]).await;
        mount_update(&server, false).await;

        let error = test_client(&server, true)?
            .create_or_update_pull_request(options(true))
            .await
            .expect_err("missing node ID should fail the draft transition");

        assert!(matches!(
            error.downcast_ref::<GitHubError>(),
            Some(GitHubError::MissingPullRequestNodeId {
                number: PULL_NUMBER
            })
        ));
        assert_received_paths(
            &server,
            &["/repos/owner/repo/pulls", "/repos/owner/repo/pulls/8"],
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn graphql_errors_include_draft_transition_context() -> Result<()> {
        let server = MockServer::start().await;
        mount_lookup(
            &server,
            vec![pull_request(PULL_NUMBER, false, Some(NODE_ID))],
        )
        .await;
        mount_update(&server, false).await;
        mount_graphql(
            &server,
            CONVERT_PULL_REQUEST_TO_DRAFT,
            json!({ "errors": [{ "message": "Draft pull requests are unavailable" }] }),
        )
        .await;

        let error = test_client(&server, true)?
            .create_or_update_pull_request(options(true))
            .await
            .expect_err("GraphQL errors should fail the draft transition");
        let message = error.to_string();

        assert!(message.contains("Failed to convert GitHub pull request to draft"));
        assert!(message.contains("Draft pull requests are unavailable"));

        Ok(())
    }
}
