use seal_cli::{ColorChoice, GlobalArgs};

/// The resolved global settings to use for any invocation of the CLI.
#[derive(Debug, Clone)]
pub(crate) struct GlobalSettings {
    pub(crate) quiet: u8,
    pub(crate) verbose: u8,
    pub(crate) no_progress: bool,
    pub(crate) color: ColorChoice,
}

impl GlobalSettings {
    /// Resolve the [`GlobalSettings`] from the CLI and filesystem configuration.
    pub(crate) fn resolve(args: &GlobalArgs) -> Self {
        Self {
            quiet: args.quiet,
            verbose: args.verbose,
            no_progress: args.no_progress,
            color: if args.no_color {
                ColorChoice::Never
            } else {
                args.color.unwrap_or(ColorChoice::Auto)
            },
        }
    }
}
