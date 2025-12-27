use std::fmt::Write as _;

use anyhow::Result;
use seal_changelog::DEFAULT_CHANGELOG_PATH;
use seal_project::ProjectWorkspace;

use crate::ExitStatus;
use crate::printer::Printer;

pub async fn generate_release_body(printer: Printer) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let workspace = ProjectWorkspace::discover()?;
    let config = workspace.config();

    let changelog_path = config
        .changelog
        .as_ref()
        .and_then(|c| c.changelog_path.clone())
        .unwrap_or_else(|| workspace.root().join(DEFAULT_CHANGELOG_PATH));

    if !changelog_path.exists() {
        anyhow::bail!("Changelog not found at `{}`", changelog_path.display());
    }

    let changelog_content = fs_err::read_to_string(&changelog_path)?;
    let release_body = seal_changelog::create_release_body(&changelog_content)?;

    let json = serde_json::to_string_pretty(&release_body)?;
    writeln!(stdout, "{json}")?;

    Ok(ExitStatus::Success)
}
