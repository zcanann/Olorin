# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/project-explorer`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- [x] Move project-item mutation flows to unprivileged command routing (implemented `project-items add` and moved scan-result add flow off privileged route).
- [x] Replace privileged `scan-results add-to-project` path with an unprivileged project-items add command, then remove the stubbed privileged executor path.
- [ ] Expand `project-items` command surface beyond `list`/`activate` to support create, delete, rename, move, and reorder operations needed by project explorer UX (currently expanded to include `add` only).
- [ ] Implement `ProjectHierarchyViewData` state model (loaded root item tree, selection, expanded directories, context/takeover state, pending operations).
- [ ] Implement `ProjectHierarchyView` rendering of nested project items (directory tree) backed by `project-items list`, including row selection and expansion.
- [ ] Render project item icons by type and add preview-value column behavior for address/pointer-style items.
- [ ] Implement non-modal delete confirmation flow in the project hierarchy panel (take-over panel content, not popup modal) and wire to delete command.
- [ ] Implement drag/drop reordering in project hierarchy and persist ordering metadata.
- [ ] Implement sort-order persistence updates in project metadata (manifest and/or per-folder metadata), including API setters and save/load consistency.
- [ ] Wire project hierarchy refresh to project/project-item change events and command callbacks so UI stays in sync after mutations.
- [ ] Implement scan-result to project shortcuts: double-click result to add single entry, plus ensure selected-range add works through the new unprivileged add path.
- [ ] Add/extend tests for new project-items commands, add-to-project flow, and sort-order persistence behavior.

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
