use anyhow::{Context, Result};
use seal_project::{ChangelogConfig, Config, ReleaseConfig, VersionFile, VersionFileTextFormat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoosterConfig {
    #[serde(default, alias = "version-files")]
    pub version_files: Vec<RoosterVersionFile>,

    #[serde(default)]
    pub submodules: Vec<String>,

    #[serde(default, alias = "require-labels")]
    pub require_labels: Vec<RequireLabel>,

    #[serde(default, alias = "ignore-labels")]
    pub ignore_labels: Vec<String>,

    #[serde(default, alias = "changelog-ignore-labels")]
    pub changelog_ignore_labels: Vec<String>,

    #[serde(default, alias = "changelog-ignore-authors")]
    pub changelog_ignore_authors: Vec<String>,

    #[serde(default, alias = "major-labels")]
    pub major_labels: Vec<String>,

    #[serde(default, alias = "minor-labels")]
    pub minor_labels: Vec<String>,

    #[serde(alias = "version-format")]
    pub version_format: Option<String>,

    #[serde(alias = "default-bump-type")]
    pub default_bump_type: Option<String>,

    #[serde(default, alias = "trim-title-prefixes")]
    pub trim_title_prefixes: Vec<String>,

    #[serde(default, alias = "section-labels")]
    pub section_labels: HashMap<String, Vec<String>>,

    #[serde(default, alias = "changelog-sections")]
    pub changelog_sections: HashMap<String, String>,

    #[serde(alias = "changelog-contributors")]
    pub changelog_contributors: Option<bool>,

    #[serde(alias = "version-tag-prefix")]
    pub version_tag_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RoosterVersionFile {
    Simple(String),
    WithField {
        #[serde(alias = "target")]
        path: String,
        field: String,
        #[serde(alias = "version-format")]
        format: Option<String>,
    },
    WithMatch {
        #[serde(alias = "path")]
        target: String,
        #[serde(rename = "match")]
        match_pattern: String,
        #[serde(alias = "version-format")]
        format: Option<String>,
    },
    WithReplace {
        #[serde(alias = "path")]
        target: String,
        replace: String,
    },
    JustPath {
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequireLabel {
    pub submodule: String,
    pub labels: Vec<String>,
}

pub fn parse_rooster_config(path: &Path) -> Result<RoosterConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let table: toml::Table = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML: {}", path.display()))?;

    let rooster_table = table
        .get("tool")
        .and_then(|t| t.as_table())
        .and_then(|t| t.get("rooster"))
        .context("No [tool.rooster] section found")?;

    let config: RoosterConfig = rooster_table
        .clone()
        .try_into()
        .context("Failed to deserialize rooster config")?;

    Ok(config)
}

pub fn migrate_rooster_config(rooster: &RoosterConfig) -> (Config, Vec<String>) {
    let mut warnings = Vec::new();

    let version_files = convert_version_files(&rooster.version_files, &mut warnings);

    let ignore_labels =
        if rooster.ignore_labels.is_empty() && !rooster.changelog_ignore_labels.is_empty() {
            rooster.changelog_ignore_labels.clone()
        } else if !rooster.ignore_labels.is_empty() {
            rooster.ignore_labels.clone()
        } else {
            Vec::new()
        };

    let ignore_contributors = if rooster.changelog_ignore_authors.is_empty() {
        Vec::new()
    } else {
        rooster.changelog_ignore_authors.clone()
    };

    let include_contributors = rooster.changelog_contributors.unwrap_or(true);

    let section_labels =
        convert_section_labels(&rooster.section_labels, &rooster.changelog_sections);

    let changelog_config = if !ignore_labels.is_empty()
        || !ignore_contributors.is_empty()
        || !include_contributors
        || !section_labels.is_empty()
    {
        Some(ChangelogConfig {
            ignore_labels: if ignore_labels.is_empty() {
                None
            } else {
                Some(ignore_labels)
            },
            ignore_contributors: if ignore_contributors.is_empty() {
                None
            } else {
                Some(ignore_contributors)
            },
            section_labels: if section_labels.is_empty() {
                None
            } else {
                Some(section_labels)
            },
            changelog_heading: None,
            include_contributors: if include_contributors {
                None
            } else {
                Some(false)
            },
            changelog_path: None,
        })
    } else {
        None
    };

    if !rooster.submodules.is_empty() {
        warnings.push(
            "submodules: Not supported in seal (monorepo members should be configured separately)"
                .to_string(),
        );
    }

    if !rooster.require_labels.is_empty() {
        warnings.push("require-labels: Not supported in seal".to_string());
    }

    if !rooster.major_labels.is_empty() || !rooster.minor_labels.is_empty() {
        warnings.push("major-labels/minor-labels: Semantic version bumping based on labels is not yet supported in seal".to_string());
    }

    if rooster.default_bump_type.is_some() {
        warnings.push(
            "default-bump-type: Not supported in seal (use 'seal bump' with explicit version)"
                .to_string(),
        );
    }

    if !rooster.trim_title_prefixes.is_empty() {
        warnings.push("trim-title-prefixes: Not supported in seal".to_string());
    }

    if !rooster.section_labels.is_empty() || !rooster.changelog_sections.is_empty() {
        warnings.push("section-labels/changelog-sections: Custom changelog sections are supported but you need to manually verify the mapping".to_string());
    }

    if let Some(prefix) = &rooster.version_tag_prefix {
        if prefix != "v" {
            warnings.push(format!("version-tag-prefix: Custom tag prefix '{prefix}' is not configurable in seal (always uses 'v')"));
        }
    }

    let release_config = if !version_files.is_empty() {
        warnings.push(
            "current-version set to placeholder '0.0.0' - update this to your actual version"
                .to_string(),
        );
        Some(ReleaseConfig {
            current_version: "0.0.0".to_string(),
            version_files: Some(version_files),
            commit_message: None,
            branch_name: None,
            push: false,
            create_pr: false,
            confirm: true,
        })
    } else {
        warnings.push(
            "NOTE: You will need to manually add the [release] section with 'current-version'"
                .to_string(),
        );
        None
    };

    (
        Config {
            release: release_config,
            changelog: changelog_config,
            members: None,
        },
        warnings,
    )
}

fn convert_section_labels(
    section_labels: &HashMap<String, Vec<String>>,
    changelog_sections: &HashMap<String, String>,
) -> BTreeMap<String, Vec<String>> {
    let mut result = BTreeMap::new();

    for (section_name, labels) in section_labels {
        result.insert(section_name.clone(), labels.clone());
    }

    for section_name in changelog_sections.keys() {
        if !result.contains_key(section_name) && section_name != "__unknown__" {
            result.insert(section_name.clone(), vec![section_name.to_lowercase()]);
        }
    }

    result
}

fn convert_version_files(
    rooster_files: &[RoosterVersionFile],
    warnings: &mut Vec<String>,
) -> Vec<VersionFile> {
    let mut result = Vec::new();

    for file in rooster_files {
        match file {
            RoosterVersionFile::Simple(path) => {
                result.push(VersionFile::Simple(path.clone()));
            }
            RoosterVersionFile::WithField {
                path,
                field,
                format,
            } => {
                if format.as_deref() == Some("cargo") || format.is_none() {
                    result.push(VersionFile::Text {
                        path: path.clone(),
                        format: VersionFileTextFormat::Toml,
                        field: Some(field.clone()),
                    });
                } else {
                    warnings.push(format!(
                        "version-file '{path}': Unsupported format '{}', treating as simple file",
                        format.as_ref().unwrap()
                    ));
                    result.push(VersionFile::Simple(path.clone()));
                }
            }
            RoosterVersionFile::WithMatch {
                target,
                match_pattern,
                format: _,
            } => {
                result.push(VersionFile::Search {
                    path: target.clone(),
                    search: match_pattern.clone(),
                });
            }
            RoosterVersionFile::WithReplace { target, replace } => {
                result.push(VersionFile::Search {
                    path: target.clone(),
                    search: replace.clone(),
                });
            }
            RoosterVersionFile::JustPath { path } => {
                result.push(VersionFile::Simple(path.clone()));
            }
        }
    }

    result
}
