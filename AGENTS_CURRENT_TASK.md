# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/error_handling`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

- [x] Define canonical error boundaries and ownership: engine/internal crates use typed errors (`thiserror`), CLI/GUI/TUI entrypoints use `anyhow::Result`.
  - Progress: Completed typed-error migration for remaining engine/internal `Result<_, String>` surfaces (`SymbolRegistry`, `ValuedStruct`, legacy string metadata parsing), while CLI/TUI/desktop GUI entrypoints remain on `anyhow::Result`.
- [x] Replace `Result<_, String>` in process query + OS provider path with typed errors.
  Files: `squalr-engine-processes/src/process_query/process_queryer.rs`, `squalr-engine/src/os/engine_os_provider.rs`, platform-specific process query files.
- [x] Replace `Result<_, String>` in interprocess bindings/pipes with typed errors and preserve context instead of flattening to `String`.
  Files: `squalr-engine/src/engine_bindings/interprocess/**`, `squalr-engine/src/engine_bindings/standalone/**`.
- [x] Replace `Result<_, String>` in snapshot region memory reader with typed scan I/O errors.
  File: `squalr-engine-scanning/src/scanners/snapshot_region_memory_reader.rs`.
- [x] Replace runtime `unwrap()` in non-test crates with safe handling (`Result` propagation, guarded fallback, or structured log and early return).
  Initial files: `squalr-tui/src/main.rs`, `squalr-cli/src/main.rs`, `squalr/src/views/struct_viewer/struct_viewer_entry_view.rs`, `squalr-engine-api/src/structures/tasks/trackable_task.rs`, `squalr-engine-processes/src/process_query/android/android_process_query.rs`, `squalr-engine-api/src/structures/results/snapshot_region_scan_results.rs`.
- [x] Replace runtime `panic!` in app entrypoints with error returns and consistent startup failure reporting.
  Initial files: `squalr/src/main.rs`, `squalr-cli/src/main.rs`, `squalr-tui/src/main.rs`, `squalr-android/src/lib.rs`.
- [x] Update API response payloads that currently embed `Result<T, String>` where appropriate to use typed serializable error payloads.
  Initial files: `squalr-engine-api/src/commands/settings/*/list/*_response.rs`.
- [x] Add focused tests for new error conversions and propagation (process query + IPC + scan memory reader), and keep panic-based test assertions only in tests.

## Important Information
Important information discovered during work about the current state of the task should be appended here.

Initial analysis
- Audit baseline (non-test runtime crates):
  - `unwrap()`: 8 original occurrences.
  - `panic!()`: 6 original occurrences.
  - `Result<_, String>`: widespread, concentrated in process query, engine bindings (interprocess/standalone), scan memory reader, and selected API payloads.
- Existing typed error foundation already present:
  - `squalr-engine-api/src/conversions/conversion_error.rs`
  - `squalr-engine-api/src/structures/data_types/data_type_error.rs`
- Architectural constraint from `README.md` task definition:
  - Engine should normalize toward struct/typed errors.
  - CLI/GUI may use `anyhow`.
  - `Result<(_), String>` is explicitly called out as bad practice.

Discovered during iteration:
- CLI, TUI, and desktop GUI entrypoints now return `anyhow::Result<()>` and no longer panic during startup/event-loop failures.
- Android startup path no longer panics on engine/gui initialization; it logs and returns from `android_main`.
- Runtime unwrap removals completed in `trackable_task`, `snapshot_region_scan_results`, and Android package cache read path.
- Process query path now uses `ProcessQueryError` across `squalr-engine-processes` + `squalr-engine` OS provider boundary (including Windows/Linux/macOS/Android implementations and `squalr-tests` mock providers), replacing prior `Result<_, String>` signatures.
- Added focused unit tests for process-query typed error formatting/constructor behavior in `squalr-engine-processes/src/process_query/process_query_error.rs`.
- Engine binding traits now use typed `EngineBindingError` instead of `Result<_, String>`, with interprocess pipe-specific `InterprocessPipeError` preserving operation context + source error chaining.
- Added focused unit tests for `EngineBindingError` constructor/display behavior in `squalr-engine-api/src/engine/engine_binding_error.rs`.
- Snapshot region memory reads now return typed `SnapshotRegionMemoryReadError` values (including chunk-first-failure context) while preserving tombstone behavior for failed read addresses.
- Settings list response payloads (`general/memory/scan`) now use typed serializable `SettingsError` instead of `String`, and engine list executors now emit scope-specific typed read failures.
- Added focused unit tests for `SnapshotRegionMemoryReadError` and `SettingsError`, and updated settings command tests to exercise typed settings-list error payloads end-to-end.
- Eliminated remaining non-test `Result<_, String>` signatures by introducing `SymbolRegistryError` + `ValuedStructError`, and removed stale commented UI `unwrap()` usage from struct viewer row rendering.

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows.
- Prioritize replacing error signatures at trait boundaries first (`ProcessQueryer`, `ProcessQueryProvider`, engine bindings), then cascade call sites.
- Keep serialized command responses backward-compatible where needed; if shape changes are required, update tests in `squalr-tests`.
- Non-test panic/unwrap cleanup should be done before deep refactors so runtime behavior is safer during migration.
- Added `thiserror` to `squalr-engine-processes` and centralized process query failures under `process_query_error.rs`.

### Concise Session Log
- Audited repository for runtime error-handling hotspots (`unwrap`, `panic`, `Result<_, String>`, existing typed errors).
- Set current task to `pr/error_handling` and created a concrete implementation tasklist for next session.
- Replaced runtime panic/unwrap hotspots in the initial target files and added `anyhow` to CLI/TUI/GUI crates.
- Ran `cargo fmt`, `cargo check -p squalr-cli`, `cargo check -p squalr-tui`, `cargo check -p squalr-engine-api`, and `cargo check -p squalr`.
- Replaced process query + OS provider `Result<_, String>` boundaries with typed `ProcessQueryError` and updated test mocks to match.
- Ran `cargo fmt`, `cargo test -p squalr-engine-processes`, and `cargo check -p squalr-engine -p squalr-tests`.
- Replaced `Result<_, String>` in interprocess/standalone engine bindings with typed errors (`EngineBindingError`, `InterprocessPipeError`) and propagated signatures through engine-api + tests.
- Ran `cargo fmt`, `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine`, `cargo check -p squalr-tests`, `cargo test -p squalr-engine-api engine_binding_error`, and `cargo test -p squalr-tests`.
- Replaced snapshot region memory reader `Result<_, String>` signatures with typed `SnapshotRegionMemoryReadError` and added structured failure propagation.
- Replaced settings list response payloads from `Result<T, String>` to `Result<T, SettingsError>` and updated command executors/tests.
- Ran `cargo fmt`, `cargo test -p squalr-engine-scanning snapshot_region_memory_read_error`, `cargo test -p squalr-engine-api settings_error`, `cargo check -p squalr-engine -p squalr-cli -p squalr-engine-scanning`, and `cargo test -p squalr-tests --test settings_command_tests`.
- Added `SymbolRegistryError` and `ValuedStructError`, migrated remaining typed-error boundaries, and ran `cargo fmt`, `cargo test -p squalr-engine-api symbol_registry_error`, `cargo test -p squalr-engine-api valued_struct_error`, `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine`, and `cargo check -p squalr`.

