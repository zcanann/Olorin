# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- Collapse Controls + Session into an info section. Reduces 1 header giving us screen space back.
- Evaluate replacing global `q`/`Esc` quit with safer quit behavior now that text input/search flows are expanding.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Implemented three page-based workspaces with fixed layouts and persistent output pane.
- `Tab`/`Shift+Tab` now cycles focus within active page only.
- Struct Viewer was removed from active navigation/layout (no page includes it), but backend/state coupling still exists and needs follow-up cleanup.
- Removed automatic Struct Viewer coupling from scan/project command paths (state mutation + command side effects no longer auto-sync struct viewer focus).
- Theme implementation moved from `theme/mod.rs` into dedicated `theme/tui_theme.rs` and re-exported by `theme/mod.rs`.
- Project workspace now uses context routing:
  - Default: Process Selector (full-width) + Output.
  - After opening a process: Project Explorer (full-width) + Output.
  - `F4` reopens Process Selector view from anywhere.
- Global key routing in `AppShell` now uses a dedicated `handle_global_key_event` path before pane-local handlers, aligning dispatch flow with a clear top-level route-then-handle pattern.
- Workspace hotkeys moved from `1/2/3` to `F1/F2/F3` to avoid scan/input collisions; process selector routing is intentionally less accessible on `F4`.
- Process Selector search flow added:
  - `/` enters search mode.
  - typing updates `search_name` filtering via process-list request.
  - `Backspace` edits, `Ctrl+u` clears, `Enter` commits mode, `Esc` cancels and clears filter.
- Summary/header declutter pass completed:
  - Process Selector, Project Explorer, Scan Results, Element Scanner, Settings, and Output summaries were reduced to concise action + status + essential state lines.
- Project Explorer mode is now context-driven (no `p/i` toggle): opening/activating a project moves to hierarchy interaction mode automatically, and closing the active project returns to project-list mode.
- `c` (close active project) is now available from hierarchy key handling, preserving the mode-return loop without manual toggles.
- Project hierarchy rows now place activation state next to names using `[ ]`/`[x]`; hierarchy mode no longer allocates row space to project list.
- Element scanner summary now avoids duplicate constraint row rendering.
- Panel focus accent is now unified across panes instead of per-pane color variants.
- `cargo test -p squalr-tui` passes with 10 tests; one existing warning remains for dormant `TuiPane::StructViewer` variant.
