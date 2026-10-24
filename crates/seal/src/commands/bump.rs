use std::fmt::Write as _;
use std::io;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use seal_bump::{VersionBump, calculate_version_file_changes};
use seal_command::CommandWrapper;
use seal_fs::FileResolver;
use seal_github::{GitHubPullRequestOptions, GitHubService};
use seal_project::{
    PreCommitFailure, ProjectWorkspace, ReleaseConfig, ensure_clean_worktree,
    ensure_release_branch_available, get_current_branch,
};

use seal_cli::BumpArgs;

use crate::ExitStatus;
use crate::printer::Printer;

/// A command with metadata about whether it's a pre-commit command.
struct TaggedCommand {
    command: CommandWrapper,
    is_pre_commit: bool,
}

pub async fn bump(args: &BumpArgs, printer: Printer) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let version_bump: VersionBump = args
        .version
        .parse()
        .context("Failed to parse version bump argument")?;

    let workspace = ProjectWorkspace::discover()?;
    let config = workspace.config();

    let Some(release_config) = config.release.as_ref() else {
        return Err(anyhow::anyhow!(
            "No release configuration found in discovered workspace at `{}`",
            workspace.root().display()
        ));
    };

    let current_version_string = &release_config.current_version;

    let new_version = seal_bump::calculate_new_version(current_version_string, &version_bump)?;

    let new_version_string = new_version.to_string();

    writeln!(
        stdout,
        "Bumping version from {current_version_string} to {new_version_string}"
    )?;

    let branch_name = release_config
        .branch_name
        .as_ref()
        .map(|name| name.as_str().replace("{version}", &new_version_string));

    let commit_message = release_config
        .commit_message
        .as_ref()
        .map(|message| message.as_str().replace("{version}", &new_version_string));

    writeln!(stdout)?;

    let version_files = release_config.version_files.as_deref().unwrap_or(&[]);

    if version_files.is_empty() {
        tracing::info!("Warning: No version files configured - only seal.toml will be updated");
    }

    let file_resolver = FileResolver::new(workspace.root().clone());

    let uses_github =
        (!args.no_changelog && config.changelog.is_some()) || release_config.pull_request.is_some();

    #[cfg(feature = "integration-test")]
    let github_client: Option<Arc<dyn GitHubService>> = if uses_github {
        #[cfg(any(test, feature = "integration-test"))]
        use seal_github::MockGithubClient;
        Some(Arc::new(MockGithubClient::new()))
    } else {
        None
    };
    #[cfg(not(feature = "integration-test"))]
    let github_client: Option<Arc<dyn GitHubService>> = if uses_github {
        use seal_github::{GitHubClient, get_git_remote_url, parse_github_repo};

        let repo_url = get_git_remote_url(workspace.root())?;
        let (owner, repo) = parse_github_repo(&repo_url)?;
        Some(Arc::new(GitHubClient::new(owner, repo)?))
    } else {
        None
    };

    let mut file_changes = calculate_version_file_changes(
        workspace.root(),
        version_files,
        current_version_string,
        &new_version,
        &file_resolver,
    )?;
    let mut changelog_body = String::new();

    if !args.no_changelog {
        if let Some(changelog_config) = config.changelog.as_ref() {
            let prepared_changelog = seal_changelog::prepare_changelog_changes(
                workspace.root(),
                &new_version_string,
                changelog_config,
                github_client
                    .as_ref()
                    .context("Changelog generation requires a GitHub client")?,
            )
            .await
            .context("Failed to prepare changelog")?;

            changelog_body = prepared_changelog.section_body;
            file_changes.extend(prepared_changelog.file_changes);
        } else {
            tracing::info!(
                "Skipping changelog update because no `[changelog]` section was found in the configuration."
            );
        }
    } else {
        tracing::info!("Skipping changelog update because `--no-changelog` was provided.");
    }

    let pull_request = if let Some(pull_request_config) = &release_config.pull_request {
        let default_title = commit_message
            .as_deref()
            .context("Pull request configuration requires commit-message")?;
        let head = branch_name
            .as_ref()
            .context("Pull request configuration requires branch-name")?;
        let base = if let Some(base) = &pull_request_config.base {
            base.clone()
        } else {
            get_current_branch(workspace.root())?
        };

        Some(GitHubPullRequestOptions {
            title: if let Some(title) = &pull_request_config.title {
                title.replace("{version}", &new_version_string)
            } else {
                default_title.to_string()
            },
            body: if let Some(body) = &pull_request_config.body {
                body.replace("{version}", &new_version_string)
            } else {
                changelog_body.clone()
            },
            head: head.clone(),
            base,
            draft: pull_request_config.draft,
        })
    } else {
        None
    };

    writeln!(stdout, "Preview of changes:")?;
    let width = seal_terminal::terminal_width();

    writeln!(stdout, "─────────────{:─^1$}", "", width.saturating_sub(13))?;

    for change in &file_changes {
        change.display_diff(&mut stdout, &file_resolver)?;
    }

    writeln!(stdout)?;

    writeln!(stdout, "Changes to be made:")?;

    for change in &file_changes {
        writeln!(
            stdout,
            "  - Update `{}`",
            file_resolver.relative_path(change.path()).display()
        )?;
    }

    writeln!(stdout)?;

    let mut commands: Vec<TaggedCommand> = Vec::new();

    if let Some(branch) = &branch_name {
        commands.push(TaggedCommand {
            command: CommandWrapper::create_branch(branch),
            is_pre_commit: false,
        });
    }

    if let Some(message) = &commit_message {
        commands.push(TaggedCommand {
            command: CommandWrapper::git_add_all(),
            is_pre_commit: false,
        });

        if let Some(pre_commit_cmds) = release_config.pre_commit_commands.as_ref() {
            for cmd in pre_commit_cmds {
                commands.push(TaggedCommand {
                    command: CommandWrapper::custom(cmd),
                    is_pre_commit: true,
                });
            }
            commands.push(TaggedCommand {
                command: CommandWrapper::git_add_all(),
                is_pre_commit: false,
            });
        }

        commands.push(TaggedCommand {
            command: CommandWrapper::git_commit(message),
            is_pre_commit: false,
        });
    }

    if release_config.push {
        if let Some(branch) = &branch_name {
            commands.push(TaggedCommand {
                command: CommandWrapper::git_push_branch(branch),
                is_pre_commit: false,
            });
        }
    }

    if args.dry_run {
        let checks = run_bump_preflight(
            args,
            release_config,
            branch_name.as_deref(),
            github_client.as_deref(),
            workspace.root(),
        )?;

        writeln!(stdout, "Preflight checks:")?;
        for check in checks {
            writeln!(stdout, "  - {check}")?;
        }
        writeln!(stdout)?;
    }

    if !args.dry_run && !commands.is_empty() {
        writeln!(stdout, "Commands to be executed:")?;

        for tagged in &commands {
            writeln!(stdout, "  `{}`", tagged.command.as_string())?;
        }

        writeln!(stdout)?;
    }

    if let Some(pull_request) = &pull_request {
        writeln!(stdout, "Pull request:")?;
        writeln!(stdout, "  Title: {}", pull_request.title)?;
        writeln!(stdout, "  Head: {}", pull_request.head)?;
        writeln!(stdout, "  Base: {}", pull_request.base)?;
        writeln!(stdout, "  Draft: {}", pull_request.draft)?;
        if pull_request.body.is_empty() {
            writeln!(stdout, "  Body: (empty)")?;
        } else {
            writeln!(stdout, "  Body:")?;
            for line in pull_request.body.lines() {
                if line.is_empty() {
                    writeln!(stdout)?;
                } else {
                    writeln!(stdout, "    {line}")?;
                }
            }
        }
        writeln!(stdout)?;
    }

    if args.dry_run {
        writeln!(stdout, "Dry run complete. No changes made.")?;
        return Ok(ExitStatus::Success);
    }

    if release_config.confirm {
        if !confirm_changes(&mut stdout)? {
            writeln!(printer.stderr())?;
            writeln!(printer.stderr(), "No changes applied.")?;
            return Ok(ExitStatus::Success);
        }
        writeln!(stdout)?;
    }

    run_bump_preflight(
        args,
        release_config,
        branch_name.as_deref(),
        github_client.as_deref(),
        workspace.root(),
    )?;

    writeln!(stdout, "Updating files...")?;

    file_changes.apply()?;

    let on_failure = release_config.on_pre_commit_failure;

    for tagged in &commands {
        if tagged.is_pre_commit && on_failure == PreCommitFailure::Continue {
            let result = tagged
                .command
                .execute_with_result(&mut stdout, workspace.root())?;
            if !result.success {
                let exit_info = result
                    .exit_code
                    .map(|code| format!(" (exit code {code})"))
                    .unwrap_or_default();
                writeln!(
                    stdout,
                    "Warning: Command `{}` failed{exit_info}, continuing...",
                    tagged.command.as_string()
                )?;
            }
        } else {
            tagged.command.execute(&mut stdout, workspace.root())?;
        }
    }

    if let Some(pull_request) = pull_request {
        let pull_request = github_client
            .as_ref()
            .context("Pull request creation requires a GitHub client")?
            .create_or_update_pull_request(pull_request)
            .await
            .context("Failed to create or update GitHub pull request")?;
        writeln!(stdout, "Pull request: {}", pull_request.url)?;
    }

    writeln!(stdout, "Successfully bumped to {new_version_string}")?;

    Ok(ExitStatus::Success)
}

fn run_bump_preflight(
    args: &BumpArgs,
    release_config: &ReleaseConfig,
    branch_name: Option<&str>,
    github_client: Option<&dyn GitHubService>,
    root: &Path,
) -> Result<Vec<String>> {
    let mut checks = Vec::new();

    if args.force {
        checks.push("Working tree and index check skipped (--force)".to_string());
    } else {
        ensure_clean_worktree(root)?;
        checks.push("Working tree and index are clean".to_string());
    }

    if let Some(branch_name) = branch_name {
        let remote = "origin";
        let remote_checked =
            ensure_release_branch_available(root, branch_name, remote, release_config.push)?;
        checks.push(format!(
            "Release branch `{branch_name}` is available locally"
        ));

        if remote_checked {
            checks.push(format!("Remote `{remote}` is configured"));
            checks.push(format!(
                "Release branch `{branch_name}` is available on `{remote}`"
            ));
        }
    }

    if release_config
        .pre_commit_commands
        .as_ref()
        .is_some_and(|commands| !commands.is_empty())
    {
        checks.push("Pre-commit commands have a commit message".to_string());
    }

    if release_config.pull_request.is_some() {
        github_client
            .context("Pull request creation requires a GitHub client")?
            .ensure_authenticated()?;
        checks.push("GitHub authentication is available".to_string());
    }

    Ok(checks)
}

fn confirm_changes(stdout: &mut impl std::fmt::Write) -> Result<bool> {
    write!(stdout, "Proceed with these changes? (y/n):")?;

    io::Write::flush(&mut io::stdout())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}
