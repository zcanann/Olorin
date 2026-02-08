# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/nightly-upgrade`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- Restore SIMD scan compilation on latest nightly by removing obsolete lane-bound API usage (`LaneCount`/`SupportedLaneCount`) and preserving current scanner behavior/performance.
- Keep SIMD abstraction centralized in existing generic/scanning helper modules to avoid scattering nightly-specific compatibility logic.
- Validate fixes in `squalr-engine-api` first (the root blocker), then re-run workspace checks to surface and handle downstream regressions in dependency order.

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

- [ ] Replace `std::simd::{LaneCount, SupportedLaneCount}` imports/usages across the 12 affected files in `squalr-engine-api` with current nightly-compatible constraints/types.
- [ ] Refactor generic trait bounds and impl targets that currently depend on `LaneCount<N>` (notably vector comparer/function dispatch) to bind directly on usable SIMD generics.
- [ ] Rebuild `squalr-engine-api` (`cargo check -p squalr-engine-api`) and resolve any second-order type errors caused by bound refactors.
- [ ] Re-run `cargo check --workspace` and capture any additional nightly regressions beyond the initial SIMD lane-count break.
- [ ] Add/adjust targeted tests for SIMD comparison path coverage if behavior-affecting refactors are required.

## Important Information
Important information discovered during work about the current state of the task should be appended here.

INITIAL AUDIT:
- Audit date: 2026-02-08.
- Toolchain under test: `rustc 1.95.0-nightly (c7f5f3e0d 2026-02-07)` on `x86_64-pc-windows-msvc`.
- Repro command: `cargo check --workspace` and `cargo check -p squalr-engine-api`.
- Current hard blocker: 12 compile errors (`E0432`) in `squalr-engine-api` due to unresolved `std::simd::LaneCount` and `std::simd::SupportedLaneCount`.
- Affected files:
  - `squalr-engine-api/src/registries/symbols/symbol_registry.rs`
  - `squalr-engine-api/src/structures/data_types/comparisons/vector_comparisons_byte_array.rs`
  - `squalr-engine-api/src/structures/data_types/comparisons/vector_comparisons_float.rs`
  - `squalr-engine-api/src/structures/data_types/comparisons/vector_comparisons_float_big_endian.rs`
  - `squalr-engine-api/src/structures/data_types/comparisons/vector_comparisons_integer.rs`
  - `squalr-engine-api/src/structures/data_types/comparisons/vector_comparisons_integer_big_endian.rs`
  - `squalr-engine-api/src/structures/data_types/generics/vector_comparer.rs`
  - `squalr-engine-api/src/structures/data_types/generics/vector_function.rs`
  - `squalr-engine-api/src/structures/data_types/generics/vector_generics.rs`
  - `squalr-engine-api/src/structures/scanning/comparisons/scan_function_vector.rs`
  - `squalr-engine-api/src/structures/scanning/constraints/scan_constraint_finalized.rs`
  - `squalr-engine-api/src/structures/scanning/plans/element_scan/snapshot_filter_element_scan_plan.rs`
- Quick compiler probes confirm `std::simd::Simd<T, N>` compiles on current nightly without lane-count bounds, indicating obsolete constraint plumbing rather than import-path-only breakage.

Information discovered during iteration:
- 

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows:

- First unblock `squalr-engine-api`; entire workspace build is blocked there.
- Use a phased refactor: generic helpers first, then scanner/registry callsites.
- Expect cascading trait-bound mismatches immediately after lane-bound removal; handle centrally.

### Concise Session Log
Append logs for each session here. Compact redundency occasionally:
- 2026-02-08: Audited nightly-upgrade breakages. `cargo check --workspace` and `cargo check -p squalr-engine-api` fail with 12 `E0432` errors caused by removed `std::simd::LaneCount`/`SupportedLaneCount`. Captured affected files and staged a remediation plan; no implementation changes yet.
