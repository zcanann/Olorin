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
- `combine_exclusive` now intersects by field name, preserving common fields even when values differ.
- Struct viewer rows now render `DataValueBoxView` + commit button and produce edit callbacks.
- Struct edit callbacks are now wired to `ScanResultsSetPropertyRequest` for selected rows.
- Added unit tests for `combine_exclusive` behavior in `squalr-engine-domain`.
- Fixed property-edit scope: struct viewer edits now send `ScanResultsSetPropertyRequest` only for selected scan results, not all visible rows.
- Added `ElementScannerResultsViewData` tests validating single-select, multi-select, and no-selection targeting behavior.
- Re-ran targeted regression tests and `cargo check -p squalr`; all passed, with existing unrelated warnings.
- Fixed struct viewer data box ID collisions by generating per-row IDs and namespacing popup area IDs with the widget ID.
- Enforced `DataValueBoxView` read-only behavior by disabling text edits and dropdown interaction when read-only.
- Updated scan-result struct projection so `is_frozen` is read-only in struct viewer; only `value` remains writable.
- Added `ScanResult::as_valued_struct` unit test ensuring only `value` is writable.
- Updated readonly struct-viewer rows to hide commit buttons while keeping interpretation/display-type selection available.
- Added `DataValueBoxView` options to permit readonly interpretation popup interaction and preview-style neutral text coloring.
- Fixed struct viewer value-box text coloring so writable fields use `foreground` while read-only fields continue using `foreground_preview`.
- Fixed readonly struct-viewer value-box dropdown labeling and behavior to use `Display as ...` and swap to the matching precomputed display string per selected format (instead of only changing format metadata).
- Struct viewer now caches per-field display strings for all supported formats using the original typed `DataValue` bytes, avoiding reinterpret/cast paths for readonly display switching.
- Fixed bool display-string anonymization for `Display as` formats so binary/decimal/hex now render as `0/1` (without `0b`/`0x` prefixes) instead of always `true/false`, preventing false validation errors and avoiding redundant prefixes in the data value box.
- Added focused bool-format anonymization unit tests in `squalr-engine-domain` covering Bool/Binary/Decimal/Hexadecimal output strings.
- Fixed readonly `DataValueBoxView` format coloring so Binary/Hexadecimal previews use `binary_blue_preview` and `hexadecimal_green_preview` instead of writable colors.
