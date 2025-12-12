use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;
use seal_cli::{Cli, ColorChoice, Commands, GenerateCommand, SelfCommand, ValidateCommand};
use seal_logging::SealFormat;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod printer;
mod settings;
mod version;

use crate::{printer::Printer, settings::GlobalSettings};

#[derive(Debug, Clone, Copy)]
enum Level {
    Default,
    Verbose,
    Debug,
    Trace,
    TraceAll,
}

fn setup_logging(level: Level, color: ColorChoice) {
    let filter = match level {
        Level::Default => EnvFilter::new("warn"),
        Level::Verbose => EnvFilter::new("info"),
        Level::Debug => EnvFilter::new("debug"),
        Level::Trace => EnvFilter::new("seal=trace"),
        Level::TraceAll => EnvFilter::new("trace"),
    };

    let (ansi, color_choice) =
        match color.and_colorchoice(anstream::Stderr::choice(&std::io::stderr())) {
            ColorChoice::Always => (true, anstream::ColorChoice::Always),
            ColorChoice::Never => (false, anstream::ColorChoice::Never),
            ColorChoice::Auto => unreachable!("anstream can't return auto as choice"),
        };
    let writer = std::sync::Mutex::new(anstream::AutoStream::new(std::io::stderr(), color_choice));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(SealFormat::default())
                .with_writer(writer)
                .with_ansi(ansi)
                .with_filter(filter),
        )
        .init();
}

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(status) => status.into(),
        Err(err) => {
            #[allow(clippy::print_stderr)]
            {
                let mut causes = err.chain();
                eprintln!(
                    "{}: {}",
                    "error".red().bold(),
                    causes.next().unwrap().to_string().trim()
                );
                for cause in causes {
                    eprintln!(
                        "  {}: {}",
                        "Caused by".red().bold(),
                        cause.to_string().trim()
                    );
                }
            }
            ExitStatus::Error.into()
        }
    }
}

async fn run(cli: Cli) -> Result<ExitStatus> {
    // Resolve the global settings.
    let globals = GlobalSettings::resolve(&cli.top_level.global_args);

    // Setup logging based on verbosity level.
    let log_level = match globals.verbose {
        0 => Level::Default,
        1 => Level::Verbose,
        2 => Level::Debug,
        3 => Level::Trace,
        _ => Level::TraceAll,
    };

    setup_logging(log_level, globals.color);

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

    match *cli.command {
        Commands::Self_(self_ns) => match self_ns.command {
            SelfCommand::Version {
                short,
                output_format,
            } => {
                commands::self_version(short, output_format, printer)?;
                Ok(ExitStatus::Success)
            }
        },
        Commands::Validate(validate_ns) => match validate_ns.command {
            ValidateCommand::Config { config_file } => {
                commands::validate_config(config_file, printer)
            }
            ValidateCommand::Project { project } => commands::validate_project(project, printer),
        },
        Commands::Bump(bump_args) => commands::bump(&bump_args, printer).await,
        Commands::Generate(generate_ns) => match generate_ns.command {
            GenerateCommand::Changelog {
                dry_run,
                max_prs,
                overwrite,
            } => commands::generate_changelog(dry_run, printer, overwrite, max_prs).await,
        },
        Commands::Help(args) => commands::help(
            args.command.unwrap_or_default().as_slice(),
            printer,
            args.no_pager,
        ),
    }
}
