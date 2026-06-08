mod compaction;
mod index;
mod log;
mod record;

pub use index::EntryIndex;
pub use record::{DecodedRecord, Record, RecordDecodeError, RecordEncodeError, RecordKind};

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::config::DatabaseConfig;
use crate::error::DatabaseError;

#[derive(Debug, Clone, Copy, Default)]
struct DatabaseCounters {
    log_records: u64,
    put_records: u64,
    delete_records: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseStats {
    pub live_keys: usize,
    pub log_records: u64,
    pub put_records: u64,
    pub delete_records: u64,
    pub log_bytes: u64,
}

impl DatabaseStats {
    pub fn summary(&self) -> String {
        format!(
            "live_keys={}\nlog_records={}\nput_records={}\ndelete_records={}\nlog_bytes={}",
            self.live_keys, self.log_records, self.put_records, self.delete_records, self.log_bytes
        )
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    config: DatabaseConfig,
    entries: EntryIndex,
    counters: DatabaseCounters,
}

impl Database {
    /// Returns `true` if the database was freshly created (log did not exist before).
    pub fn init(path: impl Into<PathBuf>) -> Result<(Self, bool), DatabaseError> {
        let config = DatabaseConfig::new(path.into());
        let is_new = !config.log_path().exists();
        let db = Self::open_or_create(config.root().to_path_buf())?;
        Ok((db, is_new))
    }

    pub fn open_or_create(path: impl Into<PathBuf>) -> Result<Self, DatabaseError> {
        let config = DatabaseConfig::new(path);

        if config.root().exists() && !config.root().is_dir() {
            return Err(DatabaseError::NotADirectory(config.root().to_path_buf()));
        }

        config.ensure_directory()?;

        let log_path = config.log_path();
        if log_path.exists() && !log_path.is_file() {
            return Err(DatabaseError::InvalidInput(format!(
                "database log path is not a file: {}",
                log_path.display()
            )));
        }

        // Recover from a previously interrupted compaction before opening the log.
        compaction::recover_interrupted_compaction(&log_path)?;

        if !log_path.exists() {
            File::create(&log_path)?;
            log::sync_directory(&log_path)?;
        }

        let (entries, counters) = recover_log(&log_path)?;

        Ok(Self {
            config,
            entries,
            counters,
        })
    }

    pub fn put(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<(), DatabaseError> {
        let key = normalize_key(key.into())?;
        let value = value.into();
        let record = Record::put(key.clone(), value.clone());

        log::SegmentLog::new(self.config.log_path()).append(&record).map_err(|err| match err {
            log::AppendError::Encode(e) => DatabaseError::Encode(e.to_string()),
            log::AppendError::Io(e) => DatabaseError::Io(e),
        })?;

        self.entries.insert(key, value);
        self.counters.log_records += 1;
        self.counters.put_records += 1;

        Ok(())
    }

    pub fn delete(&mut self, key: impl Into<String>) -> Result<bool, DatabaseError> {
        let key = normalize_key(key.into())?;
        let existed = self.entries.contains_key(&key);
        let record = Record::delete(key.clone());

        log::SegmentLog::new(self.config.log_path()).append(&record).map_err(|err| match err {
            log::AppendError::Encode(e) => DatabaseError::Encode(e.to_string()),
            log::AppendError::Io(e) => DatabaseError::Io(e),
        })?;

        if existed {
            self.entries.remove(&key);
        }
        self.counters.log_records += 1;
        self.counters.delete_records += 1;

        Ok(existed)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        if key.is_empty() {
            return None;
        }
        self.entries.get(key).map(|value| value.as_str())
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    pub fn stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let log_bytes = log::SegmentLog::new(self.config.log_path()).len()?;

        Ok(DatabaseStats {
            live_keys: self.entries.len(),
            log_records: self.counters.log_records,
            put_records: self.counters.put_records,
            delete_records: self.counters.delete_records,
            log_bytes,
        })
    }

    pub fn compact(&mut self) -> Result<(), DatabaseError> {
        compaction::rewrite_log(&self.config.log_path(), &self.entries).map_err(|err| match err {
            compaction::RewriteError::Append(log::AppendError::Encode(e)) => DatabaseError::Encode(e.to_string()),
            compaction::RewriteError::Append(log::AppendError::Io(e)) => DatabaseError::Io(e),
            compaction::RewriteError::Io(e) => DatabaseError::Io(e),
        })?;

        let live_keys = self.entries.len() as u64;
        self.counters = DatabaseCounters {
            log_records: live_keys,
            put_records: live_keys,
            delete_records: 0,
        };

        Ok(())
    }
}

fn normalize_key(key: String) -> Result<String, DatabaseError> {
    if key.is_empty() {
        return Err(DatabaseError::InvalidInput("key cannot be empty".to_string()));
    }

    Ok(key)
}

fn recover_log(log_path: &Path) -> Result<(EntryIndex, DatabaseCounters), DatabaseError> {
    let metadata = fs::metadata(log_path)?;
    if metadata.len() == 0 {
        return Ok((EntryIndex::default(), DatabaseCounters::default()));
    }

    let mut reader = File::open(log_path)?;
    let total_len = metadata.len();
    let mut offset = 0_u64;
    let mut entries = EntryIndex::default();
    let mut counters = DatabaseCounters::default();

    while offset < total_len {
        let record_offset = offset;
        match record::read_next_record(&mut reader, offset, total_len) {
            Ok(Some(decoded)) => {
                offset += decoded.bytes_read as u64;
                counters.log_records += 1;

                match decoded.record.kind {
                    RecordKind::Put => {
                        counters.put_records += 1;
                        let value = decoded.record.value.ok_or_else(|| DatabaseError::CorruptLog {
                            offset: record_offset,
                            message: "put record is missing a value".to_string(),
                        })?;

                        entries.insert(decoded.record.key, value);
                    }
                    RecordKind::Delete => {
                        counters.delete_records += 1;
                        entries.remove(&decoded.record.key);
                    }
                }
            }
            Ok(None) => break,
            Err(RecordDecodeError::Truncated { .. }) => break,
            Err(err) => {
                return Err(DatabaseError::CorruptLog {
                    offset: record_offset,
                    message: err.to_string(),
                })
            }
        }
    }

    Ok((entries, counters))
}
