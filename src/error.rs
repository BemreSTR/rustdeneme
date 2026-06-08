use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("database path is not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("database log is corrupt at offset {offset}: {message}")]
    CorruptLog { offset: u64, message: String },
    #[error("encode error: {0}")]
    Encode(String),
}
