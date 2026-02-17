use crate::views::output::pane_state::OutputPaneState;

pub const OUTPUT_FIXED_SUMMARY_LINE_COUNT: usize = 3;

pub fn build_output_summary_lines(
    output_pane_state: &OutputPaneState,
    log_preview_capacity: usize,
) -> Vec<String> {
    let mut summary_lines = vec![
        "[ACT] r refresh-log | x clear | +/- max-lines.".to_string(),
        format!(
            "[META] log_line_count={} | max_log_lines={} | auto_scroll_latest={}.",
            output_pane_state.log_lines.len(),
            output_pane_state.max_log_line_count,
            output_pane_state.did_auto_scroll_to_latest
        ),
        format!("[STAT] {}.", output_pane_state.status_message),
    ];

    if log_preview_capacity > 0 && !output_pane_state.log_lines.is_empty() {
        let preview_line_count = output_pane_state
            .log_lines
            .len()
            .min(log_preview_capacity.saturating_sub(1));
        summary_lines.push("[RECENT]".to_string());
        let start_line_index = output_pane_state
            .log_lines
            .len()
            .saturating_sub(preview_line_count);
        for preview_line in &output_pane_state.log_lines[start_line_index..] {
            summary_lines.push(format!("[LOG] {}", preview_line));
        }
    }

    summary_lines
}

#[cfg(test)]
mod tests {
    use super::build_output_summary_lines;
    use crate::views::output::pane_state::OutputPaneState;

    #[test]
    fn summary_uses_condensed_marker_group_lead_lines() {
        let output_pane_state = OutputPaneState::default();
        let summary_lines = build_output_summary_lines(&output_pane_state, 8);

        assert!(summary_lines[0].starts_with("[ACT]"));
    }

    #[test]
    fn preview_capacity_zero_hides_recent_log_section() {
        let mut output_pane_state = OutputPaneState::default();
        output_pane_state.log_lines = vec!["[INFO] line-1".to_string(), "[INFO] line-2".to_string()];

        let summary_lines = build_output_summary_lines(&output_pane_state, 0);
        let recent_line_count = summary_lines
            .iter()
            .filter(|summary_line| summary_line.starts_with("[RECENT]") || summary_line.starts_with("[LOG]"))
            .count();

        assert_eq!(recent_line_count, 0);
    }
}
