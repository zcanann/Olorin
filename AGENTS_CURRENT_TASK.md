# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- Audit GUI/TUI command and keybinding parity for core workflows (process open, scan run, project item ops, settings edit) and identify high-impact gaps.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Implemented three page-based workspaces with fixed layouts and persistent output pane:
  - `1` Project page: Process Selector + Project Explorer + Output.
  - `2` Scanner page: Element Scanner + Scan Results + Output.
  - `3` Settings page: Settings + Output.
- `Tab`/`Shift+Tab` now cycles focus within active page only.
- Struct Viewer was removed from active navigation/layout (no page includes it), but backend/state coupling still exists and needs follow-up cleanup.
- Removed automatic Struct Viewer coupling from scan/project command paths:
  - Scan results + project handlers no longer trigger `sync_struct_viewer_focus_*` as side effects.
  - Project open/rename/delete/close and project-state sync no longer clear/sync Struct Viewer state.
  - Scan-result/project-item edits continue through dedicated pane interactions (`ScanResultsSetPropertyRequest`, project-item command paths).
- Theme implementation moved from `theme/mod.rs` into dedicated `theme/tui_theme.rs` and re-exported by `theme/mod.rs`.
- Process open is now integrated into Project workspace flow: opening a process shifts focus to Project Explorer, updates Project Explorer status with process context, and refreshes hierarchy when an active project exists.
- Global key routing in `AppShell` now uses a dedicated `handle_global_key_event` path before pane-local handlers, aligning dispatch flow with a clear top-level route-then-handle pattern.
- Focus cycling now follows the active workspace pane order (Project: Process Selector -> Project Explorer -> Output, Scanner: Element Scanner -> Scan Results -> Output, Settings: Settings -> Output) rather than global pane enum ordering.
- Added focused unit tests for workspace shortcut mapping, workspace pane composition, workspace switch focus rehoming, shared Output focus persistence, and forward/backward focus loops for Project/Scanner/Settings pages.
- Header metadata now documents current-page focus-loop keybindings via a `[FOCUS]` hint.
- `cargo test -p squalr-tui` passes with 7 tests; one existing warning remains for dormant `TuiPane::StructViewer` variant.
