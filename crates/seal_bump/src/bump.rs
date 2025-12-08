use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::Version;

// Compile regex patterns once for auto-detecting version fields
// Using expect() is safe here because these are hardcoded, valid regex patterns
static VERSION_PATTERNS: LazyLock<[regex::Regex; 3]> = LazyLock::new(|| {
    [
        regex::Regex::new(r#"version\s*=\s*"[^"]+""#).expect("Invalid regex pattern"),
        regex::Regex::new(r#""version"\s*:\s*"[^"]+""#).expect("Invalid regex pattern"),
        regex::Regex::new(r#"__version__\s*=\s*"[^"]+""#).expect("Invalid regex pattern"),
    ]
});

pub struct FileChanges(Vec<FileChange>);

impl FileChanges {
    pub fn new(changes: Vec<FileChange>) -> Self {
        Self(changes)
    }

    pub fn apply(self, root: &Path) -> Result<()> {
        for change in self {
            let full_path = root.join(change.path());
            fs_err::write(&full_path, &change.new_content)
                .context(format!("Failed to write {}", full_path.display()))?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a FileChanges {
    type Item = &'a FileChange;
    type IntoIter = std::slice::Iter<'a, FileChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for FileChanges {
    type Item = FileChange;
    type IntoIter = std::vec::IntoIter<FileChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct FileChange {
    path: PathBuf,
    old_content: String,
    new_content: String,
}

impl FileChange {
    pub fn display_diff(&self, stdout: &mut impl std::fmt::Write) -> Result<()> {
        use similar::{ChangeTag, TextDiff};

        writeln!(stdout)?;
        writeln!(
            stdout,
            "{}",
            format!(
                "diff --git a/{} b/{}",
                self.path.display(),
                self.path.display()
            )
            .bold()
        )?;
        writeln!(
            stdout,
            "{}",
            format!("--- a/{}", self.path.display()).blue()
        )?;
        writeln!(
            stdout,
            "{}",
            format!("+++ b/{}", self.path.display()).blue()
        )?;

        let diff = TextDiff::from_lines(&self.old_content, &self.new_content);

        for hunk in diff.unified_diff().iter_hunks() {
            writeln!(stdout, "{}", hunk.header().yellow().bold())?;

            for change in hunk.iter_changes() {
                let (sign, value): (&str, String) = match change.tag() {
                    ChangeTag::Delete => ("-", change.value().red().to_string()),
                    ChangeTag::Insert => ("+", change.value().green().to_string()),
                    ChangeTag::Equal => (" ", change.value().dimmed().to_string()),
                };

                if change.value().ends_with('\n') {
                    write!(stdout, "{sign}{value}")?;
                } else {
                    writeln!(stdout, "{sign}{value}")?;
                }
            }
        }

        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub fn calculate_version_file_changes(
    root: &Path,
    version_files: &[seal_project::VersionFile],
    current_version: &str,
    new_version: &Version,
) -> Result<FileChanges> {
    let mut changes = Vec::new();

    for version_file in version_files {
        match version_file {
            seal_project::VersionFile::Text {
                path,
                format,
                field,
            } => todo!(),
            seal_project::VersionFile::Search {
                path,
                search,
                version_template,
            } => todo!(),
            seal_project::VersionFile::JustPath { path } => todo!(),
            seal_project::VersionFile::Simple(_) => todo!(),
        }
    }

    // Seal.toml file change
    let seal_toml_path = root.join("seal.toml");
    let old_seal_toml_content =
        fs_err::read_to_string(&seal_toml_path).context("Failed to read seal.toml")?;

    let old_line = format!(r#"current-version = "{current_version}""#);
    let new_line = format!(r#"current-version = "{new_version}""#);

    if !old_seal_toml_content.contains(&old_line) {
        anyhow::bail!("Could not find current-version = \"{current_version}\" in seal.toml");
    }

    let updated_content = old_seal_toml_content.replace(&old_line, &new_line);

    changes.push(FileChange {
        path: seal_toml_path,
        old_content: old_seal_toml_content,
        new_content: updated_content,
    });

    Ok(FileChanges(changes))
}

fn update_version_in_content(
    file_path: &Path,
    content: &str,
    current_version: &str,
    new_version: &Version,
    search_pattern: Option<&str>,
    version_template: Option<&str>,
) -> Result<String> {
    let version_str = if let Some(template) = version_template {
        format_version_with_template(new_version, template)
    } else {
        new_version.to_string()
    };

    if let Some(pattern_str) = search_pattern {
        let search_with_current = pattern_str.replace("{version}", current_version);
        let search_with_new = pattern_str.replace("{version}", &version_str);

        if !content.contains(&search_with_current) {
            anyhow::bail!("Search pattern not found in file. Expected: {search_with_current}");
        }

        return Ok(content.replace(&search_with_current, &search_with_new));
    }

    let replacements = [
        format!(r#"version = "{version_str}""#),
        format!(r#""version": "{version_str}""#),
        format!(r#"__version__ = "{version_str}""#),
    ];

    for (pattern, replacement) in VERSION_PATTERNS.iter().zip(replacements.iter()) {
        if pattern.is_match(content) {
            return Ok(pattern.replace(content, replacement).to_string());
        }
    }

    if content.contains(current_version) {
        return Ok(content.replace(current_version, &version_str));
    }

    anyhow::bail!(format!(
        "No version field found in file `{}`",
        file_path.display()
    ));
}

fn format_version_with_template(version: &Version, template: &str) -> String {
    template
        .replace("{major}", &version.major.to_string())
        .replace("{minor}", &version.minor.to_string())
        .replace("{patch}", &version.patch.to_string())
        .replace("{extra}", &version.pre.to_string())
}
