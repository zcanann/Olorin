# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/engine-refactor`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

- **Four layers (don’t mix them):**
  - **`squalr-api` (public contract):** shared types + versioned IPC request/response structs.  
    No OS calls, no caches/monitors, no GUI types, no global mutable singletons.
  - **`squalr-engine` (pure compute):** scans/rules/RLE/pagination/snapshot merge logic *given read results*.  
    No OS deps (`windows_sys`, `sysinfo`, ptrace, etc). No persistent state. No task handles.
  - **`squalr-operating-system*` (operating system layer):** process + memory primitives and permissions/IPC transport.  
    OK to do “expensive immediate computation” (icons), but **long-lived caches/monitors belong elsewhere**.
  - **State shim / runtime (name TBD):** interactive state + policy (caches, monitoring, projects, progress/cancel).  
    Compiled into GUI/TUI/interactive CLI. Links to engine directly (no IPC hop).

- **IPC rule:** do not ship huge snapshots over IPC. Prefer returning compressed filters + metadata, then read specific values on-demand.

- **Rule of thumb:**
  - OS calls → `squalr-operating-system*`
  - compute on bytes → `squalr-engine`
  - anything remembered across interactions → runtime/state shim
  - shared types/protocol → `squalr-api`

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current architecture and relevant tests against our current task, and find tasks to put here for the next session.)

- [ ] Rename/move OS crates: `squalr-engine-processes` → `squalr-operating-system`, `squalr-engine-memory` → `squalr-operating-system`.
- [ ] Purge OS deps from `squalr-engine` completely.
- [ ] Move stateful stuff out of OS/engine: caches, monitors, “current selection”, projects → runtime/state shim.
- [ ] Make engine calls blocking + stateless (no task handles). If UI needs progress, runtime owns it.
- [ ] Define/confirm versioned IPC request/response structs in `squalr-api` (processes, open/close, regions/modules, read/write, scan).
- [ ] IPC mode: privileged side returns `FilterRle` + metadata (not raw snapshots). Runtime requests values on-demand.

## Important Information
Append important discoveries. Compac

Information found in initial audit:
- `WindowsProcessQuery` is OS-layer code (process listing/opening/icons/windowed/bitness).
- `PROCESS_CACHE` + monitoring singletons are runtime/state-shim concerns (or must be owned/injected by it).

Information discovered during iteration:
- 

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows:

- Engine stays blocking/stateless. Progress/cancel is runtime-only.
- Don’t solve plugin registry sync in this branch.

### Concise Session Log
Append logs for each session here. Compact redundency occasionally:
- 
