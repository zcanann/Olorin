# AGENTS.MD

## Workflow
- Always scan `README.md` first for the project overview + architecture constraints.
- Then read **Agentic Current Task** (below) and continue from there.
- End each session by:
  - removing unused imports / dead helpers,
  - running relevant tests,
  - checkpointing with a commit,
  - updating **Agentic Current Task** + **Concise Session Log**, compacting existing notes if needed to eliminate redundency and spam.

If you must patch source files to fix bugs while testing, that’s acceptable as long as the README architecture remains intact.

## Coding Conventions
- Variable names must be coherent and specific. `i`, `idx`, and generic `index` are forbidden. Use names like `snapshot_index`, `range_index`, `scan_result_index`, etc.
- No unhandled `unwrap()`, panics, etc. On failure: return a `Result` or log via `log!` macros.
- Comments end with a period.
- Format with default Rust formatter (repo includes `.rustfmt.toml`).
- Prefer rustdoc/intellisense-friendly function comments where practical.
- Remove unused imports.
- Prefer single-responsibility. Do not inline structs that do not belong in the file.

## Agentic Current Task
### Goal
We are on `pr/unit-tests` building `squalr-tests` as a workspace crate to test **command/response contracts** for GUI/CLI commands. The test suite is about validating **request payloads, dispatch, and typed response decoding**.

### Phase 1 (done)
Contract tests for parsing + request dispatch + typed response extraction are implemented and split into per-command suites.

### Phase 2 (in progress): OS Mock / DI seam
We need deterministic tests for privileged executors that currently call static OS singletons directly (examples: `MemoryQueryer`, `MemoryReader`, `MemoryWriter`, `ProcessQuery`). Until we have DI seams, “real OS behavior” tests can’t be correct or stable.

**Deliverable:** a minimal dependency-injection seam so tests can supply mock OS behavior.

### Current State (facts)
- `squalr-tests` exists as a workspace crate.
- Tests are split by command under `squalr-tests/tests/*_command_tests.rs`.
- Phase 1 covers all currently exposed `PrivilegedCommand` + `UnprivilegedCommand` variants with contract tests.
- `EnginePrivilegedState` now supports injected OS providers (process query + memory query/read/write), with production defaults bound to existing singletons.
- `scan_results` privileged executors (`query`, `list`, `refresh`, `freeze`, `set_property`) now route memory operations through injected OS providers.
- `MockEngineBindings` is centralized in `squalr-tests/src/mocks/mock_engine_bindings.rs` and reused by all command contract suites.
- Deterministic OS-behavior tests exist in `squalr-tests/tests/os_behavior_command_tests.rs` for memory read/write, process list/open/close, scan-new page bounds merge flow, and scan-results query/list/refresh/freeze/set-property flows.
- `scan_results add_to_project` remains a stubbed executor in this branch because project-item mutation hooks are not wired yet.
- `cargo test -p squalr-tests` is currently passing (107 integration tests).

### If something is too hard to test
- Stub the test, and write down **why** (architecture limitation) + what would need to change.
- Keep notes short and aligned with the README architecture plan.

## Agent Scratchpad and Notes

### Current Tasklist (Remove as things are completed, add remaining tangible tasks)
- Keep `scan_results add_to_project` coverage at contract-level only until project-item mutation hooks are implemented.

### Architecture Plan (Modify sparringly as new information is learned. Keep minimal and simple)
- Phase 1: command parsing + request dispatch + typed response decode via engine API mocks. [done]
- Phase 2: OS-behavior tests with injectable privileged OS access. [in progress]
- Implemented seam: process/memory providers attached to `EnginePrivilegedState` as trait objects, defaulting to current singleton-backed behavior in production.

### Concise Session Log (append-and-compact-only, keep very short)
- `pr/unit-tests`: Added `squalr-tests` workspace crate.
- `pr/unit-tests`: Split integration tests into per-command suites for maintainability.
- `pr/unit-tests`: Added broad parser + command/response contract coverage.
- `pr/unit-tests`: Added `EnginePrivilegedState` OS provider DI seam (process query + memory query/read/write) with production defaults.
- `pr/unit-tests`: Added canonical OS mock surface in `squalr-tests/src/mocks/mock_os.rs`.
- `pr/unit-tests`: Centralized `MockEngineBindings` in `squalr-tests/src/mocks/mock_engine_bindings.rs` and updated all command contract suites to reuse it.
- `pr/unit-tests`: Wired `scan_results` executors (`query/list/refresh/freeze/set_property`) to injected OS providers instead of static OS singletons.
- `pr/unit-tests`: Expanded deterministic OS-behavior tests (`os_behavior_command_tests`) to cover scan-results query/list/refresh/freeze provider usage.
- `pr/unit-tests`: Added deterministic `scan_results set_property` OS-behavior tests for value writes and freeze/unfreeze toggling through injected providers.
- `pr/unit-tests`: Fixed bool deanonymization for supported formats so `set_property is_frozen` decodes boolean payloads correctly.

## Agentic Off Limits / Not ready yet
- `pr/cli-bugs`: CLI does not spawn a window / execute commands reliably; align with GUI behavior.
- `pr/error_handling`: Normalize error style (engine uses struct-based errors; cli/gui can use `anyhow!`).
- `pr/tui`: Not ready yet.
