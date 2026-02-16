# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- Get the GUI struct viewer functional. The idea is that we should be able to click on a scan result, sync it to the struct viewer, and have values displayed for all selected items.
- The columns should have a data_value_box on the right side with editable data.
- Upon committing a value (possibly a separate commit button similar to scan results), whoever sent the last struct should probably get a callback indicating field edit
- Validate behavior in GUI for mixed-value multi-select edits (`value`, `is_frozen`) and ensure the expected write target scope.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `ValuedStruct::combine_exclusive` compared full field equality (name + value + read-only), which removed fields during multi-select whenever values differed.
- Struct viewer had no value editor rendering path and no commit callback wiring despite an existing frame action enum.

Information discovered during iteration:
- `combine_exclusive` now intersects by field name, preserving common fields even when values differ.
- Struct viewer rows now render `DataValueBoxView` + commit button and produce edit callbacks.
- Struct edit callbacks are now wired to `ScanResultsSetPropertyRequest` for selected rows.
- Added unit tests for `combine_exclusive` behavior in `squalr-engine-domain`.

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- Remaining validation is primarily GUI behavior/UX verification, since automated checks only compile/test backend logic.

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- Audited struct viewer and scan-result selection wiring; fixed multi-select field intersection bug, implemented struct viewer value editing/commit UI, wired commit callbacks to scan result property writes, added domain tests for intersection behavior, and validated with `cargo fmt`, targeted tests, and `cargo check -p squalr`.
