use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use seal_migrate::{migrate_rooster_config, parse_rooster_config};

use crate::{ExitStatus, printer::Printer};

pub fn migrate_rooster(
    input: Option<&Path>,
    output: Option<&Path>,
    overwrite: Option<bool>,
    printer: Printer,
) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let input = input.unwrap_or_else(|| Path::new("pyproject.toml"));
    let output = output.unwrap_or_else(|| Path::new("seal.toml"));
    let overwrite = overwrite.unwrap_or(false);

    if output.exists() && !overwrite {
        anyhow::bail!(
            "Output file '{}' already exists. Use --overwrite to overwrite.",
            output.display()
        );
    }

    let rooster_config = parse_rooster_config(input)
        .with_context(|| format!("Failed to parse rooster config from '{}'", input.display()))?;

    let (seal_config, warnings) = migrate_rooster_config(&rooster_config);

    let toml_string =
        toml::to_string_pretty(&seal_config).context("Failed to serialize seal config to TOML")?;

    std::fs::write(output, toml_string)
        .with_context(|| format!("Failed to write output to '{}'", output.display()))?;

    writeln!(
        stdout,
        "Successfully migrated rooster config to '{}'",
        output.display()
    )?;

    if !warnings.is_empty() {
        writeln!(stdout)?;
        writeln!(stdout, "Migration warnings:")?;
        for warning in warnings {
            writeln!(stdout, "  - {warning}")?;
        }
        writeln!(stdout)?;
        writeln!(
            stdout,
            "See docs/migration.md for more information about unsupported features."
        )?;
    }

    Ok(ExitStatus::Success)
}
