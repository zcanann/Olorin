# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/installer`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- `squalr-installer` now boots directly with `eframe`, while installation logic remains in `squalr-engine::app_provisioner`.
- Legacy Slint installer assets and view-model glue have been deleted from `squalr-installer` after parity validation.
- The architecture should mirror `squalr` for gui creation / rendering as best as possible, where it makes sense to do so.

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

- No remaining actionable items recorded for `pr/installer`; tasklist is currently clear after transparency fix validation.

## Important Information
Important information discovered during work about the current state of the task should be appended here.

- We are to fix the installer build, while also migrating from Slint to egui.
- A perfect 1:1 visual migration is not necessary (ie we can use different button visuals), but the general structure should be respected.
- It is critical to use the squalr/ repo as a guide for how to handle egui rendering (ie `app.rs`, `main_window_view.rs`, `project_selector_view.rs`, etc.)
- The installer can be substantially more lightweight. We do not need docking, for instance.
- Legacy Slint assets were used as reference and then removed once parity/polish validation was completed.
- Installer now uses a dedicated egui theme preset aligned to `squalr` colors and panel structure.
- Installer visuals were re-audited against `squalr` theme/layout and are currently aligned; repeat this audit after future installer UI feature additions.
- Installer now uses `squalr` app icons for both runtime viewport icon (`IconData`) and Windows executable resources.
- Installer header now renders a centered single-line status message; log panel now fills available width before first log line.
- Installer now uses a custom themed title bar (`with_decorations(false)`) with app icon + explicit minimize/maximize/close controls, reduced inner padding, no rounded inner status/log cards, larger footer text, and output-view-style level-colored log text on dark background.
- Installer title/footer rendering now directly mirrors `squalr` main title/footer layout patterns (painted rounded bars, drag zone, icon-based window controls), with `squalr` font assets (`NotoSans`, `UbuntuMonoBold`) applied for title/header/footer/log typography.
- Installer bootstrap is now modularized into `src/app.rs`, `src/theme.rs`, `src/ui_assets.rs`, `src/ui_state.rs`, `src/installer_runtime.rs`, and `src/logging.rs`; `main.rs` is now bootstrap-only.
- Installer rendering now mirrors `squalr`'s distinct control layout with `src/views/main_window/installer_main_window_view.rs`, `src/views/main_window/installer_title_bar_view.rs`, and `src/views/main_window/installer_footer_view.rs`; `app.rs` now orchestrates state + repaint only.
- Installer transparency now matches `squalr` behavior via `ViewportBuilder::with_transparent(true)` and transparent app clear color.
- Installer log-buffer unit tests are no longer in `main.rs`; they now live in `src/logging.rs`.
- Installer repaint flow is now event-driven (log/progress updates call `Context::request_repaint`) instead of a fixed `request_repaint_after` polling loop, matching `squalr`'s render/update pattern more closely.
- Installer title-bar window controls now use a dedicated custom widget (`src/views/main_window/title_bar_button.rs`) with explicit hover/pressed visuals instead of raw `egui::Button`, keeping parity with `squalr` title-bar behavior.
- Installer log panel rendering is now modularized into `src/views/main_window/installer_log_view.rs`, and `installer_main_window_view.rs` now composes it.
- Installer title-bar button hover/pressed visuals now use `squalr`-matching tint overlays (white/black alpha) instead of blue fills/borders.
- Installer theme visuals are now reapplied every frame in `src/app.rs::update` so integration/native theme changes cannot revert panel visuals to default light/white.
- Root cause of the remaining white-background/transparency mismatch was missing `squalr`-style rounded outer app frame wiring in installer `App::update`; installer now applies a rounded `CentralPanel` frame (`corner_radius` + border stroke + outer margin) so transparent viewport corners are exposed correctly around rounded title/footer bars.

## Agent Scratchpad and Notes 
Append below and compact regularly to relevant recent, keep under ~20 lines and discard useless information as it grows.

### Concise Session Log
- Slint → `eframe/egui`; removed Slint assets/build step; tests moved to `logging.rs`.
- UI/theme parity to `squalr` (colors/layout + dark level-colored log).
- Title/footer parity (fonts + icons + custom title-bar buttons); correct app icon (runtime + Windows resources).
- Refactor: app/theme/assets/state/runtime/logging + `views/main_window/*`; `main.rs` bootstrap-only.
- Fixed transparency (transparent viewport + clear color) + switched to event-driven repaint.
- Extracted installer log rendering into dedicated `installer_log_view.rs`.
- Updated title-bar button hover/pressed states to match `squalr` tint overlays.
- `cargo fmt --all` + `cargo test -p squalr-installer` pass; workspace still has pre-existing non-installer warnings + rustfmt `fn_args_layout` deprecation.
- Re-verified on 2026-02-08: `cargo fmt --all -- --check` and `cargo test -p squalr-installer` both pass; non-installer warnings remain unchanged.
- Fixed recurring white installer background by reapplying installer visuals in every frame update, preventing fallback to default light panel visuals.
- Checkpointed on 2026-02-08: branch remains clean for `pr/installer`; `cargo fmt --all -- --check` and `cargo test -p squalr-installer` pass with only pre-existing non-installer warnings.
- Audited on 2026-02-08: no `TODO`/`FIXME`, `panic!`, or `unwrap()` usages remain under `squalr-installer`; `cargo fmt --all -- --check` and `cargo test -p squalr-installer` still pass with unchanged non-installer warnings.
- Fixed on 2026-02-08: installer now mirrors `squalr` outer app frame composition in `squalr-installer/src/app.rs` (rounded central frame + border + margin), resolving white corner background and restoring true transparent viewport corners with rounded title/footer visuals.
