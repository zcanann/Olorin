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
- Struct viewer edits for scan result `is_frozen` were routed through generic set-property handling, which skipped the scan-results view model's client-side freeze update path.
- Fixed by routing struct viewer `is_frozen` edits through `toggle_selected_scan_results_frozen`, preserving immediate checkbox/UI sync and existing freeze failure reversion behavior.
- Struct viewer value commits now trigger on Enter when the row's `DataValueBox` text editor has focus, using a stable text edit id and the same commit path as the checkmark button.
