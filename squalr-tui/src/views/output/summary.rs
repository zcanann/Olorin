use crate::views::output::pane_state::OutputPaneState;

pub fn build_output_summary_lines(output_pane_state: &OutputPaneState) -> Vec<String> {
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

    let preview_line_count = output_pane_state.log_lines.len().min(8);
    if preview_line_count > 0 {
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
        let summary_lines = build_output_summary_lines(&output_pane_state);

        assert!(summary_lines[0].starts_with("[ACT]"));
    }
}
