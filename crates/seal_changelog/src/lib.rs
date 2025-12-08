use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use octocrab::Octocrab;
use octocrab::models::pulls::PullRequest;
use seal_project::ChangelogConfig;

pub struct ChangelogGenerator {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl ChangelogGenerator {
    pub fn new(owner: String, repo: String) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(
                std::env::var("GITHUB_TOKEN")
                    .or_else(|_| std::env::var("GH_TOKEN"))
                    .context("GITHUB_TOKEN or GH_TOKEN environment variable must be set")?,
            )
            .build()?;

        Ok(Self {
            octocrab,
            owner,
            repo,
        })
    }

    pub async fn generate_changelog(
        &self,
        version: &str,
        config: &ChangelogConfig,
    ) -> Result<String> {
        let last_release = self.get_last_release().await?;
        let prs = self.get_prs_since_release(last_release.as_ref()).await?;
        let pr_entries = fetch_pr_entries(prs);

        format_changelog_content(version, pr_entries, config)
    }

    async fn get_last_release(&self) -> Result<Option<String>> {
        let releases = self
            .octocrab
            .repos(&self.owner, &self.repo)
            .releases()
            .list()
            .per_page(1)
            .send()
            .await?;

        Ok(releases.items.first().and_then(|r| {
            r.created_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        }))
    }

    async fn get_prs_since_release(&self, since: Option<&String>) -> Result<Vec<PullRequest>> {
        let mut page = 1u32;
        let mut all_prs = Vec::new();
        let cutoff_date = since.as_ref().map(|s| s.as_str());

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

                if let Some(cutoff) = cutoff_date {
                    if let Some(merged) = &pr.merged_at {
                        if merged.format("%Y-%m-%dT%H:%M:%SZ").to_string().as_str() <= cutoff {
                            return Ok(all_prs);
                        }
                    }
                }

                all_prs.push(pr);
            }

            page += 1;
        }

        Ok(all_prs)
    }
}

fn fetch_pr_entries(prs: Vec<PullRequest>) -> Vec<PREntry> {
    prs.into_iter()
        .map(|pr| PREntry {
            title: pr.title.unwrap_or_default(),
            number: pr.number,
            url: pr.html_url.map(|u| u.to_string()),
            labels: pr
                .labels
                .map(|labels| labels.iter().map(|l| l.name.clone()).collect())
                .unwrap_or_default(),
            author: pr.user.map(|u| u.login),
        })
        .collect()
}

pub struct CategorizedPRs {
    pub sections: BTreeMap<String, Vec<PREntry>>,
    pub contributors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PREntry {
    pub title: String,
    pub number: u64,
    pub url: Option<String>,
    pub labels: Vec<String>,
    pub author: Option<String>,
}

pub fn categorize_prs(prs: Vec<PREntry>, config: &ChangelogConfig) -> CategorizedPRs {
    let ignore_labels: HashSet<&String> = config.ignore_labels().iter().collect();
    let section_labels = config.section_labels();

    let mut categorized = BTreeMap::new();
    let mut contributors = HashSet::new();

    for pr in prs {
        if pr.labels.iter().any(|l| ignore_labels.contains(l)) {
            continue;
        }

        if let Some(author) = &pr.author {
            contributors.insert(author.clone());
        }

        let mut categorized_pr = false;
        for (section_name, section_label_list) in section_labels {
            for label in section_label_list {
                if pr.labels.iter().any(|l| l == label) {
                    categorized
                        .entry(section_name.clone())
                        .or_insert_with(Vec::new)
                        .push(pr.clone());
                    categorized_pr = true;
                    break;
                }
            }
            if categorized_pr {
                break;
            }
        }

        if !categorized_pr && !section_labels.is_empty() {
            categorized
                .entry("Other".to_string())
                .or_insert_with(Vec::new)
                .push(pr);
        }
    }

    CategorizedPRs {
        sections: categorized,
        contributors: contributors.into_iter().collect(),
    }
}

pub fn format_changelog_content(
    version: &str,
    prs: Vec<PREntry>,
    config: &ChangelogConfig,
) -> Result<String> {
    let categorized = categorize_prs(prs, config);

    let mut output = String::new();

    let heading = config.changelog_heading().replace("{version}", version);

    write!(output, "## {heading}\n\n")?;

    for (section_name, prs) in &categorized.sections {
        if prs.is_empty() {
            continue;
        }

        write!(output, "### {section_name}\n\n")?;

        for pr in prs {
            if let Some(url) = &pr.url {
                writeln!(output, "- {} ([#{}]({}))", pr.title, pr.number, url)?;
            } else {
                writeln!(output, "- {} (#{}) ", pr.title, pr.number)?;
            }
        }

        output.push('\n');
    }

    if config.include_contributors() && !categorized.contributors.is_empty() {
        output.push_str("### Contributors\n\n");

        let mut contributors = categorized.contributors;
        contributors.sort();

        for contributor in contributors {
            writeln!(
                output,
                "- [@{contributor}](https://github.com/{contributor})"
            )?;
        }

        output.push('\n');
    }

    Ok(output)
}

pub fn update_changelog_file(changelog_path: &Path, new_content: &str) -> Result<()> {
    let existing_content = if changelog_path.exists() {
        fs_err::read_to_string(changelog_path)?
    } else {
        String::from("# Changelog\n\n")
    };

    let updated_content = if existing_content.starts_with("# Changelog") {
        let after_header = existing_content
            .strip_prefix("# Changelog\n\n")
            .or_else(|| existing_content.strip_prefix("# Changelog\n"))
            .unwrap_or(&existing_content);

        format!("# Changelog\n\n{new_content}{after_header}")
    } else {
        format!("# Changelog\n\n{new_content}{existing_content}")
    };

    fs_err::write(changelog_path, updated_content)?;

    Ok(())
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

fn get_git_remote_url() -> Result<String> {
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

pub fn generate_and_update_changelog(
    root: &Path,
    version: &str,
    config: &ChangelogConfig,
) -> Result<()> {
    let repo_url = get_git_remote_url()?;
    let (owner, repo) = parse_github_repo(&repo_url)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let changelog_content = runtime.block_on(async {
        let generator = ChangelogGenerator::new(owner, repo)?;
        generator.generate_changelog(version, config).await
    })?;

    let changelog_path = root.join("CHANGELOG.md");
    update_changelog_file(&changelog_path, &changelog_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use seal_project::ChangelogHeading;

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

    #[test]
    fn test_format_changelog_with_section_labels() {
        let prs = vec![
            PREntry {
                title: "Breaking API change".to_string(),
                number: 1,
                url: Some("https://github.com/owner/repo/pull/1".to_string()),
                labels: vec!["breaking".to_string()],
                author: Some("alice".to_string()),
            },
            PREntry {
                title: "Add new feature".to_string(),
                number: 2,
                url: Some("https://github.com/owner/repo/pull/2".to_string()),
                labels: vec!["enhancement".to_string()],
                author: Some("bob".to_string()),
            },
            PREntry {
                title: "Fix bug".to_string(),
                number: 3,
                url: Some("https://github.com/owner/repo/pull/3".to_string()),
                labels: vec!["bug".to_string()],
                author: Some("alice".to_string()),
            },
        ];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Breaking changes".to_string(), vec!["breaking".to_string()]);
        section_labels.insert(
            "Enhancements".to_string(),
            vec!["enhancement".to_string(), "feature".to_string()],
        );
        section_labels.insert("Bug fixes".to_string(), vec!["bug".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(true),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        ### Breaking changes

        - Breaking API change ([#1](https://github.com/owner/repo/pull/1))

        ### Bug fixes

        - Fix bug ([#3](https://github.com/owner/repo/pull/3))

        ### Enhancements

        - Add new feature ([#2](https://github.com/owner/repo/pull/2))

        ### Contributors

        - [@alice](https://github.com/alice)
        - [@bob](https://github.com/bob)

        "###);
    }

    #[test]
    fn test_format_changelog_with_ignored_labels() {
        let prs = vec![
            PREntry {
                title: "Add feature".to_string(),
                number: 1,
                url: Some("https://github.com/owner/repo/pull/1".to_string()),
                labels: vec!["enhancement".to_string()],
                author: Some("alice".to_string()),
            },
            PREntry {
                title: "Internal refactor".to_string(),
                number: 2,
                url: Some("https://github.com/owner/repo/pull/2".to_string()),
                labels: vec!["internal".to_string()],
                author: Some("bob".to_string()),
            },
            PREntry {
                title: "CI improvement".to_string(),
                number: 3,
                url: Some("https://github.com/owner/repo/pull/3".to_string()),
                labels: vec!["ci".to_string()],
                author: Some("charlie".to_string()),
            },
        ];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: Some(vec!["internal".to_string(), "ci".to_string()]),
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(true),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        ### Enhancements

        - Add feature ([#1](https://github.com/owner/repo/pull/1))

        ### Contributors

        - [@alice](https://github.com/alice)

        "###);
    }

    #[test]
    fn test_format_changelog_with_custom_heading() {
        let prs = vec![PREntry {
            title: "Add feature".to_string(),
            number: 1,
            url: Some("https://github.com/owner/repo/pull/1".to_string()),
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
        }];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: Some(section_labels),
            changelog_heading: Some(
                ChangelogHeading::new("Version {version} - Released".to_string()).unwrap(),
            ),
            include_contributors: Some(false),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## Version 1.0.0 - Released

        ### Enhancements

        - Add feature ([#1](https://github.com/owner/repo/pull/1))

        "###);
    }

    #[test]
    fn test_format_changelog_without_contributors() {
        let prs = vec![PREntry {
            title: "Add feature".to_string(),
            number: 1,
            url: Some("https://github.com/owner/repo/pull/1".to_string()),
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
        }];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(false),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        ### Enhancements

        - Add feature ([#1](https://github.com/owner/repo/pull/1))

        "###);
    }

    #[test]
    fn test_format_changelog_with_other_section() {
        let prs = vec![
            PREntry {
                title: "Add feature".to_string(),
                number: 1,
                url: Some("https://github.com/owner/repo/pull/1".to_string()),
                labels: vec!["enhancement".to_string()],
                author: Some("alice".to_string()),
            },
            PREntry {
                title: "Update docs".to_string(),
                number: 2,
                url: Some("https://github.com/owner/repo/pull/2".to_string()),
                labels: vec!["documentation".to_string()],
                author: Some("bob".to_string()),
            },
        ];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(true),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        ### Enhancements

        - Add feature ([#1](https://github.com/owner/repo/pull/1))

        ### Other

        - Update docs ([#2](https://github.com/owner/repo/pull/2))

        ### Contributors

        - [@alice](https://github.com/alice)
        - [@bob](https://github.com/bob)

        "###);
    }

    #[test]
    fn test_format_changelog_empty_prs() {
        let prs = vec![];

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: None,
            changelog_heading: None,
            include_contributors: Some(true),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        "###);
    }

    #[test]
    fn test_update_changelog_file_creates_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let changelog_path = temp_dir.path().join("CHANGELOG.md");

        let content = "## 1.0.0\n\n- Feature A\n\n";
        update_changelog_file(&changelog_path, content).unwrap();

        let result = fs_err::read_to_string(&changelog_path).unwrap();
        insta::assert_snapshot!(result, @r###"
        # Changelog

        ## 1.0.0

        - Feature A

        "###);
    }

    #[test]
    fn test_update_changelog_file_prepends_to_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let changelog_path = temp_dir.path().join("CHANGELOG.md");

        fs_err::write(
            &changelog_path,
            "# Changelog\n\n## 0.9.0\n\n- Old feature\n\n",
        )
        .unwrap();

        let new_content = "## 1.0.0\n\n- New feature\n\n";
        update_changelog_file(&changelog_path, new_content).unwrap();

        let result = fs_err::read_to_string(&changelog_path).unwrap();
        insta::assert_snapshot!(result, @r###"
        # Changelog

        ## 1.0.0

        - New feature

        ## 0.9.0

        - Old feature

        "###);
    }

    #[test]
    fn test_format_changelog_without_urls() {
        let prs = vec![PREntry {
            title: "Add feature".to_string(),
            number: 1,
            url: None,
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
        }];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(false),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r###"
        ## 1.0.0

        ### Enhancements

        - Add feature (#1)
        "###);
    }
}
