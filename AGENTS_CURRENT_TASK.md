# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Verify end-to-end in GUI and TUI that activating module-backed project items both updates preview values and applies freeze behavior over time (winmine+0x100579c).

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- Completed on 2026-02-18:
  - Scan results now prefer recently-read decimal values and include `previous=` when available.
  - Manual `[EDIT VAL]` input no longer gets overwritten by periodic result refresh unless user explicitly re-syncs or changes selection.
  - `[ROWS]` telemetry for scan results was folded into `[PAGE] ... | visible_rows=...`.
  - Element scanner now has a single focus target (`DataTypes` or `Constraints`) so data-type hover and constraint editing are not simultaneous.
  - Scan result page/selection display is now 1-indexed in the TUI summary/status text.
  - Project-item navigation now refreshes struct-viewer project-item focus immediately.
  - Project-item list now refreshes address preview values using privileged memory reads (removes persistent `??` when reads succeed).
  - Project-item activation now dispatches privileged memory-freeze targets for address items (module-backed and absolute addresses).
