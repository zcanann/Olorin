# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/engine-refactor`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

- **Five layers (do not mix them):**
  - **`squalr-engine-api` (public contract):** shared protocol types + versioned IPC request/response/event structs.
    No OS calls, no caches/monitors, no session ownership, no global mutable singletons.
  - **`squalr-engine-structops` (internal domain crate):** `DataType`/`DataValue`, symbol + scan-rule registries, struct parsing/formatting, privileged string conversions.
    Not an IPC surface. Used by `squalr-engine` and `squalr-engine-session`.
  - **`squalr-engine` (pure compute):** scans/rules/RLE/pagination/snapshot merge logic given read results.
    No OS deps (`windows-sys`, `sysinfo`, ptrace, etc). No persistent state. No task handles.
  - **OS layer crates (`squalr-engine-processes`, `squalr-engine-memory`):** immediate process + memory operations only.
    No long-lived caches/monitors/singletons.
  - **`squalr-engine-session` (state/policy layer):** interactive state + orchestration.
    Owns caches, monitors, process selection, snapshots, projects, progress/cancel, and command execution policy.

- **IPC rule:** do not ship huge snapshots over IPC. Prefer compressed filters + metadata, then read specific values on demand.

- **Rule of thumb:**
  - OS calls -> OS layer crates (`squalr-engine-processes`, `squalr-engine-memory`)
  - compute on bytes -> `squalr-engine`
  - anything remembered across interactions -> `squalr-engine-session`
  - shared protocol/types -> `squalr-engine-api`
  - struct/data semantics -> `squalr-engine-structops` (not IPC)
  - if a file is getting overloaded with responsibilities, split it

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)

- [ ] Add `squalr-engine-api` to workspace members immediately to enforce boundary checks during this refactor.
- [ ] Create `squalr-engine-structops` and move `squalr-engine-api` internals there (`structures/data_types/*`, `structures/structs/*`, registries, privileged string conversion traits, conversions).
- [ ] Create `squalr-engine-session` (rename from runtime concept) and move session/state orchestration there (`src/engine/*` runtime/session behavior, event routing, log dispatch, project manager ownership).
- [ ] Remove OS crate dependencies from `squalr-engine/Cargo.toml` and keep only compute-facing dependencies.
- [ ] Move scan execution code that creates `TrackableTask` out of compute paths (`squalr-engine-scanning/*_task.rs`) so engine APIs are blocking/stateless.
- [ ] Move `EnginePrivilegedState` orchestration responsibilities (process manager, snapshot ownership, task manager, startup monitoring) into `squalr-engine-session`.
- [ ] Refactor OS crates to remove global caches/monitor singletons (`PROCESS_CACHE`, `PROCESS_MONITOR`) and expose immediate primitive operations.
- [ ] Update IPC contracts in `squalr-engine-api` so scan flows return compressed metadata/results without `TrackableTaskHandle`.
- [ ] Keep `squalr-engine-scanning` as a compute sub-crate in this branch, but enforce strict no-OS and no-task boundaries.
- [ ] Rewire CLI/TUI/GUI boot paths: one-shot CLI blocking/stateless, interactive modes use `squalr-engine-session`.
- [ ] Fix `IMemoryWriter` `IMemoryReader` `IMemoryQueryer` naming. I prefix was a vestige from the C# port, and not idiomatic to rust.

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

Decisions locked for this branch:
- Keep one public API crate: `squalr-engine-api` is the only messaging/IPC contract surface.
- Split out domain internals to a second crate: `squalr-engine-structops` is required and is internal-only (not a second public API).
- Rename runtime shim crate to `squalr-engine-session` and move all stateful orchestration there.
- Keep OS as two crates in this branch (`squalr-engine-processes`, `squalr-engine-memory`) to minimize churn; enforce primitive-only behavior and remove singleton state.
- Keep `squalr-engine-scanning` as a compute sub-crate in this branch; merging into `squalr-engine` is deferred.
- Move `TrackableTask*` out of API contracts now (not temporary); progress/cancel stays session-local.
- Keep `squalr-engine-projects` as persistence/domain support, but session owns all project lifecycle state.
- Use a single CLI binary with an interactive flag for session mode; avoid separate binary proliferation in this branch.

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- Engine stays blocking/stateless. Progress/cancel is session-only.
- Do not solve plugin registry sync in this branch.
- Keep IPC snapshot-local; return compressed scan/filter metadata.
- Keep one public API contract crate; split struct/data internals into `squalr-engine-structops`.
- Active architecture terminology uses `session`; legacy `runtime` references are retained only when describing existing code names.

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- 2026-02-08: Audited layer violations and rewrote plan/tasklist for `pr/engine-refactor` with ordered migration steps and owner questions.
- 2026-02-08: Audited `squalr-engine-api` composition and confirmed naming confusion source; documented recommendation to separate messaging contracts from struct/data operations while keeping a single public API concept.
- 2026-02-08: Converted owner TBDs into final branch decisions; locked crate naming (`squalr-engine-session`, `squalr-engine-structops`), clarified "one public API + one internal structops crate", and updated task ordering accordingly.
