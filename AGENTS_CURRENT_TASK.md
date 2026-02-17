# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- Integrate process opening into the new Project workspace flow so selecting/opening process does not feel like a detached utility.
- Remove Struct Viewer command-path coupling from scan/project actions and route edits through dedicated scan-results/project-item interactions.
- Audit command dispatch structure against CLI patterns and align naming/routing where practical without regressing TUI UX.
- Add focused behavior checks for page switching + pane focus loops (Project, Scanner, Settings) and document keybindings.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Implemented three page-based workspaces with fixed layouts and persistent output pane:
  - `1` Project page: Process Selector + Project Explorer + Output.
  - `2` Scanner page: Element Scanner + Scan Results + Output.
  - `3` Settings page: Settings + Output.
- `Tab`/`Shift+Tab` now cycles focus within active page only.
- Struct Viewer was removed from active navigation/layout (no page includes it), but backend/state coupling still exists and needs follow-up cleanup.
- Theme implementation moved from `theme/mod.rs` into dedicated `theme/tui_theme.rs` and re-exported by `theme/mod.rs`.
