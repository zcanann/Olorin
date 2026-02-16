# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- 

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `ValuedStruct::combine_exclusive` compared full field equality (name + value + read-only), which removed fields during multi-select whenever values differed.
- Struct viewer had no value editor rendering path and no commit callback wiring despite an existing frame action enum.

Information discovered during iteration:
- Enter-to-commit in scan results and struct viewer value editors failed because `DataValueBoxView` used `TextEdit::id_source`, which scopes IDs to local UI context and broke caller-side `memory.has_focus(...)` checks.
- Fixed by assigning an explicit stable `TextEdit` id (`<data_value_box_id>_text_edit`) inside `DataValueBoxView`, and by wiring Enter commit in the scan-results action bar to the same frame action used by the checkmark button.
- Struct viewer edits for scan result `is_frozen` were routed through generic set-property handling, which skipped the scan-results view model's client-side freeze update path.
- Fixed by routing struct viewer `is_frozen` edits through `toggle_selected_scan_results_frozen`, preserving immediate checkbox/UI sync and existing freeze failure reversion behavior.
- Struct viewer value commits now trigger on Enter when the row's `DataValueBox` text editor has focus, using a stable text edit id and the same commit path as the checkmark button.
- Scan result continuous refresh reliability required two fixes: query/refresh executors now fall back to a default typed `DataValue` when the original snapshot `current_value` is unavailable, and the results view now requests repaint every 100ms so background refresh updates are rendered without user input.
- Added regression coverage for query/refresh recently-read sampling when a scan result has no original `current_value` template.
- Scan result refresh could deadlock client-side if a query/refresh dispatch failed (no callback meant `is_querying_scan_results` / `is_refreshing_scan_results` never reset). Command send APIs now report dispatch success, results view data clears those flags on failed dispatch, and polling now performs an initial query bootstrap.
- Borrow checker regression in scan-results query/refresh dispatch handling was caused by moving a shared dependency clone into response closures and then reusing it for dispatch-failure cleanup; fixed by creating dedicated closure-capture clones and retaining the outer clone for failure-path flag reset.
- Enter-to-commit for `DataValueBox` needed to be generated at the widget level (`TextEdit::lost_focus() && Enter`) instead of caller focus checks; callers now consume a per-box commit-on-enter signal.
- Scan result value rendering now falls back to on-demand anonymization of `recently_read_value` for the active display format when precomputed recently-read display vectors are missing, avoiding silent fallback to stale scan values.
- `ScanResult::as_valued_struct` now prefers `recently_read_value` over scan-time `current_value` for the editable `value` field, with regression coverage added.
