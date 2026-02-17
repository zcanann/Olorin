use crate::state::TuiAppState;
use crate::state::pane::TuiPane;
use crate::theme::TuiTheme;
use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::io::{self, IsTerminal, Stdout};
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
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
    pub process_changed_update_counter: Arc<AtomicU64>,
    pub consumed_process_changed_update_counter: u64,
    pub pending_opened_process_from_event: Arc<RwLock<Option<OpenedProcessInfo>>>,
    pub has_registered_process_changed_listener: bool,
    pub last_scan_results_periodic_refresh_time: Option<Instant>,
    pub last_process_list_auto_refresh_attempt_time: Option<Instant>,
    pub last_project_list_auto_refresh_attempt_time: Option<Instant>,
    pub last_project_items_auto_refresh_attempt_time: Option<Instant>,
    pub last_settings_auto_refresh_attempt_time: Option<Instant>,
}

impl AppShell {
    pub(super) const MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 50;
    pub(super) const MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS: u64 = 5_000;
    pub(super) const MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS: u64 = 1_000;
    pub(super) const MIN_SETTINGS_AUTO_REFRESH_INTERVAL_MS: u64 = 1_000;

    pub fn new(tick_rate: Duration) -> Self {
        Self {
            should_exit: false,
            tick_rate,
            last_tick_time: Instant::now(),
            app_state: TuiAppState::default(),
            scan_results_update_counter: Arc::new(AtomicU64::new(0)),
            consumed_scan_results_update_counter: 0,
            has_registered_scan_results_updated_listener: false,
            process_changed_update_counter: Arc::new(AtomicU64::new(0)),
            consumed_process_changed_update_counter: 0,
            pending_opened_process_from_event: Arc::new(RwLock::new(None)),
            has_registered_process_changed_listener: false,
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
        frame.render_widget(Block::default().style(TuiTheme::app_background_style()), frame.area());

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(5)])
            .split(frame.area());

        let header = Paragraph::new(vec![
            Line::from(Self::engine_mode_header_text(engine_mode)),
            Line::from(self.session_opened_process_metadata_line()),
            Line::from(self.session_active_project_metadata_line()),
        ])
        .style(TuiTheme::panel_text_style())
        .block(TuiTheme::session_block("Session"));
        frame.render_widget(header, vertical_chunks[0]);

        self.draw_pane_layout(frame, vertical_chunks[1]);

        let footer = Paragraph::new(vec![
            Line::from(Self::footer_navigation_controls_line()),
            Line::from(Self::footer_exit_controls_line()),
        ])
        .style(TuiTheme::status_text_style())
        .block(TuiTheme::controls_block("Controls"));
        frame.render_widget(footer, vertical_chunks[2]);
    }

    fn engine_mode_header_text(engine_mode: EngineMode) -> &'static str {
        match engine_mode {
            EngineMode::Standalone => "[MODE] Squalr TUI | Standalone.",
            EngineMode::UnprivilegedHost => "[MODE] Squalr TUI | Unprivileged Host.",
            EngineMode::PrivilegedShell => "[MODE] Squalr TUI | Privileged Shell.",
        }
    }

    fn footer_navigation_controls_line() -> &'static str {
        "[NAV] Tab/Shift+Tab focus | 1-7 jump | Ctrl+1-7 or v toggle | 0 show-all."
    }

    fn footer_exit_controls_line() -> &'static str {
        "[EXIT] q / Esc / Ctrl+C."
    }

    fn session_opened_process_metadata_line(&self) -> String {
        match (
            self.app_state
                .process_selector_pane_state
                .opened_process_name
                .as_deref(),
            self.app_state
                .process_selector_pane_state
                .opened_process_identifier,
        ) {
            (Some(opened_process_name), Some(opened_process_identifier)) => {
                format!("[PROC] {} | PID {}.", opened_process_name, opened_process_identifier)
            }
            (Some(opened_process_name), None) => format!("[PROC] {}.", opened_process_name),
            (None, Some(opened_process_identifier)) => format!("[PROC] PID {}.", opened_process_identifier),
            (None, None) => "[PROC] none.".to_string(),
        }
    }

    fn session_active_project_metadata_line(&self) -> String {
        match (
            self.app_state
                .project_explorer_pane_state
                .active_project_name
                .as_deref(),
            self.app_state
                .project_explorer_pane_state
                .active_project_directory_path
                .as_ref(),
        ) {
            (Some(active_project_name), Some(active_project_directory_path)) => {
                format!(
                    "[PROJ] {} | {}.",
                    active_project_name,
                    Self::condense_path_for_session(active_project_directory_path)
                )
            }
            (Some(active_project_name), None) => format!("[PROJ] {}.", active_project_name),
            (None, Some(active_project_directory_path)) => {
                format!("[PROJ] {}.", Self::condense_path_for_session(active_project_directory_path))
            }
            (None, None) => "[PROJ] none.".to_string(),
        }
    }

    fn condense_path_for_session(path: &Path) -> String {
        let normalized_path = path.to_string_lossy().replace('\\', "/");
        let path_segments: Vec<&str> = normalized_path
            .split('/')
            .filter(|path_segment| !path_segment.is_empty())
            .collect();

        if path_segments.len() <= 2 {
            return normalized_path;
        }

        let second_last_segment = path_segments[path_segments.len() - 2];
        let last_segment = path_segments[path_segments.len() - 1];
        format!(".../{}/{}", second_last_segment, last_segment)
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
}
