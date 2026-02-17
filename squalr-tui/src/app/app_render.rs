use crate::app::AppShell;
use crate::app::pane_layout::pane_layout_weights;
use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRow;
use crate::theme::TuiTheme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

impl AppShell {
    pub(super) fn draw_pane_layout(
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

        let pane_weights = pane_layout_weights(panes, self.app_state.focused_pane());
        let total_pane_weight = pane_weights.iter().copied().sum::<u16>().max(1) as u32;
        let row_constraints: Vec<Constraint> = pane_weights
            .into_iter()
            .map(|pane_weight| Constraint::Ratio(pane_weight as u32, total_pane_weight))
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
        let entry_rows = self.app_state.pane_entry_rows(pane);
        let pane_lines = self.append_entry_row_lines(pane_lines, entry_rows);

        let pane_widget = Paragraph::new(pane_lines)
            .style(TuiTheme::panel_text_style())
            .block(TuiTheme::pane_block(&title, pane, is_focused));
        frame.render_widget(pane_widget, pane_area);
    }

    fn append_entry_row_lines(
        &self,
        mut pane_lines: Vec<Line<'static>>,
        entry_rows: Vec<PaneEntryRow>,
    ) -> Vec<Line<'static>> {
        if entry_rows.is_empty() {
            return pane_lines;
        }

        pane_lines.push(Line::from(String::new()));
        for entry_row in entry_rows {
            pane_lines.push(self.render_entry_row(entry_row));
        }

        pane_lines
    }

    fn render_entry_row(
        &self,
        entry_row: PaneEntryRow,
    ) -> Line<'static> {
        let marker_style = TuiTheme::pane_entry_marker_style(entry_row.tone);
        let primary_style = TuiTheme::pane_entry_primary_style(entry_row.tone);
        let secondary_style = TuiTheme::pane_entry_secondary_style(entry_row.tone);
        let marker_text = if entry_row.marker_text.is_empty() {
            "  ".to_string()
        } else {
            entry_row.marker_text
        };
        let mut entry_spans = vec![
            Span::styled(format!("{:>2} ", marker_text), marker_style),
            Span::styled(entry_row.primary_text, primary_style),
        ];

        if let Some(secondary_text) = entry_row.secondary_text {
            entry_spans.push(Span::raw("  "));
            entry_spans.push(Span::styled(secondary_text, secondary_style));
        }

        Line::from(entry_spans)
    }
}
