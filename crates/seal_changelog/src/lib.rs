use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use seal_file_change::{FileChange, FileChanges};
use seal_github::{GitHubPullRequest, GitHubService, filter_prs_by_date_range};

use seal_project::ChangelogConfig;

pub const DEFAULT_CHANGELOG_PATH: &str = "CHANGELOG.md";

fn extract_version_from_release_name(name: Option<&String>) -> Option<String> {
    name.as_ref().map(|n| {
        if let Some(stripped) = n.strip_prefix('v') {
            stripped.to_string()
        } else {
            (*n).clone()
        }
    })
}

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
            .get_prs_between(release.as_ref().map(|r| &r.created_at), None)
            .await?;

        format_changelog_content(version, prs, config)
    }
}

pub struct CategorizedPRs {
    pub sections: BTreeMap<String, Vec<GitHubPullRequest>>,
    pub contributors: Vec<String>,
}

pub fn categorize_prs(prs: Vec<GitHubPullRequest>, config: &ChangelogConfig) -> CategorizedPRs {
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
    }

    CategorizedPRs {
        sections: categorized,
        contributors: contributors.into_iter().collect(),
    }
}

pub fn format_changelog_content(
    version: &str,
    prs: Vec<GitHubPullRequest>,
    config: &ChangelogConfig,
) -> Result<String> {
    let categorized = categorize_prs(prs, config);

    let mut output = String::new();

    let heading = config.changelog_heading().replace("{version}", version);

    write!(output, "## {heading}\n\n")?;

    for (section_name, prs) in &categorized.sections {
        write!(output, "### {section_name}\n\n")?;

        for pr in prs {
            writeln!(output, "- {} ([#{}]({}))", pr.title, pr.number, pr.url)?;
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

pub async fn prepare_changelog_changes(
    root: &Path,
    version: &str,
    config: &ChangelogConfig,
    github_client: &Arc<dyn GitHubService>,
) -> Result<FileChanges> {
    let generator = ChangelogGenerator::new(github_client);
    let changelog_content = generator.generate_changelog(version, config).await?;

    let changelog_path = if let Some(path) = config.changelog_path.as_ref() {
        root.join(path)
    } else {
        root.join("CHANGELOG.md")
    };
    let change = prepare_changelog_file_change(&changelog_path, &changelog_content)?;

    Ok(FileChanges::new(vec![change]))
}

pub async fn generate_full_changelog(
    config: &ChangelogConfig,
    github_client: &Arc<dyn GitHubService>,
    max_prs: usize,
) -> Result<String> {
    let releases = github_client.get_all_releases().await?;

    let mut output = String::new();

    let all_prs = github_client.get_prs(Some(max_prs)).await?;

    let mut release_pairs: Vec<(
        Option<&seal_github::GitHubRelease>,
        &seal_github::GitHubRelease,
    )> = Vec::new();

    let Some(first_release) = releases.first() else {
        return Ok(output);
    };

    release_pairs.push((None, first_release));

    for i in 1..releases.len() {
        release_pairs.push((Some(&releases[i - 1]), &releases[i]));
    }

    for (since, until) in release_pairs.iter().rev() {
        let filter_prs_by_date_range = filter_prs_by_date_range(
            &all_prs,
            since.map(|release| &release.created_at),
            Some(&until.created_at),
        );

        if filter_prs_by_date_range.is_empty() {
            continue;
        }

        let categorized = categorize_prs(filter_prs_by_date_range, config);

        if let Some(version) = extract_version_from_release_name(until.name.as_ref()) {
            writeln!(output, "## {version}\n")?;
        } else {
            writeln!(
                output,
                "## Release {}\n",
                until.created_at.format("%Y-%m-%d")
            )?;
        }

        for (section_name, prs) in &categorized.sections {
            write!(output, "### {section_name}\n\n")?;

            for pr in prs {
                writeln!(output, "- {} ([#{}]({}))", pr.title, pr.number, pr.url)?;
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
    }

    Ok(output)
}

#[derive(Debug, Clone)]
pub struct ChangelogSection {
    pub version: String,
    pub body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReleaseBody {
    pub title: String,
    pub body: String,
    pub prerelease: bool,
}

pub fn parse_latest_changelog_section(changelog_content: &str) -> Result<ChangelogSection> {
    let lines: Vec<&str> = changelog_content.lines().collect();

    let section_start = lines
        .iter()
        .position(|line| line.starts_with("## "))
        .ok_or_else(|| anyhow::anyhow!("No version sections found in changelog"))?;

    let version = lines[section_start]
        .strip_prefix("## ")
        .unwrap()
        .trim()
        .to_string();

    let section_end = lines[section_start + 1..]
        .iter()
        .position(|line| line.starts_with("## "))
        .map(|pos| section_start + 1 + pos)
        .unwrap_or(lines.len());

    let body_lines = &lines[section_start + 1..section_end];
    let body = body_lines.join("\n").trim().to_string();

    Ok(ChangelogSection { version, body })
}

pub fn is_prerelease(version: &str) -> bool {
    let lower = version.to_lowercase();
    lower.contains("-alpha")
        || lower.contains("-beta")
        || lower.contains("-rc")
        || lower.contains("-pre")
}

pub fn create_release_body(changelog_content: &str) -> Result<ReleaseBody> {
    let section = parse_latest_changelog_section(changelog_content)?;
    let prerelease = is_prerelease(&section.version);

    Ok(ReleaseBody {
        title: section.version,
        body: section.body,
        prerelease,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use seal_project::ChangelogHeading;
    use std::collections::BTreeMap;

    #[test]
    fn test_format_changelog_with_section_labels() {
        let prs = vec![
            GitHubPullRequest {
                title: "Breaking API change".to_string(),
                number: 1,
                url: "https://github.com/owner/repo/pull/1".to_string(),
                labels: vec!["breaking".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 1, 10, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Add new feature".to_string(),
                number: 2,
                url: "https://github.com/owner/repo/pull/2".to_string(),
                labels: vec!["enhancement".to_string()],
                author: Some("bob".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 2, 14, 30, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Fix bug".to_string(),
                number: 3,
                url: "https://github.com/owner/repo/pull/3".to_string(),
                labels: vec!["bug".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 12, 3, 9, 15, 0).unwrap(),
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
            ..Default::default()
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
            GitHubPullRequest {
                title: "Add feature".to_string(),
                number: 1,
                url: "https://github.com/owner/repo/pull/1".to_string(),
                labels: vec!["enhancement".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 11, 20, 11, 0, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Internal refactor".to_string(),
                number: 2,
                url: "https://github.com/owner/repo/pull/2".to_string(),
                labels: vec!["internal".to_string()],
                author: Some("bob".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 11, 21, 13, 45, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "CI improvement".to_string(),
                number: 3,
                url: "https://github.com/owner/repo/pull/3".to_string(),
                labels: vec!["ci".to_string()],
                author: Some("charlie".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 11, 22, 16, 20, 0).unwrap(),
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
            ..Default::default()
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
        let prs = vec![GitHubPullRequest {
            title: "Add feature".to_string(),
            number: 1,
            url: "https://github.com/owner/repo/pull/1".to_string(),
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
            merged_at: Utc.with_ymd_and_hms(2025, 10, 15, 8, 30, 0).unwrap(),
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
            ..Default::default()
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
        let prs = vec![GitHubPullRequest {
            title: "Add feature".to_string(),
            number: 1,
            url: "https://github.com/owner/repo/pull/1".to_string(),
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
            merged_at: Utc.with_ymd_and_hms(2025, 9, 5, 12, 0, 0).unwrap(),
        }];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: None,
            ignore_contributors: None,
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(false),
            ..Default::default()
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
            GitHubPullRequest {
                title: "Add feature".to_string(),
                number: 1,
                url: "https://github.com/owner/repo/pull/1".to_string(),
                labels: vec!["enhancement".to_string()],
                author: Some("alice".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 8, 12, 15, 20, 0).unwrap(),
            },
            GitHubPullRequest {
                title: "Update docs".to_string(),
                number: 2,
                url: "https://github.com/owner/repo/pull/2".to_string(),
                labels: vec!["documentation".to_string()],
                author: Some("bob".to_string()),
                merged_at: Utc.with_ymd_and_hms(2025, 8, 13, 9, 45, 0).unwrap(),
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
            ..Default::default()
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @r"
        ## 1.0.0

        ### Enhancements

        - Add feature ([#1](https://github.com/owner/repo/pull/1))

        ### Contributors

        - [@alice](https://github.com/alice)
        - [@bob](https://github.com/bob)
        ");
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
            ..Default::default()
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
    fn test_format_changelog_with_ignored_contributors() {
        let prs = vec![GitHubPullRequest {
            title: "Add feature".to_string(),
            number: 1,
            url: "https://github.com/owner/repo/pull/1".to_string(),
            labels: vec!["enhancement".to_string()],
            author: Some("alice".to_string()),
            merged_at: Utc.with_ymd_and_hms(2025, 6, 25, 14, 15, 0).unwrap(),
        }];

        let mut section_labels = BTreeMap::new();
        section_labels.insert("Enhancements".to_string(), vec!["enhancement".to_string()]);

        let config = ChangelogConfig {
            ignore_labels: Some(vec!["internal".to_string(), "ci".to_string()]),
            ignore_contributors: Some(vec!["alice".to_string()]),
            section_labels: Some(section_labels),
            changelog_heading: None,
            include_contributors: Some(true),
            ..Default::default()
        };

        let result = format_changelog_content("1.0.0", prs, &config).unwrap();

        insta::assert_snapshot!(result, @"## 1.0.0");
    }

    #[test]
    fn test_parse_latest_changelog_section() {
        let changelog = r"# Changelog

## 1.0.0

### Features

- Added feature ([#1](url))

### Contributors

- [@alice](https://github.com/alice)

## 0.9.0

### Bug Fixes

- Fixed bug
";

        let section = parse_latest_changelog_section(changelog).unwrap();
        assert_eq!(section.version, "1.0.0");
        assert_eq!(
            section.body,
            "### Features\n\n- Added feature ([#1](url))\n\n### Contributors\n\n- [@alice](https://github.com/alice)"
        );
    }

    #[test]
    fn test_parse_latest_changelog_section_no_sections() {
        let changelog = "# Changelog\n\nSome text but no version sections.";

        let result = parse_latest_changelog_section(changelog);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No version sections found")
        );
    }

    #[test]
    fn test_parse_latest_changelog_section_single_version() {
        let changelog = r"# Changelog

## 2.0.0-beta.1

### Changes

- Breaking change
";

        let section = parse_latest_changelog_section(changelog).unwrap();
        assert_eq!(section.version, "2.0.0-beta.1");
        assert_eq!(section.body, "### Changes\n\n- Breaking change");
    }

    #[test]
    fn test_is_prerelease_alpha() {
        assert!(is_prerelease("1.0.0-alpha.1"));
        assert!(is_prerelease("1.0.0-ALPHA"));
        assert!(is_prerelease("2.0.0-alpha"));
    }

    #[test]
    fn test_is_prerelease_beta() {
        assert!(is_prerelease("1.0.0-beta.1"));
        assert!(is_prerelease("1.0.0-BETA"));
        assert!(is_prerelease("2.0.0-beta"));
    }

    #[test]
    fn test_is_prerelease_rc() {
        assert!(is_prerelease("1.0.0-rc.1"));
        assert!(is_prerelease("1.0.0-RC"));
        assert!(is_prerelease("2.0.0-rc"));
    }

    #[test]
    fn test_is_prerelease_pre() {
        assert!(is_prerelease("1.0.0-pre.1"));
        assert!(is_prerelease("1.0.0-PRE"));
    }

    #[test]
    fn test_is_prerelease_stable() {
        assert!(!is_prerelease("1.0.0"));
        assert!(!is_prerelease("2.3.4"));
        assert!(!is_prerelease("10.0.0"));
    }

    #[test]
    fn test_create_release_body_stable() {
        let changelog = r"# Changelog

## 1.0.0

### Features

- Added feature

## 0.9.0

### Bug Fixes

- Fixed bug
";

        let release_body = create_release_body(changelog).unwrap();
        assert_eq!(release_body.title, "1.0.0");
        assert_eq!(release_body.body, "### Features\n\n- Added feature");
        assert!(!release_body.prerelease);
    }

    #[test]
    fn test_create_release_body_prerelease() {
        let changelog = r"# Changelog

## 2.0.0-alpha.1

### Breaking Changes

- API changed
";

        let release_body = create_release_body(changelog).unwrap();
        assert_eq!(release_body.title, "2.0.0-alpha.1");
        assert_eq!(release_body.body, "### Breaking Changes\n\n- API changed");
        assert!(release_body.prerelease);
    }
}
