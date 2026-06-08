# RustKV

RustKV is a small embedded key-value database written in Rust. It is designed as a learning project that shows how a storage engine can be built with a compact and readable codebase.

## Features

- Durable key-value storage on disk
- Append-only record log with checksums
- Recovery from restart and partial writes
- Delete support through tombstones
- Manual compaction
- Simple CLI

## Commands

```bash
cargo run -- init ./data
cargo run -- put ./data user:1 Alice
cargo run -- get ./data user:1
cargo run -- delete ./data user:1
cargo run -- list ./data
cargo run -- stats ./data
cargo run -- compact ./data
```

### Command Reference

| Command | Description |
|---|---|
| `init <path>` | Create a new database directory |
| `put <path> <key> <value>` | Store a key-value pair |
| `get <path> <key>` | Retrieve the value for a key |
| `delete <path> <key>` | Delete a key from the database |
| `list <path>` | List all stored keys and values |
| `stats <path>` | Show database statistics |
| `compact <path>` | Rewrite the log keeping only live records |

## Storage Layout

Each database lives in a user-specified directory and stores its records in `store.log`. The log is scanned on startup to rebuild the in-memory index.

### Record Binary Format

Every record is encoded in a self-describing binary format with the following layout:

```
+--------+---------+------+----------+----------+----------+---------+---------+
| MAGIC  | VERSION | KIND | KEY_LEN  | VAL_LEN  | CHECKSUM | KEY     | VALUE   |
| 4 byte | 1 byte  | 1 b. | 4 byte   | 4 byte   | 4 byte   | N bytes | M bytes |
+--------+---------+------+----------+----------+----------+---------+---------+
```

| Field | Size | Description |
|---|---|---|
| MAGIC | 4 bytes | Constant `RKV1` — identifies a valid record |
| VERSION | 1 byte | Format version (currently `1`) |
| KIND | 1 byte | Record type: `1` = Put, `2` = Delete |
| KEY_LEN | 4 bytes (LE) | Length of the key in bytes |
| VAL_LEN | 4 bytes (LE) | Length of the value in bytes (`0` for deletes) |
| CHECKSUM | 4 bytes (LE) | FNV-1a 32-bit hash over version, kind, lengths, key, and value |
| KEY | KEY_LEN bytes | UTF-8 encoded key |
| VALUE | VAL_LEN bytes | UTF-8 encoded value (absent for delete records) |

**Total header size:** 18 bytes.

### Recovery Behavior

On startup the database reads the log sequentially:

1. Each record's header is read and validated (magic, version, checksum).
2. If a record is truncated (incomplete header or payload), recovery stops at the last valid record.
3. If a record has an invalid checksum or malformed data, an error is reported with the exact byte offset.
4. The in-memory index is rebuilt from the surviving valid records.

### Compaction

Running `compact` rewrites only the live (non-deleted) records into a fresh log file, then atomically renames it to replace the old log. If the process crashes during compaction, the next startup detects the orphaned temporary file and recovers from it automatically.

## Notes

- Keys and values are UTF-8 strings in v1.
- Every write is flushed and synced to disk (`fsync`) before returning.
- Compaction rewrites only the live records and is atomic on POSIX systems.
