use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use seal_file_change::{FileChange, FileChanges};
use seal_github::GitHubService;

use seal_project::ChangelogConfig;

struct ChangelogGenerator<'a> {
    github_service: &'a Arc<dyn GitHubService>,
}

impl<'a> ChangelogGenerator<'a> {
    fn new(github_service: &'a Arc<dyn GitHubService>) -> Self {
        Self { github_service }
    }

    async fn generate_changelog(&self, version: &str, config: &ChangelogConfig) -> Result<String> {
        let release = self.github_service.get_latest_release().await.ok();

        let prs = self
            .github_service
            .get_prs_since_release(release.as_ref().map(|r| &r.created_at))
            .await?;

        let pr_entries = fetch_pr_entries(prs);

        format_changelog_content(version, pr_entries, config)
    }
}

fn fetch_pr_entries(prs: Vec<seal_github::GitHubPullRequest>) -> Vec<PREntry> {
    prs.into_iter()
        .map(|pr| PREntry {
            title: pr.title,
            number: pr.number,
            url: pr.url,
            labels: pr.labels,
            author: pr.author,
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
            if let Some(ignore_contributors) = &config.ignore_contributors {
                if ignore_contributors.contains(author) {
                    continue;
                }
            }
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

pub fn prepare_changelog_file_change(
    changelog_path: &Path,
    new_content: &str,
) -> Result<FileChange> {
    let existing_content = if changelog_path.exists() {
        fs_err::read_to_string(changelog_path)?
    } else {
        String::new()
    };

    let updated_content = {
        let first_line_is_heading = existing_content
            .lines()
            .next()
            .is_some_and(|line| line.starts_with('#'));

        if first_line_is_heading {
            let newline_pos = existing_content.find('\n');
            if let Some(pos) = newline_pos {
                let heading = &existing_content[..pos];
                let after_heading = &existing_content[pos + 1..];
                let rest = after_heading.trim_start_matches('\n');

                format!("{heading}\n\n{new_content}{rest}")
            } else {
                format!("{existing_content}\n\n{new_content}")
            }
        } else {
            format!("# Changelog\n\n{new_content}{existing_content}")
        }
    };

    Ok(FileChange::new(
        changelog_path.to_path_buf(),
        existing_content,
        updated_content,
    ))
}

pub fn prepare_changelog_changes(
    root: &Path,
    version: &str,
    config: &ChangelogConfig,
    github_client: &Arc<dyn GitHubService>,
) -> Result<FileChanges> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let changelog_content = runtime.block_on(async {
        let generator = ChangelogGenerator::new(github_client);
        generator.generate_changelog(version, config).await
    })?;

    let changelog_path = root.join("CHANGELOG.md");
    let change = prepare_changelog_file_change(&changelog_path, &changelog_content)?;

    Ok(FileChanges::new(vec![change]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use seal_project::ChangelogHeading;
    use std::collections::BTreeMap;

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
            ignore_contributors: None,
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
            ignore_contributors: None,
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
            ignore_contributors: None,
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
            ignore_contributors: None,
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
            ignore_contributors: None,
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
            ignore_contributors: None,
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
        let change = prepare_changelog_file_change(&changelog_path, content).unwrap();
        change.apply().unwrap();

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
        let change = prepare_changelog_file_change(&changelog_path, new_content).unwrap();
        change.apply().unwrap();

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
            ignore_contributors: None,
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

    #[test]
    fn test_format_changelog_with_ignored_contributors() {
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
            ignore_labels: Some(vec!["internal".to_string(), "ci".to_string()]),
            ignore_contributors: Some(vec!["alice".to_string()]),
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(true),
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @"## 1.0.0");
    }
}
