use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::record::{Record, RecordEncodeError};

#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    #[error("encode error: {0}")]
    Encode(#[from] RecordEncodeError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct SegmentLog {
    path: PathBuf,
}

impl SegmentLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, record: &Record) -> Result<(), AppendError> {
        let encoded = record.encode()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        file.write_all(&encoded)?;
        file.flush()?;
        file.sync_data()?;

        Ok(())
    }

    pub fn len(&self) -> Result<u64, std::io::Error> {
        match fs::metadata(&self.path) {
            Ok(metadata) => Ok(metadata.len()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(err) => Err(err),
        }
    }

    pub fn is_empty(&self) -> Result<bool, std::io::Error> {
        Ok(self.len()? == 0)
    }
}

/// Fsync the parent directory of a log file so the new/renamed directory
/// entry is durable. Required for crash-consistency after creating or
/// renaming a log file; the file's own fsync is not enough.
pub fn sync_directory(log_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = log_path.parent() {
        if parent.as_os_str().is_empty() {
            return Ok(());
        }
        let dir = File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}
