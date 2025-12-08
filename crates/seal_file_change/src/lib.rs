use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

pub struct FileChanges(Vec<FileChange>);

impl FileChanges {
    pub fn new(changes: Vec<FileChange>) -> Self {
        Self(changes)
    }

    pub fn apply(self) -> Result<()> {
        for change in self {
            change.apply()?;
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &FileChange> {
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

impl<'a> IntoIterator for &'a FileChanges {
    type Item = &'a FileChange;
    type IntoIter = std::slice::Iter<'a, FileChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub struct FileChange {
    path: PathBuf,
    old_content: String,
    new_content: String,
}

impl FileChange {
    pub fn new(path: PathBuf, old_content: String, new_content: String) -> Self {
        Self {
            path,
            old_content,
            new_content,
        }
    }

    pub fn apply(self) -> Result<()> {
        fs_err::write(&self.path, &self.new_content)
            .context(format!("Failed to write {}", self.path.display()))?;
        Ok(())
    }

    pub fn display_diff(&self, stdout: &mut impl std::fmt::Write) -> Result<()> {
        use similar::{ChangeTag, TextDiff};

        let path_string = self.path.display().to_string();
        let path_stripped = path_string.strip_suffix("/").unwrap_or(&path_string);

        writeln!(stdout)?;
        writeln!(
            stdout,
            "{}",
            format!("diff --git a{path_stripped} b{path_stripped}").bold()
        )?;
        writeln!(stdout, "{}", format!("--- a{path_stripped}").blue())?;
        writeln!(stdout, "{}", format!("+++ b{path_stripped}").blue())?;

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

pub fn make_absolute(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}
