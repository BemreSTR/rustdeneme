use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::log::{self, AppendError, SegmentLog};
use super::record::Record;

#[derive(Debug, thiserror::Error)]
pub enum RewriteError {
    #[error("append error: {0}")]
    Append(#[from] AppendError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn rewrite_log(log_path: &Path, entries: &BTreeMap<String, String>) -> Result<(), RewriteError> {
    let temp_path = temp_log_path(log_path);

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let temp_log = SegmentLog::new(temp_path.clone());
    for (key, value) in entries {
        temp_log.append(&Record::put(key.clone(), value.clone()))?;
    }

    // Use atomic rename to replace the old log. On POSIX, fs::rename atomically
    // replaces the target if it already exists, so there is no window where both
    // files are missing. This avoids the data-loss risk of delete-then-rename.
    fs::rename(&temp_path, log_path)?;
    log::sync_directory(log_path)?;

    Ok(())
}

/// If a previous compaction was interrupted, a `.compact` file may be orphaned
/// on disk. This function recovers it by renaming it back to the active log
/// path when the active log is missing or empty.
pub fn recover_interrupted_compaction(log_path: &Path) -> Result<(), std::io::Error> {
    let temp_path = temp_log_path(log_path);

    if !temp_path.exists() {
        return Ok(());
    }

    let log_exists = log_path.exists();
    let log_is_empty = if log_exists {
        fs::metadata(log_path).map(|m| m.len() == 0).unwrap_or(true)
    } else {
        true
    };

    if !log_exists || log_is_empty {
        fs::rename(&temp_path, log_path)?;
        log::sync_directory(log_path)?;
    } else {
        // Active log is healthy; the temp file is stale, remove it.
        fs::remove_file(&temp_path)?;
    }

    Ok(())
}

fn temp_log_path(log_path: &Path) -> PathBuf {
    let mut temp_path = log_path.to_path_buf();
    temp_path.set_extension("compact");
    temp_path
}
