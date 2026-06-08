use std::convert::TryFrom;
use std::io::{ErrorKind, Read};

const MAGIC: [u8; 4] = *b"RKV1";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    Put = 1,
    Delete = 2,
}

impl RecordKind {
    fn from_byte(value: u8, offset: u64) -> Result<Self, RecordDecodeError> {
        match value {
            1 => Ok(Self::Put),
            2 => Ok(Self::Delete),
            other => Err(RecordDecodeError::UnknownKind { offset, kind: other }),
        }
    }

    fn as_byte(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub kind: RecordKind,
    pub key: String,
    pub value: Option<String>,
}

impl Record {
    pub fn put(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            kind: RecordKind::Put,
            key: key.into(),
            value: Some(value.into()),
        }
    }

    pub fn delete(key: impl Into<String>) -> Self {
        Self {
            kind: RecordKind::Delete,
            key: key.into(),
            value: None,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, RecordEncodeError> {
        let key_bytes = self.key.as_bytes();
        let value_bytes = match &self.value {
            Some(value) => value.as_bytes(),
            None => &[][..],
        };

        let key_len = u32::try_from(key_bytes.len()).map_err(|_| RecordEncodeError::KeyTooLong(key_bytes.len()))?;
        let value_len = u32::try_from(value_bytes.len()).map_err(|_| RecordEncodeError::ValueTooLong(value_bytes.len()))?;
        let checksum = compute_checksum(VERSION, self.kind.as_byte(), key_len, value_len, key_bytes, value_bytes);

        let mut bytes = Vec::with_capacity(HEADER_LEN + key_bytes.len() + value_bytes.len());
        bytes.extend_from_slice(&MAGIC);
        bytes.push(VERSION);
        bytes.push(self.kind.as_byte());
        bytes.extend_from_slice(&key_len.to_le_bytes());
        bytes.extend_from_slice(&value_len.to_le_bytes());
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes.extend_from_slice(key_bytes);
        bytes.extend_from_slice(value_bytes);
        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct DecodedRecord {
    pub record: Record,
    pub bytes_read: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum RecordEncodeError {
    #[error("key is too long: {0} bytes (max {})", u32::MAX)]
    KeyTooLong(usize),
    #[error("value is too long: {0} bytes (max {})", u32::MAX)]
    ValueTooLong(usize),
}

#[derive(Debug, thiserror::Error)]
pub enum RecordDecodeError {
    #[error("truncated record at offset {offset}")]
    Truncated { offset: u64 },
    #[error("invalid record magic at offset {offset}")]
    InvalidMagic { offset: u64 },
    #[error("unsupported record version {version} at offset {offset}")]
    UnsupportedVersion { offset: u64, version: u8 },
    #[error("unknown record kind {kind} at offset {offset}")]
    UnknownKind { offset: u64, kind: u8 },
    #[error("record length cannot be represented at offset {offset}")]
    LengthOverflow { offset: u64 },
    #[error("checksum mismatch at offset {offset}: expected {expected}, got {actual}")]
    ChecksumMismatch { offset: u64, expected: u32, actual: u32 },
    #[error("delete record contained a value at offset {offset}")]
    DeleteValueUnexpected { offset: u64 },
    #[error("invalid UTF-8 for {field} at offset {offset}")]
    InvalidUtf8 { offset: u64, field: &'static str },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_next_record<R: Read>(reader: &mut R, offset: u64, total_len: u64) -> Result<Option<DecodedRecord>, RecordDecodeError> {
    if offset >= total_len {
        return Ok(None);
    }

    let mut header = [0_u8; HEADER_LEN];
    read_exact_or_truncated(reader, &mut header, offset)?;

    if header[0..4] != MAGIC {
        return Err(RecordDecodeError::InvalidMagic { offset });
    }

    let version = header[4];
    if version != VERSION {
        return Err(RecordDecodeError::UnsupportedVersion { offset, version });
    }

    let kind_byte = header[5];
    let kind = RecordKind::from_byte(kind_byte, offset)?;

    let key_len = u32::from_le_bytes(header[6..10].try_into().unwrap()) as u64;
    let value_len = u32::from_le_bytes(header[10..14].try_into().unwrap()) as u64;
    let expected_checksum = u32::from_le_bytes(header[14..18].try_into().unwrap());

    let record_len = HEADER_LEN as u64 + key_len + value_len;
    if record_len > total_len - offset {
        return Err(RecordDecodeError::Truncated { offset });
    }

    let key_len_usize = usize::try_from(key_len).map_err(|_| RecordDecodeError::LengthOverflow { offset })?;
    let value_len_usize = usize::try_from(value_len).map_err(|_| RecordDecodeError::LengthOverflow { offset })?;

    if kind == RecordKind::Delete && value_len_usize != 0 {
        return Err(RecordDecodeError::DeleteValueUnexpected { offset });
    }

    let mut key_bytes = vec![0_u8; key_len_usize];
    read_exact_or_truncated(reader, &mut key_bytes, offset)?;

    let mut value_bytes = vec![0_u8; value_len_usize];
    if value_len_usize > 0 {
        read_exact_or_truncated(reader, &mut value_bytes, offset)?;
    }

    let actual_checksum = compute_checksum(version, kind_byte, key_len as u32, value_len as u32, &key_bytes, &value_bytes);
    if actual_checksum != expected_checksum {
        return Err(RecordDecodeError::ChecksumMismatch {
            offset,
            expected: expected_checksum,
            actual: actual_checksum,
        });
    }

    let key = String::from_utf8(key_bytes).map_err(|_| RecordDecodeError::InvalidUtf8 {
        offset,
        field: "key",
    })?;

    let value = match kind {
        RecordKind::Put => Some(String::from_utf8(value_bytes).map_err(|_| RecordDecodeError::InvalidUtf8 {
            offset,
            field: "value",
        })?),
        RecordKind::Delete => None,
    };

    Ok(Some(DecodedRecord {
        record: Record { kind, key, value },
        bytes_read: record_len as usize,
    }))
}

fn read_exact_or_truncated<R: Read>(reader: &mut R, buffer: &mut [u8], offset: u64) -> Result<(), RecordDecodeError> {
    match reader.read_exact(buffer) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => Err(RecordDecodeError::Truncated { offset }),
        Err(err) => Err(err.into()),
    }
}

fn compute_checksum(version: u8, kind: u8, key_len: u32, value_len: u32, key_bytes: &[u8], value_bytes: &[u8]) -> u32 {
    let mut hash = 0x811C_9DC5_u32;

    hash = update_hash(hash, &[version, kind]);
    hash = update_hash(hash, &key_len.to_le_bytes());
    hash = update_hash(hash, &value_len.to_le_bytes());
    hash = update_hash(hash, key_bytes);
    hash = update_hash(hash, value_bytes);

    hash
}

fn update_hash(mut hash: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash
}
