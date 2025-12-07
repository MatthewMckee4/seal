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
    print_version(version_info, None, short, output_format, printer)?;

    Ok(ExitStatus::Success)
}
fn print_version(
    old_version: VersionInfo,
    new_version: Option<VersionInfo>,
    short: bool,
    output_format: VersionFormat,
    printer: Printer,
) -> Result<()> {
    match output_format {
        VersionFormat::Text => {
            if let Some(name) = &old_version.package_name {
                if !short {
                    write!(printer.stdout(), "{name} ")?;
                }
            }
            if let Some(new_version) = new_version {
                if short {
                    writeln!(printer.stdout(), "{}", new_version.cyan())?;
                } else {
                    writeln!(
                        printer.stdout(),
                        "{} => {}",
                        old_version.cyan(),
                        new_version.cyan()
                    )?;
                }
            } else {
                writeln!(printer.stdout(), "{}", old_version.cyan())?;
            }
        }
        VersionFormat::Json => {
            let final_version = new_version.unwrap_or(old_version);
            let string = serde_json::to_string_pretty(&final_version)?;
            writeln!(printer.stdout(), "{string}")?;
        }
    }
    Ok(())
}
