use anyhow::{Context, Result};
use seal_fs::FileResolver;
use similar::TextDiff;
use std::path::{Path, PathBuf};

pub struct FileChanges(Vec<FileChange>);

impl FileChanges {
    pub fn new(changes: Vec<FileChange>) -> Self {
        Self(changes)
    }

    pub fn apply(self) -> Result<()> {
        for change in self.iter() {
            change.apply()?;
        }
        Ok(())
    }

    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub fn iter(&self) -> impl Iterator<Item = &FileChange> {
        self.0.iter()
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
    abslute_path: PathBuf,
    old_content: String,
    new_content: String,
}

impl FileChange {
    pub fn new(path: PathBuf, old_content: String, new_content: String) -> Self {
        Self {
            abslute_path: path,
            old_content,
            new_content,
        }
    }

    pub fn apply(&self) -> Result<()> {
        fs_err::write(&self.abslute_path, &self.new_content)
            .context(format!("Failed to write {}", self.abslute_path.display()))?;
        Ok(())
    }

    pub fn display_diff(
        &self,
        stdout: &mut impl std::fmt::Write,
        file_resolver: &FileResolver,
    ) -> Result<()> {
        let width = seal_terminal::terminal_width();

        let path_string = file_resolver
            .relative_path(&self.abslute_path)
            .display()
            .to_string();

        let diff = TextDiff::from_lines(&self.old_content, &self.new_content)
            .unified_diff()
            .header(&path_string, &path_string)
            .context_radius(2)
            .to_string();

        write!(stdout, "{diff}")?;

        writeln!(stdout, "────────────{:─^1$}", "", width.saturating_sub(13))?;

        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.abslute_path
    }
}

pub fn make_absolute(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_absolute() {
        let base = Path::new("/home/user");
        let path = Path::new("file.txt");
        assert_eq!(
            make_absolute(base, path),
            PathBuf::from("/home/user/file.txt")
        );

        let path = Path::new("/home/user/file.txt");
        assert_eq!(
            make_absolute(base, path),
            PathBuf::from("/home/user/file.txt")
        );
    }
}
