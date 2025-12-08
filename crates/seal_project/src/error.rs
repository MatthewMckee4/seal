use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Invalid configuration file: {0}")]
    InvalidConfigurationFile(#[from] ConfigValidationError),

    #[error("Failed to read config file {path}: {source}")]
    ConfigFileNotReadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    ConfigParseError(#[from] toml::de::Error),

    #[error("Not in a git repository: {path}")]
    NotInGitRepository { path: PathBuf },

    #[error("Git command '{command}' failed: {stderr}")]
    GitCommandFailed { command: String, stderr: String },

    #[error("Workspace member '{member}' is missing seal.toml at path: {path}")]
    MemberMissingSealToml { member: String, path: PathBuf },

    #[error("Workspace member '{member}' path does not exist: {path}")]
    MemberPathNotFound { member: String, path: PathBuf },
}

#[derive(Error, Debug)]
pub enum ConfigValidationError {
    #[error("release.version-files cannot be empty")]
    EmptyVersionFiles,

    #[error("release.version-files cannot contain empty strings")]
    EmptyVersionFilePath,

    #[error("release.commit-message cannot be empty")]
    EmptyCommitMessage,

    #[error("release.branch-name cannot be empty")]
    EmptyBranchName,

    #[error("release.{field} must contain '{{version}}' placeholder, got: '{value}'")]
    MissingVersionPlaceholder { field: String, value: String },

    #[error("release.current-version is not a valid version: '{value}'")]
    InvalidVersion { value: String },

    #[error("project name cannot be empty")]
    EmptyProjectName,

    #[error(
        "project name '{name}' contains invalid characters (only alphanumeric, dash, and underscore allowed)"
    )]
    InvalidProjectName { name: String },

    #[error("release.push = true requires branch-name to be set")]
    PushRequiresBranchName,

    #[error("release.create-pr = true requires both branch-name and push = true")]
    CreatePrRequiresBranchAndPush,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_project_error_display() {
        let err = ProjectError::NotInGitRepository {
            path: PathBuf::from("/tmp/test"),
        };
        assert_snapshot!(err.to_string(), @"Not in a git repository: /tmp/test");

        let err = ProjectError::GitCommandFailed {
            command: "git status".to_string(),
            stderr: "fatal: not a git repository".to_string(),
        };
        assert_snapshot!(
            err.to_string(),
            @"Git command 'git status' failed: fatal: not a git repository"
        );
    }

    #[test]
    fn test_config_validation_error_display() {
        let err = ConfigValidationError::EmptyCommitMessage;
        assert_snapshot!(err.to_string(), @"release.commit-message cannot be empty");

        let err = ConfigValidationError::EmptyBranchName;
        assert_snapshot!(err.to_string(), @"release.branch-name cannot be empty");

        let err = ConfigValidationError::MissingVersionPlaceholder {
            field: "commit-message".to_string(),
            value: "Release".to_string(),
        };
        assert_snapshot!(
            err.to_string(),
            @"release.commit-message must contain '{version}' placeholder, got: 'Release'"
        );

        let err = ConfigValidationError::InvalidVersion {
            value: String::new(),
        };
        assert_snapshot!(
            err.to_string(),
            @"release.current-version is not a valid version: ''"
        );
    }

    #[test]
    fn test_project_error_from_config_validation() {
        let validation_err = ConfigValidationError::EmptyCommitMessage;
        let project_err: ProjectError = validation_err.into();
        assert_snapshot!(
            project_err.to_string(),
            @"Invalid configuration file: release.commit-message cannot be empty"
        );
    }
}
