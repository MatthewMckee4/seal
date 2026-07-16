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

    #[error("No parent directory found for path: {path}")]
    NoParentDirectory { path: PathBuf },

    #[error(transparent)]
    ConfigParseError(#[from] toml::de::Error),

    #[error("Not in a git repository: {path}")]
    NotInGitRepository { path: PathBuf },

    #[error("Git command '{command}' failed: {stderr}")]
    GitCommandFailed { command: String, stderr: String },

    #[error(
        "Working tree and index are not clean:\n{changes}\nCommit or stash these changes before running `seal bump`, or use `--force` to bypass this check."
    )]
    DirtyGitState { changes: String },

    #[error("Release branch `{branch}` is not a valid Git branch name")]
    InvalidGitBranch { branch: String },

    #[error("Release branch `{branch}` already exists locally")]
    LocalGitBranchExists { branch: String },

    #[error("No `{remote}` Git remote is configured; release.push = true requires one")]
    MissingGitRemote { remote: String },

    #[error("Release branch `{branch}` already exists on remote `{remote}`")]
    RemoteGitBranchExists { branch: String, remote: String },

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

    #[error("release.pre-commit-commands requires release.commit-message to be set")]
    PreCommitCommandsRequireCommitMessage,

    #[error(
        "release.pull-request requires release.commit-message, release.branch-name, and release.push = true"
    )]
    PullRequestMissingPrerequisites,

    #[error("release.pull-request.title cannot be empty")]
    EmptyPullRequestTitle,

    #[error("release.pull-request.base cannot be empty")]
    EmptyPullRequestBase,

    #[error("release.changelog.changelog-heading cannot be empty")]
    EmptyChangelogHeading,

    #[error("release.changelog.changelog-heading cannot start with '#', got: '{value}'")]
    ChangelogHeadingStartsWithHash { value: String },
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

        let err = ConfigValidationError::PullRequestMissingPrerequisites;
        assert_snapshot!(
            err.to_string(),
            @"release.pull-request requires release.commit-message, release.branch-name, and release.push = true"
        );

        let err = ConfigValidationError::EmptyPullRequestTitle;
        assert_snapshot!(err.to_string(), @"release.pull-request.title cannot be empty");

        let err = ConfigValidationError::EmptyPullRequestBase;
        assert_snapshot!(err.to_string(), @"release.pull-request.base cannot be empty");
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

    #[test]
    fn test_dirty_git_state_error_display() {
        let error = ProjectError::DirtyGitState {
            changes: "  M src/lib.rs".to_string(),
        };
        assert_snapshot!(error.to_string(), @r#"
        Working tree and index are not clean:
          M src/lib.rs
        Commit or stash these changes before running `seal bump`, or use `--force` to bypass this check.
        "#);
    }

    #[test]
    fn test_invalid_git_branch_error_display() {
        let error = ProjectError::InvalidGitBranch {
            branch: "bad branch".to_string(),
        };
        assert_snapshot!(error.to_string(), @"Release branch `bad branch` is not a valid Git branch name");
    }

    #[test]
    fn test_local_git_branch_exists_error_display() {
        let error = ProjectError::LocalGitBranchExists {
            branch: "release/v1.2.3".to_string(),
        };
        assert_snapshot!(error.to_string(), @"Release branch `release/v1.2.3` already exists locally");
    }

    #[test]
    fn test_missing_git_remote_error_display() {
        let error = ProjectError::MissingGitRemote {
            remote: "origin".to_string(),
        };
        assert_snapshot!(error.to_string(), @"No `origin` Git remote is configured; release.push = true requires one");
    }

    #[test]
    fn test_remote_git_branch_exists_error_display() {
        let error = ProjectError::RemoteGitBranchExists {
            branch: "release/v1.2.3".to_string(),
            remote: "origin".to_string(),
        };
        assert_snapshot!(error.to_string(), @"Release branch `release/v1.2.3` already exists on remote `origin`");
    }
}
