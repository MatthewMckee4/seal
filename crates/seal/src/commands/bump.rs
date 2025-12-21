use std::fmt::Write as _;
use std::io;
use std::sync::Arc;

use anyhow::{Context, Result};
use seal_bump::{VersionBump, calculate_version_file_changes};
use seal_command::CommandWrapper;
use seal_fs::FileResolver;
use seal_github::GitHubService;
use seal_project::ProjectWorkspace;

use seal_cli::BumpArgs;

use crate::ExitStatus;
use crate::printer::Printer;

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

    #[cfg(feature = "integration-test")]
    let github_client: Arc<dyn GitHubService> = {
        #[cfg(any(test, feature = "integration-test"))]
        use seal_github::MockGithubClient;
        Arc::new(MockGithubClient::new())
    };
    #[cfg(not(feature = "integration-test"))]
    let github_client: Arc<dyn GitHubService> = {
        use seal_github::{GitHubClient, get_git_remote_url, parse_github_repo};

        let repo_url = get_git_remote_url(workspace.root())?;
        let (owner, repo) = parse_github_repo(&repo_url)?;
        Arc::new(GitHubClient::new(owner, repo)?)
    };

    let mut file_changes = calculate_version_file_changes(
        workspace.root(),
        version_files,
        current_version_string,
        &new_version,
        &file_resolver,
    )?;

    if !args.no_changelog {
        if let Some(changelog_config) = config.changelog.as_ref() {
            let changes = seal_changelog::prepare_changelog_changes(
                workspace.root(),
                &new_version_string,
                changelog_config,
                &github_client,
            )
            .await
            .context("Failed to prepare changelog")?;

            file_changes.extend(changes);
        } else {
            tracing::info!(
                "Skipping changelog update because no `[changelog]` section was found in the configuration."
            );
        }
    } else {
        tracing::info!("Skipping changelog update because `--no-changelog` was provided.");
    }

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

    let mut commands = Vec::new();

    if let Some(branch) = &branch_name {
        commands.push(CommandWrapper::create_branch(branch));
    }

    if let Some(message) = &commit_message {
        commands.push(CommandWrapper::git_add_all());
        commands.push(CommandWrapper::git_commit(message));
    }

    if release_config.push {
        if let Some(branch) = &branch_name {
            commands.push(CommandWrapper::git_push_branch(branch));
        }
    }

    if args.dry_run {
        writeln!(stdout, "Dry run complete. No changes made.")?;
        return Ok(ExitStatus::Success);
    }

    if !commands.is_empty() {
        writeln!(stdout, "Commands to be executed:")?;

        for command in &commands {
            writeln!(stdout, "  `{}`", command.as_string())?;
        }

        writeln!(stdout)?;
    }

    if release_config.confirm {
        if !confirm_changes(&mut stdout)? {
            writeln!(printer.stderr())?;
            writeln!(printer.stderr(), "No changes applied.")?;
            return Ok(ExitStatus::Success);
        }
        writeln!(stdout)?;
    }

    writeln!(stdout, "Updating files...")?;

    file_changes.apply()?;

    for command in &commands {
        command.execute(&mut stdout, workspace.root())?;
    }

    writeln!(stdout, "Successfully bumped to {new_version_string}")?;

    Ok(ExitStatus::Success)
}

fn confirm_changes(stdout: &mut impl std::fmt::Write) -> Result<bool> {
    write!(stdout, "Proceed with these changes? (y/n):")?;

    io::Write::flush(&mut io::stdout())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}
