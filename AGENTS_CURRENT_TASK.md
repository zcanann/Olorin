# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/api-contract`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- Treat `squalr-engine-api` as a stable contract crate, and move runtime/engine internals behind crate-private or engine-only modules.
- Define explicit public API boundaries: `commands`, `events`, and external-facing `structures` DTOs should remain public; scanner internals, registries internals, and vector-comparison implementation details should not.
- Decouple command DTO definitions from transport/runtime helper behavior (`send()`, binding locks, callback dispatch) so the same command shapes work for CLI/GUI/TUI/MCP and future non-session clients.
- Introduce a transition path for stateful session APIs to stateless APIs by adding explicit context handles in requests/responses before removing current session assumptions.
- Add contract tests that snapshot command/event/structure wire shapes and run as an API-semver guard.

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

- Move engine-coupled abstractions out of contract-facing types where possible (`Registries`, `ProjectItemType`, singleton registries, and scan internals).
- Separate `structopt` parsing concerns from API DTO definitions for request types.
- Design and implement a stateless command prototype for one vertical slice (process open/list/close) to validate the migration model.
- Add compatibility tests for serialization/deserialization of command/event payloads and typed response mapping.
- Run `cargo check` for affected workspace crates and update call sites incrementally.

## Important Information
Important information discovered during work about the current state of the task should be appended here.

Information found in initial audit:
- Current branch is `pr/api-contract`.
- `squalr-engine-api/src/lib.rs` exports broad top-level modules (`commands`, `conversions`, `dependency_injection`, `engine`, `events`, `registries`, `structures`, `traits`, `utils`) with many deeply nested `pub mod` paths.
- `squalr-engine-api` currently requires nightly (`#![feature(portable_simd)]` in `squalr-engine-api/src/lib.rs`), which makes the public contract crate nightly-bound.
- Command request traits (`squalr-engine-api/src/commands/privileged_command_request.rs`, `squalr-engine-api/src/commands/unprivileged_command_request.rs`) include transport execution helpers that require `EngineUnprivilegedState`, coupling DTOs to session/runtime state.
- API-exposed structures include engine-coupled internals:
- `squalr-engine-api/src/structures/projects/project_items/project_item_type.rs` depends on engine bindings and `Registries`.
- `squalr-engine-api/src/structures/snapshots/snapshot_region.rs` has public mutable fields and raw-pointer helpers; comment explicitly marks this as temporary.
- Singleton registries (`SymbolRegistry`, `ElementScanRuleRegistry`) expose unsafe global access (`unwrap_unchecked`) and have deprecation JIRA comments indicating architectural mismatch for mirrored/non-standalone scenarios.
- Naming typo exists in public module path: `engine_api_priviliged_bindings` (used across engine + api).

Information discovered during iteration:
- `cargo check -p squalr-engine-api` passes but emits multiple warnings including TODO/JIRA placeholders and unfinished paths in API-exposed code.
- `cargo check -p squalr-cli` passes, confirming current workspace remains buildable while contract issues are primarily architectural/boundary related.
- Added `squalr-engine-api::engine::engine_api_privileged_bindings` as the corrected module and retained `engine_api_priviliged_bindings` as a deprecation shim re-export for compatibility.
- Updated `squalr-engine-api` and `squalr-engine` call sites to import from `engine_api_privileged_bindings`.
- `cargo check -p squalr-engine-api` and `cargo check -p squalr-engine` both pass after the migration (warnings unchanged and pre-existing).
- Added compatibility-safe API namespace exports in `squalr-engine-api` via `api::commands`, `api::events`, and `api::types` while preserving existing top-level module exports for current callers.
- `cargo fmt`, `cargo check -p squalr-engine-api`, and `cargo check -p squalr-engine` pass after the namespace addition (existing warnings remain pre-existing).
- Added `squalr-engine-api/API_SURFACE_INVENTORY.md` to classify crate-level exports into `public contract`, `transitional`, and `internal`.
- Marked internal crate-level modules in `squalr-engine-api/src/lib.rs` as `#[doc(hidden)]` and documented `api` as the preferred semver-sensitive namespace.
- `cargo fmt`, `cargo check -p squalr-engine-api`, and `cargo check -p squalr-engine` pass after the classification/boundary-hardening pass (pre-existing warnings remain).

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows:

- Start with boundary map, then perform low-risk namespace hardening (re-export/publish surface), then migrate internals.
- Preserve backward compatibility with deprecation shims where possible before hard removals.

### Concise Session Log
Append logs for each session here. Compact redundency occasionally:
- Audited README + API contract task scope, scanned `squalr-engine-api` public surface, validated compile health with `cargo check`, and produced staged plan for boundary hardening + stateless API migration.
- Implemented `engine_api_priviliged_bindings` -> `engine_api_privileged_bindings` migration with a deprecated compatibility shim, updated engine/api imports, ran `cargo fmt`, then verified with `cargo check -p squalr-engine-api` and `cargo check -p squalr-engine`.
- Implemented a new `squalr-engine-api::api` namespace with `commands`, `events`, and `types` re-export modules, formatted, and re-validated API/engine crates with `cargo check`.
- Added API surface inventory documentation, marked non-contract root modules as `#[doc(hidden)]`, documented `api` as preferred contract entrypoint, and re-validated with `cargo check -p squalr-engine-api` + `cargo check -p squalr-engine`.
