use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
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

struct AppShell {
    should_exit: bool,
    tick_rate: Duration,
    last_tick_time: Instant,
}

impl AppShell {
    fn new(tick_rate: Duration) -> Self {
        Self {
            should_exit: false,
            tick_rate,
            last_tick_time: Instant::now(),
        }
    }

    fn run(
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
                let event = event::read().context("Failed while reading terminal event.")?;
                self.handle_event(event);
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
        let horizontal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        let header_text = match engine_mode {
            EngineMode::Standalone => "Squalr TUI (Standalone)",
            EngineMode::UnprivilegedHost => "Squalr TUI (Unprivileged Host)",
            EngineMode::PrivilegedShell => "Squalr TUI (Privileged Shell)",
        };

        let header = Paragraph::new(header_text)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL).title("Session"));
        frame.render_widget(header, horizontal_chunks[0]);

        let body = Paragraph::new(vec![
            Line::from("App shell initialized."),
            Line::from("Controls: q / Esc / Ctrl+C to exit."),
            Line::from("Next milestone: pane state + keyboard routing."),
        ])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(body, horizontal_chunks[1]);
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
                _ => {}
            }
        }
    }

    fn on_tick(&mut self) {}
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let engine_mode = if args.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    // Start Squalr engine.
    let mut squalr_engine = SqualrEngine::new(engine_mode).context("Fatal error initializing Squalr engine.")?;

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        squalr_engine
            .get_engine_unprivileged_state()
            .as_ref()
            .context("Engine unprivileged state was unavailable in standalone mode.")?;
    } else if engine_mode == EngineMode::UnprivilegedHost {
        log::info!("TUI running in unprivileged host mode.");
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("TUI running as a privileged IPC shell.");
    } else {
        bail!("Unsupported TUI state.");
    }

    let mut terminal_guard = TerminalGuard::new()?;
    let mut app_shell = AppShell::new(Duration::from_millis(100));
    app_shell.run(&mut terminal_guard, engine_mode, &mut squalr_engine)?;

    Ok(())
}
