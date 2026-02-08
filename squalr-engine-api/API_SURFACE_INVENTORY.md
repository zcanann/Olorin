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
