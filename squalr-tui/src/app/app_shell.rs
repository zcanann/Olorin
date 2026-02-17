use crate::state::TuiAppState;
use crate::state::pane::TuiPane;
use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use std::io::{self, IsTerminal, Stdout};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct TerminalGuard {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() || !io::stderr().is_terminal() {
            bail!(
                "Squalr TUI requires an interactive terminal. In VS Code CodeLLDB launch configs, set `terminal` to `external` or `integrated` for the squalr-tui target."
            );
        }

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
    pub last_process_list_auto_refresh_attempt_time: Option<Instant>,
    pub last_project_list_auto_refresh_attempt_time: Option<Instant>,
    pub last_project_items_auto_refresh_attempt_time: Option<Instant>,
    pub last_settings_auto_refresh_attempt_time: Option<Instant>,
}

impl AppShell {
    const MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 50;
    const MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 5_000;
    const MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS: u64 = 1_000;
    const MIN_SETTINGS_AUTO_REFRESH_INTERVAL_MS: u64 = 1_000;

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
            last_process_list_auto_refresh_attempt_time: None,
            last_project_list_auto_refresh_attempt_time: None,
            last_project_items_auto_refresh_attempt_time: None,
            last_settings_auto_refresh_attempt_time: None,
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
        let current_tick_time = Instant::now();
        self.synchronize_active_project_from_engine_state(squalr_engine);
        self.register_scan_results_updated_listener_if_needed(squalr_engine);
        let did_requery_after_scan_results_update = self.query_scan_results_page_if_engine_event_pending(squalr_engine);
        if !did_requery_after_scan_results_update {
            self.refresh_scan_results_on_interval_if_eligible(squalr_engine);
        }

        self.refresh_output_log_history(squalr_engine);

        if self.should_refresh_process_list_on_tick(current_tick_time) {
            self.last_process_list_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_process_list_with_feedback(squalr_engine, false);
        }

        if self.should_refresh_project_list_on_tick(current_tick_time) {
            self.last_project_list_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_project_list_with_feedback(squalr_engine, false);
        }

        if self.should_refresh_project_items_list_on_tick(current_tick_time) {
            self.last_project_items_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_project_items_list_with_feedback(squalr_engine, false);
        }

        self.refresh_settings_on_tick_if_eligible(squalr_engine);
    }

    fn synchronize_active_project_from_engine_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            return;
        };

        let opened_project_lock = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let (engine_active_project_name, engine_active_project_directory_path) = match opened_project_lock.read() {
            Ok(opened_project_read_guard) => match opened_project_read_guard.as_ref() {
                Some(opened_project) => (
                    Some(opened_project.get_project_info().get_name().to_string()),
                    opened_project.get_project_info().get_project_directory(),
                ),
                None => (None, None),
            },
            Err(lock_error) => {
                log::error!("Failed to acquire opened project lock for TUI project-state synchronization: {}", lock_error);
                return;
            }
        };

        self.apply_engine_active_project_state(engine_active_project_name, engine_active_project_directory_path);
    }

    fn apply_engine_active_project_state(
        &mut self,
        engine_active_project_name: Option<String>,
        engine_active_project_directory_path: Option<PathBuf>,
    ) {
        let did_active_project_change = self.app_state.project_explorer_pane_state.active_project_name != engine_active_project_name
            || self
                .app_state
                .project_explorer_pane_state
                .active_project_directory_path
                != engine_active_project_directory_path;
        if !did_active_project_change {
            return;
        }

        let did_active_project_directory_change = self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            != engine_active_project_directory_path;

        self.app_state
            .project_explorer_pane_state
            .set_active_project(engine_active_project_name, engine_active_project_directory_path);

        if did_active_project_directory_change {
            self.app_state.project_explorer_pane_state.clear_project_items();
            self.app_state
                .struct_viewer_pane_state
                .clear_focus("Cleared struct viewer after project-state synchronization.");
            self.last_project_items_auto_refresh_attempt_time = None;
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
        let _ = self.query_scan_results_current_page_with_feedback(squalr_engine, false);
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

    fn refresh_settings_on_tick_if_eligible(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_tick_time = Instant::now();
        if !self.should_refresh_settings_on_tick(current_tick_time) {
            return;
        }

        self.last_settings_auto_refresh_attempt_time = Some(current_tick_time);
        self.refresh_all_settings_categories_with_feedback(squalr_engine, false);
    }

    fn should_refresh_settings_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self.app_state.settings_pane_state.has_loaded_settings_once || self.app_state.settings_pane_state.is_refreshing_settings {
            return false;
        }

        match self.last_settings_auto_refresh_attempt_time {
            Some(last_settings_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_settings_auto_refresh_attempt_time) >= Duration::from_millis(Self::MIN_SETTINGS_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    fn should_refresh_process_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .process_selector_pane_state
            .has_loaded_process_list_once
            || self
                .app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response
        {
            return false;
        }

        match self.last_process_list_auto_refresh_attempt_time {
            Some(last_process_list_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_process_list_auto_refresh_attempt_time)
                    >= Duration::from_millis(Self::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    fn should_refresh_project_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once
            || self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_list_response
        {
            return false;
        }

        match self.last_project_list_auto_refresh_attempt_time {
            Some(last_project_list_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_project_list_auto_refresh_attempt_time)
                    >= Duration::from_millis(Self::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    fn should_refresh_project_items_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            .is_none()
            || self
                .app_state
                .project_explorer_pane_state
                .has_loaded_project_item_list_once
            || self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_item_list_response
        {
            return false;
        }

        match self.last_project_items_auto_refresh_attempt_time {
            Some(last_project_items_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_project_items_auto_refresh_attempt_time)
                    >= Duration::from_millis(Self::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
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
    use crate::views::project_explorer_pane_state::ProjectHierarchyEntry;
    use crate::views::settings_pane_state::SettingsCategory;
    use crate::views::struct_viewer_pane_state::StructViewerSource;
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
    use std::path::PathBuf;
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
    fn output_tick_refresh_preserves_existing_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.output_pane_state.status_message = "Manual output status.".to_string();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for output status test");

        app_shell.refresh_output_log_history(&mut squalr_engine);

        assert_eq!(app_shell.app_state.output_pane_state.status_message, "Manual output status.");
    }

    #[test]
    fn process_auto_refresh_preserves_existing_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.process_selector_pane_state.status_message = "Manual process status.".to_string();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for process auto-refresh status test");

        app_shell.refresh_process_list_with_feedback(&mut squalr_engine, false);

        assert_eq!(app_shell.app_state.process_selector_pane_state.status_message, "Manual process status.");
    }

    #[test]
    fn project_auto_refresh_preserves_existing_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.project_explorer_pane_state.status_message = "Manual project status.".to_string();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for project auto-refresh status test");

        app_shell.refresh_project_list_with_feedback(&mut squalr_engine, false);

        assert_eq!(app_shell.app_state.project_explorer_pane_state.status_message, "Manual project status.");
    }

    #[test]
    fn project_items_auto_refresh_preserves_existing_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.project_explorer_pane_state.status_message = "Manual project item status.".to_string();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for project-item auto-refresh status test");

        app_shell.refresh_project_items_list_with_feedback(&mut squalr_engine, false);

        assert_eq!(app_shell.app_state.project_explorer_pane_state.status_message, "Manual project item status.");
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
    fn scan_results_engine_update_requery_preserves_existing_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for scan-results engine-update status test");
        app_shell.app_state.scan_results_pane_state.status_message = "Manual scan-results status.".to_string();
        app_shell
            .scan_results_update_counter
            .store(1, std::sync::atomic::Ordering::Relaxed);

        let did_dispatch_query = app_shell.query_scan_results_page_if_engine_event_pending(&mut squalr_engine);

        assert!(did_dispatch_query);
        assert_eq!(app_shell.app_state.scan_results_pane_state.status_message, "Manual scan-results status.");
        assert_eq!(app_shell.consumed_scan_results_update_counter, 1);
    }

    #[test]
    fn scan_results_manual_query_updates_status_message() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for scan-results manual query status test");
        app_shell.app_state.scan_results_pane_state.status_message = "Manual scan-results status.".to_string();

        app_shell.query_scan_results_current_page(&mut squalr_engine);

        assert!(
            app_shell
                .app_state
                .scan_results_pane_state
                .status_message
                .starts_with("Loaded page ")
        );
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
    fn process_list_auto_refresh_eligibility_uses_load_state_and_interval() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell
            .app_state
            .process_selector_pane_state
            .has_loaded_process_list_once = false;
        app_shell
            .app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = false;

        assert!(app_shell.should_refresh_process_list_on_tick(current_tick_time));

        app_shell.last_process_list_auto_refresh_attempt_time = Some(current_tick_time);
        assert!(!app_shell.should_refresh_process_list_on_tick(current_tick_time));

        app_shell.last_process_list_auto_refresh_attempt_time =
            Some(current_tick_time - Duration::from_millis(AppShell::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS + 1));
        assert!(app_shell.should_refresh_process_list_on_tick(current_tick_time));

        app_shell
            .app_state
            .process_selector_pane_state
            .has_loaded_process_list_once = true;
        assert!(!app_shell.should_refresh_process_list_on_tick(current_tick_time));
    }

    #[test]
    fn project_list_auto_refresh_eligibility_uses_load_state_and_interval() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once = false;
        app_shell
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = false;

        assert!(app_shell.should_refresh_project_list_on_tick(current_tick_time));

        app_shell.last_project_list_auto_refresh_attempt_time = Some(current_tick_time);
        assert!(!app_shell.should_refresh_project_list_on_tick(current_tick_time));

        app_shell.last_project_list_auto_refresh_attempt_time =
            Some(current_tick_time - Duration::from_millis(AppShell::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS + 1));
        assert!(app_shell.should_refresh_project_list_on_tick(current_tick_time));

        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once = true;
        assert!(!app_shell.should_refresh_project_list_on_tick(current_tick_time));
    }

    #[test]
    fn project_items_auto_refresh_eligibility_requires_active_project_and_interval() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = false;
        app_shell
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = false;

        assert!(!app_shell.should_refresh_project_items_list_on_tick(current_tick_time));

        app_shell
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path = Some("C:/Projects/Alpha/project".into());
        assert!(app_shell.should_refresh_project_items_list_on_tick(current_tick_time));

        app_shell.last_project_items_auto_refresh_attempt_time = Some(current_tick_time);
        assert!(!app_shell.should_refresh_project_items_list_on_tick(current_tick_time));

        app_shell.last_project_items_auto_refresh_attempt_time =
            Some(current_tick_time - Duration::from_millis(AppShell::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS + 1));
        assert!(app_shell.should_refresh_project_items_list_on_tick(current_tick_time));

        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = true;
        assert!(!app_shell.should_refresh_project_items_list_on_tick(current_tick_time));
    }

    #[test]
    fn engine_project_sync_clears_hierarchy_and_resets_project_item_refresh_timing_on_directory_change() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell
            .app_state
            .project_explorer_pane_state
            .set_active_project(Some("Alpha".to_string()), Some(PathBuf::from("C:/Projects/Alpha/project")));
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = true;
        app_shell
            .app_state
            .project_explorer_pane_state
            .project_item_visible_entries = vec![ProjectHierarchyEntry {
            project_item_path: PathBuf::from("C:/Projects/Alpha/project/project_items/Health.json"),
            display_name: "Health".to_string(),
            depth: 0,
            is_directory: false,
            is_expanded: false,
            is_activated: false,
        }];
        app_shell
            .app_state
            .project_explorer_pane_state
            .selected_project_item_visible_index = Some(0);
        app_shell
            .app_state
            .project_explorer_pane_state
            .selected_item_path = Some("C:/Projects/Alpha/project/project_items/Health.json".to_string());
        app_shell.last_project_items_auto_refresh_attempt_time = Some(Instant::now());

        app_shell.apply_engine_active_project_state(Some("Beta".to_string()), Some(PathBuf::from("C:/Projects/Beta/project")));

        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .active_project_name,
            Some("Beta".to_string())
        );
        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .active_project_directory_path,
            Some(PathBuf::from("C:/Projects/Beta/project"))
        );
        assert!(
            !app_shell
                .app_state
                .project_explorer_pane_state
                .has_loaded_project_item_list_once
        );
        assert!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .project_item_visible_entries
                .is_empty()
        );
        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .selected_project_item_visible_index,
            None
        );
        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .selected_item_path,
            None
        );
        assert_eq!(app_shell.last_project_items_auto_refresh_attempt_time, None);
    }

    #[test]
    fn engine_project_sync_preserves_hierarchy_when_directory_is_unchanged() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let active_directory_path = PathBuf::from("C:/Projects/Alpha/project");
        let active_project_item_entry = ProjectHierarchyEntry {
            project_item_path: PathBuf::from("C:/Projects/Alpha/project/project_items/Health.json"),
            display_name: "Health".to_string(),
            depth: 0,
            is_directory: false,
            is_expanded: false,
            is_activated: false,
        };
        let previous_refresh_attempt_time = Some(Instant::now());
        app_shell
            .app_state
            .project_explorer_pane_state
            .set_active_project(Some("Alpha".to_string()), Some(active_directory_path.clone()));
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = true;
        app_shell
            .app_state
            .project_explorer_pane_state
            .project_item_visible_entries = vec![active_project_item_entry];
        app_shell
            .app_state
            .project_explorer_pane_state
            .selected_project_item_visible_index = Some(0);
        app_shell.last_project_items_auto_refresh_attempt_time = previous_refresh_attempt_time;

        app_shell.apply_engine_active_project_state(Some("Alpha Renamed".to_string()), Some(active_directory_path));

        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .active_project_name,
            Some("Alpha Renamed".to_string())
        );
        assert!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .has_loaded_project_item_list_once
        );
        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .project_item_visible_entries
                .len(),
            1
        );
        assert_eq!(
            app_shell
                .app_state
                .project_explorer_pane_state
                .selected_project_item_visible_index,
            Some(0)
        );
        assert_eq!(app_shell.last_project_items_auto_refresh_attempt_time, previous_refresh_attempt_time);
    }

    #[test]
    fn engine_project_sync_allows_immediate_project_item_auto_refresh_after_directory_change() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell
            .app_state
            .project_explorer_pane_state
            .set_active_project(Some("Alpha".to_string()), Some(PathBuf::from("C:/Projects/Alpha/project")));
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = true;
        app_shell
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = false;
        app_shell.last_project_items_auto_refresh_attempt_time = Some(current_tick_time);

        app_shell.apply_engine_active_project_state(Some("Beta".to_string()), Some(PathBuf::from("C:/Projects/Beta/project")));

        assert!(app_shell.should_refresh_project_items_list_on_tick(current_tick_time));
    }

    #[test]
    fn settings_auto_refresh_eligibility_uses_load_state_and_interval() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        let current_tick_time = Instant::now();
        app_shell.app_state.settings_pane_state.status_message = "Applied general settings.".to_string();
        app_shell.app_state.settings_pane_state.has_loaded_settings_once = false;

        assert!(app_shell.should_refresh_settings_on_tick(current_tick_time));

        app_shell.last_settings_auto_refresh_attempt_time = Some(current_tick_time);
        assert!(!app_shell.should_refresh_settings_on_tick(current_tick_time));

        app_shell.last_settings_auto_refresh_attempt_time =
            Some(current_tick_time - Duration::from_millis(AppShell::MIN_SETTINGS_AUTO_REFRESH_INTERVAL_MS + 1));
        assert!(app_shell.should_refresh_settings_on_tick(current_tick_time));

        app_shell.app_state.settings_pane_state.has_loaded_settings_once = true;
        assert!(!app_shell.should_refresh_settings_on_tick(current_tick_time));
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
    fn focused_struct_viewer_reports_status_for_read_only_edit_attempt() {
        let mut app_shell = AppShell::new(Duration::from_millis(100));
        app_shell.app_state.set_focus_to_pane(TuiPane::StructViewer);
        app_shell.app_state.struct_viewer_pane_state.source = StructViewerSource::ScanResults;
        let selected_scan_results = vec![create_scan_result(13)];
        let selected_scan_result_refs = vec![ScanResultRef::new(13)];
        app_shell
            .app_state
            .struct_viewer_pane_state
            .focus_scan_results(&selected_scan_results, selected_scan_result_refs);
        let read_only_field_position = app_shell
            .app_state
            .struct_viewer_pane_state
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| {
                focused_struct
                    .get_fields()
                    .iter()
                    .position(|focused_field| focused_field.get_is_read_only())
            })
            .expect("scan-result struct should include a read-only field");
        app_shell
            .app_state
            .struct_viewer_pane_state
            .selected_field_position = Some(read_only_field_position);
        app_shell.app_state.struct_viewer_pane_state.selected_field_name = app_shell
            .app_state
            .struct_viewer_pane_state
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| {
                focused_struct
                    .get_fields()
                    .get(read_only_field_position)
                    .map(|focused_field| focused_field.get_name().to_string())
            });
        app_shell.app_state.struct_viewer_pane_state.status_message = "Ready.".to_string();
        let initial_pending_edit_text = app_shell
            .app_state
            .struct_viewer_pane_state
            .pending_edit_text
            .clone();
        let mut squalr_engine = SqualrEngine::new(EngineMode::Standalone).expect("engine should initialize for routing test");

        app_shell.handle_focused_pane_event(KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE), &mut squalr_engine);

        assert_eq!(app_shell.app_state.struct_viewer_pane_state.pending_edit_text, initial_pending_edit_text);
        assert!(
            app_shell
                .app_state
                .struct_viewer_pane_state
                .status_message
                .contains("read-only")
        );
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
