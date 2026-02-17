use crate::state::TuiAppState;
use crate::state::pane::TuiPane;
use crate::state::project_explorer_pane_state::{ProjectExplorerFocusTarget, ProjectSelectorInputMode};
use crate::state::settings_pane_state::SettingsCategory;
use crate::state::struct_viewer_pane_state::StructViewerSource;
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
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
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
use squalr_engine_api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::io::{self, Stdout};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
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
    pub scan_results_update_counter: Arc<AtomicU64>,
    pub consumed_scan_results_update_counter: u64,
    pub has_registered_scan_results_updated_listener: bool,
    pub last_scan_results_periodic_refresh_time: Option<Instant>,
}

impl AppShell {
    const MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 50;
    const MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 5_000;

    pub fn new(tick_rate: Duration) -> Self {
        Self {
            should_exit: false,
            tick_rate,
            last_tick_time: Instant::now(),
            app_state: TuiAppState::default(),
            scan_results_update_counter: Arc::new(AtomicU64::new(0)),
            consumed_scan_results_update_counter: 0,
            has_registered_scan_results_updated_listener: false,
            last_scan_results_periodic_refresh_time: None,
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
        self.register_scan_results_updated_listener_if_needed(squalr_engine);
        let did_requery_after_scan_results_update = self.query_scan_results_page_if_engine_event_pending(squalr_engine);
        if !did_requery_after_scan_results_update {
            self.refresh_scan_results_on_interval_if_eligible(squalr_engine);
        }

        self.refresh_output_log_history(squalr_engine);

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

        if !self.app_state.settings_pane_state.is_refreshing_settings && self.app_state.settings_pane_state.status_message == "Ready." {
            self.refresh_all_settings_categories(squalr_engine);
        }
    }

    fn register_scan_results_updated_listener_if_needed(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.has_registered_scan_results_updated_listener {
            return;
        }

        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            return;
        };
        let scan_results_update_counter = self.scan_results_update_counter.clone();
        engine_unprivileged_state.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |_scan_results_updated_event| {
            scan_results_update_counter.fetch_add(1, Ordering::Relaxed);
        });

        self.has_registered_scan_results_updated_listener = true;
    }

    fn query_scan_results_page_if_engine_event_pending(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) -> bool {
        let latest_scan_results_update_counter = self.scan_results_update_counter.load(Ordering::Relaxed);
        if latest_scan_results_update_counter == self.consumed_scan_results_update_counter {
            return false;
        }
        if self.app_state.scan_results_pane_state.is_querying_scan_results {
            return false;
        }

        self.consumed_scan_results_update_counter = latest_scan_results_update_counter;
        self.query_scan_results_current_page(squalr_engine);
        true
    }

    fn refresh_scan_results_on_interval_if_eligible(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_tick_time = Instant::now();
        if !self.should_refresh_scan_results_page_on_tick(current_tick_time) {
            return;
        }

        if self.refresh_scan_results_page_with_feedback(squalr_engine, false) {
            self.last_scan_results_periodic_refresh_time = Some(Instant::now());
        }
    }

    fn should_refresh_scan_results_page_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if !self.app_state.is_pane_visible(TuiPane::ScanResults) {
            return false;
        }
        if self.app_state.scan_results_pane_state.scan_results.is_empty() {
            return false;
        }
        if self.app_state.scan_results_pane_state.is_querying_scan_results
            || self
                .app_state
                .scan_results_pane_state
                .is_refreshing_scan_results
            || self.app_state.scan_results_pane_state.is_freezing_scan_results
            || self.app_state.scan_results_pane_state.is_deleting_scan_results
            || self.app_state.scan_results_pane_state.is_committing_value_edit
        {
            return false;
        }

        let refresh_interval = self.scan_results_periodic_refresh_interval();
        match self.last_scan_results_periodic_refresh_time {
            Some(last_scan_results_periodic_refresh_time) => current_tick_time.duration_since(last_scan_results_periodic_refresh_time) >= refresh_interval,
            None => true,
        }
    }

    fn scan_results_periodic_refresh_interval(&self) -> Duration {
        let configured_results_read_interval_ms = self
            .app_state
            .settings_pane_state
            .scan_settings
            .results_read_interval_ms;
        let bounded_results_read_interval_ms =
            configured_results_read_interval_ms.clamp(Self::MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS, Self::MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS);

        Duration::from_millis(bounded_results_read_interval_ms)
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
            TuiPane::StructViewer => self.handle_struct_viewer_key_event(key_event, squalr_engine),
            TuiPane::Output => self.handle_output_key_event(key_event.code, squalr_engine),
            TuiPane::Settings => self.handle_settings_key_event(key_event.code, squalr_engine),
        }
    }

    fn handle_output_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_output_log_history(squalr_engine),
            KeyCode::Char('x') | KeyCode::Delete => self.app_state.output_pane_state.clear_log_lines(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.app_state.output_pane_state.increase_max_line_count(),
            KeyCode::Char('-') => self.app_state.output_pane_state.decrease_max_line_count(),
            _ => {}
        }
    }

    fn handle_settings_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_all_settings_categories(squalr_engine),
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

    fn handle_struct_viewer_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
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
            KeyCode::Backspace => self.app_state.struct_viewer_pane_state.backspace_pending_edit(),
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app_state.struct_viewer_pane_state.clear_pending_edit();
            }
            KeyCode::Char(pending_character) => self
                .app_state
                .struct_viewer_pane_state
                .append_pending_edit_character(pending_character),
            _ => {}
        }
    }

    fn refresh_output_log_history(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.output_pane_state.status_message = "No unprivileged engine state is available for output logs.".to_string();
                return;
            }
        };

        let log_history_guard = match engine_unprivileged_state.get_logger().get_log_history().read() {
            Ok(log_history_guard) => log_history_guard,
            Err(lock_error) => {
                self.app_state.output_pane_state.status_message = format!("Failed to lock output log history: {}", lock_error);
                return;
            }
        };
        let log_history_snapshot = log_history_guard.iter().cloned().collect();
        self.app_state
            .output_pane_state
            .apply_log_history(log_history_snapshot);
    }

    fn refresh_all_settings_categories(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.settings_pane_state.is_refreshing_settings {
            self.app_state.settings_pane_state.status_message = "Settings refresh is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings refresh.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_refreshing_settings = true;

        let general_settings_list_request = GeneralSettingsListRequest {};
        let (general_response_sender, general_response_receiver) = mpsc::sync_channel(1);
        general_settings_list_request.send(engine_unprivileged_state, move |general_settings_list_response| {
            let _ = general_response_sender.send(general_settings_list_response);
        });

        match general_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(general_settings_list_response) => {
                if let Ok(general_settings) = general_settings_list_response.general_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_general_settings(general_settings);
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read general settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings response: {}", receive_error);
            }
        }

        let memory_settings_list_request = MemorySettingsListRequest {};
        let (memory_response_sender, memory_response_receiver) = mpsc::sync_channel(1);
        memory_settings_list_request.send(engine_unprivileged_state, move |memory_settings_list_response| {
            let _ = memory_response_sender.send(memory_settings_list_response);
        });

        match memory_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(memory_settings_list_response) => {
                if let Ok(memory_settings) = memory_settings_list_response.memory_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_memory_settings(memory_settings);
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read memory settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings response: {}", receive_error);
            }
        }

        let scan_settings_list_request = ScanSettingsListRequest {};
        let (scan_response_sender, scan_response_receiver) = mpsc::sync_channel(1);
        scan_settings_list_request.send(engine_unprivileged_state, move |scan_settings_list_response| {
            let _ = scan_response_sender.send(scan_settings_list_response);
        });

        match scan_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_settings_list_response) => {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_scan_settings(scan_settings);
                    self.app_state.settings_pane_state.status_message = "Settings refreshed.".to_string();
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read scan settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings response: {}", receive_error);
            }
        }

        self.app_state.settings_pane_state.is_refreshing_settings = false;
    }

    fn apply_selected_settings_category(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.settings_pane_state.is_applying_settings {
            self.app_state.settings_pane_state.status_message = "Settings update is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings update.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_applying_settings = true;
        let selected_settings_category = self.app_state.settings_pane_state.selected_category;

        match selected_settings_category {
            SettingsCategory::General => {
                let general_settings_set_request = GeneralSettingsSetRequest {
                    engine_request_delay: Some(
                        self.app_state
                            .settings_pane_state
                            .general_settings
                            .engine_request_delay_ms,
                    ),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                general_settings_set_request.send(engine_unprivileged_state, move |general_settings_set_response| {
                    let _ = response_sender.send(general_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_general_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied general settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Memory => {
                let memory_settings = self.app_state.settings_pane_state.memory_settings;
                let memory_settings_set_request = MemorySettingsSetRequest {
                    memory_type_none: Some(memory_settings.memory_type_none),
                    memory_type_private: Some(memory_settings.memory_type_private),
                    memory_type_image: Some(memory_settings.memory_type_image),
                    memory_type_mapped: Some(memory_settings.memory_type_mapped),
                    required_write: Some(memory_settings.required_write),
                    required_execute: Some(memory_settings.required_execute),
                    required_copy_on_write: Some(memory_settings.required_copy_on_write),
                    excluded_write: Some(memory_settings.excluded_write),
                    excluded_execute: Some(memory_settings.excluded_execute),
                    excluded_copy_on_write: Some(memory_settings.excluded_copy_on_write),
                    start_address: Some(memory_settings.start_address),
                    end_address: Some(memory_settings.end_address),
                    only_query_usermode: Some(memory_settings.only_query_usermode),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                memory_settings_set_request.send(engine_unprivileged_state, move |memory_settings_set_response| {
                    let _ = response_sender.send(memory_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_memory_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied memory settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Scan => {
                let scan_settings = self.app_state.settings_pane_state.scan_settings;
                let scan_settings_set_request = ScanSettingsSetRequest {
                    results_page_size: Some(scan_settings.results_page_size),
                    results_read_interval_ms: Some(scan_settings.results_read_interval_ms),
                    project_read_interval_ms: Some(scan_settings.project_read_interval_ms),
                    freeze_interval_ms: Some(scan_settings.freeze_interval_ms),
                    memory_alignment: scan_settings.memory_alignment,
                    memory_read_mode: Some(scan_settings.memory_read_mode),
                    floating_point_tolerance: Some(scan_settings.floating_point_tolerance),
                    is_single_threaded_scan: Some(scan_settings.is_single_threaded_scan),
                    debug_perform_validation_scan: Some(scan_settings.debug_perform_validation_scan),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                scan_settings_set_request.send(engine_unprivileged_state, move |scan_settings_set_response| {
                    let _ = response_sender.send(scan_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_scan_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied scan settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings set response: {}", receive_error);
                    }
                }
            }
        }

        self.app_state.settings_pane_state.is_applying_settings = false;
    }

    fn refresh_struct_viewer_focus_from_source(&mut self) {
        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected.".to_string();
            }
            StructViewerSource::ScanResults => self.sync_struct_viewer_focus_from_scan_results(),
            StructViewerSource::ProjectItems => self.sync_struct_viewer_focus_from_project_items(),
        }
    }

    fn sync_struct_viewer_focus_from_scan_results(&mut self) {
        let selected_scan_results = self.app_state.scan_results_pane_state.selected_scan_results();
        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        self.app_state
            .struct_viewer_pane_state
            .focus_scan_results(&selected_scan_results, selected_scan_result_refs);
    }

    fn sync_struct_viewer_focus_from_project_items(&mut self) {
        let selected_project_items = self
            .app_state
            .project_explorer_pane_state
            .selected_project_items_for_struct_viewer();
        self.app_state
            .struct_viewer_pane_state
            .focus_project_items(selected_project_items);
    }

    fn commit_struct_viewer_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.struct_viewer_pane_state.is_committing_edit {
            self.app_state.struct_viewer_pane_state.status_message = "Struct field edit is already in progress.".to_string();
            return;
        }

        let edited_field = match self
            .app_state
            .struct_viewer_pane_state
            .build_edited_field_from_pending_text()
        {
            Ok(edited_field) => edited_field,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };

        self.app_state.struct_viewer_pane_state.is_committing_edit = true;
        self.app_state.struct_viewer_pane_state.status_message = format!("Committing field '{}'.", edited_field.get_name());

        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected for commit.".to_string();
            }
            StructViewerSource::ScanResults => self.commit_scan_result_struct_field_edit(squalr_engine, edited_field),
            StructViewerSource::ProjectItems => self.commit_project_item_struct_field_edit(squalr_engine, edited_field),
        }

        self.app_state.struct_viewer_pane_state.is_committing_edit = false;
    }

    fn commit_scan_result_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_scan_result_refs = self
            .app_state
            .struct_viewer_pane_state
            .selected_scan_result_refs
            .clone();
        if selected_scan_result_refs.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No scan results are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for scan result struct edits.".to_string();
                return;
            }
        };

        if edited_field.get_name() == ScanResult::PROPERTY_NAME_IS_FROZEN {
            let target_frozen_state = edited_field
                .get_data_value()
                .map(|edited_data_value| {
                    edited_data_value
                        .get_value_bytes()
                        .iter()
                        .any(|edited_value_byte| *edited_value_byte != 0)
                })
                .unwrap_or(false);

            let scan_results_freeze_request = ScanResultsFreezeRequest {
                scan_result_refs: selected_scan_result_refs,
                is_frozen: target_frozen_state,
            };
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
                let _ = response_sender.send(scan_results_freeze_response);
            });

            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result freeze request from struct viewer.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(scan_results_freeze_response) => {
                    if scan_results_freeze_response
                        .failed_freeze_toggle_scan_result_refs
                        .is_empty()
                    {
                        self.app_state.struct_viewer_pane_state.status_message = if target_frozen_state {
                            "Committed frozen state from struct viewer.".to_string()
                        } else {
                            "Committed unfrozen state from struct viewer.".to_string()
                        };
                    } else {
                        self.app_state.struct_viewer_pane_state.status_message = format!(
                            "Freeze commit partially failed for {} scan results.",
                            scan_results_freeze_response
                                .failed_freeze_toggle_scan_result_refs
                                .len()
                        );
                    }
                    self.refresh_scan_results_page(squalr_engine);
                    self.sync_struct_viewer_focus_from_scan_results();
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result freeze response: {}", receive_error);
                }
            }
            return;
        }

        let scan_results_set_property_request = match Self::build_scan_results_set_property_request_for_struct_edit(selected_scan_result_refs, &edited_field) {
            Ok(scan_results_set_property_request) => scan_results_set_property_request,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result property request from struct viewer.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.struct_viewer_pane_state.status_message =
                    format!("Committed scan result field '{}' from struct viewer.", edited_field.get_name());
                self.refresh_scan_results_page(squalr_engine);
                self.sync_struct_viewer_focus_from_scan_results();
            }
            Err(receive_error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result property response: {}", receive_error);
            }
        }
    }

    fn commit_project_item_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_project_item_paths = self
            .app_state
            .struct_viewer_pane_state
            .selected_project_item_paths
            .clone();
        if selected_project_item_paths.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No project items are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for project item struct edits.".to_string();
                return;
            }
        };

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let edited_field_name = edited_field.get_name().to_string();
        let edited_name = if edited_field_name == ProjectItem::PROPERTY_NAME {
            Self::extract_string_value_from_edited_field(&edited_field)
        } else {
            None
        };

        let mut pending_memory_write_requests = Vec::new();
        let mut pending_rename_requests = Vec::new();
        let mut has_persisted_property_edit = false;
        let mut opened_project_write_guard = match opened_project_lock.write() {
            Ok(opened_project_write_guard) => opened_project_write_guard,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Failed to acquire opened project lock for struct edit: {}", error);
                return;
            }
        };
        let opened_project = match opened_project_write_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "Cannot apply struct edit because no project is currently open.".to_string();
                return;
            }
        };
        let root_project_item_path = opened_project
            .get_project_root_ref()
            .get_project_item_path()
            .clone();

        for selected_project_item_path in &selected_project_item_paths {
            if edited_field_name == ProjectItem::PROPERTY_NAME && selected_project_item_path == &root_project_item_path {
                continue;
            }

            let project_item_ref = ProjectItemRef::new(selected_project_item_path.clone());
            let selected_project_item = match opened_project.get_project_item_mut(&project_item_ref) {
                Some(selected_project_item) => selected_project_item,
                None => continue,
            };
            let project_item_type_id = selected_project_item
                .get_item_type()
                .get_project_item_type_id()
                .to_string();
            let should_apply_edited_field = Self::should_apply_struct_field_edit_to_project_item(&project_item_type_id, &edited_field_name);

            if should_apply_edited_field {
                selected_project_item.get_properties_mut().set_field_data(
                    edited_field.get_name(),
                    edited_field.get_field_data().clone(),
                    edited_field.get_is_read_only(),
                );
                selected_project_item.set_has_unsaved_changes(true);
                has_persisted_property_edit = true;
            }

            if let Some(edited_name) = &edited_name {
                if let Some(project_items_rename_request) =
                    Self::build_project_item_rename_request(selected_project_item_path, &project_item_type_id, edited_name)
                {
                    pending_rename_requests.push(project_items_rename_request);
                }
            }

            if let Some(memory_write_request) = Self::build_memory_write_request_for_project_item_edit(selected_project_item, &edited_field) {
                pending_memory_write_requests.push(memory_write_request);
            }
        }

        if !has_persisted_property_edit && pending_rename_requests.is_empty() && pending_memory_write_requests.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "Selected project item field cannot be committed through TUI struct routing.".to_string();
            return;
        }

        drop(opened_project_write_guard);

        if has_persisted_property_edit {
            if let Ok(mut opened_project_write_guard) = opened_project_lock.write() {
                if let Some(opened_project) = opened_project_write_guard.as_mut() {
                    opened_project
                        .get_project_info_mut()
                        .set_has_unsaved_changes(true);
                }
            }

            let project_save_request = ProjectSaveRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            project_save_request.send(engine_unprivileged_state, move |project_save_response| {
                let _ = response_sender.send(project_save_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_save_response) => {
                    if !project_save_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project save failed while committing project item struct field.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project save response: {}", receive_error);
                    return;
                }
            }

            project_manager.notify_project_items_changed();
        }

        for pending_rename_request in pending_rename_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            pending_rename_request.send(engine_unprivileged_state, move |project_items_rename_response| {
                let _ = response_sender.send(project_items_rename_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_items_rename_response) => {
                    if !project_items_rename_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project item rename failed during struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project item rename response: {}", receive_error);
                    return;
                }
            }
        }

        for pending_memory_write_request in pending_memory_write_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = pending_memory_write_request.send(engine_unprivileged_state, move |memory_write_response| {
                let _ = response_sender.send(memory_write_response);
            });
            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch memory write request during struct edit commit.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(memory_write_response) => {
                    if !memory_write_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Memory write failed during project item struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for memory write response: {}", receive_error);
                    return;
                }
            }
        }

        self.app_state
            .struct_viewer_pane_state
            .apply_committed_field(&edited_field);
        self.app_state.struct_viewer_pane_state.status_message = format!("Committed project item field '{}' from struct viewer.", edited_field.get_name());
        self.refresh_project_items_list(squalr_engine);
        self.sync_struct_viewer_focus_from_project_items();
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

    fn refresh_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let _ = self.refresh_scan_results_page_with_feedback(squalr_engine, true);
    }

    fn refresh_scan_results_page_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) -> bool {
        if self
            .app_state
            .scan_results_pane_state
            .is_refreshing_scan_results
        {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Scan results refresh already in progress.".to_string();
            }
            return false;
        }

        let scan_result_refs_for_current_page = self
            .app_state
            .scan_results_pane_state
            .scan_results
            .iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect::<Vec<_>>();
        if scan_result_refs_for_current_page.is_empty() {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "No scan results are available to refresh.".to_string();
            }
            return false;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results refresh.".to_string();
                }
                return false;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = true;
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message =
                format!("Refreshing {} scan results on the current page.", scan_result_refs_for_current_page.len());
        }

        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: scan_result_refs_for_current_page,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_refresh_request.send(engine_unprivileged_state, move |scan_results_refresh_response| {
            let _ = response_sender.send(scan_results_refresh_response);
        });

        if !request_dispatched {
            self.app_state
                .scan_results_pane_state
                .is_refreshing_scan_results = false;
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results refresh request.".to_string();
            }
            return false;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_refresh_response) => {
                let refreshed_result_count = scan_results_refresh_response.scan_results.len();
                self.app_state
                    .scan_results_pane_state
                    .apply_refreshed_results(scan_results_refresh_response.scan_results);
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Refreshed {} scan results.", refreshed_result_count);
                }
                self.last_scan_results_periodic_refresh_time = Some(Instant::now());
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results refresh response: {}", receive_error);
                }
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = false;
        true
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
                self.refresh_scan_results_page(squalr_engine);
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
                self.refresh_scan_results_page(squalr_engine);
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
        self.sync_struct_viewer_focus_from_scan_results();
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
                self.sync_struct_viewer_focus_from_project_items();
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
                    self.app_state
                        .struct_viewer_pane_state
                        .clear_focus("Cleared struct viewer after project open.");
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
                        self.sync_struct_viewer_focus_from_project_items();
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
                        self.app_state
                            .struct_viewer_pane_state
                            .clear_focus("Cleared struct viewer after project delete.");
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
                    self.app_state
                        .struct_viewer_pane_state
                        .clear_focus("Cleared struct viewer after project close.");
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

    fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let edited_data_value = edited_field.get_data_value()?;
        let edited_name = String::from_utf8(edited_data_value.get_value_bytes().clone()).ok()?;
        let edited_name = edited_name.trim();

        if edited_name.is_empty() { None } else { Some(edited_name.to_string()) }
    }

    fn build_project_item_rename_request(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);
            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();
        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }

    fn build_memory_write_request_for_project_item_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }
        if edited_field.get_name() != ProjectItemTypeAddress::PROPERTY_ADDRESS {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        Some(MemoryWriteRequest {
            address,
            module_name,
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn build_scan_results_set_property_request_for_struct_edit(
        scan_result_refs: Vec<ScanResultRef>,
        edited_field: &ValuedStructField,
    ) -> Result<ScanResultsSetPropertyRequest, String> {
        let edited_data_value = edited_field
            .get_data_value()
            .ok_or_else(|| "Nested struct scan result edits are not supported in the TUI yet.".to_string())?;
        let symbol_registry = SymbolRegistry::get_instance();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(edited_data_value.get_data_type_ref());
        let edited_anonymous_value = symbol_registry
            .anonymize_value(edited_data_value, default_edit_format)
            .map_err(|error| format!("Failed to format edited scan result value: {}", error))?;

        Ok(ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace: edited_field.get_name().to_string(),
            anonymous_value_string: edited_anonymous_value,
        })
    }

    fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
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

#[cfg(test)]
mod tests {
    use crate::app::AppShell;
    use crate::state::pane::TuiPane;
    use crate::state::settings_pane_state::SettingsCategory;
    use crate::state::struct_viewer_pane_state::StructViewerSource;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use squalr_engine::engine_mode::EngineMode;
    use squalr_engine::squalr_engine::SqualrEngine;
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
    use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
    use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;
    use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
    use std::time::{Duration, Instant};

    fn create_scan_result(scan_result_global_index: u64) -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            0x1000 + scan_result_global_index,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataTypeU8::get_value_from_primitive(42)),
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(scan_result_global_index),
        );

        ScanResult::new(scan_result_valued, String::new(), 0, None, Vec::new(), false)
    }

    #[test]
    fn focused_settings_pane_routes_category_cycle_key() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.set_focus_to_pane(TuiPane::Settings);
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for routing test");

        app_shell.handle_focused_pane_event(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE), &mut squalr_engine);

        assert_eq!(app_shell.app_state.settings_pane_state.selected_category, SettingsCategory::Memory);
    }

    #[test]
    fn focused_output_pane_routes_clear_key() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.set_focus_to_pane(TuiPane::Output);
        app_shell.app_state.output_pane_state.log_lines = vec!["existing log".to_string()];
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for routing test");

        app_shell.handle_focused_pane_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE), &mut squalr_engine);

        assert!(app_shell.app_state.output_pane_state.log_lines.is_empty());
    }

    #[test]
    fn scan_results_engine_update_waits_while_query_is_active() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for signal routing test");

        app_shell
            .scan_results_update_counter
            .store(1, std::sync::atomic::Ordering::Relaxed);
        app_shell
            .app_state
            .scan_results_pane_state
            .is_querying_scan_results = true;

        let did_dispatch_query = app_shell.query_scan_results_page_if_engine_event_pending(&mut squalr_engine);

        assert!(!did_dispatch_query);
        assert_eq!(app_shell.consumed_scan_results_update_counter, 0);
    }

    #[test]
    fn scan_results_periodic_refresh_requires_visible_pane_and_results() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell.app_state.scan_results_pane_state.scan_results = vec![create_scan_result(1)];

        let _hide_succeeded = app_shell.app_state.toggle_pane_visibility(TuiPane::ScanResults);
        assert!(!app_shell.app_state.is_pane_visible(TuiPane::ScanResults));

        assert!(!app_shell.should_refresh_scan_results_page_on_tick(current_tick_time));

        app_shell.app_state.set_focus_to_pane(TuiPane::ScanResults);

        assert!(app_shell.should_refresh_scan_results_page_on_tick(current_tick_time));
    }

    #[test]
    fn scan_results_periodic_refresh_respects_bounded_interval() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell.app_state.scan_results_pane_state.scan_results = vec![create_scan_result(7)];

        app_shell
            .app_state
            .settings_pane_state
            .scan_settings
            .results_read_interval_ms = 1;
        assert_eq!(
            app_shell.scan_results_periodic_refresh_interval(),
            Duration::from_millis(AppShell::MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS)
        );

        app_shell
            .app_state
            .settings_pane_state
            .scan_settings
            .results_read_interval_ms = 100_000;
        assert_eq!(
            app_shell.scan_results_periodic_refresh_interval(),
            Duration::from_millis(AppShell::MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS)
        );

        app_shell
            .app_state
            .settings_pane_state
            .scan_settings
            .results_read_interval_ms = 1_000;
        app_shell.last_scan_results_periodic_refresh_time = Some(current_tick_time - Duration::from_millis(500));
        assert!(!app_shell.should_refresh_scan_results_page_on_tick(current_tick_time));

        app_shell.last_scan_results_periodic_refresh_time = Some(current_tick_time - Duration::from_millis(1_100));
        assert!(app_shell.should_refresh_scan_results_page_on_tick(current_tick_time));
    }

    #[test]
    fn focused_struct_viewer_routes_format_cycle_key() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.set_focus_to_pane(TuiPane::StructViewer);
        app_shell.app_state.struct_viewer_pane_state.source = StructViewerSource::ScanResults;
        let selected_scan_results = vec![create_scan_result(3)];
        let selected_scan_result_refs = vec![ScanResultRef::new(3)];
        app_shell
            .app_state
            .struct_viewer_pane_state
            .focus_scan_results(&selected_scan_results, selected_scan_result_refs);
        let previous_pending_edit_text = app_shell
            .app_state
            .struct_viewer_pane_state
            .pending_edit_text
            .clone();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for routing test");

        app_shell.handle_focused_pane_event(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE), &mut squalr_engine);

        assert_ne!(app_shell.app_state.struct_viewer_pane_state.pending_edit_text, previous_pending_edit_text);
    }

    #[test]
    fn build_scan_results_set_property_request_for_struct_edit_uses_default_value_format() {
        let edited_data_value = DataTypeU8::get_value_from_primitive(42);
        let edited_field = ValuedStructField::new(
            ScanResult::PROPERTY_NAME_VALUE.to_string(),
            ValuedStructFieldData::Value(edited_data_value),
            false,
        );
        let scan_result_refs = vec![ScanResultRef::new(55)];
        let scan_results_set_property_request =
            AppShell::build_scan_results_set_property_request_for_struct_edit(scan_result_refs.clone(), &edited_field).expect("request should be created");

        assert_eq!(scan_results_set_property_request.field_namespace, ScanResult::PROPERTY_NAME_VALUE);
        let request_scan_result_global_indices = scan_results_set_property_request
            .scan_result_refs
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();
        let expected_scan_result_global_indices = scan_result_refs
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();
        assert_eq!(request_scan_result_global_indices, expected_scan_result_global_indices);
        assert_eq!(
            scan_results_set_property_request
                .anonymous_value_string
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Decimal
        );
        assert_eq!(
            scan_results_set_property_request
                .anonymous_value_string
                .get_anonymous_value_string(),
            "42"
        );
    }
}
