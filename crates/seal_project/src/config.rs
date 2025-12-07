use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ProjectName;
use crate::error::{ConfigValidationError, ProjectError};

const DEFAULT_COMMIT_MESSAGE: &str = "Release v{version}";
const DEFAULT_BRANCH_NAME: &str = "release/v{version}";
const DEFAULT_TAG_FORMAT: &str = "v{version}";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub members: Option<MembersConfig>,
    pub release: ReleaseConfig,
}

impl Config {
    pub fn from_toml_str(content: &str) -> Result<Self, ProjectError> {
        toml::from_str(content).map_err(ProjectError::ConfigParseError)
    }

    pub fn from_file(path: &Path) -> Result<Self, ProjectError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ProjectError::ConfigFileNotReadable {
                path: path.to_path_buf(),
                source: e,
            })?;
        Self::from_toml_str(&content)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MembersConfig {
    pub members: BTreeMap<ProjectName, PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VersionFile {
    /// Detailed configuration with search pattern
    Detailed {
        path: String,
        search: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "version-template")]
        version_template: Option<String>,
    },
    /// Simple string path: "Cargo.toml"
    Simple(String),
}

impl VersionFile {
    pub fn path(&self) -> &str {
        match self {
            Self::Detailed { path, .. } => path,
            Self::Simple(path) => path,
        }
    }

    pub fn search_pattern(&self) -> Option<&str> {
        match self {
            Self::Detailed { search, .. } => Some(search),
            Self::Simple(_) => None,
        }
    }

    pub fn version_template(&self) -> Option<&str> {
        match self {
            Self::Detailed {
                version_template, ..
            } => version_template.as_deref(),
            Self::Simple(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReleaseConfig {
    pub current_version: String,

    #[serde(default = "default_version_files")]
    pub version_files: Vec<VersionFile>,

    #[serde(default = "default_commit_message_value")]
    pub commit_message: CommitMessage,

    #[serde(default = "default_branch_name_value")]
    pub branch_name: BranchName,

    #[serde(default = "default_tag_format_value")]
    pub tag_format: TagFormat,

    #[serde(default = "default_push")]
    pub push: bool,

    #[serde(default = "default_create_pr")]
    pub create_pr: bool,

    #[serde(default = "default_confirm")]
    pub confirm: bool,
}

fn default_version_files() -> Vec<VersionFile> {
    vec![]
}

fn default_commit_message_value() -> CommitMessage {
    CommitMessage::new(DEFAULT_COMMIT_MESSAGE.to_string()).expect("default is valid")
}

fn default_branch_name_value() -> BranchName {
    BranchName::new(DEFAULT_BRANCH_NAME.to_string()).expect("default is valid")
}

fn default_tag_format_value() -> TagFormat {
    TagFormat::new(DEFAULT_TAG_FORMAT.to_string()).expect("default is valid")
}

fn default_push() -> bool {
    true
}

fn default_create_pr() -> bool {
    true
}

fn default_confirm() -> bool {
    true
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TagFormat(String);

impl TagFormat {
    pub fn new(value: String) -> Result<Self, ConfigValidationError> {
        if value.trim().is_empty() {
            return Err(ConfigValidationError::EmptyTagFormat);
        }
        if !value.contains("{version}") {
            return Err(ConfigValidationError::MissingVersionPlaceholder {
                field: "tag-format".to_string(),
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for TagFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TagFormat {
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
commit-message = "chore: release v{version}"
branch-name = "release-{version}"
tag-format = "{version}"
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
            "commit-message": "chore: release v{version}",
            "branch-name": "release-{version}",
            "tag-format": "{version}",
            "push": true,
            "create-pr": true,
            "confirm": true
          }
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
            "commit-message": "Release v{version}",
            "branch-name": "release/v{version}",
            "tag-format": "v{version}",
            "push": true,
            "create-pr": true,
            "confirm": true
          }
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
            TomlError {
                message: "unknown field `unknown-field`, expected one of `current-version`, `version-files`, `commit-message`, `branch-name`, `tag-format`, `push`, `create-pr`, `confirm`",
                raw: Some(
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
            TomlError {
                message: "release.commit-message must contain '{version}' placeholder, got: 'Release without placeholder'",
                raw: Some(
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
            TomlError {
                message: "release.branch-name must contain '{version}' placeholder, got: 'release-branch'",
                raw: Some(
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
    fn test_missing_version_placeholder_in_tag_format() {
        let toml = r#"
[release]
current-version = "1.0.0"
tag-format = "release"
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            TomlError {
                message: "release.tag-format must contain '{version}' placeholder, got: 'release'",
                raw: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\ntag-format = \"release\"\n",
                ),
                keys: [
                    "release",
                    "tag-format",
                ],
                span: Some(
                    50..59,
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
            TomlError {
                message: "release.commit-message cannot be empty",
                raw: Some(
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
            TomlError {
                message: "release.branch-name cannot be empty",
                raw: Some(
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
    fn test_empty_tag_format() {
        let toml = r#"
[release]
current-version = "1.0.0"
tag-format = ""
"#;

        let result = Config::from_toml_str(toml);
        let err = result.unwrap_err();
        assert_debug_snapshot!(err, @r#"
        ConfigParseError(
            TomlError {
                message: "release.tag-format cannot be empty",
                raw: Some(
                    "\n[release]\ncurrent-version = \"1.0.0\"\ntag-format = \"\"\n",
                ),
                keys: [
                    "release",
                    "tag-format",
                ],
                span: Some(
                    50..52,
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
    fn test_tag_format_new_valid() {
        let tag = TagFormat::new("v{version}".to_string()).unwrap();
        insta::assert_snapshot!(tag.as_str(), @"v{version}");
        insta::assert_snapshot!(tag.to_string(), @"v{version}");
    }

    #[test]
    fn test_tag_format_new_empty() {
        let result = TagFormat::new(String::new());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigValidationError::EmptyTagFormat
        ));
    }

    #[test]
    fn test_tag_format_new_missing_placeholder() {
        let result = TagFormat::new("release".to_string());
        assert!(result.is_err());
        assert_debug_snapshot!(result.unwrap_err(), @r#"
        MissingVersionPlaceholder {
            field: "tag-format",
            value: "release",
        }
        "#);
    }

    #[test]
    fn test_serialization_round_trip() {
        let config = Config {
            members: None,
            release: ReleaseConfig {
                current_version: "1.2.3".to_string(),
                version_files: vec![VersionFile::Simple("Cargo.toml".to_string())],
                commit_message: CommitMessage::new("Release v{version}".to_string()).unwrap(),
                branch_name: BranchName::new("release/v{version}".to_string()).unwrap(),
                tag_format: TagFormat::new("v{version}".to_string()).unwrap(),
                push: true,
                create_pr: true,
                confirm: true,
            },
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
            "tag-format": "v{version}",
            "push": true,
            "create-pr": true,
            "confirm": true
          }
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
            release: ReleaseConfig {
                current_version: "1.0.0",
                version_files: [],
                commit_message: CommitMessage(
                    "Release {version} with {version} tag",
                ),
                branch_name: BranchName(
                    "release/v{version}",
                ),
                tag_format: TagFormat(
                    "v{version}",
                ),
                push: true,
                create_pr: true,
                confirm: true,
            },
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
            TomlError {
                message: "release.commit-message must contain '{version}' placeholder, got: 'Release {VERSION}'",
                raw: Some(
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
            release: ReleaseConfig {
                current_version: "1.0.0",
                version_files: [
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
                commit_message: CommitMessage(
                    "Release v{version}",
                ),
                branch_name: BranchName(
                    "release/v{version}",
                ),
                tag_format: TagFormat(
                    "v{version}",
                ),
                push: true,
                create_pr: true,
                confirm: true,
            },
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
            release: ReleaseConfig {
                current_version: "1.0.0",
                version_files: [],
                commit_message: CommitMessage(
                    "Release v{version}",
                ),
                branch_name: BranchName(
                    "release/v{version}",
                ),
                tag_format: TagFormat(
                    "v{version}",
                ),
                push: true,
                create_pr: true,
                confirm: true,
            },
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
        assert_eq!(config.release.version_files.len(), 2);
        assert_eq!(config.release.version_files[0].path(), "version.sh");
        assert_eq!(
            config.release.version_files[0].search_pattern(),
            Some("export PUBLIC_VERSION=\"{version}\"")
        );
        assert_eq!(config.release.version_files[1].path(), "Cargo.toml");
        assert_eq!(
            config.release.version_files[1].search_pattern(),
            Some("version = \"{version}\"")
        );
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
        assert_eq!(config.release.version_files.len(), 2);

        assert_eq!(config.release.version_files[0].path(), "package.json");
        assert_eq!(config.release.version_files[0].search_pattern(), None);

        assert_eq!(config.release.version_files[1].path(), "version.sh");
        assert_eq!(
            config.release.version_files[1].search_pattern(),
            Some("VERSION=\"{version}\"")
        );
    }
}
