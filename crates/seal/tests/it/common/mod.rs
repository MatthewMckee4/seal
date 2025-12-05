// The `unreachable_pub` is to silence false positives in RustRover.
#![allow(dead_code, unreachable_pub)]

use assert_cmd::Command;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Test context for running seal commands.
pub struct TestContext {
    pub root: PathBuf,
    /// Standard filters for this test context.
    filters: Vec<(String, String)>,
    /// The root temporary directory for this test.
    _root: tempfile::TempDir,
}

impl TestContext {
    /// Create a new test context with a temporary directory.
    pub fn new() -> Self {
        let _root = tempfile::TempDir::new().expect("Failed to create temp dir");

        let filters = Vec::new();

        Self {
            root: _root.path().to_path_buf(),
            filters,
            _root,
        }
    }

    /// Get the path to the temporary directory.
    pub fn temp_path(&self) -> PathBuf {
        self.root.clone()
    }

    /// Standard snapshot filters _plus_ those for this test context.
    pub fn filters(&self) -> Vec<(&str, &str)> {
        // Put test context snapshots before the default filters
        // This ensures we don't replace other patterns inside paths from the test context first
        self.filters
            .iter()
            .map(|(p, r)| (p.as_str(), r.as_str()))
            .chain(INSTA_FILTERS.iter().copied())
            .collect()
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
    // Rewrite Windows output to Unix output
    (r"\\([\w\d]|\.)", "/$1"),
    (r"seal\.exe", "seal"),
    // seal version display
    (
        r"seal(-.*)? \d+\.\d+\.\d+(-(alpha|beta|rc)\.\d+)?",
        r"seal [VERSION]",
    ),
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

/// Helper method to apply filters to a string. Useful when `!seal_snapshot` cannot be used.
pub fn apply_filters<T: AsRef<str>>(mut snapshot: String, filters: impl AsRef<[(T, T)]>) -> String {
    for (matcher, replacement) in filters.as_ref() {
        // TODO(konstin): Cache regex compilation
        let re = Regex::new(matcher.as_ref()).expect("Do you need to regex::escape your filter?");
        if re.is_match(&snapshot) {
            snapshot = re.replace_all(&snapshot, replacement.as_ref()).to_string();
        }
    }
    snapshot
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

    let snapshot = apply_filters(
        format!(
            "success: {:?}\nexit_code: {}\n----- stdout -----\n{}\n----- stderr -----\n{}",
            output.status.success(),
            output.status.code().unwrap_or(!0),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        filters,
    );

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
