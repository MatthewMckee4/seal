use std::path::PathBuf;

use crate::Config;

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub root: PathBuf,
    pub config: Config,
}

impl WorkspaceMember {
    pub fn new(root: PathBuf, config: Config) -> Self {
        Self { root, config }
    }
}
