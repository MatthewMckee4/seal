// The `unreachable_pub` is to silence false positives in RustRover.
#![allow(dead_code, unreachable_pub)]

use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Test context for running seal commands.
pub struct TestContext {
    pub root: ChildPath,
    /// Standard filters for this test context.
    filters: Vec<(String, String)>,
    /// The temporary directory for this test.
    pub _root: tempfile::TempDir,
}

impl TestContext {
    /// Create a new test context with a temporary directory.
    pub fn new() -> Self {
        let root = tempfile::TempDir::with_prefix("seal-test")
            .expect("Failed to create test root directory");

        let mut filters = Vec::new();

        filters.extend(
            Self::path_patterns(root.path())
                .into_iter()
                .map(|pattern| (pattern, "[TEMP]/".to_string())),
        );

        if cfg!(windows) {
            // Windows temp directory pattern
            let pattern = regex::escape(
                &dunce::simplified(root.path())
                    .display()
                    .to_string()
                    .replace('/', "\\"),
            );
            filters.push((pattern, "[TEMP]".to_string()));
        }

        Self {
            root: ChildPath::new(root.path()),
            _root: root,
            filters,
        }
    }

    /// Create a seal.toml file with the given content.
    pub fn seal_toml(&self, content: &str) -> &Self {
        self.root
            .child("seal.toml")
            .write_str(content)
            .expect("Failed to write seal.toml");
        self
    }

    /// Create a minimal seal.toml with just current-version.
    pub fn minimal_seal_toml(&self, version: &str) -> &Self {
        self.seal_toml(&format!(
            r#"
[release]
current-version = "{version}"
"#
        ))
    }

    /// Generate various escaped regex patterns for the given path.
    pub fn path_patterns(path: impl AsRef<Path>) -> Vec<String> {
        let mut patterns = Vec::new();

        // We can only canonicalize paths that exist already
        if path.as_ref().exists() {
            patterns.push(Self::path_pattern(
                path.as_ref()
                    .canonicalize()
                    .expect("Failed to create canonical path"),
            ));
        }

        // Include a non-canonicalized version
        patterns.push(Self::path_pattern(path));

        patterns
    }

    /// Generate an escaped regex pattern for the given path.
    fn path_pattern(path: impl AsRef<Path>) -> String {
        format!(
            // Trim the trailing separator for cross-platform directories filters
            r"{}\\?/?",
            regex::escape(&dunce::simplified(path.as_ref()).display().to_string())
                // Make separators platform agnostic because on Windows we will display
                // paths with Unix-style separators sometimes
                .replace(r"\\", r"(\\|\/)")
        )
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

    /// Add extra standard filtering for Windows-compatible missing file errors.
    pub fn with_filtered_missing_file_error(mut self) -> Self {
        // The exact message string depends on the system language, so we remove it.
        // We want to only remove the phrase after `Caused by:`
        self.filters.push((
            r"[^:\n]* \(os error 2\)".to_string(),
            " [OS ERROR 2]".to_string(),
        ));
        // Replace the Windows "The system cannot find the path specified. (os error 3)"
        // with the Unix "No such file or directory (os error 2)"
        // and mask the language-dependent message.
        self.filters.push((
            r"[^:\n]* \(os error 3\)".to_string(),
            " [OS ERROR 2]".to_string(),
        ));
        self
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
        let mut command = Self::new_command();
        command.current_dir(self.root.path());
        command
    }

    /// Initialize a git repository in the test context.
    pub fn init_git(&self) -> &Self {
        std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to init git");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to set git user.email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to set git user.name");

        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to git add");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to git commit");

        self
    }

    /// Get the current git branch name.
    pub fn git_current_branch(&self) -> String {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to get current branch");

        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .to_string()
    }

    /// Get the latest git commit message.
    pub fn git_last_commit_message(&self) -> String {
        let output = std::process::Command::new("git")
            .args(["log", "-1", "--pretty=%B"])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to get commit message");

        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .to_string()
    }

    /// Check if a git branch exists.
    pub fn git_branch_exists(&self, branch: &str) -> bool {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--verify", branch])
            .current_dir(self.root.path())
            .output()
            .expect("Failed to check branch");

        output.status.success()
    }

    /// Read a file and return its contents as a string.
    pub fn read_file(&self, path: &str) -> String {
        std::fs::read_to_string(self.root.join(path))
            .unwrap_or_else(|_| panic!("Failed to read file: {path}"))
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
    // Strip ANSI color codes (match ESC character using character class)
    (r"[\x1b]\[[0-9;]*m", ""),
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
