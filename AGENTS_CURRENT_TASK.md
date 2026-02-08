# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/engine-refactor`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

- **Four layers (do not mix them):**
  - **`squalr-engine-api` (public contract):** shared types + versioned IPC request/response structs.
    No OS calls, no caches/monitors, no GUI/runtime session ownership, no global mutable singletons.
  - **`squalr-engine` (pure compute):** scans/rules/RLE/pagination/snapshot merge logic given read results.
    No OS deps (`windows-sys`, `sysinfo`, ptrace, etc). No persistent state. No task handles.
  - **`squalr-operating-system` (OS layer):** process + memory api calls.
    No long-lived caches/monitors/singletons.
  - **Runtime (`squalr-runtime`, name TBD):** interactive state + policy.
    Owns caches, monitors, process selection, snapshots, projects, and progress/cancel.

- **IPC rule:** do not ship huge snapshots over IPC. Prefer compressed filters + metadata, then read specific values on demand.

- **Rule of thumb:**
  - OS calls -> `squalr-operating-system`
  - compute on bytes -> `squalr-engine`
  - anything remembered across interactions -> runtime
  - shared protocol/types -> `squalr-engine-api`
  - if a file is getting overloaded with responsibilities, split it

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)

- [ ] Decide final crate naming and split strategy (`squalr-operating-system` single crate vs separate `-processes`/`-memory`; runtime crate name).
- [ ] Make `squalr-engine-api` contract-only by moving `src/engine/*` runtime/session behavior to runtime crate.
- [ ] Remove OS crate dependencies from `squalr-engine/Cargo.toml` and keep only compute-facing dependencies.
- [ ] Move scan execution code that creates `TrackableTask` out of compute paths (`squalr-engine-scanning/*_task.rs`) so engine APIs become blocking/stateless.
- [ ] Move `EnginePrivilegedState` orchestration responsibilities (process manager, snapshot ownership, task manager, startup monitoring) into runtime.
- [ ] Refactor OS crates to remove global caches/monitor singletons (`PROCESS_CACHE`, `PROCESS_MONITOR`) and expose immediate primitive operations.
- [ ] Update IPC contracts in `squalr-engine-api` so scan flows return compressed metadata/results without task handles.
- [ ] Rewire CLI/TUI/GUI boot paths: one-shot CLI blocking/stateless, interactive modes use runtime shim.
- [ ] Fix `IMemoryWriter` `IMemoryReader` `IMemoryQueryer` naming. I prefix was a vestige from the C# port, and not idiomatic to rust.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `WindowsProcessQuery` is OS-layer code (process listing/opening/icons/windowed/bitness).
- `PROCESS_CACHE` + monitoring singletons are runtime/state-shim concerns (or must be owned/injected by it).

Information discovered during iteration:
- `squalr-engine` currently depends on `squalr-engine-memory`, `squalr-engine-processes`, `squalr-engine-projects`, and `squalr-engine-scanning` directly.
- `squalr-engine-scanning` directly depends on `squalr-engine-memory` and currently owns task-oriented execution entry points.
- `squalr-engine-api` contains runtime/session behavior today (`engine_unprivileged_state`, bindings, log dispatch, project manager ownership).
- Scan responses currently expose optional `TrackableTaskHandle` in API surface.

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- Engine stays blocking/stateless. Progress/cancel is runtime-only.
- Do not solve plugin registry sync in this branch.
- Keep IPC snapshot-local; return compressed scan/filter metadata.

### QUESTIONS FOR OWNER
- Crate naming decision: one `squalr-operating-system` crate, or keep separate `squalr-operating-system-memory` and `squalr-operating-system-processes`?
- Should `squalr-engine-scanning` be merged into `squalr-engine` in this branch, or kept as a compute sub-crate with strict no-OS boundaries?
- Should `TrackableTask*` types stay in `squalr-engine-api` temporarily for compatibility, or be moved behind runtime-only APIs now?
- Should project management live entirely in runtime for this branch, or stay partially in existing `squalr-engine-projects` until a follow-up branch?
- For interactive CLI mode: prefer separate binary or single binary with an interactive boot flag?

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- 2026-02-08: Audited layer violations and rewrote plan/tasklist for `pr/engine-refactor` with ordered migration steps and owner questions.