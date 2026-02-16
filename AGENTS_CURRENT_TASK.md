# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/project-explorer`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- No remaining concrete `pr/project-explorer` regressions currently queued.

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
- `project-items add` now accepts optional target directory routing (`--target-directory-path`) and defaults to hidden root directory `project/` when no directory is selected.
- Element scanner add flows (toolbar add-selection + row double-click) now resolve selected directory from `ProjectHierarchyViewData`; selected files route adds to their parent directory.
- Project hierarchy now treats hidden root `project/` as the hierarchy root when present, so the root storage directory itself is not rendered as a visible folder entry.
- Project serialization now uses hidden root item path `project/` on load, and project creation initializes root ref at `project/` instead of empty path.
- Selecting project hierarchy entries now focuses struct viewer on project item properties; editing fields persists to project item files and refreshes hierarchy.
- Address project-item struct edit of the `address` field now dispatches privileged `memory write` with edited bytes to the edited address.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-engine project_items_add_request_executor` (3 passed).
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr --no-run` (build successful).
- Audited project-item edit memory-write trigger path in `ProjectHierarchyView::build_memory_write_request_for_project_item_edit`: memory writes are gated to `address` project-item type and only when editing field `address`.
- Added unit coverage in `squalr/src/views/project_explorer/project_hierarchy/project_hierarchy_view.rs` for memory-write request gating: positive case for address-field edits on address items and negative cases for non-address field edits and non-address item types.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr build_memory_write_request_for_ -- --nocapture` (3 passed), `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed).
- Project hierarchy now renders the root directory as a visible row (depth 0), keeping children one level indented for drag/drop target clarity.
- Added per-item project hierarchy context menus with `New Folder` and `Delete` actions.
- `New Folder` is available on every project-item row; directory targets create nested folders, non-directory targets create sibling folders under the selected item's parent.
- Added spacebar keyboard toggle support for selected hierarchy item activation state.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed), and `cargo test -p squalr --no-run` (build successful).
- Implemented project hierarchy multi-select with Ctrl/Cmd toggle and Shift range selection, including bulk Delete (toolbar/context/keyboard), bulk Space activation toggles, checkbox toggles over entire selection, and struct viewer focus/edit over all selected items.
- Project item struct edits now apply to all selected project items in one save cycle, with per-item optional memory-write dispatch preserved for address `address` field edits.
- Address project item creation now normalizes blank or placeholder (`TODO`) names to `New Address`.
- Project item string field getters now deserialize raw UTF-8 bytes directly for name/module/description/icon/symbolic-struct fields instead of relying on placeholder display-string paths.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr project_hierarchy_view_data` (5 passed), `cargo test -p squalr-engine-api project_item_type_address` (2 passed), `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed).
- Implemented concrete pointer project-item display-string field support in `ProjectItemTypePointer` via `freeze_data_value_interpreter` helpers and default-name normalization (`New Pointer`) for blank/placeholder names.
- Project hierarchy pointer rows now use pointer freeze-display interpreter text for right-column preview with `??` fallback, replacing the static `Pointer` placeholder preview.
- Added unit tests for pointer preview rendering in `ProjectHierarchyViewData` and pointer default-name normalization in `ProjectItemTypePointer`.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr project_hierarchy_view_data` (7 passed), `cargo test -p squalr-engine-api project_item_type_pointer` (2 passed), `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed).
- Session checkpoint (2026-02-16): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); no new concrete `pr/project-explorer` TODOs identified.
- Hidden project root directory constant is now `project_items/` (`Project::PROJECT_DIR`), with legacy read compatibility for `project/` (`Project::LEGACY_PROJECT_DIR`) in project deserialization and hierarchy root resolution.
- Project hierarchy root row now displays the opened project name (`ProjectInfo::get_name()`) instead of the hidden on-disk root directory name.
- Root project directory name edits are ignored in `ProjectHierarchyView::apply_project_item_edits`, preventing persistence of the synthetic root display label.
- New-folder creation now expands ancestor directories before refresh so newly created rows remain visible in hierarchy context.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-engine project_items_add_request_executor` (3 passed), `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), and `cargo test -p squalr project_hierarchy_view_data` (7 passed).
- Fixed address preview defaulting to `??`: `ProjectItemTypeAddress::new_project_item` now initializes `freeze_data_value_interpreter` from the provided freeze value via symbol-registry anonymization.
- Fixed project-item struct-viewer string field population: implemented `DataTypeStringUtf8::anonymize_value_bytes` for `String` format so UTF-8 properties (name/module/description/etc.) no longer render as empty values.
- Added regressions: address preview assertion in `ProjectHierarchyViewData`, UTF-8 struct-viewer edit-value assertion in `StructViewerViewData`, UTF-8 anonymization tests in `squalr-engine-domain`, and ancestor expansion test covering folder-create hierarchy stability.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-engine-domain data_type_string_utf8` (2 passed), `cargo test -p squalr-engine-api project_item_type_address` (3 passed), `cargo test -p squalr project_hierarchy_view_data` (9 passed), `cargo test -p squalr struct_viewer_view_data` (1 passed), `cargo test -p squalr-tests --test project_items_command_tests` (19 passed), and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed).
- Session checkpoint (2026-02-16, follow-up): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); repository is clean and no new concrete `pr/project-explorer` regressions were identified in targeted audit.
- Session checkpoint (2026-02-16, audit): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); targeted `pr/project-explorer` TODO/stub audit found no new concrete regressions to queue.
- Session checkpoint (2026-02-16, audit): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); focused `pr/project-explorer` TODO/JIRA review found only deferred/stubbed non-explorer architecture notes and no new concrete project-explorer regressions to queue.
- Session checkpoint (2026-02-16, follow-up): Re-ran `cargo test -p squalr-tests --test project_items_command_tests` (19 passed) and `cargo test -p squalr-tests --test scan_results_command_tests` (20 passed); tasklist remains unchanged with no concrete `pr/project-explorer` regressions currently queued.
