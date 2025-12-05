use clap::{
    Args, Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum VersionFormat {
    /// Display the version as plain text.
    Text,
    /// Display the version as JSON.
    Json,
}

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "seal", author, version = crate::version::seal_version_string())]
#[command(about = "An extremely fast release management tool.")]
#[command(
    after_help = "Use `seal help` for more details.",
    after_long_help = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[command(styles=STYLES)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Box<Commands>,

    #[command(flatten)]
    pub top_level: TopLevelArgs,
}

#[derive(Parser)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub struct TopLevelArgs {
    #[command(flatten)]
    pub global_args: Box<GlobalArgs>,

    #[arg(global = true, short, long, action = clap::ArgAction::HelpShort, help_heading = "Global options")]
    help: Option<bool>,

    /// Display the seal version.
    #[arg(short = 'V', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "Global options", next_display_order = 1000)]
pub struct GlobalArgs {
    /// Use quiet output.
    ///
    /// Repeating this option, e.g., `-qq`, will enable a silent mode in which
    /// seal will write no output to stdout.
    #[arg(global = true, action = clap::ArgAction::Count, long, short, conflicts_with = "verbose")]
    pub quiet: u8,

    /// Use verbose output.
    #[arg(global = true, action = clap::ArgAction::Count, long, short, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Hide all progress outputs.
    ///
    /// For example, spinners or progress bars.
    #[arg(global = true, long, value_parser = clap::builder::BoolishValueParser::new())]
    pub no_progress: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "self")]
    Self_(SelfNamespace),
}

#[derive(Args)]
pub struct SelfNamespace {
    #[command(subcommand)]
    pub command: SelfCommand,
}

#[derive(Subcommand)]
pub enum SelfCommand {
    /// Display seal's version
    Version {
        /// Only print the version
        #[arg(long)]
        short: bool,
        #[arg(long, value_enum, default_value = "text")]
        output_format: VersionFormat,
    },
}
