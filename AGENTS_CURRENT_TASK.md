# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/installer`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- `squalr-installer` now boots directly with `eframe`, while installation logic remains in `squalr-engine::app_provisioner`.
- Legacy Slint installer assets and view-model glue have been deleted from `squalr-installer` after parity validation.

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

## Important Information
Important information discovered during work about the current state of the task should be appended here.

- We are to fix the installer build, while also migrating from Slint to egui.
- A perfect 1:1 visual migration is not necessary (ie we can use different button visuals), but the general structure should be respected.
- It is critical to use the squalr/ repo as a guide for how to handle egui rendering (ie `app.rs`, `main_window_view.rs`, `project_selector_view.rs`, etc.)
- The installer can be substantially more lightweight. We do not need docking, for instance.
- Legacy Slint assets were used as reference and then removed once parity/polish validation was completed.

## Agent Scratchpad and Notes
Smaller notes should go here, and can be erased and compacted as needed during iteration.

- It is currently unclear if it makes sense to attempt to share elements (buttons, title bar, footer) between the `squalr` project, and `squalr-installer`. The alternative is just to maintain a copy. This may be acceptable given that the number of total elements is small (header, footer, progress bar, buttons), however maintaining two themes can be tedius.

### Concise Session Log (append-or-compact-only, keep very short and compact as it grows)
- 2026-02-08: Replaced `squalr-installer` Slint startup with `eframe` UI (status text, progress bar, log panel, launch button); removed Slint compile step from `build.rs`; added log-buffer unit tests; `cargo test -p squalr-installer` passes.
- 2026-02-08: Validated installer UI structure parity against legacy Slint view and removed stale `squalr-installer/ui/slint` + `squalr-installer/src/view_models` glue.
