# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/error_handling`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

- [ ] Define canonical error boundaries and ownership: engine/internal crates use typed errors (`thiserror`), CLI/GUI/TUI entrypoints use `anyhow::Result`.
  - Progress: CLI/TUI/desktop GUI entrypoints now use `anyhow::Result`; process query + OS provider now uses typed errors; engine/internal typed-error migration remains.
- [x] Replace `Result<_, String>` in process query + OS provider path with typed errors.
  Files: `squalr-engine-processes/src/process_query/process_queryer.rs`, `squalr-engine/src/os/engine_os_provider.rs`, platform-specific process query files.
- [ ] Replace `Result<_, String>` in interprocess bindings/pipes with typed errors and preserve context instead of flattening to `String`.
  Files: `squalr-engine/src/engine_bindings/interprocess/**`, `squalr-engine/src/engine_bindings/standalone/**`.
- [ ] Replace `Result<_, String>` in snapshot region memory reader with typed scan I/O errors.
  File: `squalr-engine-scanning/src/scanners/snapshot_region_memory_reader.rs`.
- [x] Replace runtime `unwrap()` in non-test crates with safe handling (`Result` propagation, guarded fallback, or structured log and early return).
  Initial files: `squalr-tui/src/main.rs`, `squalr-cli/src/main.rs`, `squalr/src/views/struct_viewer/struct_viewer_entry_view.rs`, `squalr-engine-api/src/structures/tasks/trackable_task.rs`, `squalr-engine-processes/src/process_query/android/android_process_query.rs`, `squalr-engine-api/src/structures/results/snapshot_region_scan_results.rs`.
- [x] Replace runtime `panic!` in app entrypoints with error returns and consistent startup failure reporting.
  Initial files: `squalr/src/main.rs`, `squalr-cli/src/main.rs`, `squalr-tui/src/main.rs`, `squalr-android/src/lib.rs`.
- [ ] Update API response payloads that currently embed `Result<T, String>` where appropriate to use typed serializable error payloads.
  Initial files: `squalr-engine-api/src/commands/settings/*/list/*_response.rs`.
- [ ] Add focused tests for new error conversions and propagation (process query + IPC + scan memory reader), and keep panic-based test assertions only in tests.

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

