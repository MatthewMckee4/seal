use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod printer;
mod settings;
mod version;

use cli::{Cli, Commands, SelfCommand};

use crate::{printer::Printer, settings::GlobalSettings};

#[derive(Copy, Clone)]
pub(crate) enum ExitStatus {
    /// The command succeeded.
    Success,

    /// The command failed due to an error in the user input.
    #[expect(unused)]
    Failure,

    /// The command failed with an unexpected error.
    Error,

    /// The command's exit status is propagated from an external command.
    #[expect(unused)]
    External(u8),
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => Self::from(0),
            ExitStatus::Failure => Self::from(1),
            ExitStatus::Error => Self::from(2),
            ExitStatus::External(code) => Self::from(code),
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    run(&cli).unwrap_or(ExitStatus::Error).into()
}

fn run(cli: &Cli) -> Result<ExitStatus> {
    // Resolve the global settings.
    let globals = GlobalSettings::resolve(&cli.top_level.global_args);

    // Configure the `Printer`, which controls user-facing output in the CLI.
    let printer = if globals.quiet == 1 {
        Printer::Quiet
    } else if globals.quiet > 1 {
        Printer::Silent
    } else if globals.verbose > 0 {
        Printer::Verbose
    } else if globals.no_progress {
        Printer::NoProgress
    } else {
        Printer::Default
    };

    match cli.command.as_ref() {
        Commands::Self_(self_ns) => match self_ns.command {
            SelfCommand::Version {
                short,
                output_format,
            } => {
                commands::self_version(short, output_format, printer)?;
                Ok(ExitStatus::Success)
            }
        },
    }
}
