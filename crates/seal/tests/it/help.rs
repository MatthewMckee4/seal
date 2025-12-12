use crate::{common::TestContext, seal_snapshot};

#[test]
fn help_command() {
    let context = TestContext::new();
    let mut cmd = context.help();

    seal_snapshot!(&mut cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    An extremely fast release management tool.

    Usage: seal [OPTIONS] <COMMAND>

    Commands:
      self      Manage the seal executable
      validate  Validate project configuration and structure
      bump      Bump version and create release branch
      generate  Generate project files
      help      Display documentation for a command

    Global options:
      -q, --quiet...     Use quiet output
      -v, --verbose...   Use verbose output
          --no-progress  Hide all progress outputs
      -h, --help         Display the concise help for this command
      -V, --version      Display the seal version

    Use `seal help <command>` for more information on a specific command.


    ----- stderr -----
    ");
}

#[test]
fn help_flag() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("--help"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    An extremely fast release management tool.

    Usage: seal [OPTIONS] <COMMAND>

    Commands:
      self      Manage the seal executable
      validate  Validate project configuration and structure
      bump      Bump version and create release branch
      generate  Generate project files
      help      Display documentation for a command

    Global options:
      -q, --quiet...     Use quiet output
      -v, --verbose...   Use verbose output
          --no-progress  Hide all progress outputs
      -h, --help         Display the concise help for this command
      -V, --version      Display the seal version

    Use `seal help` for more details.

    ----- stderr -----
    ");
}

#[test]
fn help_short_flag() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("-h"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    An extremely fast release management tool.

    Usage: seal [OPTIONS] <COMMAND>

    Commands:
      self      Manage the seal executable
      validate  Validate project configuration and structure
      bump      Bump version and create release branch
      generate  Generate project files
      help      Display documentation for a command

    Global options:
      -q, --quiet...     Use quiet output
      -v, --verbose...   Use verbose output
          --no-progress  Hide all progress outputs
      -h, --help         Display the concise help for this command
      -V, --version      Display the seal version

    Use `seal help` for more details.

    ----- stderr -----
    ");
}

#[test]
fn help_self_command() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("help").arg("self"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Manage the seal executable

    Usage: seal self [OPTIONS] <COMMAND>

    Commands:
      version  Display seal's version

    Global options:
      -q, --quiet...
              Use quiet output.
              
              Repeating this option, e.g., `-qq`, will enable a silent mode in which seal will write no
              output to stdout.

      -v, --verbose...
              Use verbose output

          --no-progress
              Hide all progress outputs.
              
              For example, spinners or progress bars.

      -h, --help
              Display the concise help for this command

    Use `seal help self <command>` for more information on a specific command.


    ----- stderr -----
    ");
}

#[test]
fn help_invalid_command() {
    let context = TestContext::new();

    seal_snapshot!(context.help().arg("invalid"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: There is no command `invalid` for `seal`. Did you mean one of:
        self
        validate
        bump
        generate
    ");
}

#[test]
fn help_invalid_command_nearest() {
    let context = TestContext::new();

    seal_snapshot!(context.help().arg("self").arg("vversion"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: There is no command `vversion` for `seal self`. Did you mean one of:
        version
    ");
}

#[test]
fn help_self_version_command() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("help").arg("self").arg("version"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Display seal's version

    Usage: seal self version [OPTIONS]

    Options:
          --short
              Only print the version

          --output-format <OUTPUT_FORMAT>
              Possible values:
              - text: Display the version as plain text
              - json: Display the version as JSON[default: text]

    Global options:
      -q, --quiet...
              Use quiet output.
              
              Repeating this option, e.g., `-qq`, will enable a silent mode in which seal will write no
              output to stdout.

      -v, --verbose...
              Use verbose output

          --no-progress
              Hide all progress outputs.
              
              For example, spinners or progress bars.

      -h, --help
              Display the concise help for this command


    ----- stderr -----
    ");
}

#[test]
fn help_command_no_pager() {
    let context = TestContext::new();

    seal_snapshot!(context.help().arg("--no-pager"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    An extremely fast release management tool.

    Usage: seal [OPTIONS] <COMMAND>

    Commands:
      self      Manage the seal executable
      validate  Validate project configuration and structure
      bump      Bump version and create release branch
      generate  Generate project files
      help      Display documentation for a command

    Global options:
      -q, --quiet...     Use quiet output
      -v, --verbose...   Use verbose output
          --no-progress  Hide all progress outputs
      -h, --help         Display the concise help for this command
      -V, --version      Display the seal version

    Use `seal help <command>` for more information on a specific command.


    ----- stderr -----
    ");
}
