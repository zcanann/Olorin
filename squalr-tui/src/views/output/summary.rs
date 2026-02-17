use crate::views::output::pane_state::OutputPaneState;

pub fn build_output_summary_lines(output_pane_state: &OutputPaneState) -> Vec<String> {
    let mut summary_lines = vec![
        "Actions: r refresh log history, x clear, +/- max lines.".to_string(),
        format!("log_line_count={}", output_pane_state.log_lines.len()),
        format!("max_log_lines={}", output_pane_state.max_log_line_count),
        format!("auto_scroll_latest={}", output_pane_state.did_auto_scroll_to_latest),
        format!("status={}", output_pane_state.status_message),
    ];

    let preview_line_count = output_pane_state.log_lines.len().min(8);
    if preview_line_count > 0 {
        summary_lines.push("Recent:".to_string());
        let start_line_index = output_pane_state
            .log_lines
            .len()
            .saturating_sub(preview_line_count);
        for preview_line in &output_pane_state.log_lines[start_line_index..] {
            summary_lines.push(preview_line.clone());
        }
    }

    summary_lines
}
