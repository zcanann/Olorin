# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/package-upgrades`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- Goal: upgrade all workspace crate dependencies in every `Cargo.toml` to latest available releases while preserving existing architecture boundaries (command/response split, engine/api layering, and platform crate separation).
- Strategy: perform upgrades in controlled waves (core shared crates first, then engine crates, then frontend/platform crates), validating compile/test health at each wave before moving on.
- Constraint handling: expect intentional breakages from major-version jumps and pinned `=` dependencies; handle with targeted refactors and keep behavior stable via existing integration tests.

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)
- [ ] Build dependency inventory and classify crates by upgrade risk (low/medium/high).
- [ ] Upgrade shared foundational dependencies used across many crates (serialization/logging/error/concurrency) and resolve breakages.
- [ ] Upgrade engine-side crates and APIs, then run targeted engine + command tests.
- [ ] Upgrade GUI/CLI/TUI/installer/android crate dependencies and resolve platform-specific breakages.
- [ ] Run full workspace validation (`cargo check`, relevant tests), remove dead code/imports, and prepare commit.

## Important Information
Important information discovered during work about the current state of the task should be appended here.
- Workspace size: 15 Rust crates (`cargo metadata --no-deps`) with no centralized `[workspace.dependencies]` block, so updates are currently per-crate.
- `cargo update --workspace --dry-run` reports `0 packages` updatable under current constraints and `194 unchanged dependencies behind latest`, indicating version requirement widening is required for full upgrades.
- Several crates use strict `=` pins (for example `bincode =1.3.3`, `sysinfo =0.34.2`, `rodio =0.20.1`, `slint =1.11.0`), which are likely high-friction upgrade points.
- `cargo outdated` is not installed in this environment; audit currently relies on `cargo metadata` + `cargo update --dry-run`.

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows:
- Keep package-upgrade implementation phased to avoid cross-crate breakage storms.
- Favor shared dependency alignment where possible to reduce duplicate transitive trees.

### Concise Session Log
Append logs for each session here. Compact redundency occasionally:
- 2026-02-08: Read `README.md`, `AGENTS.md`, and current task file; completed brief workspace dependency audit without applying upgrades; added phased upgrade plan and audit findings.
