use std::path::PathBuf;

pub use clap::builder::Styles;
pub use clap::builder::styling::{AnsiColor, Effects, Style};
pub use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum VersionFormat {
    /// Display the version as plain text.
    Text,
    /// Display the version as JSON.
    Json,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ColorChoice {
    /// Enables colored output only when the output is going to a terminal or TTY with support.
    Auto,

    /// Enables colored output regardless of the detected environment.
    Always,

    /// Disables colored output.
    Never,
}

impl ColorChoice {
    /// Combine self (higher priority) with an [`anstream::ColorChoice`] (lower priority).
    ///
    /// This method allows prioritizing the user choice, while using the inferred choice for a
    /// stream as default.
    #[must_use]
    pub fn and_colorchoice(self, next: anstream::ColorChoice) -> Self {
        match self {
            Self::Auto => match next {
                anstream::ColorChoice::Auto => Self::Auto,
                anstream::ColorChoice::Always | anstream::ColorChoice::AlwaysAnsi => Self::Always,
                anstream::ColorChoice::Never => Self::Never,
            },
            Self::Always | Self::Never => self,
        }
    }
}

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "seal", author, version = seal_version::version())]
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

    /// Display the concise help for this command.
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

    /// Disable colors.
    #[arg(global = true, long, hide = true, conflicts_with = "color")]
    pub no_color: bool,

    /// Control the use of color in output.
    ///
    /// By default, seal will automatically detect support for colors when writing to a terminal.
    #[arg(
        global = true,
        long,
        value_enum,
        conflicts_with = "no_color",
        value_name = "COLOR_CHOICE"
    )]
    pub color: Option<ColorChoice>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage the seal executable.
    #[command(name = "self")]
    Self_(SelfNamespace),
    /// Validate project configuration and structure.
    Validate(ValidateNamespace),
    /// Bump version and create release branch.
    Bump(BumpArgs),
    /// Generate project files.
    Generate(GenerateNamespace),
    /// Display documentation for a command.
    #[command(help_template = "\
{about-with-newline}
{usage-heading} {usage}{after-help}
",
        after_help = format!("\
{heading}Options:{heading:#}
  {option}--no-pager{option:#} Disable pager when printing help
",
            heading = Style::new().bold().underline(),
            option = Style::new().bold(),
        ),
    )]
    Help(HelpArgs),
}

#[derive(Args, Debug)]
pub struct BumpArgs {
    /// Version bump to perform (e.g., 'major', 'minor', 'patch', 'alpha', 'major-beta', or '1.2.3')
    pub version: String,

    /// Show what would be done without making any changes
    #[arg(long)]
    pub dry_run: bool,

    /// Skip generating or updating the changelog
    #[arg(long)]
    pub no_changelog: bool,
}

#[derive(Args, Debug)]
pub struct HelpArgs {
    /// Disable pager when printing help
    #[arg(long)]
    pub no_pager: bool,

    pub command: Option<Vec<String>>,
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

#[derive(Args)]
pub struct ValidateNamespace {
    #[command(subcommand)]
    pub command: ValidateCommand,
}

#[derive(Subcommand)]
pub enum ValidateCommand {
    /// Validate workspace configuration file
    ///
    /// If no config path is provided, discovers seal.toml in the current directory.
    Config {
        /// Path to the config file (seal.toml)
        #[arg(long)]
        config_file: Option<PathBuf>,
    },
    /// Validate full project workspace including members
    ///
    /// If no project path is provided, uses the current directory.
    Project {
        /// Path to the project directory
        #[arg(long, short)]
        project: Option<PathBuf>,
    },
}

#[derive(Args)]
pub struct GenerateNamespace {
    #[command(subcommand)]
    pub command: GenerateCommand,
}

#[derive(Subcommand)]
pub enum GenerateCommand {
    /// Generate changelog
    ///
    /// We look at all releases, and get all PRs from that release.
    /// Then add them to the changelog.
    ///
    /// We do not include PRs since the latest release.
    Changelog {
        /// Perform a dry run without modifying files and print the result to stdout
        #[arg(long)]
        dry_run: bool,

        /// Maximum number of PRs to fetch.
        ///
        /// Be aware that this can be slow or can fail due to high number of requests if the number is too high.
        ///
        /// Note that this does not mean that you will see this number of PRs in the changelog, this just means
        /// before filtering, we will fetch this number of PRs.
        ///
        /// Defaults to 100.
        #[arg(long)]
        max_prs: Option<usize>,

        /// Overwrite the changelog file if it already exists
        #[arg(long, default_missing_value = "true", num_args = 0..1)]
        overwrite: Option<bool>,
    },
}
