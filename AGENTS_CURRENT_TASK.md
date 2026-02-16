# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- 

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- 

Information discovered during iteration:
- Removed legacy project-root compatibility paths and now always resolve project roots via `Project::PROJECT_DIR` (`project_items/`) in project deserialization and project hierarchy root resolution.
- Session checkpoint (2026-02-16): Ran `cargo test -p squalr-engine-projects is_project_item_file_path` (2 passed), `cargo test -p squalr project_hierarchy_view_data` (11 passed), and `cargo test -p squalr-tests --test project_items_command_tests` (19 passed).
