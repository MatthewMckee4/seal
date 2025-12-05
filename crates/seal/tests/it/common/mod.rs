// The `unreachable_pub` is to silence false positives in RustRover.
#![allow(dead_code, unreachable_pub)]

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// Test context for running seal commands.
pub struct TestContext {
    pub root: PathBuf,
    /// The root temporary directory for this test.
    _root: tempfile::TempDir,
}

impl TestContext {
    /// Create a new test context with a temporary directory.
    pub fn new() -> Self {
        let _root = tempfile::TempDir::new().expect("Failed to create temp dir");
        Self {
            root: _root.path().to_path_buf(),
            _root,
        }
    }

    /// Get the path to the temporary directory.
    pub fn temp_path(&self) -> PathBuf {
        self.root.clone()
    }

    /// Create a `seal help` command with options shared across scenarios.
    #[allow(clippy::unused_self)]
    pub fn help(&self) -> Command {
        let mut command = Self::new_command();
        command.arg("help");
        command
    }

    /// Create a seal command for testing.
    #[allow(clippy::unused_self)]
    pub fn command(&self) -> Command {
        Self::new_command()
    }

    /// Creates a new `Command` that is intended to be suitable for use in
    /// all tests.
    fn new_command() -> Command {
        Self::new_command_with(&get_bin())
    }

    /// Creates a new `Command` that is intended to be suitable for use in
    /// all tests, but with the given binary.
    fn new_command_with(bin: &Path) -> Command {
        Command::new(bin)
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the seal binary that cargo built before launching the tests.
///
/// <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates>
pub fn get_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_seal"))
}

/// Common filters for snapshot testing.
pub static INSTA_FILTERS: &[(&str, &str)] = &[
    // Normalize Windows line endings
    (r"\r\n", "\n"),
    // Normalize Windows paths
    (r"\\", "/"),
];

/// Get the function name for snapshot naming.
///
/// This macro captures the function name at compile time.
#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // Extract the function name from the full path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

/// Execute the command and format its output status, stdout and stderr into a snapshot string.
#[allow(clippy::print_stderr)]
pub fn run_and_format(
    cmd: &mut Command,
    filters: &[(&str, &str)],
    _test_name: &str,
) -> (String, std::process::Output) {
    let program = cmd.get_program().to_string_lossy().to_string();

    let output = cmd
        .output()
        .unwrap_or_else(|err| panic!("Failed to spawn {program}: {err}"));

    eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Unfiltered output ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!(
        "----- stdout -----\n{}\n----- stderr -----\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    eprintln!("────────────────────────────────────────────────────────────────────────────────\n");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut snapshot = format!(
        "success: {}\nexit_code: {}\n----- stdout -----\n{}----- stderr -----\n{}",
        output.status.success(),
        output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    );

    // Apply filters
    for (pattern, replacement) in filters {
        snapshot = snapshot.replace(pattern, replacement);
    }

    (snapshot, output)
}

/// Run snapshot testing with the seal command.
///
/// By default, applies common filters for cross-platform compatibility.
#[allow(unused_macros)]
#[macro_export]
macro_rules! seal_snapshot {
    ($cmd:expr, @$snapshot:literal) => {{
        seal_snapshot!($crate::common::INSTA_FILTERS.to_vec(), $cmd, @$snapshot)
    }};
    ($filters:expr, $cmd:expr, @$snapshot:literal) => {{
        let (snapshot, output) = $crate::common::run_and_format(
            $cmd,
            &$filters,
            $crate::function_name!(),
        );
        ::insta::assert_snapshot!(snapshot, @$snapshot);
        output
    }};
}

#[allow(unused_imports)]
pub(crate) use seal_snapshot;
