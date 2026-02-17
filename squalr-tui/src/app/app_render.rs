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

        let pane_content_height = pane_area.height.saturating_sub(2) as usize;
        let pane_content_width = pane_area.width.saturating_sub(2) as usize;
        let fitted_summary_lines = self
            .fit_summary_lines_to_width(self.app_state.pane_summary_lines(pane, pane_content_height), pane_content_width)
            .into_iter()
            .collect::<Vec<String>>();
        let clamped_summary_lines = Self::clamp_summary_lines_for_entry_safeguard(pane, fitted_summary_lines, pane_content_height);
        let (summary_lines, entry_row_capacity) =
            Self::reconcile_row_telemetry_for_capacity(pane, clamped_summary_lines, pane_content_height, |pane_entry_row_capacity| {
                self.app_state
                    .pane_row_telemetry_line(pane, pane_entry_row_capacity)
            });
        let display_summary_lines = self.fit_summary_lines_to_width(summary_lines, pane_content_width);
        let pane_lines: Vec<Line<'static>> = display_summary_lines.into_iter().map(Line::from).collect();
        let entry_rows = self.app_state.pane_entry_rows(pane, entry_row_capacity);
        let pane_lines = self.append_entry_row_lines(pane_lines, entry_rows, pane_content_width);

        let pane_widget = Paragraph::new(pane_lines)
            .style(TuiTheme::panel_text_style())
            .block(TuiTheme::pane_block(&title, pane, is_focused));
        frame.render_widget(pane_widget, pane_area);
    }

    fn append_entry_row_lines(
        &self,
        mut pane_lines: Vec<Line<'static>>,
        entry_rows: Vec<PaneEntryRow>,
        content_width: usize,
    ) -> Vec<Line<'static>> {
        if entry_rows.is_empty() {
            return pane_lines;
        }

        if !pane_lines.is_empty() {
            pane_lines.push(Line::from(String::new()));
        }
        for entry_row in entry_rows {
            pane_lines.push(Self::render_entry_row(entry_row, content_width));
        }

        pane_lines
    }

    fn render_entry_row(
        entry_row: PaneEntryRow,
        content_width: usize,
    ) -> Line<'static> {
        let marker_style = TuiTheme::pane_entry_marker_style(entry_row.tone);
        let primary_style = TuiTheme::pane_entry_primary_style(entry_row.tone);
        let secondary_style = TuiTheme::pane_entry_secondary_style(entry_row.tone);
        let marker_text = Self::format_marker_prefix(entry_row.marker_text, content_width);
        let marker_prefix_width = marker_text.chars().count();
        let available_content_width = content_width.saturating_sub(marker_prefix_width);
        let (primary_text, secondary_text) = Self::fit_entry_row_content(entry_row.primary_text, entry_row.secondary_text, available_content_width);
        let mut entry_spans = vec![
            Span::styled(marker_text, marker_style),
            Span::styled(primary_text, primary_style),
        ];

        if let Some(secondary_text) = secondary_text {
            entry_spans.push(Span::raw("  "));
            entry_spans.push(Span::styled(secondary_text, secondary_style));
        }

        Line::from(entry_spans)
    }

    fn fit_summary_lines_to_width(
        &self,
        summary_lines: Vec<String>,
        content_width: usize,
    ) -> Vec<String> {
        summary_lines
            .into_iter()
            .map(|summary_line| Self::truncate_line_with_ellipsis(summary_line, content_width))
            .collect()
    }

    fn replace_row_telemetry_line(
        mut summary_lines: Vec<String>,
        row_telemetry_line: Option<String>,
    ) -> Vec<String> {
        let Some(row_telemetry_line) = row_telemetry_line else {
            return summary_lines;
        };
        let Some(row_summary_line_index) = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[ROWS]"))
        else {
            return summary_lines;
        };
        summary_lines[row_summary_line_index] = row_telemetry_line;
        summary_lines
    }

    fn upsert_row_telemetry_line(
        pane: TuiPane,
        mut summary_lines: Vec<String>,
        row_telemetry_line: Option<String>,
        pane_content_height: usize,
    ) -> Vec<String> {
        let Some(row_telemetry_line) = row_telemetry_line else {
            return summary_lines;
        };
        let Some(row_summary_line_index) = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[ROWS]"))
        else {
            if !Self::is_entry_heavy_pane(pane) || pane_content_height == 0 {
                return summary_lines;
            }
            if summary_lines.is_empty() {
                summary_lines.push(row_telemetry_line);
            } else {
                let last_summary_line_index = summary_lines.len() - 1;
                summary_lines[last_summary_line_index] = row_telemetry_line;
            }
            return summary_lines;
        };
        summary_lines[row_summary_line_index] = row_telemetry_line;
        summary_lines
    }

    fn strip_row_telemetry_line(summary_lines: Vec<String>) -> Vec<String> {
        summary_lines
            .into_iter()
            .filter(|summary_line| !summary_line.starts_with("[ROWS]"))
            .collect()
    }

    fn reconcile_row_telemetry_for_capacity<F>(
        pane: TuiPane,
        summary_lines: Vec<String>,
        pane_content_height: usize,
        row_telemetry_line_builder: F,
    ) -> (Vec<String>, usize)
    where
        F: Fn(usize) -> Option<String>,
    {
        let baseline_entry_row_capacity = Self::pane_entry_row_capacity(pane, pane_content_height, summary_lines.len());
        if baseline_entry_row_capacity == 0 {
            return (Self::strip_row_telemetry_line(summary_lines), 0);
        }

        let summary_lines_with_telemetry = Self::upsert_row_telemetry_line(
            pane,
            summary_lines.clone(),
            row_telemetry_line_builder(baseline_entry_row_capacity),
            pane_content_height,
        );
        let telemetry_entry_row_capacity = Self::pane_entry_row_capacity(pane, pane_content_height, summary_lines_with_telemetry.len());
        if telemetry_entry_row_capacity == 0 {
            return (Self::strip_row_telemetry_line(summary_lines), baseline_entry_row_capacity);
        }

        (
            Self::replace_row_telemetry_line(summary_lines_with_telemetry, row_telemetry_line_builder(telemetry_entry_row_capacity)),
            telemetry_entry_row_capacity,
        )
    }

    fn clamp_summary_lines_for_entry_safeguard(
        pane: TuiPane,
        mut summary_lines: Vec<String>,
        pane_content_height: usize,
    ) -> Vec<String> {
        let minimum_entry_row_count = Self::minimum_entry_row_count_for_pane(pane);
        if minimum_entry_row_count == 0 {
            return summary_lines;
        }

        let maximum_summary_line_count = pane_content_height.saturating_sub(minimum_entry_row_count.saturating_add(1));
        if summary_lines.len() > maximum_summary_line_count {
            summary_lines.truncate(maximum_summary_line_count);
        }

        summary_lines
    }

    fn pane_entry_row_capacity(
        pane: TuiPane,
        pane_content_height: usize,
        summary_line_count: usize,
    ) -> usize {
        let separator_line_count = usize::from(summary_line_count > 0);
        let computed_entry_row_capacity = pane_content_height.saturating_sub(summary_line_count.saturating_add(separator_line_count));
        let minimum_entry_row_count = Self::minimum_entry_row_count_for_pane(pane);
        if minimum_entry_row_count == 0 {
            return computed_entry_row_capacity;
        }

        if pane_content_height < minimum_entry_row_count {
            return computed_entry_row_capacity;
        }

        if computed_entry_row_capacity == 0 {
            return 0;
        }

        computed_entry_row_capacity.max(minimum_entry_row_count)
    }

    fn is_entry_heavy_pane(pane: TuiPane) -> bool {
        matches!(pane, TuiPane::ProcessSelector | TuiPane::ScanResults | TuiPane::ProjectExplorer)
    }

    fn minimum_entry_row_count_for_pane(pane: TuiPane) -> usize {
        usize::from(Self::is_entry_heavy_pane(pane))
    }

    fn format_marker_prefix(
        marker_text: String,
        content_width: usize,
    ) -> String {
        if content_width == 0 {
            return String::new();
        }

        if content_width == 1 {
            return Self::single_column_marker(marker_text);
        }

        if content_width == 2 {
            let truncated_marker = Self::truncate_line_with_ellipsis(marker_text, 2);
            return format!("{:>2}", truncated_marker);
        }

        let truncated_marker = Self::truncate_line_with_ellipsis(marker_text, 2);
        format!("{:>2} ", truncated_marker)
    }

    fn single_column_marker(marker_text: String) -> String {
        if marker_text.is_empty() {
            return String::new();
        }

        if let Some(first_visible_marker) = marker_text
            .chars()
            .find(|marker_character| !marker_character.is_whitespace())
        {
            return first_visible_marker.to_string();
        }

        " ".to_string()
    }

    fn fit_entry_row_content(
        primary_text: String,
        secondary_text: Option<String>,
        available_content_width: usize,
    ) -> (String, Option<String>) {
        if available_content_width == 0 {
            return (String::new(), None);
        }

        let truncated_primary = Self::truncate_line_with_ellipsis(primary_text.clone(), available_content_width);
        let Some(secondary_text) = secondary_text else {
            return (truncated_primary, None);
        };

        let secondary_separator_width = 2usize;
        let minimum_secondary_width = 4usize;
        if available_content_width <= secondary_separator_width + minimum_secondary_width {
            return (truncated_primary, None);
        }

        let primary_text_length = primary_text.chars().count();
        let secondary_text_length = secondary_text.chars().count();
        if primary_text_length + secondary_separator_width + secondary_text_length <= available_content_width {
            return (primary_text, Some(secondary_text));
        }

        if available_content_width <= 24 {
            return (truncated_primary, None);
        }

        let primary_width = available_content_width.saturating_sub(secondary_separator_width + minimum_secondary_width);
        if primary_width == 0 {
            return (truncated_primary, None);
        }

        let secondary_width = available_content_width.saturating_sub(primary_width + secondary_separator_width);
        if secondary_width == 0 {
            return (Self::truncate_line_with_ellipsis(primary_text, available_content_width), None);
        }

        let fitted_primary = Self::truncate_line_with_ellipsis(primary_text, primary_width);
        let fitted_secondary = Self::truncate_line_with_ellipsis(secondary_text, secondary_width);
        (fitted_primary, Some(fitted_secondary))
    }

    fn truncate_line_with_ellipsis(
        text: String,
        max_width: usize,
    ) -> String {
        if max_width == 0 {
            return String::new();
        }

        let text_character_count = text.chars().count();
        if text_character_count <= max_width {
            return text;
        }

        if max_width == 1 {
            return ".".to_string();
        }

        let kept_text: String = text.chars().take(max_width - 1).collect();
        format!("{}.", kept_text)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppShell;
    use crate::state::pane::TuiPane;
    use crate::state::pane_entry_row::PaneEntryRow;

    #[test]
    fn truncation_leaves_short_lines_unchanged() {
        assert_eq!(AppShell::truncate_line_with_ellipsis("[STAT] ready.".to_string(), 64), "[STAT] ready.");
    }

    #[test]
    fn truncation_adds_ellipsis_for_narrow_width() {
        assert_eq!(AppShell::truncate_line_with_ellipsis("[STAT] long status text".to_string(), 10), "[STAT] lo.");
    }

    #[test]
    fn truncation_handles_single_character_width() {
        assert_eq!(AppShell::truncate_line_with_ellipsis("[STAT]".to_string(), 1), ".");
    }

    #[test]
    fn entry_marker_prefix_stays_aligned() {
        assert_eq!(AppShell::format_marker_prefix("*".to_string(), 8), " * ");
        assert_eq!(AppShell::format_marker_prefix("LONG".to_string(), 8), "L. ");
    }

    #[test]
    fn entry_marker_prefix_preserves_marker_visibility_in_tiny_widths() {
        assert_eq!(AppShell::format_marker_prefix("*".to_string(), 1), "*");
        assert_eq!(AppShell::format_marker_prefix("*".to_string(), 2), " *");
    }

    #[test]
    fn entry_marker_prefix_uses_visible_character_for_two_character_markers_at_single_width() {
        assert_eq!(AppShell::format_marker_prefix("*+".to_string(), 1), "*");
        assert_eq!(AppShell::format_marker_prefix(" +".to_string(), 1), "+");
        assert_eq!(AppShell::format_marker_prefix(" -".to_string(), 1), "-");
        assert_eq!(AppShell::format_marker_prefix("  ".to_string(), 1), " ");
    }

    #[test]
    fn entry_row_omits_secondary_when_too_narrow() {
        let (primary_text, secondary_text) = AppShell::fit_entry_row_content("primary-text".to_string(), Some("secondary-text".to_string()), 10);

        assert_eq!(primary_text, "primary-t.");
        assert_eq!(secondary_text, None);
    }

    #[test]
    fn entry_row_truncates_primary_and_secondary_when_wide_enough() {
        let (primary_text, secondary_text) = AppShell::fit_entry_row_content("012345678901234567890123456789".to_string(), Some("abcdefghij".to_string()), 28);

        assert_eq!(primary_text, "012345678901234567890.");
        assert_eq!(secondary_text, Some("abc.".to_string()));
    }

    #[test]
    fn entry_row_render_keeps_fixed_marker_column_width() {
        let rendered_line = AppShell::render_entry_row(
            PaneEntryRow::selected("F".to_string(), "primary".to_string(), Some("secondary".to_string())),
            12,
        );
        let rendered_text: String = rendered_line
            .spans
            .iter()
            .map(|span| span.content.to_string())
            .collect();

        assert_eq!(rendered_text, " F primary");
    }

    #[test]
    fn entry_row_render_keeps_marker_visible_at_single_column_width() {
        let rendered_line = AppShell::render_entry_row(PaneEntryRow::selected("*".to_string(), "primary".to_string(), Some("secondary".to_string())), 1);
        let rendered_text: String = rendered_line
            .spans
            .iter()
            .map(|span| span.content.to_string())
            .collect();

        assert_eq!(rendered_text, "*");
    }

    #[test]
    fn entry_heavy_panes_clamp_summary_lines_to_preserve_rows() {
        let summary_lines = vec![
            "[ACT] one".to_string(),
            "[NAV] two".to_string(),
            "[META] three".to_string(),
            "[STAT] four".to_string(),
        ];

        let clamped_summary_lines = AppShell::clamp_summary_lines_for_entry_safeguard(TuiPane::ProcessSelector, summary_lines, 3);

        assert_eq!(clamped_summary_lines.len(), 1);
    }

    #[test]
    fn entry_heavy_panes_allow_row_without_separator_when_summary_omitted() {
        let entry_row_capacity = AppShell::pane_entry_row_capacity(TuiPane::ScanResults, 1, 0);

        assert_eq!(entry_row_capacity, 1);
    }

    #[test]
    fn non_entry_heavy_panes_keep_existing_capacity_behavior() {
        let entry_row_capacity = AppShell::pane_entry_row_capacity(TuiPane::Output, 4, 3);

        assert_eq!(entry_row_capacity, 0);
    }

    #[test]
    fn replaces_rows_telemetry_line_when_present() {
        let summary_lines = vec!["[STAT] ok.".to_string(), "[ROWS] top=5.".to_string()];
        let updated_summary_lines = AppShell::replace_row_telemetry_line(summary_lines, Some("[ROWS] visible=3.".to_string()));

        assert_eq!(updated_summary_lines[1], "[ROWS] visible=3.");
    }

    #[test]
    fn leaves_summary_lines_unchanged_without_rows_marker() {
        let summary_lines = vec!["[STAT] ok.".to_string()];
        let updated_summary_lines = AppShell::replace_row_telemetry_line(summary_lines.clone(), Some("[ROWS] visible=3.".to_string()));

        assert_eq!(updated_summary_lines, summary_lines);
    }

    #[test]
    fn upsert_rows_telemetry_replaces_last_line_for_entry_heavy_panes() {
        let summary_lines = vec!["[ACT] action.".to_string(), "[STAT] ok.".to_string()];
        let updated_summary_lines = AppShell::upsert_row_telemetry_line(TuiPane::ProcessSelector, summary_lines, Some("[ROWS] visible=1.".to_string()), 3);

        assert_eq!(updated_summary_lines[1], "[ROWS] visible=1.");
    }

    #[test]
    fn upsert_rows_telemetry_inserts_single_line_when_summary_empty() {
        let updated_summary_lines = AppShell::upsert_row_telemetry_line(TuiPane::ScanResults, Vec::new(), Some("[ROWS] visible=1.".to_string()), 1);
        let entry_row_capacity = AppShell::pane_entry_row_capacity(TuiPane::ScanResults, 1, updated_summary_lines.len());

        assert_eq!(updated_summary_lines, vec!["[ROWS] visible=1.".to_string()]);
        assert_eq!(entry_row_capacity, 0);
    }

    #[test]
    fn reconcile_rows_telemetry_falls_back_to_row_capacity_when_telemetry_would_starve_rows() {
        let (summary_lines, entry_row_capacity) =
            AppShell::reconcile_row_telemetry_for_capacity(TuiPane::ScanResults, Vec::new(), 1, |pane_entry_row_capacity| {
                Some(format!("[ROWS] visible={}.", pane_entry_row_capacity))
            });

        assert_eq!(summary_lines, Vec::<String>::new());
        assert_eq!(entry_row_capacity, 1);
    }

    #[test]
    fn reconcile_rows_telemetry_strips_rows_when_no_entry_rows_fit() {
        let (summary_lines, entry_row_capacity) = AppShell::reconcile_row_telemetry_for_capacity(
            TuiPane::ProcessSelector,
            vec!["[STAT] ok.".to_string(), "[ROWS] visible=1.".to_string()],
            1,
            |pane_entry_row_capacity| Some(format!("[ROWS] visible={}.", pane_entry_row_capacity)),
        );

        assert_eq!(summary_lines, vec!["[STAT] ok.".to_string()]);
        assert_eq!(entry_row_capacity, 0);
    }

    #[test]
    fn reconcile_rows_telemetry_preserves_rows_marker_when_capacity_allows_rows() {
        let (summary_lines, entry_row_capacity) =
            AppShell::reconcile_row_telemetry_for_capacity(TuiPane::ProcessSelector, vec!["[ACT] refresh.".to_string()], 3, |pane_entry_row_capacity| {
                Some(format!("[ROWS] visible={}.", pane_entry_row_capacity))
            });

        assert_eq!(summary_lines, vec!["[ROWS] visible=1.".to_string()]);
        assert_eq!(entry_row_capacity, 1);
    }
}
