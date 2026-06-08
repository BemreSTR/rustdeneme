use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    root: PathBuf,
}

impl DatabaseConfig {
    pub const LOG_FILE_NAME: &'static str = "store.log";

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn log_path(&self) -> PathBuf {
        self.root.join(Self::LOG_FILE_NAME)
    }

    pub fn ensure_directory(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)
    }
}
