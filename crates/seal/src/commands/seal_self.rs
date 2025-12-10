use std::fmt::Write;

use anyhow::Result;
use owo_colors::OwoColorize;
use seal_cli::VersionFormat;

use crate::{ExitStatus, printer::Printer, version::VersionInfo};

/// Display version information for seal itself (`seal self version`)
pub fn self_version(
    short: bool,
    output_format: VersionFormat,
    printer: Printer,
) -> Result<ExitStatus> {
    let version_info = crate::version::seal_self_version();
    print_version(&version_info, short, output_format, printer)?;

    Ok(ExitStatus::Success)
}
fn print_version(
    version: &VersionInfo,
    short: bool,
    output_format: VersionFormat,
    printer: Printer,
) -> Result<()> {
    match output_format {
        VersionFormat::Text => {
            if let Some(name) = &version.package_name {
                if !short {
                    write!(printer.stdout(), "{name} ")?;
                }
            }

            writeln!(printer.stdout(), "{}", version.cyan())?;
        }
        VersionFormat::Json => {
            let string = serde_json::to_string_pretty(&version)?;
            writeln!(printer.stdout(), "{string}")?;
        }
    }
    Ok(())
}
