use std::fmt::Write as _;
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
    let current_version = &config.release.current_version;

    let new_version =
        seal_bump::calculate_version(current_version, &version_bump).context(format!(
            "Failed to calculate new version from '{current_version}' with bump '{version_bump}'"
        ))?;

    writeln!(
        stdout,
        "Bumping version from {current_version} to {new_version}"
    )?;

    let branch_name = config
        .release
        .branch_name
        .as_str()
        .replace("{version}", &new_version);
    let commit_message = config
        .release
        .commit_message
        .as_str()
        .replace("{version}", &new_version);

    writeln!(stdout, "Creating branch: {branch_name}")?;
    create_git_branch(&branch_name)?;

    writeln!(stdout, "Updating version files...")?;
    update_version_files(
        workspace.root(),
        &config.release.version_files,
        current_version,
        &new_version,
    )?;

    writeln!(stdout, "Committing changes...")?;
    commit_changes(&commit_message)?;

    if !args.no_push {
        writeln!(stdout, "Pushing branch to remote...")?;
        push_branch(&branch_name)?;

        if !args.no_pr {
            writeln!(stdout, "Creating pull request...")?;
            create_pull_request(&new_version)?;
        }
    }

    writeln!(stdout, "Successfully bumped to {new_version}")?;

    Ok(ExitStatus::Success)
}

fn create_git_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create git branch: {stderr}");
    }

    Ok(())
}

fn update_version_files(
    root: &std::path::Path,
    version_files: &[seal_project::VersionFile],
    current_version: &str,
    new_version: &str,
) -> Result<()> {
    for version_file in version_files {
        let full_path = root.join(version_file.path());
        if !full_path.exists() {
            anyhow::bail!("Version file not found: {}", full_path.display());
        }

        let content = std::fs::read_to_string(&full_path)
            .context(format!("Failed to read {}", full_path.display()))?;

        let updated_content = update_version_in_content(
            &content,
            current_version,
            new_version,
            version_file.search_pattern(),
            version_file.version_template(),
        )?;

        std::fs::write(&full_path, updated_content)
            .context(format!("Failed to write {}", full_path.display()))?;
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
