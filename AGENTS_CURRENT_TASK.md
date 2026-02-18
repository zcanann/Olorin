# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).

- Manual runtime verification in both GUI and TUI against `winmine+0x100579c`: confirm activated module-backed project items keep preview values updated and freeze behavior persists over time.
    - Owner: Polling refreshes of values should send a flag to not log, otherwise you get log spam...
    - Verify client/IPC builds retain recently-read scan-result values across query/refresh cycles when a read returns no payload.

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
  - TUI project-item auto-refresh no longer stops after first load; tick polling now uses `scan_settings.project_read_interval_ms` (bounded to 50..5000ms).
  - TUI scan-result periodic refresh now runs even when the scan-results pane is not focused (still guarded by in-flight query/refresh/edit/freeze/delete state).
  - GUI vs TUI scan-result cadence audit: GUI still runs a hardcoded 100ms background refresh loop in `ElementScannerResultsViewData::poll_scan_results`, while TUI now uses `results_read_interval_ms`.
  - GUI project hierarchy now performs periodic project-item refreshes driven by `scan_settings.project_read_interval_ms` (bounded to 50..5000ms), with a 1s scan-settings sync and pending-operation guards.
  - TUI and GUI scan-result query/refresh merges now preserve previously-recently-read value payloads per global result index when incoming responses have no recently-read payload, preventing regressions to stale scanned defaults in client/IPC flows.
