use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRowTone;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};

pub struct TuiTheme;

impl TuiTheme {
    pub fn app_background_style() -> Style {
        Style::default()
            .bg(Color::Rgb(14, 18, 24))
            .fg(Color::Rgb(226, 232, 240))
    }

    pub fn panel_text_style() -> Style {
        Style::default().fg(Color::Rgb(214, 222, 235))
    }

    pub fn status_text_style() -> Style {
        Style::default().fg(Color::Rgb(148, 163, 184))
    }

    pub fn pane_entry_marker_style(pane_entry_row_tone: PaneEntryRowTone) -> Style {
        match pane_entry_row_tone {
            PaneEntryRowTone::Selected => Style::default()
                .fg(Color::Rgb(125, 211, 252))
                .add_modifier(Modifier::BOLD),
            PaneEntryRowTone::Normal => Style::default().fg(Color::Rgb(94, 234, 212)),
            PaneEntryRowTone::Disabled => Style::default().fg(Color::Rgb(100, 116, 139)),
        }
    }

    pub fn pane_entry_primary_style(pane_entry_row_tone: PaneEntryRowTone) -> Style {
        match pane_entry_row_tone {
            PaneEntryRowTone::Selected => Style::default()
                .fg(Color::Rgb(241, 245, 249))
                .add_modifier(Modifier::BOLD),
            PaneEntryRowTone::Normal => Style::default().fg(Color::Rgb(214, 222, 235)),
            PaneEntryRowTone::Disabled => Style::default().fg(Color::Rgb(148, 163, 184)),
        }
    }

    pub fn pane_entry_secondary_style(pane_entry_row_tone: PaneEntryRowTone) -> Style {
        match pane_entry_row_tone {
            PaneEntryRowTone::Selected => Style::default().fg(Color::Rgb(191, 219, 254)),
            PaneEntryRowTone::Normal => Style::default().fg(Color::Rgb(148, 163, 184)),
            PaneEntryRowTone::Disabled => Style::default().fg(Color::Rgb(100, 116, 139)),
        }
    }

    pub fn session_block<'a>(title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Rgb(100, 116, 139)))
            .style(
                Style::default()
                    .bg(Color::Rgb(20, 26, 34))
                    .fg(Color::Rgb(226, 232, 240)),
            )
    }

    pub fn controls_block<'a>(title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Rgb(71, 85, 105)))
            .style(
                Style::default()
                    .bg(Color::Rgb(18, 23, 31))
                    .fg(Color::Rgb(184, 202, 222)),
            )
    }

    pub fn pane_block<'a>(
        title: &'a str,
        pane: TuiPane,
        is_focused: bool,
    ) -> Block<'a> {
        let accent_color = Self::pane_accent_color(pane);
        let unfocused_border_color = Self::mix_color(accent_color, Color::Rgb(71, 85, 105));
        let border_color = if is_focused { accent_color } else { unfocused_border_color };
        let background_color = if is_focused {
            Self::mix_color(Color::Rgb(18, 24, 32), accent_color)
        } else {
            Color::Rgb(18, 24, 32)
        };

        let title_style = if is_focused {
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(186, 199, 214))
        };

        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(title_style)
            .border_style(Style::default().fg(border_color))
            .style(
                Style::default()
                    .bg(background_color)
                    .fg(Color::Rgb(214, 222, 235)),
            )
    }

    fn pane_accent_color(pane: TuiPane) -> Color {
        match pane {
            TuiPane::ProcessSelector => Color::Rgb(56, 189, 248),
            TuiPane::ElementScanner => Color::Rgb(34, 197, 94),
            TuiPane::ScanResults => Color::Rgb(245, 158, 11),
            TuiPane::ProjectExplorer => Color::Rgb(99, 102, 241),
            TuiPane::StructViewer => Color::Rgb(236, 72, 153),
            TuiPane::Output => Color::Rgb(20, 184, 166),
            TuiPane::Settings => Color::Rgb(168, 85, 247),
        }
    }

    fn mix_color(
        left_color: Color,
        right_color: Color,
    ) -> Color {
        match (left_color, right_color) {
            (Color::Rgb(left_red, left_green, left_blue), Color::Rgb(right_red, right_green, right_blue)) => Color::Rgb(
                ((left_red as u16 + right_red as u16) / 2) as u8,
                ((left_green as u16 + right_green as u16) / 2) as u8,
                ((left_blue as u16 + right_blue as u16) / 2) as u8,
            ),
            _ => left_color,
        }
    }
}
