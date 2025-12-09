use std::fmt::Write as _;
use std::io;
use std::process::Command;

use anyhow::{Context, Result};
use seal_bump::{VersionBump, calculate_version_file_changes};
use seal_fs::FileResolver;
use seal_project::ProjectWorkspace;

use seal_cli::BumpArgs;

use crate::ExitStatus;
use crate::printer::Printer;

pub fn bump(args: &BumpArgs, printer: Printer) -> Result<ExitStatus> {
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

    let new_version = seal_bump::calculate_new_version(current_version_string, &version_bump)
        .context(format!(
            "Failed to calculate new version from '{current_version_string}' with bump '{version_bump}'"
        ))?
       ;

    let new_version_string = new_version.to_string();

    writeln!(
        stdout,
        "Bumping version from {current_version_string} to {new_version_string}"
    )?;

    let branch_name = release_config
        .branch_name
        .as_ref()
        .map(|bn| bn.as_str().replace("{version}", &new_version_string));
    let commit_message = release_config
        .commit_message
        .as_ref()
        .map(|cm| cm.as_str().replace("{version}", &new_version_string));

    writeln!(stdout)?;

    let version_files = release_config.version_files.as_deref().unwrap_or(&[]);

    if version_files.is_empty() {
        writeln!(
            stdout,
            "Warning: No version files configured - only seal.toml will be updated"
        )?;
        writeln!(stdout)?;
    }

    writeln!(stdout, "Preview of changes:")?;
    writeln!(stdout, "-------------------")?;

    let changes = calculate_version_file_changes(
        workspace.root(),
        version_files,
        current_version_string,
        &new_version,
    )?;

    let file_resolver = FileResolver::new(workspace.root().clone());

    for change in &changes {
        change.display_diff(&mut stdout, &file_resolver)?;
    }

    let changelog_changes = if !args.no_changelog {
        if let Some(changelog_config) = config.changelog.as_ref() {
            match seal_changelog::prepare_changelog_changes(
                workspace.root(),
                &new_version_string,
                changelog_config,
            ) {
                Ok(changes) => {
                    for change in &changes {
                        change.display_diff(&mut stdout, &file_resolver)?;
                    }
                    Some(changes)
                }
                Err(e) => {
                    writeln!(stdout, "Warning: Failed to prepare changelog: {e}")?;
                    None
                }
            }
        } else {
            writeln!(
                stdout,
                "Skipping changelog update because no `[changelog]` section was found in the configuration."
            )?;
            None
        }
    } else {
        writeln!(
            stdout,
            "Skipping changelog update because `--no-changelog` was provided."
        )?;
        None
    };

    writeln!(stdout)?;

    let has_git_operations = branch_name.is_some() || commit_message.is_some();

    writeln!(stdout, "Changes to be made:")?;
    for change in &changes {
        writeln!(stdout, "  - Update `{}`", change.path().display())?;
    }
    if let Some(ref changelog) = changelog_changes {
        for change in changelog {
            writeln!(stdout, "  - Update `{}`", change.path().display())?;
        }
    }
    writeln!(stdout)?;

    if has_git_operations {
        writeln!(stdout, "Commands to be executed:")?;
        if let Some(branch) = &branch_name {
            writeln!(stdout, "  `git checkout -b {branch}`")?;
        }

        if let Some(message) = &commit_message {
            writeln!(stdout, "  `git add -A`")?;
            writeln!(stdout, "  `git commit -m \"{message}\"`")?;
        }
        if release_config.push {
            if let Some(branch) = &branch_name {
                writeln!(stdout, "  `git push -u origin {branch}`")?;
            }
            if release_config.create_pr {
                writeln!(
                    stdout,
                    "  `gh pr create --title \"Release v{new_version_string}\" --body \"Automated release for version {new_version_string}\"`"
                )?;
            }
        }
    } else {
        writeln!(
            stdout,
            "Note: No branch or commit will be created (branch-name and commit-message not configured)"
        )?;
    }

    if args.dry_run {
        writeln!(stdout)?;
        writeln!(stdout, "Dry run complete. No changes made.")?;
        return Ok(ExitStatus::Success);
    }

    if release_config.confirm {
        writeln!(stdout)?;
        if !confirm_changes(&mut stdout)? {
            writeln!(stdout)?;
            writeln!(stdout, "No changes applied.")?;
            return Ok(ExitStatus::Success);
        }
    }

    writeln!(stdout)?;

    if let Some(branch) = &branch_name {
        writeln!(stdout, "Creating branch: {branch}")?;
        create_git_branch(branch)?;
    }

    writeln!(stdout, "Updating version files...")?;
    changes.apply()?;

    if let Some(changelog) = changelog_changes {
        writeln!(stdout, "Updating changelog...")?;
        changelog.apply()?;
    }

    if let Some(message) = &commit_message {
        writeln!(stdout, "Committing changes...")?;
        commit_changes(message)?;
    }

    if release_config.push {
        if let Some(branch) = &branch_name {
            writeln!(stdout, "Pushing branch to remote...")?;
            push_branch(branch)?;

            if release_config.create_pr {
                writeln!(stdout, "Creating pull request...")?;
                create_pull_request(&new_version_string)?;
            }
        }
    }

    writeln!(stdout, "Successfully bumped to {new_version_string}")?;

    if branch_name.is_none() && commit_message.is_none() {
        writeln!(stdout, "Note: No git branch or commit was created")?;
    } else if branch_name.is_none() {
        writeln!(stdout, "Note: No git branch was created")?;
    } else if commit_message.is_none() {
        writeln!(stdout, "Note: No git commit was created")?;
    }

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

fn create_git_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        anyhow::bail!("Failed to create git branch");
    }

    Ok(())
}

fn commit_changes(message: &str) -> Result<()> {
    Command::new("git")
        .args(["add", "-A"])
        .output()
        .context("Failed to execute git add")?;

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to commit changes: {stderr}");
    }

    Ok(())
}

fn push_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "-u", "origin", branch_name])
        .output()
        .context("Failed to execute git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to push branch: {stderr}");
    }

    Ok(())
}

fn create_pull_request(version: &str) -> Result<()> {
    let title = format!("Release v{version}");
    let body = format!("Automated release for version {version}");

    let output = Command::new("gh")
        .args(["pr", "create", "--title", &title, "--body", &body])
        .output()
        .context("Failed to execute gh pr create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create pull request: {stderr}");
    }

    Ok(())
}
