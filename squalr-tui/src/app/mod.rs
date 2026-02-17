use crate::state::TuiAppState;
use crate::state::pane::TuiPane;
use crate::state::project_explorer_pane_state::{ProjectExplorerFocusTarget, ProjectSelectorInputMode};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::io::{self, Stdout};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub struct TerminalGuard {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();

        enable_raw_mode().context("Failed to enable terminal raw mode.")?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide).context("Failed to switch to alternate screen.")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to initialize terminal backend.")?;

        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen, cursor::Show);
        let _ = self.terminal.show_cursor();
    }
}

pub struct AppShell {
    pub should_exit: bool,
    pub tick_rate: Duration,
    pub last_tick_time: Instant,
    pub app_state: TuiAppState,
}

impl AppShell {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            should_exit: false,
            tick_rate,
            last_tick_time: Instant::now(),
            app_state: TuiAppState::default(),
        }
    }

    pub fn run(
        &mut self,
        terminal_guard: &mut TerminalGuard,
        engine_mode: EngineMode,
        squalr_engine: &mut SqualrEngine,
    ) -> Result<()> {
        while !self.should_exit {
            terminal_guard
                .terminal
                .draw(|frame| self.draw(frame, engine_mode))
                .context("Failed to draw TUI frame.")?;

            let timeout_duration = self.tick_rate.saturating_sub(self.last_tick_time.elapsed());
            if event::poll(timeout_duration).context("Failed while polling terminal events.")? {
                let incoming_event = event::read().context("Failed while reading terminal event.")?;
                self.handle_event(incoming_event, squalr_engine);
            }

            if self.last_tick_time.elapsed() >= self.tick_rate {
                self.on_tick(squalr_engine);
                self.last_tick_time = Instant::now();
            }
        }

        Ok(())
    }

    fn draw(
        &self,
        frame: &mut ratatui::Frame<'_>,
        engine_mode: EngineMode,
    ) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        let header_text = match engine_mode {
            EngineMode::Standalone => "Squalr TUI (Standalone)",
            EngineMode::UnprivilegedHost => "Squalr TUI (Unprivileged Host)",
            EngineMode::PrivilegedShell => "Squalr TUI (Privileged Shell)",
        };

        let header = Paragraph::new(vec![
            Line::from(header_text),
            Line::from("Focus: Tab / Shift+Tab. Focus pane: 1-7. Toggle pane: Ctrl+1-7 or v. Show all: 0."),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Session"));
        frame.render_widget(header, vertical_chunks[0]);

        self.draw_pane_layout(frame, vertical_chunks[1]);

        let footer = Paragraph::new(vec![Line::from(
            "Global: q / Esc / Ctrl+C to exit. Non-mouse workflow enabled for pane navigation and visibility.",
        )])
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(footer, vertical_chunks[2]);
    }

    fn handle_event(
        &mut self,
        incoming_event: Event,
        squalr_engine: &mut SqualrEngine,
    ) {
        if let Event::Key(key_event) = incoming_event {
            if key_event.kind != KeyEventKind::Press {
                return;
            }

            let mut was_handled_by_global_shortcut = true;
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => self.should_exit = true,
                KeyCode::Tab => self.app_state.cycle_focus_forward(),
                KeyCode::BackTab => self.app_state.cycle_focus_backward(),
                KeyCode::Char('v') => {
                    let _toggle_succeeded = self.app_state.toggle_focused_pane_visibility();
                }
                KeyCode::Char('0') => self.app_state.show_all_panes(),
                KeyCode::Char(shortcut_digit) => {
                    if let Some(target_pane) = TuiPane::from_shortcut_digit(shortcut_digit) {
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            let _toggle_succeeded = self.app_state.toggle_pane_visibility(target_pane);
                        } else {
                            self.app_state.set_focus_to_pane(target_pane);
                        }
                    }
                }
                _ => was_handled_by_global_shortcut = false,
            }

            if !was_handled_by_global_shortcut {
                self.handle_focused_pane_event(key_event, squalr_engine);
            }
        }
    }

    fn on_tick(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .process_selector_pane_state
            .process_list_entries
            .is_empty()
            && !self
                .app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response
        {
            self.refresh_process_list(squalr_engine);
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once
            && !self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_list_response
        {
            self.refresh_project_list(squalr_engine);
        }

        if self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            .is_some()
            && !self
                .app_state
                .project_explorer_pane_state
                .has_loaded_project_item_list_once
            && !self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_item_list_response
        {
            self.refresh_project_items_list(squalr_engine);
        }
    }

    fn handle_focused_pane_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.focused_pane() {
            TuiPane::ProcessSelector => self.handle_process_selector_key_event(key_event.code, squalr_engine),
            TuiPane::ElementScanner => self.handle_element_scanner_key_event(key_event, squalr_engine),
            TuiPane::ScanResults => self.handle_scan_results_key_event(key_event, squalr_engine),
            TuiPane::ProjectExplorer => self.handle_project_explorer_key_event(key_event, squalr_engine),
            _ => {}
        }
    }

    fn handle_process_selector_key_event(
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

    fn handle_element_scanner_key_event(
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

    fn handle_scan_results_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        let is_range_extend_modifier_active = key_event.modifiers.contains(KeyModifiers::SHIFT);

        match key_event.code {
            KeyCode::Char('r') => self.query_scan_results_current_page(squalr_engine),
            KeyCode::Char('R') => self.refresh_selected_scan_results(squalr_engine),
            KeyCode::Char(']') => self.query_next_scan_results_page(squalr_engine),
            KeyCode::Char('[') => self.query_previous_scan_results_page(squalr_engine),
            KeyCode::Down | KeyCode::Char('j') => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_next_result(is_range_extend_modifier_active);
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
            }
            KeyCode::Char('f') => self.toggle_selected_scan_results_frozen_state(squalr_engine),
            KeyCode::Char('a') => self.add_selected_scan_results_to_project(squalr_engine),
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_scan_results(squalr_engine),
            KeyCode::Enter => self.commit_selected_scan_results_value_edit(squalr_engine),
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
    }

    fn handle_project_explorer_key_event(
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

    fn handle_project_list_key_event(
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

    fn handle_project_hierarchy_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('h') => self.refresh_project_items_list(squalr_engine),
            KeyCode::Down | KeyCode::Char('j') => self
                .app_state
                .project_explorer_pane_state
                .select_next_project_item(),
            KeyCode::Up | KeyCode::Char('k') => self
                .app_state
                .project_explorer_pane_state
                .select_previous_project_item(),
            KeyCode::Right | KeyCode::Char('l') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .expand_selected_project_item_directory()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No expandable directory is selected.".to_string();
                }
            }
            KeyCode::Left => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .collapse_selected_project_item_directory_or_select_parent()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No collapsible directory is selected.".to_string();
                }
            }
            KeyCode::Char(' ') => self.toggle_selected_project_item_activation(squalr_engine),
            KeyCode::Char('n') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .begin_create_project_directory_input()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No project item directory target is selected.".to_string();
                }
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
            }
            KeyCode::Char('b') => self.move_staged_project_items_to_selected_directory(squalr_engine),
            KeyCode::Char('u') => {
                self.app_state
                    .project_explorer_pane_state
                    .clear_pending_move_source_paths();
                self.app_state.project_explorer_pane_state.status_message = "Cleared staged project item move.".to_string();
            }
            KeyCode::Char('[') => self.reorder_selected_project_item(squalr_engine, true),
            KeyCode::Char(']') => self.reorder_selected_project_item(squalr_engine, false),
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_project_item_with_confirmation(squalr_engine),
            _ => {}
        }
    }

    fn commit_project_selector_input(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.project_explorer_pane_state.input_mode {
            ProjectSelectorInputMode::CreatingProject => self.create_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::RenamingProject => self.rename_selected_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::CreatingProjectDirectory => self.create_project_directory_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::None => {}
        }
    }

    fn reset_scan_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for scan reset.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Resetting active scan.".to_string();

        let scan_reset_request = ScanResetRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_reset_request.send(engine_unprivileged_state, move |scan_reset_response| {
            let _ = response_sender.send(scan_reset_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan reset request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_reset_response) => {
                if scan_reset_response.success {
                    self.app_state.element_scanner_pane_state.has_scan_results = false;
                    self.app_state.element_scanner_pane_state.last_result_count = 0;
                    self.app_state
                        .element_scanner_pane_state
                        .last_total_size_in_bytes = 0;
                    self.app_state.scan_results_pane_state.clear_results();
                    self.app_state.element_scanner_pane_state.status_message = "Scan state reset.".to_string();
                } else {
                    self.app_state.element_scanner_pane_state.status_message = "Scan reset request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for scan reset response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    fn collect_scan_values(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for value collection.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Collecting scan values.".to_string();

        let scan_collect_values_request = ScanCollectValuesRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_collect_values_request.send(engine_unprivileged_state, move |scan_collect_values_response| {
            let _ = response_sender.send(scan_collect_values_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan collect values request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_collect_values_response) => {
                self.app_state.element_scanner_pane_state.last_result_count = scan_collect_values_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = scan_collect_values_response
                    .scan_results_metadata
                    .total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message = format!(
                    "Collected values for {} results.",
                    scan_collect_values_response.scan_results_metadata.result_count
                );
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for collect values response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    fn start_element_scan(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for element scanning.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Starting scan.".to_string();

        if !self.app_state.element_scanner_pane_state.has_scan_results {
            let scan_new_request = ScanNewRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_new_request.send(engine_unprivileged_state, move |scan_new_response| {
                let _ = response_sender.send(scan_new_response);
            });

            if !request_dispatched {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch new scan request.".to_string();
                return;
            }

            if let Err(receive_error) = response_receiver.recv_timeout(Duration::from_secs(3)) {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for new scan response: {}", receive_error);
                return;
            }
        }

        let element_scan_request = ElementScanRequest {
            scan_constraints: self
                .app_state
                .element_scanner_pane_state
                .build_anonymous_scan_constraints(),
            data_type_refs: vec![
                self.app_state
                    .element_scanner_pane_state
                    .selected_data_type_ref(),
            ],
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = element_scan_request.send(engine_unprivileged_state, move |element_scan_response| {
            let _ = response_sender.send(element_scan_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch element scan request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(element_scan_response) => {
                self.app_state.element_scanner_pane_state.has_scan_results = true;
                self.app_state.element_scanner_pane_state.last_result_count = element_scan_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = element_scan_response.scan_results_metadata.total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message =
                    format!("Scan complete with {} results.", element_scan_response.scan_results_metadata.result_count);
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for element scan response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    fn query_scan_results_current_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_querying_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Scan results query already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results query.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_querying_scan_results = true;
        self.app_state.scan_results_pane_state.status_message =
            format!("Querying scan results page {}.", self.app_state.scan_results_pane_state.current_page_index);

        let page_index = self.app_state.scan_results_pane_state.current_page_index;
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_query_request.send(engine_unprivileged_state, move |scan_results_query_response| {
            let _ = response_sender.send(scan_results_query_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_querying_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results query request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_query_response) => {
                self.apply_scan_results_query_response(scan_results_query_response);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results query response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_querying_scan_results = false;
    }

    fn query_next_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_add(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    fn query_previous_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_sub(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    fn refresh_selected_scan_results(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .scan_results_pane_state
            .is_refreshing_scan_results
        {
            self.app_state.scan_results_pane_state.status_message = "Scan results refresh already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to refresh.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results refresh.".to_string();
                return;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = format!("Refreshing {} selected scan results.", selected_scan_result_refs.len());

        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: selected_scan_result_refs,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_refresh_request.send(engine_unprivileged_state, move |scan_results_refresh_response| {
            let _ = response_sender.send(scan_results_refresh_response);
        });

        if !request_dispatched {
            self.app_state
                .scan_results_pane_state
                .is_refreshing_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results refresh request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_refresh_response) => {
                let refreshed_result_count = scan_results_refresh_response.scan_results.len();
                self.app_state
                    .scan_results_pane_state
                    .apply_refreshed_results(scan_results_refresh_response.scan_results);
                self.app_state.scan_results_pane_state.status_message = format!("Refreshed {} scan results.", refreshed_result_count);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results refresh response: {}", receive_error);
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = false;
    }

    fn toggle_selected_scan_results_frozen_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_freezing_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Scan results freeze request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to freeze/unfreeze.".to_string();
            return;
        }

        let target_frozen_state = !self
            .app_state
            .scan_results_pane_state
            .any_selected_result_frozen();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for freeze toggles.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_freezing_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = if target_frozen_state {
            "Freezing selected scan results.".to_string()
        } else {
            "Unfreezing selected scan results.".to_string()
        };

        let scan_results_freeze_request = ScanResultsFreezeRequest {
            scan_result_refs: selected_scan_result_refs,
            is_frozen: target_frozen_state,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
            let _ = response_sender.send(scan_results_freeze_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results freeze request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_freeze_response) => {
                let failed_toggle_count = scan_results_freeze_response
                    .failed_freeze_toggle_scan_result_refs
                    .len();
                self.app_state.scan_results_pane_state.status_message = if failed_toggle_count == 0 {
                    if target_frozen_state {
                        "Selected scan results frozen.".to_string()
                    } else {
                        "Selected scan results unfrozen.".to_string()
                    }
                } else {
                    format!("Freeze toggle partially failed for {} entries.", failed_toggle_count)
                };
                self.refresh_selected_scan_results(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results freeze response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
    }

    fn add_selected_scan_results_to_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project
        {
            self.app_state.scan_results_pane_state.status_message = "Add to project request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to add to project.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for project item creation.".to_string();
                return;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = true;
        self.app_state.scan_results_pane_state.status_message = format!("Adding {} scan results to project.", selected_scan_result_refs.len());

        let project_items_add_request = ProjectItemsAddRequest {
            scan_result_refs: selected_scan_result_refs,
            target_directory_path: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_add_request.send(engine_unprivileged_state, move |project_items_add_response| {
            let _ = response_sender.send(project_items_add_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_add_response) => {
                self.app_state.scan_results_pane_state.status_message = if project_items_add_response.success {
                    format!(
                        "Added {} project items from selected scan results.",
                        project_items_add_response.added_project_item_count
                    )
                } else {
                    "Add-to-project request failed.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for add-to-project response: {}", receive_error);
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = false;
    }

    fn delete_selected_scan_results(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_deleting_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Delete request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to delete.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for deletion.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_deleting_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = format!("Deleting {} selected scan results.", selected_scan_result_refs.len());

        let scan_results_delete_request = ScanResultsDeleteRequest {
            scan_result_refs: selected_scan_result_refs,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_delete_request.send(engine_unprivileged_state, move |scan_results_delete_response| {
            let _ = response_sender.send(scan_results_delete_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results delete request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_delete_response) => {
                self.app_state.scan_results_pane_state.status_message = "Deleted selected scan results.".to_string();
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results delete response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
    }

    fn commit_selected_scan_results_value_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_committing_value_edit {
            self.app_state.scan_results_pane_state.status_message = "Value commit request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to commit value edits.".to_string();
            return;
        }

        let pending_value_edit_text = self
            .app_state
            .scan_results_pane_state
            .pending_value_edit_text
            .trim()
            .to_string();
        if pending_value_edit_text.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "Edit value is empty.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for value commits.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_committing_value_edit = true;
        self.app_state.scan_results_pane_state.status_message = format!(
            "Committing value edit '{}' for {} selected results.",
            pending_value_edit_text,
            selected_scan_result_refs.len()
        );

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs: selected_scan_result_refs,
            anonymous_value_string: AnonymousValueString::new(pending_value_edit_text, AnonymousValueStringFormat::Decimal, ContainerType::None),
            field_namespace: ScanResult::PROPERTY_NAME_VALUE.to_string(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_committing_value_edit = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results set property request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.scan_results_pane_state.status_message = "Committed selected scan result values.".to_string();
                self.refresh_selected_scan_results(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results set property response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_committing_value_edit = false;
    }

    fn apply_scan_results_query_response(
        &mut self,
        scan_results_query_response: squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse,
    ) {
        let result_count = scan_results_query_response.result_count;
        let page_index = scan_results_query_response.page_index;
        self.app_state
            .scan_results_pane_state
            .apply_query_response(scan_results_query_response);
        self.app_state.scan_results_pane_state.status_message = format!("Loaded page {} ({} total results).", page_index, result_count);
    }

    fn refresh_process_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response
        {
            self.app_state.process_selector_pane_state.status_message = "Process list request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process queries.".to_string();
                return;
            }
        };

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = true;
        self.app_state.process_selector_pane_state.status_message = "Refreshing process list.".to_string();

        let process_list_request = ProcessListRequest {
            require_windowed: self
                .app_state
                .process_selector_pane_state
                .show_windowed_processes_only,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_list_request.send(engine_unprivileged_state, move |process_list_response| {
            let _ = response_sender.send(process_list_response);
        });

        if !request_dispatched {
            self.app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response = false;
            self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process list request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_list_response) => {
                let process_count = process_list_response.processes.len();
                self.app_state
                    .process_selector_pane_state
                    .apply_process_list(process_list_response.processes);
                self.app_state.process_selector_pane_state.status_message = format!("Loaded {} processes.", process_count);
            }
            Err(receive_error) => {
                self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process list response: {}", receive_error);
            }
        }

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = false;
    }

    fn open_selected_process(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.process_selector_pane_state.is_opening_process {
            self.app_state.process_selector_pane_state.status_message = "Process open request already in progress.".to_string();
            return;
        }

        let selected_process_identifier = match self.app_state.process_selector_pane_state.selected_process_id() {
            Some(selected_process_identifier) => selected_process_identifier,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No process is selected.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process opening.".to_string();
                return;
            }
        };

        self.app_state.process_selector_pane_state.is_opening_process = true;
        self.app_state.process_selector_pane_state.status_message = format!("Opening process {}.", selected_process_identifier);

        let process_open_request = ProcessOpenRequest {
            process_id: Some(selected_process_identifier),
            search_name: None,
            match_case: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_open_request.send(engine_unprivileged_state, move |process_open_response| {
            let _ = response_sender.send(process_open_response);
        });

        if !request_dispatched {
            self.app_state.process_selector_pane_state.is_opening_process = false;
            self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process open request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_open_response) => {
                let opened_process = process_open_response.opened_process_info;
                self.app_state
                    .process_selector_pane_state
                    .set_opened_process(opened_process.clone());
                self.app_state.process_selector_pane_state.status_message = if let Some(opened_process_info) = opened_process {
                    format!(
                        "Opened process {} ({}).",
                        opened_process_info.get_name(),
                        opened_process_info.get_process_id_raw()
                    )
                } else {
                    "Open process request completed with no process.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process open response: {}", receive_error);
            }
        }

        self.app_state.process_selector_pane_state.is_opening_process = false;
    }

    fn refresh_project_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response
        {
            self.app_state.project_explorer_pane_state.status_message = "Project list request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project queries.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once = true;
        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = true;
        self.app_state.project_explorer_pane_state.status_message = "Refreshing project list.".to_string();

        let project_list_request = ProjectListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_list_request.send(engine_unprivileged_state, move |project_list_response| {
            let _ = response_sender.send(project_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_list_response) => {
                let project_count = project_list_response.projects_info.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_list(project_list_response.projects_info);
                self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} projects.", project_count);
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project list response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = false;
    }

    fn refresh_project_items_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item list request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project item listing.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = true;
        self.app_state.project_explorer_pane_state.status_message = "Refreshing project item hierarchy.".to_string();

        let project_items_list_request = ProjectItemsListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_list_request.send(engine_unprivileged_state, move |project_items_list_response| {
            let _ = response_sender.send(project_items_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_list_response) => {
                let project_item_count = project_items_list_response.opened_project_items.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_items_list(project_items_list_response.opened_project_items);
                self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} project items.", project_item_count);
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item list response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = false;
    }

    fn create_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_creating_project {
            self.app_state.project_explorer_pane_state.status_message = "Project create request already in progress.".to_string();
            return;
        }

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project creation.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_creating_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Creating project '{}'.", new_project_name);

        let project_create_request = ProjectCreateRequest {
            project_directory_path: None,
            project_name: Some(new_project_name.clone()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_create_request.send(engine_unprivileged_state, move |project_create_response| {
            let _ = response_sender.send(project_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_create_response) => {
                if project_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!(
                        "Created project '{}' at {}.",
                        new_project_name,
                        project_create_response.new_project_path.display()
                    );
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_create_response.new_project_path);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project create response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_creating_project = false;
    }

    fn create_project_directory_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_creating_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item create request already in progress.".to_string();
            return;
        }

        let parent_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(parent_directory_path) => parent_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No directory target is selected for project item create.".to_string();
                return;
            }
        };

        let project_item_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(project_item_name) => project_item_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project item name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project item create.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Creating directory '{}' under {}.", project_item_name, parent_directory_path.display());

        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path,
            project_item_name: project_item_name.clone(),
            project_item_type: "directory".to_string(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_create_request.send(engine_unprivileged_state, move |project_items_create_response| {
            let _ = response_sender.send(project_items_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_create_response) => {
                if project_items_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Created project directory '{}'.", project_item_name);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item create response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = false;
    }

    fn toggle_selected_project_item_activation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item activation request already in progress.".to_string();
            return;
        }

        let selected_project_item_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_path()
        {
            Some(selected_project_item_path) => selected_project_item_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for activation.".to_string();
                return;
            }
        };
        let is_target_activated = !self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_is_activated();

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message =
                    "No unprivileged engine state is available for project item activation.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Setting activation={} for {}.", is_target_activated, selected_project_item_path.display());

        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: vec![selected_project_item_path.display().to_string()],
            is_activated: is_target_activated,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_activate_request.send(engine_unprivileged_state, move |project_items_activate_response| {
            let _ = response_sender.send(project_items_activate_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_) => {
                self.app_state.project_explorer_pane_state.status_message = "Updated selected project item activation.".to_string();
                self.refresh_project_items_list(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message =
                    format!("Timed out waiting for project item activation response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = false;
    }

    fn move_staged_project_items_to_selected_directory(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_moving_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item move request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_move_source_paths()
        {
            self.app_state.project_explorer_pane_state.status_message = "No staged project items to move.".to_string();
            return;
        }

        let target_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(target_directory_path) => target_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No target directory is selected for move.".to_string();
                return;
            }
        };

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .pending_move_source_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for move.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Moving {} project items to {}.", project_item_paths.len(), target_directory_path.display());

        let project_items_move_request = ProjectItemsMoveRequest {
            project_item_paths,
            target_directory_path,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_move_request.send(engine_unprivileged_state, move |project_items_move_response| {
            let _ = response_sender.send(project_items_move_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_move_response) => {
                if project_items_move_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .clear_pending_move_source_paths();
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Moved {} project items.", project_items_move_response.moved_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item move request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item move response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = false;
    }

    fn reorder_selected_project_item(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        move_toward_previous_position: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_reordering_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item reorder request already in progress.".to_string();
            return;
        }

        let project_item_paths = match self
            .app_state
            .project_explorer_pane_state
            .build_reorder_request_paths_for_selected_project_item(move_toward_previous_position)
        {
            Some(project_item_paths) => project_item_paths,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Selected project item cannot be reordered in that direction.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for reorder.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = "Reordering project items.".to_string();

        let project_items_reorder_request = ProjectItemsReorderRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_reorder_request.send(engine_unprivileged_state, move |project_items_reorder_response| {
            let _ = response_sender.send(project_items_reorder_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_reorder_response) => {
                if project_items_reorder_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Reordered {} project items.", project_items_reorder_response.reordered_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item reorder request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item reorder response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = false;
    }

    fn delete_selected_project_item_with_confirmation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_deleting_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item delete request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_delete_confirmation_for_selected_project_item()
        {
            if self
                .app_state
                .project_explorer_pane_state
                .arm_delete_confirmation_for_selected_project_item()
            {
                self.app_state.project_explorer_pane_state.status_message = "Press x again to confirm deleting selected project item.".to_string();
            } else {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for delete.".to_string();
            }
            return;
        }

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .take_pending_delete_confirmation_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for delete.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting {} project items.", project_item_paths.len());

        let project_items_delete_request = ProjectItemsDeleteRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_delete_request.send(engine_unprivileged_state, move |project_items_delete_response| {
            let _ = response_sender.send(project_items_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_delete_response) => {
                if project_items_delete_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Deleted {} project items.", project_items_delete_response.deleted_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item delete response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = false;
    }

    fn open_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_opening_project {
            self.app_state.project_explorer_pane_state.status_message = "Project open request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project opening.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_opening_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Opening project '{}'.", selected_project_name);

        let project_open_request = ProjectOpenRequest {
            open_file_browser: false,
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_open_request.send(engine_unprivileged_state, move |project_open_response| {
            let _ = response_sender.send(project_open_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_open_response) => {
                if project_open_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(Some(selected_project_name.clone()), Some(selected_project_directory_path.clone()));
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state.project_explorer_pane_state.status_message = format!("Opened project '{}'.", selected_project_name);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project open request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project open response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_opening_project = false;
    }

    fn rename_selected_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_renaming_project {
            self.app_state.project_explorer_pane_state.status_message = "Project rename request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for rename.".to_string();
                return;
            }
        };
        let selected_project_directory_path_for_active_check = selected_project_directory_path.clone();

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project renaming.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_renaming_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Renaming project to '{}'.", new_project_name);

        let project_rename_request = ProjectRenameRequest {
            project_directory_path: selected_project_directory_path,
            new_project_name: new_project_name.clone(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_rename_request.send(engine_unprivileged_state, move |project_rename_response| {
            let _ = response_sender.send(project_rename_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_rename_response) => {
                if project_rename_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Renamed project to '{}'.", new_project_name);
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_rename_response.new_project_path);
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path_for_active_check)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(Some(new_project_name), Some(project_rename_response.new_project_path));
                    }
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project rename request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project rename response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_renaming_project = false;
    }

    fn delete_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_deleting_project {
            self.app_state.project_explorer_pane_state.status_message = "Project delete request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for delete.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project deletion.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_deleting_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting project '{}'.", selected_project_name);

        let project_delete_request = ProjectDeleteRequest {
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_delete_request.send(engine_unprivileged_state, move |project_delete_response| {
            let _ = response_sender.send(project_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_delete_response) => {
                if project_delete_response.success {
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(None, None);
                        self.app_state.project_explorer_pane_state.clear_project_items();
                    }
                    self.app_state.project_explorer_pane_state.status_message = format!("Deleted project '{}'.", selected_project_name);
                    self.refresh_project_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project delete response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_deleting_project = false;
    }

    fn close_active_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_closing_project {
            self.app_state.project_explorer_pane_state.status_message = "Project close request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project close.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_closing_project = true;
        self.app_state.project_explorer_pane_state.status_message = "Closing active project.".to_string();

        let project_close_request = ProjectCloseRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_close_request.send(engine_unprivileged_state, move |project_close_response| {
            let _ = response_sender.send(project_close_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_close_response) => {
                if project_close_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(None, None);
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state.project_explorer_pane_state.status_message = "Closed active project.".to_string();
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project close request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project close response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_closing_project = false;
    }

    fn draw_pane_layout(
        &self,
        frame: &mut ratatui::Frame<'_>,
        body_area: Rect,
    ) {
        let left_column_panes: Vec<TuiPane> = [
            TuiPane::ProcessSelector,
            TuiPane::ProjectExplorer,
            TuiPane::Settings,
        ]
        .into_iter()
        .filter(|pane| self.app_state.is_pane_visible(*pane))
        .collect();
        let right_column_panes: Vec<TuiPane> = [
            TuiPane::ElementScanner,
            TuiPane::ScanResults,
            TuiPane::StructViewer,
            TuiPane::Output,
        ]
        .into_iter()
        .filter(|pane| self.app_state.is_pane_visible(*pane))
        .collect();

        match (left_column_panes.is_empty(), right_column_panes.is_empty()) {
            (false, false) => {
                let columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                    .split(body_area);
                self.draw_pane_column(frame, columns[0], &left_column_panes);
                self.draw_pane_column(frame, columns[1], &right_column_panes);
            }
            (false, true) => self.draw_pane_column(frame, body_area, &left_column_panes),
            (true, false) => self.draw_pane_column(frame, body_area, &right_column_panes),
            (true, true) => {}
        }
    }

    fn draw_pane_column(
        &self,
        frame: &mut ratatui::Frame<'_>,
        column_area: Rect,
        panes: &[TuiPane],
    ) {
        if panes.is_empty() {
            return;
        }

        let row_constraints: Vec<Constraint> = panes
            .iter()
            .map(|_| Constraint::Ratio(1, panes.len() as u32))
            .collect();
        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(column_area);

        for (row_index, pane) in panes.iter().enumerate() {
            self.draw_single_pane(frame, row_areas[row_index], *pane);
        }
    }

    fn draw_single_pane(
        &self,
        frame: &mut ratatui::Frame<'_>,
        pane_area: Rect,
        pane: TuiPane,
    ) {
        let is_focused = self.app_state.focused_pane() == pane;
        let mut title = format!("{} [{}]", pane.title(), pane.shortcut_digit());
        if is_focused {
            title.push_str(" *");
        }

        let pane_lines: Vec<Line<'static>> = self
            .app_state
            .pane_summary_lines(pane)
            .into_iter()
            .map(Line::from)
            .collect();

        let pane_widget = Paragraph::new(pane_lines).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(pane_widget, pane_area);
    }
}
