use crate::state::TuiAppState;
use crate::state::pane::TuiPane;
use anyhow::{Context, Result};
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
use std::io::{self, Stdout};
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
        _squalr_engine: &mut SqualrEngine,
    ) -> Result<()> {
        while !self.should_exit {
            terminal_guard
                .terminal
                .draw(|frame| self.draw(frame, engine_mode))
                .context("Failed to draw TUI frame.")?;

            let timeout_duration = self.tick_rate.saturating_sub(self.last_tick_time.elapsed());
            if event::poll(timeout_duration).context("Failed while polling terminal events.")? {
                let incoming_event = event::read().context("Failed while reading terminal event.")?;
                self.handle_event(incoming_event);
            }

            if self.last_tick_time.elapsed() >= self.tick_rate {
                self.on_tick();
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
    ) {
        if let Event::Key(key_event) = incoming_event {
            if key_event.kind != KeyEventKind::Press {
                return;
            }

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
                _ => {}
            }
        }
    }

    fn on_tick(&mut self) {}

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
