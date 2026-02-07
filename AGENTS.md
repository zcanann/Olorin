# AGENTS.MD
Always scan the README.md file to get a good project overview.

You should look at the Agentic Current Task section to pick up on your previous work.

After each session, you should attempt to checkpoint your work with a commit, and fill out the relevant sections of Agentic Current Task.

If you have to modify source files to patch bugs, this is understandable as long as the README architecture is in-tact.

## Coding Conventions
- All variable names should be coherent. This means 'i' is forbidden. 'idx' is forbidden. In fact, even 'index' is often bad, because you should generally say what the index is for, ie 'snapshot_index'. You are a systems programmer, not an academic.
- No unhandled unwraps, panics, etc. On failure, either return a result, or log an error or warning with the log! macro.
- Comments end with a period.
- All code is to be formatted by the default rust formatter. This project already has a bundled .rustfmt.toml, so this should get picked up automatically by tooling.
- Commenting functions with intellisense friendly comments is preferred when possible.

## Agentic Current Task
We are working on a pr/unit-tests branch, to create squalr-tests as a project within this repository. The goal is to test the commands sent by the gui/cli. In other words, we are testing our command/response system, and ensuring the commands do what we expect them to.

The tricky part is doing dependency injection over the OS APIs to emulate what an OS might return. OS APIs are abstracted, so this is should be extensible.

This should cover various cases like scanning for different data types, writing memory, reading memory, querying memory ranges, etc...

If functionality is too hard to test, note down why its better to not have the test yet and instead wait for the implementation to change. Always cross-reference this with the Architecture Plan. If the plan seems too complicated, cut scope. If the tests can't comply due to poor architecture, just note it down and leave the test stubbed.

#### Scratchpad (Agents can modify this!)
If this starts to become sprawling, compact this.

- `squalr-tests` crate now exists in the workspace.
- Initial scope is command/response contract tests that do not require real OS process handles.
- Parser blocker resolved for current command set: duplicate aliases and duplicate short flags no longer panic `PrivilegedCommand::from_iter_safe(...)` in `squalr-tests` parser coverage.
- Parser coverage now includes `settings memory set` with long-form flags to guard against future clap metadata regressions in settings request definitions.
- Parser coverage now includes `scan pointer-scan` long-form flags and validates parsed field extraction for target address, pointer data type ref, max depth, and offset size.
- Parser coverage now includes `process list` long-form flags and validates parsed field extraction for require-windowed, search-name, match-case, limit, and fetch-icons.
- Parser coverage now includes `process open` long-form flags and validates parsed field extraction for process-id, search-name, and match-case.
- Parser coverage now includes `settings scan set` long-form flags and validates parsed field extraction for page size, memory alignment, read mode, floating-point tolerance, and single-threaded mode.
- Parser coverage now includes `settings general set` long-form flags and validates parsed field extraction for engine request delay.
- Parser coverage now includes `scan element-scan` long-form flags and validates parsed field extraction for vectorized scan constraints and data type refs.
- Parser coverage now includes `results list` long-form flags and validates parsed `page_index` field extraction.
- Parser coverage now includes `results set-property` long-form flags and validates parsed `scan_result_refs`, `anonymous_value_string`, and `field_namespace` extraction.
- Parser coverage now includes `results freeze` long-form flags and validates parsed `scan_result_refs` and `is_frozen` extraction.
- Parser coverage now includes `results query` and `results delete` long-form flags and validates parsed `page_index` and `scan_result_refs` extraction.
- Parser coverage now includes `results refresh` long-form flags and validates parsed `scan_result_refs` extraction.
- Parser coverage now includes `results add-to-project` long-form flags and validates parsed `scan_result_refs` extraction.
- Parser coverage now includes privileged `memory read`, `scan reset`, `scan collect-values`, and `tasks cancel` command shapes.
- Parser coverage now includes privileged `memory write`, `scan struct-scan`, `process close`, and settings `general|memory|scan list` command shapes, with field extraction checks for memory write and struct scan payloads.
- Parser coverage now includes unprivileged `project create`, `project rename`, and `project-items activate` long-form flags and validates field extraction.
- Parser coverage now includes unprivileged `project open`, `project delete`, `project export`, `project list`, `project close`, `project save`, and `project-items list` command shapes, with long-form field extraction checks where applicable.
- Contract coverage now includes unprivileged `project list` request dispatch through `send_unprivileged(...)`, with typed response decode and callback invocation verification via `EngineApiUnprivilegedBindings` mocks.
- Contract coverage now includes unprivileged `project open` request dispatch through `send_unprivileged(...)`, with typed response decode, callback invocation verification, and captured payload checks for file-browser toggle, project path, and project name.
- Contract coverage now includes unprivileged `project create` request dispatch through `send_unprivileged(...)`, with typed response decode verification for success and returned project path plus captured request payload checks.
- Contract coverage now includes unprivileged `project delete` request dispatch through `send_unprivileged(...)`, with typed response decode verification and captured request payload checks.
- Contract coverage now includes unprivileged `project rename` request dispatch through `send_unprivileged(...)`, with typed response decode verification for success and returned project path plus captured request payload checks.
- Contract coverage now includes unprivileged `project-items activate` request dispatch through `send_unprivileged(...)`, with typed response decode verification and captured activation payload checks.
- Contract coverage now includes unprivileged `project export`, `project close`, `project save`, and `project-items list` request dispatch through `send_unprivileged(...)`, with typed response decode verification and command payload propagation checks where applicable.
- Contract coverage now includes unprivileged typed-response mismatch handling for `project save` dispatch through `send_unprivileged(...)`, verifying callback suppression on wrong response variant while preserving command dispatch capture.
- Contract coverage now includes unprivileged typed-response mismatch handling for `project-items list` dispatch through `send_unprivileged(...)`, verifying callback suppression on wrong response variant while preserving command dispatch capture.
- Contract coverage now includes privileged `process open` request dispatch through `send_unprivileged(...)`, with typed response decode verification and captured payload checks for process id, search name, and match-case.
- Contract coverage now includes privileged `process close` request dispatch through `send_unprivileged(...)`, with typed response decode verification and callback suppression checks for wrong response variants.
- Contract coverage now includes privileged `process list` request dispatch through `send_unprivileged(...)`, with typed response decode verification for returned process metadata and captured payload checks for list filters.
- Contract coverage now includes privileged typed-response mismatch handling for `process list` dispatch through `send_unprivileged(...)`, verifying callback suppression on wrong response variant while preserving command dispatch capture.
- Contract coverage now includes privileged `tasks list` and `tasks cancel` request dispatch through `send_unprivileged(...)`, with typed response decode verification and captured cancel payload checks.
- Contract coverage now includes privileged typed-response mismatch handling for `tasks cancel` dispatch through `send_unprivileged(...)`, verifying callback suppression on wrong response variant while preserving command dispatch capture.
- Contract coverage now includes privileged `scan reset` and `scan collect-values` request dispatch through `send_unprivileged(...)`, with typed response decode verification and command dispatch capture checks.
- Contract coverage now includes privileged typed-response mismatch handling for `scan collect-values` dispatch through `send_unprivileged(...)`, verifying callback suppression on wrong response variants while preserving command dispatch capture.
- Contract coverage now includes privileged `memory read` and `memory write` request dispatch through `send_unprivileged(...)`, with typed response decode verification, request payload propagation checks, and wrong-variant callback suppression coverage.
- `squalr-tests` integration coverage is now split into per-command suites under `squalr-tests/tests/*_command_tests.rs` to avoid single-file test sprawl.

#### Architecture Plan (Agents can modify this!)
Iterate on this section with the architecture plan. Prefer simplicty, while staying within the bounds of the README.md plan. If this starts to become sprawling, compact it into the core skeleton of the intended architecture that is meant to guide all edits.

- Phase 1 (implemented): validate command request dispatch and typed response extraction through `EngineApiUnprivilegedBindings` mocks.
- Phase 1 (extended): validate unprivileged request dispatch and typed response extraction through `UnprivilegedCommandRequest::send_unprivileged(...)` mock bindings.
- Phase 1 (extended): expand unprivileged dispatch contract coverage beyond `project list` by validating typed callback decode and payload propagation for additional project commands.
- Phase 1 (extended): validate unprivileged dispatch contract coverage for `project create` and `project delete` command paths.
- Phase 1 (extended): validate unprivileged dispatch contract coverage for `project rename` and `project-items activate` command paths.
- Phase 1 (extended): validate unprivileged dispatch contract coverage for `project export`, `project close`, `project save`, and `project-items list` command paths.
- Phase 1 (extended): validate unprivileged typed-response mismatch behavior for `send_unprivileged(...)` request callbacks (wrong response variant should not invoke typed callback).
- Phase 1 (extended): validate unprivileged typed-response mismatch behavior for `project-items` request callbacks in `send_unprivileged(...)` coverage.
- Phase 1 (extended): validate privileged process-open dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode and request payload propagation.
- Phase 1 (extended): validate privileged process-close dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode and wrong-variant callback suppression.
- Phase 1 (extended): validate privileged process-list dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode and request payload propagation.
- Phase 1 (extended): validate privileged typed-response mismatch behavior for process-list request callbacks in `send_unprivileged(...)` coverage.
- Phase 1 (extended): validate privileged trackable-task list/cancel dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode and cancel payload propagation.
- Phase 1 (extended): validate privileged typed-response mismatch behavior for trackable-task cancel request callbacks in `send_unprivileged(...)` coverage.
- Phase 1 (extended): validate privileged scan reset/collect-values dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode and wrong-variant callback suppression for collect-values.
- Phase 1 (extended): validate privileged memory read/write dispatch contract coverage for `send_unprivileged(...)`, including typed callback decode, payload propagation, and wrong-variant callback suppression.
- Phase 1 (extended): add parser contract regression coverage for privileged command parsing to prevent clap construction regressions.
- Phase 1 (extended): add parser contract regression coverage for unprivileged project and project-item command parsing to prevent clap regressions outside privileged command trees.
- Phase 1 (extended): broaden unprivileged parser coverage to include all currently exposed `project` and `project-items` subcommands.
- Phase 1 (extended): reorganize `squalr-tests` integration coverage into per-command test suites for maintainability and ownership.
- Phase 2 (deferred): add OS-behavior tests for memory read/write, page query, and scan flows once OS query/reader/writer singletons support dependency injection overrides in test context.
- Scope cut rationale: privileged executors call static OS-facing singletons directly (`MemoryQueryer`, `MemoryReader`, `MemoryWriter`, `ProcessQuery`), so deterministic command executor tests cannot currently emulate OS data without architectural changes.
- Proposed minimal future seam: trait-object providers on `EnginePrivilegedState` for process/memory/query APIs, with production defaults bound to current implementations.

#### Concise Session logs (Agents can modify this!)
For each PR, append to this section a summary of the work accomplished. If this starts to become sprawling, compact this.
- `pr/unit-tests`: Added new workspace member `squalr-tests`.
- `pr/unit-tests`: Added initial tests in `squalr-tests/tests/command_response_tests.rs`
- NOTE FROM OWNER: This test format is unsustainable and retarded. Stop dumping everything in one file. One test suite per command.
- `pr/unit-tests`: Split `squalr-tests/tests/command_response_tests.rs` into command-specific suites (`memory`, `process`, `project`, `project-items`, `scan`, `scan-results`, `settings`, `trackable-tasks`) while preserving existing parser and dispatch contract coverage.
- `pr/unit-tests`: Added `project-items list` unprivileged typed-response mismatch contract coverage to assert callback suppression when the engine returns a different response variant while still capturing command dispatch.
- `pr/unit-tests`: Added privileged `process open` success-path contract coverage to assert typed callback decode and request payload propagation through `send_unprivileged(...)`.
- `pr/unit-tests`: Added privileged `process close` contract coverage to assert typed callback decode on success and callback suppression on wrong typed response variants through `send_unprivileged(...)`.
- `pr/unit-tests`: Added privileged `process list` contract coverage to assert typed callback decode for returned process metadata, request payload propagation, and callback suppression on wrong typed response variants through `send_unprivileged(...)`.
- `pr/unit-tests`: Added privileged `tasks list` and `tasks cancel` contract coverage to assert typed callback decode, cancel request payload propagation, and callback suppression on wrong typed response variants through `send_unprivileged(...)`.
- `pr/unit-tests`: Added privileged `scan reset` and `scan collect-values` contract coverage to assert typed callback decode, command dispatch propagation, and callback suppression on wrong typed response variants through `send_unprivileged(...)`.
- `pr/unit-tests`: Added privileged `memory read` and `memory write` contract coverage to assert typed callback decode, payload propagation, and callback suppression on wrong typed response variants through `send_unprivileged(...)`.

## Agentic Eventually TODO list
- pr/cli-bugs - The cli build currently does not even spawn a window. The cli should be able to spawn visibly and execute commands. It has not been functional for many months, causing drift. Observe the gui project (squalr) for reference to functional code. Both projects leverage squalr-engine / squalr-engine-api for the heavy lifting.
- pr/error_handling - We currently have a mixed use of Result<(), String>, anyhow! based errors, and error enums. Update the project to the following: Within squalr-engine, errors should be struct based. Within squalr-cli and squalr, anyhow! based errors are fine (ignore squalr-android and squalr-tui for now).


## Agentic Off Limits List
These are not ready to be picked up yet.
- pr/tui - We want a TUI project at some point. Would be good to get this working loosely based on both the cli and 
