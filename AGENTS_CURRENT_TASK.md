# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/engine-refactor`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

- **Locked crate naming (do not bikeshed in this branch):**
  - **`squalr-engine-domain`** is the internal domain-model crate (rename plan target for prior `squalr-engine-structops` wording).
    Reason: `structops` is ambiguous; `domain` clearly communicates "shared engine data semantics and registries."
  - **`squalr-operating-system`** is the only OS integration crate.
    It replaces `squalr-engine-processes` + `squalr-engine-memory` during this refactor.

- **Five layers (do not mix them):**
  - **`squalr-engine-api` (public contract):** shared protocol types + versioned IPC request/response/event structs.
    No OS calls, no caches/monitors, no session ownership, no global mutable singletons.
  - **`squalr-engine-domain` (internal domain semantics):** `DataType`/`DataValue`, symbol + scan-rule registries, struct parsing/formatting, privileged string conversions.
    Not an IPC surface. Used by `squalr-engine` and `squalr-engine-session`.
  - **`squalr-engine` (pure compute):** scans/rules/RLE/pagination/snapshot merge logic given read results.
    No OS deps (`windows-sys`, `sysinfo`, ptrace, etc). No persistent state. No task handles.
  - **`squalr-operating-system` (OS primitives):** process enumeration, handle lifecycle, module/region enumeration, read/write memory, icons/bitness/permissions, IPC transport primitives.
    No long-lived caches/monitors/singletons.
  - **`squalr-engine-session` (state/policy layer):** interactive state + orchestration.
    Owns caches, monitors, process selection, snapshots, projects, progress/cancel, and command execution policy.

- **IPC rule:** do not ship huge snapshots over IPC. Prefer compressed filters + metadata, then read specific values on demand.

- **Rule of thumb:**
  - OS calls -> `squalr-operating-system`
  - compute on bytes -> `squalr-engine`
  - anything remembered across interactions -> `squalr-engine-session`
  - shared protocol/types -> `squalr-engine-api`
  - struct/data semantics -> `squalr-engine-domain` (not IPC)
  - if a file is getting overloaded with responsibilities, split it

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)

- [ ] No remaining concrete tasks are queued in this branch plan; awaiting next `pr/engine-refactor` objective.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `WindowsProcessQuery` is OS-layer code (process listing/opening/icons/windowed/bitness).
- `PROCESS_CACHE` + monitoring singletons are session/state-shim concerns (or must be owned/injected by it).

Information discovered during iteration:
- `squalr-engine` now depends on `squalr-operating-system` (legacy direct dependencies on `squalr-engine-memory`/`squalr-engine-processes` were removed).
- `squalr-engine-scanning` now depends on `squalr-operating-system`; scan entry points are blocking compute with cancellation/progress callbacks but no task ownership.
- `squalr-engine-api` contains session behavior today (`engine_unprivileged_state`, bindings, log dispatch, project manager ownership).
- Scan command responses (`element/pointer/collect-values/struct`) now expose compact `ScanResultsMetadata` instead of `TrackableTaskHandle`.
- `squalr-engine-api` currently mixes protocol + session + domain logic:
  - session: `src/engine/engine_unprivileged_state.rs`, `structures/projects/project_manager.rs`, log dispatcher;
  - domain/structops: `structures/data_types/*`, `structures/structs/*`, registries;
  - protocol/messaging: `commands/*`, `events/*`.
- `squalr-engine-api` currently requires nightly SIMD (`#![feature(portable_simd)]`), which is a red flag for a pure messaging contract crate.
- `UnprivilegedCommandRequest` and `EngineApiUnprivilegedBindings` were previously coupled to concrete `EngineUnprivilegedState`; now they target `EngineExecutionContext` trait abstraction.
- `squalr-engine-api` is depended on by most crates but is not listed as a workspace member, increasing boundary drift risk during this refactor.
- `squalr-engine-api` `Cargo.toml` currently pulls in session/domain-heavy deps (`sysinfo`, `rayon`, `notify`, `structopt`, etc), confirming it is not yet contract-only.
- `squalr-engine-processes` and `squalr-engine-memory` are no longer target end-state crates; their responsibilities consolidate into `squalr-operating-system`.
- Root `Cargo.toml` workspace members now include `squalr-engine-api`, and targeted workspace wiring check passes via `cargo check -p squalr-engine-api`.
- Memory interface traits were renamed to `MemoryWriterTrait`, `MemoryReaderTrait`, and `MemoryQueryerTrait`; dependent imports/impls in `squalr-engine-memory`, `squalr-engine-scanning`, and `squalr-engine` were updated and compile checks pass.
- Root `Cargo.toml` workspace members now include `squalr-engine-domain`, and targeted crate wiring check passes via `cargo check -p squalr-engine-domain`.
- Moving full `structures`/`registries`/`traits` directly into `squalr-engine-domain` currently fails due coupling to session-facing `engine` bindings through project item types (`EngineApiPrivilegedBindings` dependency), so migration should proceed in smaller decoupled slices.
- Conversion core is now hosted in `squalr-engine-domain`, with compatibility preserved via re-exports in `squalr-engine-api::conversions`.
- `AnonymousValueStringFormat` now lives in `squalr-engine-domain::structures::data_values` with compatibility preserved via `squalr-engine-api::structures::data_values::anonymous_value_string_format` re-export.
- Format-aware conversion adapters (`conversions_from_binary`, `conversions_from_decimal`, `conversions_from_hexadecimal`) are now hosted in `squalr-engine-domain::conversions`, re-exported by `squalr-engine-api::conversions`.
- Privileged string conversion traits are now hosted in `squalr-engine-domain::traits` with a generic context type, and re-exported via `squalr-engine-api::traits` for compatibility.
- Foundational struct-domain types `SymbolicStructRef` and `ValuedStructError` are now hosted in `squalr-engine-domain::structures::structs`, with compatibility preserved through `squalr-engine-api` module re-exports.
- `ContainerType` and `AnonymousValueString` now live in `squalr-engine-domain::structures::data_values`, with compatibility preserved via `squalr-engine-api::structures::data_values` re-export modules.
- `DataTypeRef`, `DataTypeError`, `DataTypeSizingData`, and `FloatingPointTolerance` now live in `squalr-engine-domain::structures::data_types`, with compatibility preserved via `squalr-engine-api::structures::data_types` re-export modules.
- `FloatingPointTolerance::get_value` now avoids unchecked unwraps and falls back to epsilon when conversion fails, aligning with the branch rule against panic-style failure paths.
- Root `Cargo.toml` workspace members now include `squalr-engine-session`, with initial scaffold crate wiring in place for incremental session-orchestration migration.
- Session orchestration concrete type `EngineUnprivilegedState` and logging types (`LogDispatcher`, `LogHistoryAppender`) now live in `squalr-engine-session`; `squalr-engine-api` no longer contains `engine_unprivileged_state` or engine logging modules.
- Introduced `squalr-engine-api::engine::engine_execution_context::EngineExecutionContext` so API command/binding traits depend on context capabilities instead of concrete session type.
- Updated consumers (`squalr-engine`, `squalr`, `squalr-cli`, `squalr-tests`) to depend on `squalr-engine-session` for concrete unprivileged state access.
- Added `squalr-operating-system` crate, migrated `squalr-engine-memory` and `squalr-engine-processes` source modules into it, and rewired active consumers (`squalr-engine`, `squalr-engine-scanning`, `squalr-tests`) to use the unified OS crate.
- `squalr-engine` no longer directly depends on `squalr-operating-system` or `sysinfo`; OS provider abstraction (`EngineOsProviders`) and OS type imports were moved behind `squalr-engine-session::os`.
- Removed `squalr-engine/src/os/*` and rewired privileged state + command executors to consume session-layer OS exports, preserving behavior while tightening crate boundaries.
- `squalr-engine-scanning` scan executors now run as blocking compute (`ElementScanExecutor`, `ValueCollector`, `PointerScanExecutor`) with `ScanExecutionContext` cancellation/progress callbacks; `TrackableTask` creation moved to `squalr-engine` scan command executors.
- `squalr-operating-system` process queryers no longer use global `PROCESS_CACHE`/`PROCESS_MONITOR` singletons; Windows/macOS/Android now perform immediate per-call queries and monitoring start/stop are no-op compatibility hooks.
- `EnginePrivilegedState` now lives in `squalr-engine-session` and owns process manager, snapshot, registries, task manager, freeze bootstrap task startup, and process monitoring startup.
- `TrackableTaskManager` moved from `squalr-engine` into `squalr-engine-session::tasks`; `squalr-engine` now provides bootstrap helpers (`create_engine_privileged_state*`) and re-exports the session state type.
- `squalr-engine-scanning` no longer depends on `squalr-operating-system`/`sysinfo` and no longer exposes `freeze_task`; memory reads are now supplied through `ScanExecutionContext` callbacks.
- `SnapshotScanResultFreezeTask` moved into `squalr-engine-session::tasks` and now uses `EngineOsProviders` for memory query/write operations, keeping task ownership in the session layer.
- `squalr-cli` now supports one-shot blocking command execution (`squalr-cli <command ...>`) by dispatching a single command through `EngineUnprivilegedState`, waiting for the response callback, then exiting; interactive loop behavior remains default when no command tokens are provided.
- `DataValue`, `SymbolicFieldDefinition`, `SymbolicStructDefinition`, `ValuedStruct`, and `ValuedStructField` now live in `squalr-engine-domain`; `squalr-engine-api` counterparts are re-export modules to preserve path compatibility.
- Added domain trait `SymbolResolver` so moved struct-domain logic can resolve defaults/symbolic structs without depending on `squalr-engine-api` registries; `squalr-engine-api::SymbolRegistry` now implements this trait.
- `FromStringPrivileged` implementations for moved domain types are now context-generic (`ContextType`) instead of hard-coupled to concrete registry aggregates, reducing cross-layer coupling.
- `squalr-engine-api::structures::data_types` is now a domain re-export surface; full data type implementation modules (`built_in_types`, `comparisons`, `generics`, `data_type`) now live in `squalr-engine-domain`.
- Scan contract structures required by moved data types (`structures::scanning::comparisons` and `structures::scanning::constraints`) now live in `squalr-engine-domain` and are re-exported from `squalr-engine-api`.
- `Endian` now lives in `squalr-engine-domain::structures::memory`, and `squalr-engine-api::structures::memory::endian` is a compatibility re-export module.
- `SymbolRegistry` + `SymbolRegistryError` now live in `squalr-engine-domain::registries::symbols`; `squalr-engine-api::registries::symbols` is now a compatibility re-export module.
- Aggregate registry ownership moved to `squalr-engine-session::registries::Registries`; `squalr-engine-api` now exposes `RegistryContext` trait so API/project-item contracts no longer require an API-owned aggregate state container.
- `squalr-cli` no longer carries unused logging scaffolding (`logging::cli_log_listener`); placeholder response handlers now intentionally underscore unused response arguments to keep warning output clean during incremental CLI command wiring.

Decisions locked for this branch:
- Keep one public API crate: `squalr-engine-api` is the only messaging/IPC contract surface.
- Split out domain internals to a second crate: `squalr-engine-domain` is required and is internal-only (not a second public API).
- Rename runtime shim crate to `squalr-engine-session` and move all stateful orchestration there.
- Consolidate OS integration into one crate: `squalr-operating-system` replaces `squalr-engine-processes` + `squalr-engine-memory`.
- Keep `squalr-engine-scanning` as a compute sub-crate in this branch; merging into `squalr-engine` is deferred.
- Move `TrackableTask*` out of API contracts now (not temporary); progress/cancel stays session-local.
- Keep `squalr-engine-projects` as persistence/domain support, but session owns all project lifecycle state.
- Use a single CLI binary with an interactive flag for session mode; avoid separate binary proliferation in this branch.

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- Engine stays blocking/stateless. Progress/cancel is session-only.
- Do not solve plugin registry sync in this branch.
- Keep IPC snapshot-local; return compressed scan/filter metadata.
- Keep one public API contract crate; split struct/data internals into `squalr-engine-domain`.
- Active architecture terminology uses `session`; legacy `runtime` references are retained only when describing existing code names.

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- 2026-02-08: Audited layer violations and rewrote plan/tasklist for `pr/engine-refactor` with ordered migration steps and owner questions.
- 2026-02-08: Audited `squalr-engine-api` composition and confirmed naming confusion source; documented recommendation to separate messaging contracts from struct/data operations while keeping a single public API concept.
- 2026-02-08: Converted owner TBDs into final branch decisions; locked crate naming (`squalr-engine-session`, internal domain crate), clarified "one public API + one internal domain crate", and updated task ordering accordingly.
- 2026-02-08: Clarified naming for first-time readability (`squalr-engine-domain` replacing ambiguous `squalr-engine-structops`) and updated migration plan to consolidate OS work into `squalr-operating-system`.
- 2026-02-08: Added `squalr-engine-api` to root workspace members and validated with `cargo check -p squalr-engine-api`.
- 2026-02-08: Completed trait rename cleanup for memory interfaces (`IMemory*` -> `*Trait`) across engine/memory/scanning crates and validated with targeted cargo checks.
- 2026-02-08: Added scaffold crate `squalr-engine-domain` and wired it into workspace membership; validated with `cargo check -p squalr-engine-domain`.
- 2026-02-08: Moved conversion core modules into `squalr-engine-domain`, re-exported them from `squalr-engine-api`, validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Moved privileged string conversion traits (`FromStringPrivileged`, `ToStringPrivileged`) into `squalr-engine-domain`, preserved API compatibility via re-exports, and validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Moved foundational struct-domain types (`SymbolicStructRef`, `ValuedStructError`) into `squalr-engine-domain::structures::structs`, preserved API compatibility with re-export modules in `squalr-engine-api`, and validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Moved `AnonymousValueStringFormat` and format-aware conversion adapters (`conversions_from_binary`, `conversions_from_decimal`, `conversions_from_hexadecimal`) into `squalr-engine-domain`, preserved API compatibility via re-exports in `squalr-engine-api`, and validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Moved `ContainerType` and `AnonymousValueString` into `squalr-engine-domain::structures::data_values`, preserved API compatibility via re-export modules in `squalr-engine-api`, and validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Moved `DataTypeRef`, `DataTypeError`, `DataTypeSizingData`, and `FloatingPointTolerance` into `squalr-engine-domain::structures::data_types`, preserved API compatibility via re-export modules in `squalr-engine-api`, and validated with `cargo check -p squalr-engine-domain`, `cargo check -p squalr-engine-api`, and `cargo test -p squalr-engine-domain`.
- 2026-02-08: Added scaffold crate `squalr-engine-session`, wired it into root workspace members, and validated with `cargo check -p squalr-engine-session`.
- 2026-02-08: Migrated `EngineUnprivilegedState` and engine logging implementation from `squalr-engine-api` into `squalr-engine-session`; introduced `EngineExecutionContext` abstraction in API traits to avoid crate cycles; rewired engine/cli/gui/tests imports and validated with `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine-session`, `cargo check -p squalr-engine`, `cargo check -p squalr-cli`, `cargo check -p squalr-tests`, `cargo test -p squalr-engine-session`, and `cargo test -p squalr-tests --no-run`.
- 2026-02-08: Added `squalr-operating-system` and migrated memory/process modules into it; rewired engine/scanning/tests imports and dependencies from `squalr-engine-memory`/`squalr-engine-processes` to `squalr-operating-system`; validated with `cargo check -p squalr-operating-system`, `cargo check -p squalr-engine-scanning`, `cargo check -p squalr-engine`, `cargo check -p squalr-tests`, `cargo test -p squalr-operating-system`, and `cargo test -p squalr-tests --no-run`.
- 2026-02-08: Removed `squalr-engine-memory` and `squalr-engine-processes` from root workspace members after migration completion; validated with `cargo check -p squalr-operating-system -p squalr-engine-scanning -p squalr-engine -p squalr-tests`, `cargo test -p squalr-operating-system`, and `cargo test -p squalr-tests --no-run`.
- 2026-02-08: Removed direct OS deps from `squalr-engine` by moving `EngineOsProviders` into `squalr-engine-session::os`, re-exporting required OS types from the session layer, deleting `squalr-engine/src/os/*`, and validating with `cargo check -p squalr-engine-session -p squalr-engine`, `cargo test -p squalr-engine-session`, and `cargo test -p squalr-engine --no-run`.
- 2026-02-08: Refactored scan execution to remove `TrackableTask` creation from compute paths by introducing blocking `squalr-engine-scanning` executors with `ScanExecutionContext`, then creating/registering tasks in `squalr-engine` scan command executors; validated with `cargo check -p squalr-engine-scanning`, `cargo check -p squalr-engine`, `cargo test -p squalr-engine-scanning --no-run`, and `cargo test -p squalr-engine --no-run`.
- 2026-02-09: Removed OS-layer process query singletons (`PROCESS_CACHE`, `PROCESS_MONITOR`) from `squalr-operating-system` by switching Windows/macOS/Android process enumeration to immediate per-call querying and deleting obsolete monitor helpers; validated with `cargo check -p squalr-operating-system`, `cargo check -p squalr-engine-session`, and `cargo check -p squalr-engine`.
- 2026-02-09: Moved privileged orchestration state into `squalr-engine-session` by relocating `EnginePrivilegedState` and `TrackableTaskManager`, updated `squalr-engine` to bootstrap via `create_engine_privileged_state*`, rewired tests/mocks for session OS providers, and validated with `cargo check -p squalr-engine-session -p squalr-engine -p squalr-tests`, `cargo test -p squalr-engine --no-run`, and `cargo test -p squalr-tests --no-run`.
- 2026-02-09: Updated scan IPC contracts to remove `TrackableTaskHandle` from scan responses in favor of `ScanResultsMetadata`; made scan command executors return metadata after blocking execution and kept update notifications via `ScanResultsUpdatedEvent`; validated with `cargo check -p squalr-engine-api`, `cargo check -p squalr-engine -p squalr-tests`, and `cargo test -p squalr-tests --test scan_command_tests`.
- 2026-02-09: Enforced `squalr-engine-scanning` compute boundaries by removing OS/task ownership (`freeze_task` moved to `squalr-engine-session`), injecting process memory reads via `ScanExecutionContext`, and dropping OS deps from scanning crate; validated with `cargo check -p squalr-engine-scanning -p squalr-engine-session -p squalr-engine` and `cargo test -p squalr-engine-scanning --no-run -p squalr-engine-session --no-run -p squalr-engine --no-run`.
- 2026-02-09: Rewired CLI boot behavior for `pr/engine-refactor` one-shot mode by adding blocking single-command execution in `squalr-cli` (`squalr-cli <command ...>`) while preserving interactive loop and IPC shell behavior; validated with `cargo check -p squalr-cli` and `cargo test -p squalr-cli --no-run`.
- 2026-02-09: Migrated `DataValue` and remaining struct-domain models into `squalr-engine-domain`, added `SymbolResolver` abstraction + `SymbolRegistry` impl for decoupled symbol lookup, re-exported moved modules from `squalr-engine-api`, and validated with `cargo fmt --all`, `cargo check -p squalr-engine-domain -p squalr-engine-api`, `cargo check -p squalr-engine -p squalr-engine-session -p squalr-cli -p squalr-tests`, `cargo test -p squalr-engine-domain`, and `cargo test -p squalr-engine-api --no-run`.
- 2026-02-09: Migrated full `structures/data_types/*` implementation into `squalr-engine-domain` (plus required `structures/scanning::{comparisons,constraints}`, `structures/memory::endian`, and `registries::symbols`), converted corresponding `squalr-engine-api` modules to compatibility re-exports, and validated with `cargo fmt --all`, `cargo check -p squalr-engine-domain -p squalr-engine-api`, `cargo test -p squalr-engine-domain`, and `cargo test -p squalr-engine-api --no-run`.
- 2026-02-09: Re-scoped aggregate registry ownership by moving concrete `Registries` into `squalr-engine-session`, added `squalr-engine-api::registries::registry_context::RegistryContext` for API-level registry access contracts, updated project item contracts to depend on the trait instead of a concrete API registry container, and validated with `cargo fmt --all`, `cargo check -p squalr-engine-api -p squalr-engine-session -p squalr-engine -p squalr-tests`, `cargo test -p squalr-engine-session --no-run`, and `cargo test -p squalr-tests --no-run`.
- 2026-02-09: Removed unused CLI logging scaffolding (`squalr-cli/src/logging/*`), cleaned placeholder response handler argument warnings, and validated with `cargo fmt --all`, `cargo check -p squalr-cli -p squalr-engine -p squalr-engine-session -p squalr-tests`, `cargo check -p squalr-engine -p squalr-engine-session -p squalr-tests`, and `cargo test -p squalr-cli --no-run`.
- 2026-02-09: Re-validated refactor-critical crates without source changes by running `cargo check -p squalr-engine-api -p squalr-engine-domain -p squalr-engine-session -p squalr-operating-system -p squalr-engine-scanning -p squalr-engine -p squalr-cli -p squalr-tests` and `cargo test -p squalr-engine-api -p squalr-engine-domain -p squalr-engine-session -p squalr-operating-system -p squalr-engine-scanning -p squalr-engine -p squalr-cli -p squalr-tests --no-run`.
- 2026-02-09: Compacted `Current Tasklist` to remaining work only (none queued) and repeated refactor-critical validation with `cargo check -p squalr-engine-api -p squalr-engine-domain -p squalr-engine-session -p squalr-operating-system -p squalr-engine-scanning -p squalr-engine -p squalr-cli -p squalr-tests` plus `cargo test -p squalr-engine-api -p squalr-engine-domain -p squalr-engine-session -p squalr-operating-system -p squalr-engine-scanning -p squalr-engine -p squalr-cli -p squalr-tests --no-run`.
