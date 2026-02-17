# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- [ ] Audit GUI struct viewer behavior against TUI for any remaining non-command parity gaps after display-format cycling (for example nested-field UX, edit affordances, and field-level status semantics).

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
- TUI scan results pane now dispatches `ScanResultsQueryRequest`, `ScanResultsRefreshRequest`, `ScanResultsFreezeRequest`, `ScanResultsDeleteRequest`, `ProjectItemsAddRequest`, and `ScanResultsSetPropertyRequest` via keyboard-first controls (`r`, `R`, `[`/`]`, `Up`/`Down`, `Shift+Up`/`Shift+Down`, `f`, `a`, `x`, `Enter`).
- Added `scan_results_pane_state` reducer tests for page-change selection reset and range-based selected scan-result ref collection.
- Checkpoint commit for scan results parity: `1e89edc0` (`Implement TUI scan results pane command parity`).
- TUI project selector now dispatches `ProjectListRequest`, `ProjectCreateRequest`, `ProjectOpenRequest`, `ProjectRenameRequest`, `ProjectDeleteRequest`, and `ProjectCloseRequest` with keyboard-first controls (`r`, `n`, `Enter`/`o`, `e`, `x`, `c`, `j`/`k`, plus inline name input commit/cancel).
- Added `project_explorer_pane_state` reducer tests for default selection on list load, wraparound project selection, and rename-input guard behavior.
- Checkpoint commit for project selector parity: `58e938ef` (`Implement TUI project selector command parity`).
- TUI project hierarchy now dispatches `ProjectItemsListRequest`, `ProjectItemsCreateRequest`, `ProjectItemsDeleteRequest`, `ProjectItemsActivateRequest`, `ProjectItemsMoveRequest`, and `ProjectItemsReorderRequest` with keyboard-first controls (`i` hierarchy mode, `h` refresh, `j`/`k` select, `l`/`Left` expand-collapse, `Space` activate, `n` create folder, `x` confirm-delete, `m` stage move, `b` move here, `[`/`]` reorder).
- Checkpoint commit for project hierarchy parity: `7d69407d` (`Implement TUI project hierarchy keyboard command parity`).
- TUI struct viewer now tracks focused source (`scan results` or `project items`), supports keyboard-first field navigation/edit buffering (`j`/`k`, `Enter`, text input), and routes commits through scan-result/property commands (`ScanResultsSetPropertyRequest`, `ScanResultsFreezeRequest`) and project-item edit routing (`ProjectSaveRequest`, `ProjectItemsRenameRequest`, `MemoryWriteRequest`) with selection sync from scan-results/project-hierarchy reducers.
- TUI settings pane now has keyboard-first category/field reducers and command parity for general/memory/scan list+set requests, including in-pane state summaries and per-category apply actions.
- TUI output pane now reads log history from `EngineUnprivilegedState` on tick, supports periodic redraw/preview, and adds keyboard actions for refresh (`r`), clear (`x`), and max-line bounds (`+`/`-`).
- Added focused tests for settings and output reducers plus app-level focused-pane keyboard routing in `squalr-tui`; validated with `cargo test -p squalr-tui` (25 passed).
- Checkpoint commit for settings/output/test parity: `468acfcb` (`Implement TUI settings/output parity and routing tests`).
- GUI vs TUI parity audit (this pass): command request surface in `squalr/src/views` and `squalr-tui/src` is now matched for all high-value panes (process, element scanner, scan results, project selector/hierarchy, struct viewer, settings).
- Remaining high-value parity gap is behavior-level, not command-level: GUI wires `ScanResultsUpdatedEvent` and periodic `ScanResultsRefreshRequest` in `element_scanner_results_view_data`, while TUI currently relies on manual refresh triggers and selected-only refresh actions.
- Next concrete `pr/tui` implementation target is to add event-driven and periodic scan-results synchronization in `squalr-tui` with keyboard-first status visibility and reducer/app-shell tests for throttling and requery behavior.
- TUI app shell now registers a one-time `ScanResultsUpdatedEvent` listener, tracks event counters thread-safely, and requeries the current scan-results page on tick when pending updates exist and no query is in flight.
- TUI scan-results refresh now supports a bounded periodic loop driven by scan setting `results_read_interval_ms` clamped to 50-5000ms, gated to visible scan-results pane + active selection and suppressed during conflicting in-flight operations.
- Added app-shell tests for engine-update signal gating, visibility/selection gating for periodic refresh, and bounded interval behavior; validated with `cargo test -p squalr-tui` (28 passed).
- GUI vs TUI parity audit (this pass): GUI refreshes values for the entire queried scan-results page on its loop; TUI previously refreshed only selected rows. TUI now refreshes all current-page rows for periodic/manual refresh paths and no longer requires an active selection for periodic refresh gating.
- Updated scan-results periodic refresh tests to validate pane-visibility + non-empty-page gating and bounded interval behavior against the page-level refresh model; validated with `cargo test -p squalr-tui` (28 passed).
- Next behavior-level parity target identified: GUI struct viewer exposes multiple display representations per field (`anonymize_value_to_supported_formats`), while TUI currently surfaces/edits only a single default-format representation.
- TUI struct viewer now materializes supported per-field display representations via `SymbolRegistry::anonymize_value_to_supported_formats`, tracks active format per field, and supports keyboard-first format cycling (`[` previous, `]` next) with in-pane status and summary feedback.
- Added struct-viewer reducer tests for format cycling and uncommitted-edit guard behavior, plus app-shell tests for focused `]` key routing and struct-edit payload formatting via a request-builder helper (`build_scan_results_set_property_request_for_struct_edit`).
- Validation pass: `cargo test -p squalr-tui` (32 passed).


