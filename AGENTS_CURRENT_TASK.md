# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/project-explorer`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- Double-click add to project should always prioritize adding to the selected directory item
- There should be a root level folder (which is not displayed a folder) with all project items. This is to avoid conflicts with project.json
- If no directory is selected, adding new items to the project should default to this root folder.
- Struct viewer should work for project items (ie name).
- Struct viewer should allow editing addresses to trigger a write to memory.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- `ProjectHierarchyViewData` is currently empty and `ProjectHierarchyView` only renders toolbar, so opened-project hierarchy UX is not implemented.
- `ProjectHierarchyFrameAction` currently only has `None`, so hierarchy interactions (select, expand, delete, reorder, etc.) are not wired.
- `project-items` API/executor currently supports only `list` and `activate`, which is insufficient for project explorer editing workflows.
- `scan-results add-to-project` is currently a privileged command with a fully commented-out/stubbed executor (`ScanResultsAddToProjectResponse::default()`), so add-to-project behavior is effectively incomplete.
- Element scanner results view contains `// JIRA: Double click logic.` for row double-click, so required direct add flow from double-click is not implemented.
- Project manifest already has sort order storage (`ProjectManifest.project_item_sort_order`) but currently exposes only a getter, with no mutation path for reorder persistence.

Information discovered during iteration:
- Added unprivileged `project-items add --scan-result-refs ...` command/response path and executor in `squalr-engine` to mutate opened projects without using privileged project mutation.
- `project-items add` resolves selected scan results via privileged `results refresh`, then creates address items and persists the project from unprivileged context.
- Removed privileged `scan-results add-to-project` command/response wiring and deleted the stubbed executor path in engine + API command surface.
- Updated GUI add-to-project flow in `ElementScannerResultsViewData` to use `ProjectItemsAddRequest`.
- Added/updated tests in `squalr-tests/tests/project_items_command_tests.rs` and `squalr-tests/tests/scan_results_command_tests.rs` to reflect the new routing.
- Expanded `project-items` API/executor surface with `create`, `delete`, `rename`, `move`, and `reorder` commands routed through unprivileged executors.
- New mutation executors (`create/delete/rename/move`) apply filesystem operations then reload the opened project from disk to keep in-memory project state consistent.
- Reorder now persists manifest sort order through `ProjectManifest::set_project_item_sort_order`, `ProjectInfo::get_project_manifest_mut`, and `project-items reorder` save path.
- Added parser/dispatch/response tests for the expanded project-items surface in `squalr-tests/tests/project_items_command_tests.rs`.
- Restored `project-items list` executor to return opened project info, root item, and all project items from unprivileged project manager state.
- Implemented project hierarchy GUI state + rendering using `project-items list`, including nested tree flattening, selection, directory expand/collapse, type-based icons, and address/pointer preview text.
- Implemented project hierarchy non-modal delete take-over flow with `ProjectHierarchyTakeOverState::DeleteConfirmation`, toolbar delete action, keyboard `Delete` shortcut, and wired confirmation to unprivileged `project-items delete`.
- Implemented element scanner result row double-click shortcut to add a single scan result to project via unprivileged `project-items add`.
- Added unit coverage in `ElementScannerResultsViewData` for single-index scan-result-ref collection used by double-click add flow.
- Strengthened project hierarchy sync checks to refresh when opened project item paths or manifest sort order drift from loaded hierarchy state (not just when opened project path changes).
- Implemented project hierarchy drag/drop reorder flow (same-parent reordering) with `project-items reorder` dispatch, in-panel drag target highlighting, and pending-operation handling for reordering.
- Added `project-items reorder` request dispatch tests in `squalr-tests` and unit tests for reorder manifest-path normalization in `squalr-engine`.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-tests --test project_items_command_tests` (18 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed).
- Session checkpoint (2026-02-16, follow-up): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (18 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); workspace remains clean and task is still awaiting next concrete `pr/project-explorer` requirement.
- Fixed UI deadlock on scan-result double-click add-to-project: `ProjectManager::notify_project_items_changed` now uses non-blocking `try_read()` on the opened-project lock instead of blocking `read()`, preventing self-deadlock when called while project mutation executors hold the write lock.
- Added regression test coverage in `squalr-engine-api` to verify `notify_project_items_changed` does not block while an opened-project write lock is held.
- Project hierarchy entries now render as single-line rows: name at left and preview at right, with address preview sourced from `freeze_data_value_interpreter` or `??` fallback.
- Project hierarchy display name now comes from the project item `name` property (`ProjectItem::get_field_name`) with filename fallback for empty values.
- Added activation checkboxes to all hierarchy rows and wired them to `project-items activate`; implemented recursive path-based activation in the unprivileged activate executor (folder toggles include descendants).
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-engine project_items_activate_request_executor` (2 passed), `cargo test -p squalr-tests --test project_items_command_tests` (18 passed), `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed), and `cargo test -p squalr --no-run` (build successful).
