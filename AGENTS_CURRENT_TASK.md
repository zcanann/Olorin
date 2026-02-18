# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Freezing project items still does not actually freeze them from the TUI. Selecting the project item does nothing. Tested on a module (ie winmine+0x100579c). Either its bugged, you arent respecting modules, or something else. Again, consult gui project.

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Completed on 2026-02-18:
  - Scan results now prefer recently-read decimal values and include `previous=` when available.
  - Manual `[EDIT VAL]` input no longer gets overwritten by periodic result refresh unless user explicitly re-syncs or changes selection.
  - `[ROWS]` telemetry for scan results was folded into `[PAGE] ... | visible_rows=...`.
  - Element scanner now has a single focus target (`DataTypes` or `Constraints`) so data-type hover and constraint editing are not simultaneous.
  - Scan result page/selection display is now 1-indexed in the TUI summary/status text.
  - Project-item navigation now refreshes struct-viewer project-item focus immediately.
- Remaining blocker:
  - Project item activation does not currently drive freeze behavior for module-backed addresses in the underlying engine path. TUI activation request is dispatched and list refreshes, but freeze-side effects are not observed.
