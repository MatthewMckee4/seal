use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::Result;
use seal_project::ProjectWorkspace;

use crate::{ExitStatus, printer::Printer};

/// Validate only the configuration file
/// If `config_file` is None, discovers seal.toml in the current directory
pub fn validate_config(config_file: Option<PathBuf>, printer: Printer) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let workspace = if let Some(path) = config_file {
        ProjectWorkspace::from_config_file(&path)?
    } else {
        ProjectWorkspace::discover()?
    };

    writeln!(
        stdout,
        "Config file `{}` is valid",
        workspace.config_file().display()
    )?;
    Ok(ExitStatus::Success)
}

/// Validate full project workspace including members
/// If `project_path` is None, uses the current directory
pub fn validate_project(project_path: Option<PathBuf>, printer: Printer) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let workspace = if let Some(path) = project_path {
        ProjectWorkspace::from_project_path(&path)?
    } else {
        ProjectWorkspace::discover()?
    };

    writeln!(stdout, "Project validation successful")?;
    if !workspace.members().is_empty() {
        writeln!(
            stdout,
            "Found {} workspace member(s)",
            workspace.members().len()
        )?;
    }
    Ok(ExitStatus::Success)
}
