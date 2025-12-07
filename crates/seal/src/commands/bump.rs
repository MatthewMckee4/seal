use std::fmt::Write as _;
use std::io;
use std::process::Command;

use anyhow::{Context, Result};
use seal_bump::VersionBump;
use seal_project::ProjectWorkspace;

use crate::ExitStatus;
use crate::cli::BumpArgs;
use crate::printer::Printer;

pub fn bump(args: &BumpArgs, printer: Printer) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let version_bump: VersionBump = args
        .version
        .parse()
        .context("Failed to parse version bump argument")?;

    let workspace = ProjectWorkspace::discover()?;
    let config = workspace.config();
    let current_version_string = &config.release.current_version;

    let new_version_string = seal_bump::calculate_version(current_version_string, &version_bump)
        .context(format!(
            "Failed to calculate new version from '{current_version_string}' with bump '{version_bump}'"
        ))?
        .to_string();

    writeln!(
        stdout,
        "Bumping version from {current_version_string} to {new_version_string}"
    )?;

    let branch_name = config
        .release
        .branch_name
        .as_ref()
        .map(|bn| bn.as_str().replace("{version}", &new_version_string));
    let commit_message = config
        .release
        .commit_message
        .as_ref()
        .map(|cm| cm.as_str().replace("{version}", &new_version_string));

    writeln!(stdout)?;

    let version_files = config.release.version_files.as_deref().unwrap_or(&[]);

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
        &new_version_string,
    )?;

    for change in &changes {
        display_diff(
            &mut stdout,
            &change.path,
            &change.old_content,
            &change.new_content,
        )?;
    }

    writeln!(stdout)?;

    let has_git_operations = branch_name.is_some() || commit_message.is_some();

    if has_git_operations {
        writeln!(stdout, "Commands to be executed:")?;
        if let Some(branch) = &branch_name {
            writeln!(stdout, "  git checkout -b {branch}")?;
        }
        writeln!(stdout, "  # Update version files")?;
        writeln!(stdout, "  # Update seal.toml")?;
        if let Some(message) = &commit_message {
            writeln!(stdout, "  git add -A")?;
            writeln!(stdout, "  git commit -m \"{message}\"")?;
        }
        if config.release.push {
            if let Some(branch) = &branch_name {
                writeln!(stdout, "  git push -u origin {branch}")?;
            }
            if config.release.create_pr {
                writeln!(
                    stdout,
                    "  gh pr create --title \"Release v{new_version_string}\" --body \"Automated release for version {new_version_string}\""
                )?;
            }
        }
    } else {
        writeln!(stdout, "Changes to be made:")?;
        writeln!(stdout, "  # Update version files")?;
        writeln!(stdout, "  # Update seal.toml")?;
        writeln!(stdout)?;
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

    if config.release.confirm {
        writeln!(stdout)?;
        if !confirm_changes(&mut stdout)? {
            writeln!(stdout, "Aborted.")?;
            return Ok(ExitStatus::Success);
        }
    }

    writeln!(stdout)?;

    if let Some(branch) = &branch_name {
        writeln!(stdout, "Creating branch: {branch}")?;
        create_git_branch(branch)?;
    }

    writeln!(stdout, "Updating version files...")?;
    apply_version_file_changes(workspace.root(), &changes)?;

    writeln!(stdout, "Updating seal.toml...")?;
    update_seal_toml(
        workspace.root(),
        current_version_string,
        &new_version_string,
    )?;

    if let Some(message) = &commit_message {
        writeln!(stdout, "Committing changes...")?;
        commit_changes(message)?;
    }

    if config.release.push {
        if let Some(branch) = &branch_name {
            writeln!(stdout, "Pushing branch to remote...")?;
            push_branch(branch)?;

            if config.release.create_pr {
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

struct FileChange {
    path: String,
    old_content: String,
    new_content: String,
}

fn calculate_version_file_changes(
    root: &std::path::Path,
    version_files: &[seal_project::VersionFile],
    current_version: &str,
    new_version: &str,
) -> Result<Vec<FileChange>> {
    let mut changes = Vec::new();

    for version_file in version_files {
        let full_path = root.join(version_file.path());
        if !full_path.exists() {
            anyhow::bail!("Version file not found: {}", full_path.display());
        }

        let old_content = std::fs::read_to_string(&full_path)
            .context(format!("Failed to read {}", full_path.display()))?;

        let new_content = update_version_in_content(
            &old_content,
            current_version,
            new_version,
            version_file.search_pattern(),
            version_file.version_template(),
        )?;

        changes.push(FileChange {
            path: version_file.path().to_string(),
            old_content,
            new_content,
        });
    }

    Ok(changes)
}

fn apply_version_file_changes(root: &std::path::Path, changes: &[FileChange]) -> Result<()> {
    for change in changes {
        let full_path = root.join(&change.path);
        std::fs::write(&full_path, &change.new_content)
            .context(format!("Failed to write {}", full_path.display()))?;
    }

    Ok(())
}

fn display_diff(
    stdout: &mut impl std::fmt::Write,
    path: &str,
    old_content: &str,
    new_content: &str,
) -> Result<()> {
    writeln!(stdout)?;
    writeln!(stdout, "diff --git a/{path} b/{path}")?;
    writeln!(stdout, "--- a/{path}")?;
    writeln!(stdout, "+++ b/{path}")?;

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    for (i, (old_line, new_line)) in old_lines.iter().zip(new_lines.iter()).enumerate() {
        if old_line != new_line {
            writeln!(stdout, "@@ -{},{} +{},{} @@", i + 1, 1, i + 1, 1)?;
            writeln!(stdout, "-{old_line}")?;
            writeln!(stdout, "+{new_line}")?;
        }
    }

    Ok(())
}

fn confirm_changes(stdout: &mut impl std::fmt::Write) -> Result<bool> {
    write!(stdout, "Proceed with these changes? (y/n): ")?;

    // Flush stdout to ensure the prompt is displayed before reading input
    io::Write::flush(&mut io::stdout())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn update_seal_toml(
    root: &std::path::Path,
    current_version: &str,
    new_version: &str,
) -> Result<()> {
    let seal_toml_path = root.join("seal.toml");
    let content = std::fs::read_to_string(&seal_toml_path).context("Failed to read seal.toml")?;

    let old_line = format!(r#"current-version = "{current_version}""#);
    let new_line = format!(r#"current-version = "{new_version}""#);

    if !content.contains(&old_line) {
        anyhow::bail!("Could not find current-version = \"{current_version}\" in seal.toml");
    }

    let updated_content = content.replace(&old_line, &new_line);

    std::fs::write(&seal_toml_path, updated_content).context("Failed to write seal.toml")?;

    Ok(())
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

fn update_version_in_content(
    content: &str,
    current_version: &str,
    new_version: &str,
    search_pattern: Option<&str>,
    version_template: Option<&str>,
) -> Result<String> {
    let version_str = if let Some(template) = version_template {
        format_version_with_template(new_version, template)?
    } else {
        new_version.to_string()
    };

    if let Some(pattern_str) = search_pattern {
        let search_with_current = pattern_str.replace("{version}", current_version);
        let search_with_new = pattern_str.replace("{version}", &version_str);

        if !content.contains(&search_with_current) {
            anyhow::bail!("Search pattern not found in file. Expected: {search_with_current}");
        }

        return Ok(content.replace(&search_with_current, &search_with_new));
    }

    let patterns = [
        (
            regex::Regex::new(r#"version\s*=\s*"[^"]+""#)?,
            format!(r#"version = "{version_str}""#),
        ),
        (
            regex::Regex::new(r#""version"\s*:\s*"[^"]+""#)?,
            format!(r#""version": "{version_str}""#),
        ),
        (
            regex::Regex::new(r#"__version__\s*=\s*"[^"]+""#)?,
            format!(r#"__version__ = "{version_str}""#),
        ),
    ];

    for (pattern, replacement) in &patterns {
        if pattern.is_match(content) {
            return Ok(pattern.replace(content, replacement).to_string());
        }
    }

    anyhow::bail!("No version field found in file");
}

fn format_version_with_template(version: &str, template: &str) -> Result<String> {
    let parsed_version = semver::Version::parse(version)
        .context(format!("Failed to parse version '{version}' for template"))?;

    let extra = if parsed_version.pre.is_empty() {
        String::new()
    } else {
        parsed_version.pre.to_string()
    };

    let result = template
        .replace("{major}", &parsed_version.major.to_string())
        .replace("{minor}", &parsed_version.minor.to_string())
        .replace("{patch}", &parsed_version.patch.to_string())
        .replace("{extra}", &extra);

    Ok(result)
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
