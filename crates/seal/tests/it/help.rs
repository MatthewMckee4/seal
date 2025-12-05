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
      self  Manage the seal executable
      help  Display documentation for a command

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
      self  Manage the seal executable
      help  Display documentation for a command

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
      self  Manage the seal executable
      help  Display documentation for a command

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
