# squalr-engine-api Public Surface Inventory

This file classifies the current crate-level public modules for `pr/api-contract`.

## Public Contract
- `api` (`api::commands`, `api::events`, `api::types`) as the stable semver target.

## Transitional (Compatibility)
- `commands` (legacy top-level command module path).
- `events` (legacy top-level event module path).
- `structures` (legacy top-level types module path).

## Internal (Not Intended as External Contract)
- `conversions`.
- `dependency_injection`.
- `engine`.
- `registries`.
- `traits`.
- `utils`.

## Notes
- Internal modules remain `pub` today to avoid immediate breakage while migration is in progress.
- Internal modules are marked `#[doc(hidden)]` to discourage new external usage.
- Future steps can move internal modules to crate-private visibility once call sites are migrated.
- `api::types::projects` is now narrowed to contract-safe project DTO modules; engine-coupled project item internals remain available only via `api::types::projects_legacy`.
- `api::types::scanning` now exposes only contract-safe scanning DTO paths (`comparisons`, `constraints`, `memory_read_mode`, `plans`); scanner internals remain available only via `api::types::scanning_legacy`.
- `api::types::snapshots` now exposes only `snapshot`; `snapshot_region` remains available only via `api::types::snapshots_legacy`.
- `api::commands::stateless` now includes `process`, `trackable_tasks`, `memory`, `project`, `project_items`, `scan`, `scan_results`, and `settings` command DTO contracts intentionally free of CLI parsing derives.
- `api::commands::stateless::project_items::ProjectItemsListResponse` now uses contract-safe `ProjectItemRef` (`opened_project_root_ref`) instead of engine-coupled `ProjectItem`.
- `api::commands` runtime-coupled command helper modules (`{un,}privileged_command{,_request,_response}`) remain available only as doc-hidden compatibility exports.
