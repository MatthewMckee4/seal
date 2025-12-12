use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use seal_macros::OptionsMetadata;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ProjectName;
use crate::error::{ConfigValidationError, ProjectError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, OptionsMetadata)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The members of the project.
    #[field(
        default = r"{}",
        value_type = "dict",
        example = r#"
            [members]
            pkg1 = "packages/pkg1"
            pkg2 = "packages/pkg2"
        "#
    )]
    pub members: Option<BTreeMap<ProjectName, PathBuf>>,

    #[option_group]
    /// Release configuration for versioning and publishing.
    pub release: Option<ReleaseConfig>,

    /// Changelog configuration for release notes generation.
    #[option_group]
    pub changelog: Option<ChangelogConfig>,
}

impl Config {
    pub fn from_toml_str(content: &str) -> Result<Self, ProjectError> {
        let config: Self = toml::from_str(content).map_err(ProjectError::ConfigParseError)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file(path: &Path) -> Result<Self, ProjectError> {
        let content =
            fs_err::read_to_string(path).map_err(|e| ProjectError::ConfigFileNotReadable {
                path: path.to_path_buf(),
                source: e,
            })?;
        Self::from_toml_str(&content)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if let Some(release) = &self.release {
            release.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VersionFile {
    /// Text or TOML replacement (format determines behavior)
    Text {
        /// Glob pattern
        path: String,
        /// Format of the file
        format: VersionFileTextFormat,
        /// Field to update in the file
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field: Option<String>,
    },
    /// Search and replace with optional template
    Search {
        /// Glob pattern
        path: String,
        /// Pattern to search for.
        ///
        /// Should contain `{version}` placeholder.
        search: String,
    },
    /// Just path, does a straight string replacement
    JustPath {
        path: String, // Glob pattern allowed
    },
    /// Simple path, also does string replacement
    Simple(String), // Path as string, glob allowed
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionFileTextFormat {
    Toml,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, OptionsMetadata)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReleaseConfig {
    /// The current version of the project.
    #[field(value_type = "string", example = r#"current-version = "0.1.0""#)]
    pub current_version: String,

    /// The version files that need to be updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "[]",
        value_type = "list",
        example = r#"
            [[release.version-files]]
            path = "**/Cargo.toml"
            format = "toml"
            field = "package.version"

            [[release.version-files]]
            path = "version.sh"
            format = "text"

            [[release.version-files]]
            path = "version.sh"
            search = "export FULL_VERSION = '{version}'"

            [[release.version-files]]
            path = "README.md"

            [release]
            version-files = [
                "docs/version.txt"
            ]
        "#
    )]
    pub version_files: Option<Vec<VersionFile>>,

    /// The commit message to use when committing the release changes.
    #[field(
        default = "null",
        value_type = "string",
        example = r#"
        commit-message = "Release {version}"
    "#
    )]
    pub commit_message: Option<CommitMessage>,

    /// The branch name to use when creating a new release branch.
    #[field(
        default = "null",
        value_type = "string",
        example = r#"
        branch-name = "release-{version}"
    "#
    )]
    pub branch_name: Option<BranchName>,

    /// Whether to push the release changes to the remote repository.
    #[serde(default = "default_push")]
    #[field(
        default = "false",
        value_type = "boolean",
        example = r#"
        push = false"#
    )]
    pub push: bool,

    /// Whether to create a pull request for the release changes.
    #[serde(default = "default_create_pr")]
    #[field(
        default = "false",
        value_type = "boolean",
        example = r#"
        create-pr = true"#
    )]
    pub create_pr: bool,

    /// Whether to confirm the release changes with the user before proceeding.
    #[serde(default = "default_confirm")]
    #[field(
        default = "true",
        value_type = "boolean",
        example = r#"
    confirm = true"#
    )]
    pub confirm: bool,
}

fn default_push() -> bool {
    false
}

fn default_create_pr() -> bool {
    false
}

fn default_confirm() -> bool {
    true
}

impl ReleaseConfig {
    fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.push && self.branch_name.is_none() {
            return Err(ConfigValidationError::PushRequiresBranchName);
        }

        if self.create_pr && (self.branch_name.is_none() || !self.push) {
            return Err(ConfigValidationError::CreatePrRequiresBranchAndPush);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CommitMessage(String);

impl CommitMessage {
    pub fn new(value: String) -> Result<Self, ConfigValidationError> {
        if value.trim().is_empty() {
            return Err(ConfigValidationError::EmptyCommitMessage);
        }
        if !value.contains("{version}") {
            return Err(ConfigValidationError::MissingVersionPlaceholder {
                field: "commit-message".to_string(),
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommitMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for CommitMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CommitMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BranchName(String);

impl BranchName {
    pub fn new(value: String) -> Result<Self, ConfigValidationError> {
        if value.trim().is_empty() {
            return Err(ConfigValidationError::EmptyBranchName);
        }
        if !value.contains("{version}") {
            return Err(ConfigValidationError::MissingVersionPlaceholder {
                field: "branch-name".to_string(),
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for BranchName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BranchName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, OptionsMetadata, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ChangelogConfig {
    /// Labels to ignore when generating changelog.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "[]",
        value_type = "list",
        example = r#"
        ignore-labels = ["internal", "ci", "testing"]
        "#
    )]
    pub ignore_labels: Option<Vec<String>>,

    /// Contributors to ignore when generating changelog.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "[]",
        value_type = "list",
        example = r#"
        ignore-contributors = ["dependabot[bot]"]
        "#
    )]
    pub ignore_contributors: Option<Vec<String>>,

    /// Mapping of section names to labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "{}",
        value_type = "dict",
        example = r#"
        [changelog.section-labels]
        "Breaking changes" = ["breaking"]
        "Enhancements" = ["enhancement", "compatibility"]
        "#
    )]
    pub section_labels: Option<BTreeMap<String, Vec<String>>>,

    /// Template for the changelog heading. Must contain {version} placeholder.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = r#""{version}""#,
        value_type = "string",
        example = r#"
        changelog-heading = "{version}"
        "#
    )]
    pub changelog_heading: Option<ChangelogHeading>,

    /// Whether to include contributors in the changelog. Defaults to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "true",
        value_type = "boolean",
        example = r#"
        include-contributors = true
        "#
    )]
    pub include_contributors: Option<bool>,

    /// Path to the changelog file. Defaults to `CHANGELOG.md`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[field(
        default = "CHANGELOG.md",
        value_type = "string",
        example = r#"
        changelog-path = "CHANGELOG.md"
        "#
    )]
    pub changelog_path: Option<PathBuf>,
}

impl ChangelogConfig {
    pub fn ignore_labels(&self) -> &[String] {
        self.ignore_labels.as_deref().unwrap_or(&[])
    }

    pub fn section_labels(&self) -> &BTreeMap<String, Vec<String>> {
        static EMPTY: BTreeMap<String, Vec<String>> = BTreeMap::new();
        self.section_labels.as_ref().unwrap_or(&EMPTY)
    }

    pub fn changelog_heading(&self) -> &str {
        self.changelog_heading
            .as_ref()
            .map(ChangelogHeading::as_str)
            .unwrap_or("{version}")
    }

    pub fn include_contributors(&self) -> bool {
        self.include_contributors.unwrap_or(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ChangelogHeading(String);

impl ChangelogHeading {
    pub fn new(value: String) -> Result<Self, ConfigValidationError> {
        if value.trim().is_empty() {
            return Err(ConfigValidationError::EmptyChangelogHeading);
        }
        if !value.contains("{version}") {
            return Err(ConfigValidationError::MissingVersionPlaceholder {
                field: "changelog-heading".to_string(),
                value,
            });
        }
        if value.trim_start().starts_with('#') {
            return Err(ConfigValidationError::ChangelogHeadingStartsWithHash { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChangelogHeading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ChangelogHeading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ChangelogHeading {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_json_snapshot};

    use super::*;

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[release]
current-version = "1.2.3"
version-files = ["Cargo.toml", "package.json"]
commit-message = "release v{version}"
branch-name = "release-{version}"
"#;

        let config = Config::from_toml_str(toml).unwrap();
        assert_json_snapshot!(config, @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.2.3",
            "version-files": [
              "Cargo.toml",
              "package.json"
            ],
            "commit-message": "release v{version}",
            "branch-name": "release-{version}",
            "push": false,
            "create-pr": false,
            "confirm": true
          },
          "changelog": null
        }
        "#);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
[release]
current-version = "0.1.0"
version-files = ["VERSION"]
"#;

        let config = Config::from_toml_str(toml).unwrap();
        assert_json_snapshot!(config, @r#"
        {
          "members": null,
          "release": {
            "current-version": "0.1.0",
            "version-files": [
              "VERSION"
            ],
            "commit-message": null,
            "branch-name": null,
            "push": false,
            "create-pr": false,
            "confirm": true
          },
          "changelog": null
        }
        "#);
    }

    #[test]
    fn test_parse_empty_config_requires_current_version() {
        let toml = "[release]";
        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_field_error() {
        let toml = r#"
[release]
unknown-field = "value"
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "unknown field `unknown-field`, expected one of `current-version`, `version-files`, `commit-message`, `branch-name`, `push`, `create-pr`, `confirm`",
                input: Some(
                    "\n[release]\nunknown-field = \"value\"\n",
                ),
                keys: [
                    "release",
                ],
                span: Some(
                    11..24,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_missing_version_placeholder_in_commit_message() {
        let toml = r#"
[release]
current-version = "1.0.0"
commit-message = "Release without placeholder"
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "release.commit-message must contain '{version}' placeholder, got: 'Release without placeholder'",
                input: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\ncommit-message = \"Release without placeholder\"\n",
                ),
                keys: [
                    "release",
                    "commit-message",
                ],
                span: Some(
                    54..83,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_missing_version_placeholder_in_branch_name() {
        let toml = r#"
[release]
current-version = "1.0.0"
branch-name = "release-branch"
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "release.branch-name must contain '{version}' placeholder, got: 'release-branch'",
                input: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\nbranch-name = \"release-branch\"\n",
                ),
                keys: [
                    "release",
                    "branch-name",
                ],
                span: Some(
                    51..67,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_empty_commit_message() {
        let toml = r#"
[release]
current-version = "1.0.0"
commit-message = ""
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "release.commit-message cannot be empty",
                input: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\ncommit-message = \"\"\n",
                ),
                keys: [
                    "release",
                    "commit-message",
                ],
                span: Some(
                    54..56,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_empty_branch_name() {
        let toml = r#"
[release]
current-version = "1.0.0"
branch-name = ""
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            Error {
                message: "release.branch-name cannot be empty",
                input: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\nbranch-name = \"\"\n",
                ),
                keys: [
                    "release",
                    "branch-name",
                ],
                span: Some(
                    51..53,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_commit_message_new_valid() {
        let msg = CommitMessage::new("Release v{version}".to_string()).unwrap();
        insta::assert_snapshot!(msg.as_str(), @"Release v{version}");
        insta::assert_snapshot!(msg.to_string(), @"Release v{version}");
    }

    #[test]
    fn test_commit_message_new_empty() {
        let result = CommitMessage::new(String::new());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @"EmptyCommitMessage");
    }

    #[test]
    fn test_commit_message_new_missing_placeholder() {
        let result = CommitMessage::new("Release".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        MissingVersionPlaceholder {
            field: "commit-message",
            value: "Release",
        }
        "#);
    }

    #[test]
    fn test_commit_message_whitespace_only() {
        let result = CommitMessage::new("   ".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @"EmptyCommitMessage");
    }

    #[test]
    fn test_branch_name_new_valid() {
        let name = BranchName::new("release/v{version}".to_string()).unwrap();
        insta::assert_snapshot!(name.as_str(), @"release/v{version}");
        insta::assert_snapshot!(name.to_string(), @"release/v{version}");
    }

    #[test]
    fn test_branch_name_new_empty() {
        let result = BranchName::new(String::new());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @"EmptyBranchName");
    }

    #[test]
    fn test_branch_name_new_missing_placeholder() {
        let result = BranchName::new("release".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        MissingVersionPlaceholder {
            field: "branch-name",
            value: "release",
        }
        "#);
    }

    #[test]
    fn test_changelog_heading_new_valid() {
        let name = ChangelogHeading::new("Version {version}".to_string()).unwrap();
        insta::assert_snapshot!(name.as_str(), @"Version {version}");
        insta::assert_snapshot!(name.to_string(), @"Version {version}");
    }

    #[test]
    fn test_changelog_heading_new_empty() {
        let result = ChangelogHeading::new(String::new());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @"EmptyChangelogHeading");
    }

    #[test]
    fn test_changelog_heading_new_missing_placeholder() {
        let result = ChangelogHeading::new("release".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        MissingVersionPlaceholder {
            field: "changelog-heading",
            value: "release",
        }
        "#);
    }

    #[test]
    fn test_changelog_heading_new_starts_with_hash() {
        let result = ChangelogHeading::new("# release-{version}".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r##"
        ChangelogHeadingStartsWithHash {
            value: "# release-{version}",
        }
        "##);
    }

    #[test]
    fn test_serialization_round_trip() {
        let config = Config {
            members: None,
            release: Some(ReleaseConfig {
                current_version: "1.2.3".to_string(),
                version_files: Some(vec![VersionFile::Simple("Cargo.toml".to_string())]),
                commit_message: Some(CommitMessage::new("Release v{version}".to_string()).unwrap()),
                branch_name: Some(BranchName::new("release/v{version}".to_string()).unwrap()),
                push: true,
                create_pr: true,
                confirm: true,
            }),
            changelog: None,
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed = Config::from_toml_str(&toml_str).unwrap();

        assert_json_snapshot!(parsed, @r#"
        {
          "members": null,
          "release": {
            "current-version": "1.2.3",
            "version-files": [
              "Cargo.toml"
            ],
            "commit-message": "Release v{version}",
            "branch-name": "release/v{version}",
            "push": true,
            "create-pr": true,
            "confirm": true
          },
          "changelog": null
        }
        "#);
    }

    #[test]
    fn test_version_placeholder_multiple_times() {
        let toml = r#"
[release]
current-version = "1.0.0"
commit-message = "Release {version} with {version} tag"
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_ok());
        assert_debug_snapshot!(result.unwrap(), @r#"
        Config {
            members: None,
            release: Some(
                ReleaseConfig {
                    current_version: "1.0.0",
                    version_files: None,
                    commit_message: Some(
                        CommitMessage(
                            "Release {version} with {version} tag",
                        ),
                    ),
                    branch_name: None,
                    push: false,
                    create_pr: false,
                    confirm: true,
                },
            ),
            changelog: None,
        }
        "#);
    }

    #[test]
    fn test_version_placeholder_case_sensitive() {
        let toml = r#"
[release]
current-version = "1.0.0"
commit-message = "Release {VERSION}"
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        ConfigParseError(
            Error {
                message: "release.commit-message must contain '{version}' placeholder, got: 'Release {VERSION}'",
                input: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\ncommit-message = \"Release {VERSION}\"\n",
                ),
                keys: [
                    "release",
                    "commit-message",
                ],
                span: Some(
                    54..73,
                ),
            },
        )
        "#);
    }

    #[test]
    fn test_multiple_version_files() {
        let toml = r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "package.json", "VERSION"]
"#;

        let config = Config::from_toml_str(toml).unwrap();
        assert_debug_snapshot!(config, @r#"
        Config {
            members: None,
            release: Some(
                ReleaseConfig {
                    current_version: "1.0.0",
                    version_files: Some(
                        [
                            Simple(
                                "Cargo.toml",
                            ),
                            Simple(
                                "package.json",
                            ),
                            Simple(
                                "VERSION",
                            ),
                        ],
                    ),
                    commit_message: None,
                    branch_name: None,
                    push: false,
                    create_pr: false,
                    confirm: true,
                },
            ),
            changelog: None,
        }
        "#);
    }

    #[test]
    fn test_empty_version_files_array() {
        let toml = r#"
[release]
current-version = "1.0.0"
version-files = []
"#;

        let config = Config::from_toml_str(toml);
        assert_debug_snapshot!(config.unwrap(), @r#"
        Config {
            members: None,
            release: Some(
                ReleaseConfig {
                    current_version: "1.0.0",
                    version_files: Some(
                        [],
                    ),
                    commit_message: None,
                    branch_name: None,
                    push: false,
                    create_pr: false,
                    confirm: true,
                },
            ),
            changelog: None,
        }
        "#);
    }

    #[test]
    fn test_version_file_with_custom_search_pattern() {
        let toml = r#"
[release]
current-version = "1.0.0"

[[release.version-files]]
path = "version.sh"
search = "export PUBLIC_VERSION=\"{version}\""

[[release.version-files]]
path = "Cargo.toml"
search = "version = \"{version}\""
"#;

        let config = Config::from_toml_str(toml).unwrap();
        let version_files = config
            .release
            .as_ref()
            .unwrap()
            .version_files
            .as_ref()
            .unwrap();

        assert_debug_snapshot!(version_files, @r#"
        [
            Search {
                path: "version.sh",
                search: "export PUBLIC_VERSION=\"{version}\"",
            },
            Search {
                path: "Cargo.toml",
                search: "version = \"{version}\"",
            },
        ]
        "#);
    }

    #[test]
    fn test_version_file_mixed_simple_and_detailed() {
        let toml = r#"
[release]
current-version = "1.0.0"
version-files = [
    "package.json",
    { path = "version.sh", search = "VERSION=\"{version}\"" }
]
"#;

        let config = Config::from_toml_str(toml).unwrap();
        let version_files = config
            .release
            .as_ref()
            .unwrap()
            .version_files
            .as_ref()
            .unwrap();

        assert_debug_snapshot!(version_files, @r#"
        [
            Simple(
                "package.json",
            ),
            Search {
                path: "version.sh",
                search: "VERSION=\"{version}\"",
            },
        ]
        "#);
    }

    #[test]
    fn test_validation_push_requires_branch_name() {
        let toml = r#"
[release]
current-version = "1.0.0"
push = true
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        InvalidConfigurationFile(
            PushRequiresBranchName,
        )
        "#);
    }

    #[test]
    fn test_validation_create_pr_requires_branch_and_push() {
        let toml = r#"
[release]
current-version = "1.0.0"
create-pr = true
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        InvalidConfigurationFile(
            CreatePrRequiresBranchAndPush,
        )
        "#);
    }

    #[test]
    fn test_validation_create_pr_requires_push() {
        let toml = r#"
[release]
current-version = "1.0.0"
branch-name = "release/{version}"
create-pr = true
push = false
"#;

        let result = Config::from_toml_str(toml);
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        InvalidConfigurationFile(
            CreatePrRequiresBranchAndPush,
        )
        "#);
    }

    #[test]
    fn test_validation_valid_with_branch_and_push() {
        let toml = r#"
[release]
current-version = "1.0.0"
branch-name = "release/{version}"
push = true
create-pr = true
"#;

        let config = Config::from_toml_str(toml).unwrap();
        assert!(config.release.as_ref().unwrap().push);
        assert!(config.release.as_ref().unwrap().create_pr);
    }

    #[test]
    fn test_validate_changelog_config() {
        let toml = r#"
[changelog]
ignore-labels = ["ignore"]
ignore-contributors = ["bot"]
include-contributors = true
changelog-heading = "Release {version}"

[changelog.section-labels]
"Breaking changes" = ["breaking"]
"Enhancements" = ["enhancement", "compatibility"]
"#;

        let config = Config::from_toml_str(toml).unwrap();
        assert_json_snapshot!(config, @r#"
        {
          "members": null,
          "release": null,
          "changelog": {
            "ignore-labels": [
              "ignore"
            ],
            "ignore-contributors": [
              "bot"
            ],
            "section-labels": {
              "Breaking changes": [
                "breaking"
              ],
              "Enhancements": [
                "enhancement",
                "compatibility"
              ]
            },
            "changelog-heading": "Release {version}",
            "include-contributors": true
          }
        }
        "#);
    }
}
