use std::path::{Path, PathBuf};

pub struct FileResolver {
    root: PathBuf,
}

impl FileResolver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn relative_path<'a>(&self, path: &'a Path) -> &'a Path {
        let cwd = self.root.clone();

        if let Ok(path) = path.strip_prefix(cwd) {
            return path;
        }

        path
    }
}
