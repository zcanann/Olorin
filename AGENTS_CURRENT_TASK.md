# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- Toggling a project item address does not freeze the address. This works fine in gui build.
- Settings editing UX is abysmal (inputs dont even work?)

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- TUI now runs as three fixed workspace pages with persistent Output pane and in-page `Tab`/`Shift+Tab` focus loops.
- Workspace switching is mapped to `F1/F2/F3/F4` with explicit behavior: `F1` always enters Project Explorer context and `F4` always enters Process Selector context (no toggle behavior).
- Global key routing is top-level first (`handle_global_key_event`) before pane-local handlers.
- App exit is intentionally restricted to `Ctrl+Q` or `Ctrl+C`; plain `q` and `Esc` are not global quit keys (covered by `AppShell` tests).
- Project Explorer is context-driven (no manual `p/i` mode switch); open/activate project enters hierarchy mode, close active project returns to list mode.
- Hierarchy rows render activation state inline with names (`[ ]` / `[x]`) and support `h/Left` collapse-to-parent behavior.
- Project hierarchy supports `Home`/`End` jump to first/last visible item.
- Process Selector supports client-side search over cached process data with a dedicated `search:` row and no per-keystroke engine round-trip.
- In Process Selector search mode, only the `search:` row is highlighted; selected process rows no longer show duplicate `>` marker while typing.
- `Up` on the first process row now returns focus to search input mode for quick refinement.
- Project list supports the same client-side search pattern over cached project entries with dedicated search row UX.
- Process selector and project list both support `Home`/`End` jump navigation with matching summary hints and tests.
- Settings pane now supports `Home`/`End` field jump parity, matching other list-like panes.
- Settings summary controls line now advertises `Home/End` jump behavior explicitly.
- Struct Viewer is removed from active page layouts, but dormant state/backend coupling cleanup remains.
- Theme logic lives in `theme/tui_theme.rs` and per-pane accent differences were removed for a unified focused-pane accent.
- Header space was reclaimed by collapsing Session + Controls into a single `Info` block; `[EXIT]` controls are now merged into `[NAV]` to save one line.
- Workspace layouts now allocate more default height to Output (`40%` of body area).
- Element Scanner data types now render as a full `[ ]` / `[x]` grid with all supported types visible at once; navigation uses arrow keys (Left/Right/Up/Down), and constraint-row selection moved to `[` / `]`.
- `j/k` pane navigation bindings were removed from TUI key handlers and summary hints; list-like navigation is now arrow-key based.
- Element Scanner now supports multi-select data types via `Space`/`Enter`; hovered type is rendered with `>` and selected state remains `[ ]` / `[x]`.
- Element Scanner summary now prioritizes `[SCAN]` / `[STAT]` above type/constraint rows, renders constraints as `[CONSTRAINT #n]`, and immediate comparisons as `x <op> value`.
- Project Explorer summary is context-aware: project-list controls render only in list mode, hierarchy controls + `[PROJ]` render only in hierarchy mode.
- Scan Results now mirror selected scanner data types as active client-side filters over the current page, with summary lines for active/available type ids.
- Element Scanner removed `[LAST]`; result count/bytes now show in Scan Results summary.
- `cargo test -p squalr-tui` currently passes (30 tests); one pre-existing dead-code warning remains for dormant `TuiPane::StructViewer`.
