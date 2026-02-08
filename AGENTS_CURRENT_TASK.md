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

- [x] Add `squalr-engine-api` to workspace members immediately to enforce boundary checks during this refactor.
- [ ] Create `squalr-engine-domain` and move domain semantics from `squalr-engine-api` there (`structures/data_types/*`, `structures/structs/*`, registries, privileged string conversion traits, conversions).
- [x] Move conversion core modules (`base_system_conversions`, `command_line_conversions`, `conversion_error`, `conversions_from_primitives`, `storage_size_conversions`) into `squalr-engine-domain` and re-export them from `squalr-engine-api`.
- [x] Move format-aware conversion adapters (`conversions_from_binary`, `conversions_from_decimal`, `conversions_from_hexadecimal`) into `squalr-engine-domain` and re-export them from `squalr-engine-api`.
- [x] Move privileged string conversion traits into `squalr-engine-domain` and re-export them from `squalr-engine-api`.
- [ ] Create `squalr-engine-session` and move session/state orchestration there (`src/engine/*` runtime/session behavior, event routing, log dispatch, project manager ownership).
- [ ] Create `squalr-operating-system`, move code from `squalr-engine-processes` + `squalr-engine-memory` into it, and switch dependents to the unified crate.
- [ ] Remove `squalr-engine-processes` and `squalr-engine-memory` from the workspace once their code has been migrated and callers are updated.
- [ ] Remove OS dependencies from `squalr-engine/Cargo.toml` and keep only compute-facing dependencies.
- [ ] Move scan execution code that creates `TrackableTask` out of compute paths (`squalr-engine-scanning/*_task.rs`) so engine APIs are blocking/stateless.
- [ ] Move `EnginePrivilegedState` orchestration responsibilities (process manager, snapshot ownership, task manager, startup monitoring) into `squalr-engine-session`.
- [ ] Refactor OS layer implementation to remove global caches/monitor singletons (`PROCESS_CACHE`, `PROCESS_MONITOR`) and expose immediate primitive operations only.
- [ ] Update IPC contracts in `squalr-engine-api` so scan flows return compressed metadata/results without `TrackableTaskHandle`.
- [ ] Keep `squalr-engine-scanning` as a compute sub-crate in this branch, but enforce strict no-OS and no-task boundaries.
- [ ] Rewire CLI/TUI/GUI boot paths: one-shot CLI blocking/stateless, interactive modes use `squalr-engine-session`.
- [x] Rename `IMemoryWriter`, `IMemoryReader`, `IMemoryQueryer` to idiomatic Rust trait names without `I` prefixes.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `WindowsProcessQuery` is OS-layer code (process listing/opening/icons/windowed/bitness).
- `PROCESS_CACHE` + monitoring singletons are session/state-shim concerns (or must be owned/injected by it).

Information discovered during iteration:
- `squalr-engine` currently depends on `squalr-engine-memory`, `squalr-engine-processes`, `squalr-engine-projects`, and `squalr-engine-scanning` directly.
- `squalr-engine-scanning` directly depends on `squalr-engine-memory` and currently owns task-oriented execution entry points.
- `squalr-engine-api` contains session behavior today (`engine_unprivileged_state`, bindings, log dispatch, project manager ownership).
- Scan responses currently expose optional `TrackableTaskHandle` in API surface.
- `squalr-engine-api` currently mixes protocol + session + domain logic:
  - session: `src/engine/engine_unprivileged_state.rs`, `structures/projects/project_manager.rs`, log dispatcher;
  - domain/structops: `structures/data_types/*`, `structures/structs/*`, registries;
  - protocol/messaging: `commands/*`, `events/*`.
- `squalr-engine-api` currently requires nightly SIMD (`#![feature(portable_simd)]`), which is a red flag for a pure messaging contract crate.
- `UnprivilegedCommandRequest` is coupled to session state (`EngineUnprivilegedState`) instead of pure transport contracts.
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
