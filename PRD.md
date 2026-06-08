# Product Requirements Document

## Rust Embedded Key-Value Database

**Working name:** RustKV
**Status:** Draft
**Version:** 0.1
**Audience:** Solo developer / learner / portfolio project

## 1. Summary

RustKV is a small embedded key-value database written in Rust. It is designed as a practical learning project that demonstrates how a storage engine works from the inside: command parsing, persistence, recovery, indexing, compaction, and safe file handling.

The project is intentionally narrower than a general-purpose database. The first version focuses on a simple command-line interface, durable storage on disk, and reliable retrieval of data after restart. Later versions can expand into scans, transactions, and a richer query surface, but the initial product should remain easy to reason about and test.

## 2. Problem Statement

Many beginner Rust projects are either too trivial or too broad. A database project sits in the middle: it is small enough to complete, but deep enough to exercise real systems concepts. The challenge is to build something that is simple enough for one person to maintain while still feeling like a real storage engine.

RustKV should solve a practical educational problem: provide a compact codebase where the developer can learn how durable storage, crash recovery, and indexing work without needing a full SQL engine or distributed architecture.

## 3. Product Goals

1. Store key-value pairs durably on disk.
2. Retrieve values by key with predictable behavior.
3. Support delete operations through tombstones or equivalent markers.
4. Recover correctly after process restarts and partial writes.
5. Provide a clean CLI for manual usage and testing.
6. Keep the implementation small, readable, and testable.

## 4. Non-Goals

RustKV v1 is not intended to be:

1. A full SQL database.
2. A networked service or distributed system.
3. A multi-user ACID database with isolation levels.
4. A replacement for SQLite, RocksDB, or sled.
5. A generic analytics engine.

The project should avoid scope creep. If a feature does not help teach core storage-engine concepts, it should probably not be in v1.

## 5. Target Users

### Primary user

A Rust developer or student who wants to learn storage-engine fundamentals by building a small embedded database.

### Secondary user

A reviewer, mentor, or future maintainer who wants to understand the code quickly and verify correctness through tests and documentation.

## 6. User Stories

1. As a user, I want to create a database in a folder so that data survives restarts.
2. As a user, I want to write a value under a key so that I can retrieve it later.
3. As a user, I want to read a value by key so that I can verify the database stored it correctly.
4. As a user, I want to delete a value so that it is no longer returned.
5. As a user, I want to list stored keys so that I can inspect the database contents.
6. As a developer, I want the database to recover from a crash so that I do not lose all data after an interrupted write.
7. As a developer, I want tests to cover persistence and recovery so that behavior stays stable while refactoring.

## 7. Functional Requirements

### 7.1 Database lifecycle

- The application must create or open a database in a user-specified directory.
- The database must persist data across process restarts.
- The database must fail gracefully if the storage directory is invalid or inaccessible.

### 7.2 Data operations

- `put` must store a value under a key.
- `get` must return the current value for a key, if present.
- `delete` must mark a key as removed.
- `list` must enumerate stored keys and optionally show values or metadata.
- `stats` must report simple metrics such as record count, log size, and compaction status.

### 7.3 Storage behavior

- Writes must be append-only at the lowest storage layer, or equivalent in a well-documented format.
- The database must be able to reconstruct its in-memory index from on-disk data.
- The system must ignore incomplete trailing records caused by crashes or power loss.
- The format must be versioned so that future file changes are manageable.

### 7.4 Recovery

- On startup, the database must scan existing storage files and rebuild the latest valid state.
- If the last record is corrupted or incomplete, the system must stop at the last valid record instead of crashing.
- A corrupted file should produce a clear error message when recovery cannot safely continue.

### 7.5 Compaction

- The database must support a basic compaction path that rewrites only live records.
- Compaction can be manual in v1, but the architecture should make automatic compaction possible later.
- After compaction, the active storage footprint should be smaller or at least not grow unbounded.

### 7.6 CLI requirements

- The binary must expose help text and command examples.
- The CLI must support at least `init`, `put`, `get`, `delete`, `list`, `stats`, and `compact`.
- Commands must return useful exit codes for success and failure.

## 8. Data Model

### v1 data model

- Keys: UTF-8 strings
- Values: UTF-8 strings
- Tombstones: explicit markers for deleted keys

This keeps the first version approachable and easy to test from the terminal. A later version can move to raw bytes if binary payload support becomes desirable.

### Logical record fields

- Record type: put or delete
- Key
- Value length
- Value bytes
- Timestamp or sequence number, if needed for debugging and ordering
- Optional checksum for integrity verification

## 9. Product Scope by Release

### MVP

- Create/open database directory
- Put/get/delete
- Persistence across restarts
- Recovery from clean shutdowns and partial trailing records
- Basic CLI
- Unit and integration tests

### v1.1

- List keys
- Stats command
- Manual compaction
- Better logging

### v1.2

- Range scans or prefix scans
- Binary values
- Benchmark harness
- Automatic compaction heuristics

## 10. Success Criteria

RustKV can be considered successful if all of the following are true:

1. A user can store data, restart the process, and read the same data back.
2. Delete operations behave as expected and survive restarts.
3. Corrupted or incomplete trailing records do not destroy previously valid data.
4. The codebase remains small enough to understand without a large architecture diagram.
5. Tests cover the main persistence and recovery paths.
6. The repository can serve as a portfolio-quality example of Rust systems programming.

## 11. UX Requirements

The CLI should be simple and predictable. Example usage should feel like this:

```bash
rustkv init ./data
rustkv put ./data user:1 Alice
rustkv get ./data user:1
rustkv delete ./data user:1
rustkv list ./data
rustkv stats ./data
rustkv compact ./data
```

Output should be concise and human-readable. Errors should explain what went wrong and, when possible, how to fix it.

## 12. Technical Constraints

- Use Rust 2021 edition.
- Prefer standard library APIs where practical.
- Keep dependencies minimal and justified.
- Avoid adding a server, RPC layer, or async runtime unless a later milestone explicitly needs it.
- Ensure the storage format is portable across macOS, Linux, and Windows as much as possible.

## 13. Quality Requirements

### Reliability

- The database must not silently lose valid records during normal operation.
- Error messages must be deterministic and actionable where possible.

### Maintainability

- Modules should separate CLI, storage encoding, index management, and tests.
- Core logic should be easy to unit test without running the full binary.

### Testability

- Recovery behavior must be tested with temporary directories.
- Corruption scenarios must be covered with targeted tests.
- Command-level behavior should be verified with integration tests.

### Performance

- Single-key reads should remain fast for a small dataset.
- Writes should be cheap enough for the intended learning use case.
- Compaction should be understandable rather than aggressively optimized.

## 14. Risks and Mitigations

### Risk: scope creep

Mitigation: keep v1 strictly key-value based and defer SQL, networking, and transactions.

### Risk: fragile file format

Mitigation: version the record format, add checksums, and test crash recovery.

### Risk: too many dependencies

Mitigation: use crates only where they significantly improve clarity or testability.

### Risk: compaction becomes complex

Mitigation: start with manual compaction and a single active log file before introducing more advanced segment management.

## 15. Open Questions

1. Should v1 support only UTF-8 strings, or raw bytes as well?
2. Should the database accept a directory path or a single database file?
3. Should compaction run manually or automatically in the first release?
4. Should the project prioritize learning value or benchmark performance?
5. Should the CLI expose a library API for embedding in other Rust code?

## 16. Definition of Done

The first release is done when:

- The CLI works end to end.
- Data persists across restarts.
- Delete and recovery behavior are tested.
- The storage format is documented.
- The implementation plan has been executed or updated to reflect the final state.
