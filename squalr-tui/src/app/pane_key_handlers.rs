use super::app_shell::AppShell;
use crate::state::pane::TuiPane;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectSelectorInputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use squalr_engine::squalr_engine::SqualrEngine;

impl AppShell {
    pub(super) fn handle_focused_pane_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.focused_pane() {
            TuiPane::ProcessSelector => self.handle_process_selector_key_event(key_event.code, squalr_engine),
            TuiPane::ElementScanner => self.handle_element_scanner_key_event(key_event, squalr_engine),
            TuiPane::ScanResults => self.handle_scan_results_key_event(key_event, squalr_engine),
            TuiPane::ProjectExplorer => self.handle_project_explorer_key_event(key_event, squalr_engine),
            TuiPane::StructViewer => self.handle_struct_viewer_key_event(key_event, squalr_engine),
            TuiPane::Output => self.handle_output_key_event(key_event.code, squalr_engine),
            TuiPane::Settings => self.handle_settings_key_event(key_event.code, squalr_engine),
        }
    }

    pub(super) fn handle_output_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_output_log_history_with_feedback(squalr_engine, true),
            KeyCode::Char('x') | KeyCode::Delete => self.app_state.output_pane_state.clear_log_lines(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.app_state.output_pane_state.increase_max_line_count(),
            KeyCode::Char('-') => self.app_state.output_pane_state.decrease_max_line_count(),
            _ => {}
        }
    }

    pub(super) fn handle_settings_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_all_settings_categories_with_feedback(squalr_engine, true),
            KeyCode::Char(']') => self.app_state.settings_pane_state.cycle_category_forward(),
            KeyCode::Char('[') => self.app_state.settings_pane_state.cycle_category_backward(),
            KeyCode::Down | KeyCode::Char('j') => self.app_state.settings_pane_state.select_next_field(),
            KeyCode::Up | KeyCode::Char('k') => self.app_state.settings_pane_state.select_previous_field(),
            KeyCode::Char(' ') => {
                if self
                    .app_state
                    .settings_pane_state
                    .toggle_selected_boolean_field()
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self
                    .app_state
                    .settings_pane_state
                    .step_selected_numeric_field(true)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('-') => {
                if self
                    .app_state
                    .settings_pane_state
                    .step_selected_numeric_field(false)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('>') | KeyCode::Char('.') => {
                if self
                    .app_state
                    .settings_pane_state
                    .cycle_selected_enum_field(true)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('<') | KeyCode::Char(',') => {
                if self
                    .app_state
                    .settings_pane_state
                    .cycle_selected_enum_field(false)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Enter => self.apply_selected_settings_category(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_process_selector_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_process_list(squalr_engine),
            KeyCode::Char('w') => {
                let updated_windowed_filter = !self
                    .app_state
                    .process_selector_pane_state
                    .show_windowed_processes_only;
                self.app_state
                    .process_selector_pane_state
                    .set_windowed_filter(updated_windowed_filter);
                self.refresh_process_list(squalr_engine);
            }
            KeyCode::Down | KeyCode::Char('j') => self.app_state.process_selector_pane_state.select_next_process(),
            KeyCode::Up | KeyCode::Char('k') => self
                .app_state
                .process_selector_pane_state
                .select_previous_process(),
            KeyCode::Enter | KeyCode::Char('o') => self.open_selected_process(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_element_scanner_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_event.code {
            KeyCode::Char('s') => self.start_element_scan(squalr_engine),
            KeyCode::Char('n') => self.reset_scan_state(squalr_engine),
            KeyCode::Char('c') => self.collect_scan_values(squalr_engine),
            KeyCode::Char('t') => self
                .app_state
                .element_scanner_pane_state
                .cycle_data_type_forward(),
            KeyCode::Char('T') => self
                .app_state
                .element_scanner_pane_state
                .cycle_data_type_backward(),
            KeyCode::Char('j') | KeyCode::Down => self
                .app_state
                .element_scanner_pane_state
                .select_next_constraint(),
            KeyCode::Char('k') | KeyCode::Up => self
                .app_state
                .element_scanner_pane_state
                .select_previous_constraint(),
            KeyCode::Char('m') => self
                .app_state
                .element_scanner_pane_state
                .cycle_selected_constraint_compare_type_forward(),
            KeyCode::Char('M') => self
                .app_state
                .element_scanner_pane_state
                .cycle_selected_constraint_compare_type_backward(),
            KeyCode::Char('a') => {
                if !self.app_state.element_scanner_pane_state.add_constraint() {
                    self.app_state.element_scanner_pane_state.status_message = "Maximum of five constraints reached.".to_string();
                }
            }
            KeyCode::Char('x') => {
                if !self
                    .app_state
                    .element_scanner_pane_state
                    .remove_selected_constraint()
                {
                    self.app_state.element_scanner_pane_state.status_message = "At least one constraint is required.".to_string();
                }
            }
            KeyCode::Backspace => self
                .app_state
                .element_scanner_pane_state
                .backspace_selected_constraint_value(),
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app_state
                    .element_scanner_pane_state
                    .clear_selected_constraint_value();
            }
            KeyCode::Char(scan_value_character) => {
                self.app_state
                    .element_scanner_pane_state
                    .append_selected_constraint_value_character(scan_value_character);
            }
            _ => {}
        }
    }

    pub(super) fn handle_scan_results_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        let is_range_extend_modifier_active = key_event.modifiers.contains(KeyModifiers::SHIFT);
        let mut should_refresh_struct_viewer_focus = false;

        match key_event.code {
            KeyCode::Char('r') => {
                self.query_scan_results_current_page(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('R') => {
                self.refresh_scan_results_page(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char(']') => {
                self.query_next_scan_results_page(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('[') => {
                self.query_previous_scan_results_page(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_next_result(is_range_extend_modifier_active);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_previous_result(is_range_extend_modifier_active);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Home => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_first_result(is_range_extend_modifier_active);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::End => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_last_result(is_range_extend_modifier_active);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('f') => {
                self.toggle_selected_scan_results_frozen_state(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('a') => {
                self.add_selected_scan_results_to_project(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                self.delete_selected_scan_results(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Enter => {
                self.commit_selected_scan_results_value_edit(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Backspace => self
                .app_state
                .scan_results_pane_state
                .backspace_pending_value_edit(),
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app_state
                    .scan_results_pane_state
                    .clear_pending_value_edit();
            }
            KeyCode::Char('y') => self
                .app_state
                .scan_results_pane_state
                .sync_pending_value_edit_from_selection(),
            KeyCode::Char(scan_value_character) => self
                .app_state
                .scan_results_pane_state
                .append_pending_value_edit_character(scan_value_character),
            _ => {}
        }

        if should_refresh_struct_viewer_focus {
            self.sync_struct_viewer_focus_from_scan_results();
        }
    }

    pub(super) fn handle_project_explorer_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.input_mode != ProjectSelectorInputMode::None {
            match key_event.code {
                KeyCode::Esc => self
                    .app_state
                    .project_explorer_pane_state
                    .cancel_project_name_input(),
                KeyCode::Backspace => self
                    .app_state
                    .project_explorer_pane_state
                    .backspace_pending_project_name(),
                KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.app_state
                        .project_explorer_pane_state
                        .clear_pending_project_name();
                }
                KeyCode::Enter => self.commit_project_selector_input(squalr_engine),
                KeyCode::Char(project_name_character) => {
                    self.app_state
                        .project_explorer_pane_state
                        .append_pending_project_name_character(project_name_character);
                }
                _ => {}
            }

            return;
        }

        match key_event.code {
            KeyCode::Char('p') => self.app_state.project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectList,
            KeyCode::Char('i') => self.app_state.project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectHierarchy,
            _ => {}
        }

        match self.app_state.project_explorer_pane_state.focus_target {
            ProjectExplorerFocusTarget::ProjectList => self.handle_project_list_key_event(key_event.code, squalr_engine),
            ProjectExplorerFocusTarget::ProjectHierarchy => self.handle_project_hierarchy_key_event(key_event.code, squalr_engine),
        }
    }

    pub(super) fn handle_project_list_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_project_list(squalr_engine),
            KeyCode::Down | KeyCode::Char('j') => self.app_state.project_explorer_pane_state.select_next_project(),
            KeyCode::Up | KeyCode::Char('k') => self
                .app_state
                .project_explorer_pane_state
                .select_previous_project(),
            KeyCode::Enter | KeyCode::Char('o') => self.open_selected_project(squalr_engine),
            KeyCode::Char('n') => self
                .app_state
                .project_explorer_pane_state
                .begin_create_project_input(),
            KeyCode::Char('e') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .begin_rename_selected_project_input()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No project is selected for rename.".to_string();
                }
            }
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_project(squalr_engine),
            KeyCode::Char('c') => self.close_active_project(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_project_hierarchy_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        let mut should_refresh_struct_viewer_focus = false;

        match key_code {
            KeyCode::Char('h') => {
                self.refresh_project_items_list(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.app_state
                    .project_explorer_pane_state
                    .select_next_project_item();
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.app_state
                    .project_explorer_pane_state
                    .select_previous_project_item();
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .expand_selected_project_item_directory()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No expandable directory is selected.".to_string();
                }
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Left => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .collapse_selected_project_item_directory_or_select_parent()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No collapsible directory is selected.".to_string();
                }
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char(' ') => {
                self.toggle_selected_project_item_activation(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('n') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .begin_create_project_directory_input()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No project item directory target is selected.".to_string();
                }
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('m') => {
                if self
                    .app_state
                    .project_explorer_pane_state
                    .stage_selected_project_item_for_move()
                {
                    self.app_state.project_explorer_pane_state.status_message =
                        "Staged selected project item for move. Select destination and press b.".to_string();
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "No project item is selected for move.".to_string();
                }
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('b') => {
                self.move_staged_project_items_to_selected_directory(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('u') => {
                self.app_state
                    .project_explorer_pane_state
                    .clear_pending_move_source_paths();
                self.app_state.project_explorer_pane_state.status_message = "Cleared staged project item move.".to_string();
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('[') => {
                self.reorder_selected_project_item(squalr_engine, true);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char(']') => {
                self.reorder_selected_project_item(squalr_engine, false);
                should_refresh_struct_viewer_focus = true;
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                self.delete_selected_project_item_with_confirmation(squalr_engine);
                should_refresh_struct_viewer_focus = true;
            }
            _ => {}
        }

        if should_refresh_struct_viewer_focus {
            self.sync_struct_viewer_focus_from_project_items();
        }
    }

    pub(super) fn handle_struct_viewer_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        let apply_edit_input_guard = |struct_viewer_pane_state: &mut crate::views::struct_viewer_pane_state::StructViewerPaneState| -> bool {
            if let Some(block_reason) = struct_viewer_pane_state.selected_field_edit_block_reason() {
                struct_viewer_pane_state.status_message = block_reason;
                return false;
            }

            true
        };
        match key_event.code {
            KeyCode::Char('r') => self.refresh_struct_viewer_focus_from_source(),
            KeyCode::Down | KeyCode::Char('j') => self.app_state.struct_viewer_pane_state.select_next_field(),
            KeyCode::Up | KeyCode::Char('k') => self.app_state.struct_viewer_pane_state.select_previous_field(),
            KeyCode::Char('[') => {
                let selected_field_name = self
                    .app_state
                    .struct_viewer_pane_state
                    .selected_field_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                match self
                    .app_state
                    .struct_viewer_pane_state
                    .cycle_selected_field_display_format_backward()
                {
                    Ok(active_display_format) => {
                        self.app_state.struct_viewer_pane_state.status_message =
                            format!("Set display format for field '{}' to {}.", selected_field_name, active_display_format);
                    }
                    Err(error) => {
                        self.app_state.struct_viewer_pane_state.status_message = error;
                    }
                }
            }
            KeyCode::Char(']') => {
                let selected_field_name = self
                    .app_state
                    .struct_viewer_pane_state
                    .selected_field_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                match self
                    .app_state
                    .struct_viewer_pane_state
                    .cycle_selected_field_display_format_forward()
                {
                    Ok(active_display_format) => {
                        self.app_state.struct_viewer_pane_state.status_message =
                            format!("Set display format for field '{}' to {}.", selected_field_name, active_display_format);
                    }
                    Err(error) => {
                        self.app_state.struct_viewer_pane_state.status_message = error;
                    }
                }
            }
            KeyCode::Enter => self.commit_struct_viewer_field_edit(squalr_engine),
            KeyCode::Backspace => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state.struct_viewer_pane_state.backspace_pending_edit();
                }
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state.struct_viewer_pane_state.clear_pending_edit();
                }
            }
            KeyCode::Char(pending_character) => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state
                        .struct_viewer_pane_state
                        .append_pending_edit_character(pending_character);
                }
            }
            _ => {}
        }
    }
}
