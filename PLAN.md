# Implementation Plan

## Rust Embedded Key-Value Database

**Working name:** RustKV
**Reference document:** [PRD.md](PRD.md)

## 1. Goal

Build a small embedded key-value database in Rust that teaches storage-engine fundamentals while remaining simple enough to complete as a solo project. The first release should focus on correctness, recovery, and clarity rather than raw throughput or feature breadth.

## 2. Proposed Repository Layout

```text
rustkv/
├── Cargo.toml
├── README.md
├── PRD.md
├── PLAN.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── error.rs
│   ├── config.rs
│   └── storage/
│       ├── mod.rs
│       ├── record.rs
│       ├── log.rs
│       ├── index.rs
│       └── compaction.rs
└── tests/
    ├── cli.rs
    ├── recovery.rs
    └── compaction.rs
```

This layout keeps the binary thin and pushes behavior into reusable library modules.

## 3. Suggested Tech Stack

- Rust 2021 edition
- `clap` for CLI parsing
- `anyhow` for top-level application errors
- `thiserror` for domain-specific error types
- `serde` and `serde_json` if metadata needs a simple structured format
- `tracing` or `log` for human-readable diagnostics
- `tempfile` for integration tests
- `assert_cmd` and `predicates` for CLI verification

Keep dependencies lean. If a task can be handled by the standard library, prefer that first.

## 4. Architecture Overview

### 4.1 CLI layer

The CLI should accept the database path and the command to execute. It should parse arguments, validate inputs, and call library functions.

Recommended commands:

- `init`
- `put`
- `get`
- `delete`
- `list`
- `stats`
- `compact`

### 4.2 Storage engine

The storage engine should own the on-disk format, recovery rules, and compaction logic. A simple append-only log is the most practical starting point.

### 4.3 In-memory index

Maintain an in-memory map from key to the latest record location or value state. Rebuild it from disk on startup.

### 4.4 Record format

Each record should be self-describing enough to survive partial writes and future format changes. A sensible v1 design is:

- record type
- key length
- value length
- payload
- checksum or integrity marker

### 4.5 Compaction

Compaction should rewrite only live records into a fresh file or segment and then atomically replace the old active data.

## 5. Execution Phases

### Phase 0: Project bootstrap

**Tasks**

- Create the Cargo project.
- Add the chosen dependencies.
- Set up formatting and linting conventions.
- Create the module skeleton.
- Add a minimal `main.rs` that prints help or dispatches commands.

**Exit criteria**

- `cargo build` succeeds.
- The project layout matches the intended structure.
- The binary starts and accepts basic arguments.

### Phase 1: CLI and command model

**Tasks**

- Define command enums and argument structs.
- Normalize how the database path is provided.
- Implement friendly help output.
- Introduce typed errors for invalid arguments and user mistakes.

**Exit criteria**

- Every supported command parses correctly.
- Invalid input produces clear failures.
- Help text is readable and examples are present.

### Phase 2: Persistent record writing

**Tasks**

- Implement record encoding and decoding.
- Append records to disk in a stable format.
- Ensure writes are flushed in a controlled way.
- Add support for tombstones.

**Exit criteria**

- A `put` writes a durable record.
- A `delete` writes a durable tombstone.
- Records can be read back from disk.

### Phase 3: Recovery and indexing

**Tasks**

- Rebuild the in-memory index at startup.
- Ignore incomplete trailing writes safely.
- Detect malformed records and stop recovery at the last valid point.
- Make `get` resolve to the newest live value.

**Exit criteria**

- Restarting the database preserves visible state.
- Corruption handling does not erase earlier valid data.
- Reads use the reconstructed index rather than a linear full-file scan.

### Phase 4: List, stats, and inspection commands

**Tasks**

- Implement `list` for visible keys.
- Implement `stats` for simple file and record metrics.
- Report the active log size and estimated live data size.

**Exit criteria**

- Users can inspect the database contents from the CLI.
- The output is stable enough to test.

### Phase 5: Compaction

**Tasks**

- Add a manual compaction command.
- Rewrite live records into a new file or segment.
- Replace the old storage atomically.
- Rebuild the index after compaction.

**Exit criteria**

- Compaction reduces redundant storage.
- Deleted records do not reappear.
- The database still opens normally after compaction.

### Phase 6: Testing and hardening

**Tasks**

- Add unit tests for record encoding and decoding.
- Add integration tests for CLI behavior.
- Add crash-recovery tests using temporary directories.
- Add corruption tests for truncated records.
- Add compaction tests with repeated overwrites and deletes.

**Exit criteria**

- Tests cover the main persistence path.
- Recovery and compaction paths have automated coverage.
- The project is stable enough to refactor safely.

### Phase 7: Polish and documentation

**Tasks**

- Write a clear README with examples.
- Document the storage format at a high level.
- Add usage examples and expected outputs.
- Optionally add benchmark scripts or sample datasets.

**Exit criteria**

- A new contributor can understand the project from the docs.
- The repository looks complete and presentable.

## 6. Development Order

Follow this order to avoid rework:

1. Define the CLI shape.
2. Define the record format.
3. Implement append-only writes.
4. Implement recovery.
5. Add the in-memory index.
6. Add delete semantics.
7. Add list and stats.
8. Add compaction.
9. Add tests for every major path.
10. Polish documentation and examples.

This order ensures the storage format is stable before the higher-level commands depend on it.

## 7. Testing Strategy

### Unit tests

- Record encoding and decoding
- Index updates
- Error conversion
- Compaction selection logic

### Integration tests

- CLI command parsing
- Data persistence across restarts
- Delete and recovery behavior
- List and stats output

### Failure-mode tests

- Truncated log entries
- Invalid checksums or malformed records
- Missing directories
- Permission errors where practical on the current platform

### Manual smoke tests

```bash
cargo run -- init ./tmpdb
cargo run -- put ./tmpdb user:1 Alice
cargo run -- get ./tmpdb user:1
cargo run -- delete ./tmpdb user:1
cargo run -- list ./tmpdb
cargo run -- stats ./tmpdb
cargo run -- compact ./tmpdb
```

## 8. Milestones and Estimated Effort

| Milestone | Scope | Estimated Effort |
| --- | --- | --- |
| M0 | Bootstrap project and CLI skeleton | 0.5 day |
| M1 | Record format and persistence | 1 day |
| M2 | Recovery and indexing | 1 day |
| M3 | List, stats, and delete polish | 0.5 day |
| M4 | Compaction | 1 day |
| M5 | Tests and documentation | 1 day |

These estimates assume a solo learning project with moderate experience in Rust.

## 9. Definition of Done

The implementation is done when:

- The project builds cleanly.
- The CLI supports the planned commands.
- Data survives restart.
- Truncated or partial records are handled safely.
- Tests cover the core behavior.
- The README and the plan reflect the final design.

## 10. Immediate Next Steps

1. Decide whether v1 should store only UTF-8 strings or raw bytes.
2. Create the Rust Cargo project scaffold.
3. Implement the CLI and basic storage model.
4. Lock the record format before adding advanced features.

## 11. Notes for Future Expansion

- If you want transactional behavior later, introduce write batches and a formal WAL.
- If you want range queries later, add sorted segment files or a more advanced index.
- If you want a server later, keep the storage engine library separate from the transport layer.
