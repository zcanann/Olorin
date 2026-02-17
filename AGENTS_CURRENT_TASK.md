# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- [x] Audit the GUI project and produce a ratatui-first TUI parity plan.
- [x] Implement ratatui app shell in `squalr-tui` (terminal init/restore, tick loop, input loop, graceful shutdown).
- [x] Add TUI state model split by pane: process selector, element scanner, scan results, project explorer, struct viewer, output, settings.
- [x] Implement top-level layout and pane focus navigation (tab cycling, global shortcuts, visible pane toggles, non-mouse workflow).
- [x] Implement process selector pane with command parity: `ProcessListRequest` (windowed/full) + `ProcessOpenRequest`.
- [x] Implement element scanner toolbar parity: new scan, collect values, start scan, data type select, up to 5 constraints.
- [ ] Implement scan results pane parity: page navigation, selection range, freeze toggles, add to project, delete, commit edited value.
- [ ] Implement project selector parity: list/create/open/rename/delete/close project.
- [ ] Implement project hierarchy parity (keyboard-first): expand/collapse, select, activate toggle, create folder, delete confirm, move/reorder (non-drag alternatives).
- [ ] Implement struct viewer parity for selected scan results/project items, including edit commit callback routing.
- [ ] Implement settings panes parity (general/memory/scan list+set requests).
- [ ] Implement output pane parity using log history stream + periodic redraw.
- [ ] Add focused unit tests in `squalr-tui` for pure state reducers and keyboard command routing.
- [x] Run `cargo fmt` and targeted tests (`cargo test -p squalr-tui` + selected engine/view-model tests as needed).
- [x] Checkpoint commit and keep this task file compact as milestones complete.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `squalr-tui` currently initializes `SqualrEngine` only; no terminal event loop or UI rendering exists yet.
- GUI default surface is 7 docked windows: Process Selector, Project Explorer, Struct Viewer, Output, Element Scanner, Pointer Scanner, Settings.
- Pointer Scanner is currently a stub in GUI too, so TUI can keep it placeholder without parity regression.
- High-value parity path is command/view-data parity, not pixel/docking parity. Mouse-heavy drag interactions should become keyboard commands in TUI.

Information discovered during iteration:
- Process selector command parity requirements identified: `ProcessListRequest` (windowed + full) and `ProcessOpenRequest`.
- Element scanner command parity requirements identified: `ScanNewRequest`, `ElementScanRequest`, `ScanCollectValuesRequest`, `ScanResetRequest`.
- Scan results command parity requirements identified: `ScanResultsQueryRequest`, `ScanResultsRefreshRequest`, `ScanResultsSetPropertyRequest`, `ScanResultsFreezeRequest`, `ScanResultsDeleteRequest`, `ProjectItemsAddRequest`.
- Project selector command parity requirements identified: `ProjectListRequest`, `ProjectCreateRequest`, `ProjectOpenRequest`, `ProjectRenameRequest`, `ProjectDeleteRequest`, `ProjectCloseRequest`.
- Project hierarchy command parity requirements identified: `ProjectItemsListRequest`, `ProjectItemsCreateRequest`, `ProjectItemsDeleteRequest`, `ProjectItemsActivateRequest`, `ProjectItemsMoveRequest`, `ProjectItemsReorderRequest`, plus edit side effects (`ProjectSaveRequest`, `ProjectItemsRenameRequest`, `MemoryWriteRequest`).
- Settings command parity requirements identified: list/set pairs for general, memory, and scan settings.
- Existing GUI view-data modules already encapsulate most command logic and are a strong extraction target for shared UI-agnostic state/actions to reduce duplication between egui and ratatui.
- `squalr-tui` now has a working ratatui+crossterm shell with alternate-screen setup/restore, raw-mode guard via `Drop`, tick-based redraw loop, and keyboard exit handling (`q`, `Esc`, `Ctrl+C`).
- `ratatui` is pinned to `0.30.0` with `crossterm_0_29` feature to avoid workspace dependency resolution conflicts seen with `0.29.0`.
- `squalr-tui` state is now split into dedicated pane modules under `src/state/` with a single `TuiAppState` aggregator, and app shell/runtime is separated into `src/app/mod.rs` to match the owner’s anti-bloat guidance.
- Checkpoint commit for this milestone: `f1236be0` (`Add pane-split TUI state model scaffold`).
- TUI now renders a real multi-pane top-level layout with keyboard-only workflow: focus cycle (`Tab`/`Shift+Tab`), direct pane focus (`1-7`), pane visibility toggles (`Ctrl+1-7` or `v` for focused pane), and restore-all (`0`).
- Added reducer tests in `squalr-tui` for focus cycling across hidden panes, hidden-pane focus restore, and guard rails preventing all panes from being hidden.
- TUI process selector now dispatches `ProcessListRequest` (windowed/full toggle) and `ProcessOpenRequest` from keyboard-first controls (`r`, `w`, `Up/Down`, `Enter`), with synchronous response handling and visible status messages in-pane.
- Checkpoint commit for process selector parity: `92cad535` (`Implement TUI process selector command parity`).
- TUI element scanner now dispatches `ScanResetRequest`, `ScanCollectValuesRequest`, `ScanNewRequest`, and `ElementScanRequest` with keyboard-first controls (`n`, `c`, `s`, `t`/`T`, `a`, `x`, `j`/`k`, `m`/`M`) and per-pane status/result metadata updates.
- Added `element_scanner_pane_state` reducer tests for constraint cap/retention, data type cycling, and relative constraint serialization behavior.
- Checkpoint commit for element scanner parity: `55a6a38f` (`Implement TUI element scanner command parity`).


