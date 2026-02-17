use crate::views::scan_results::pane_state::ScanResultsPaneState;

pub fn build_scan_results_summary_lines(scan_results_pane_state: &ScanResultsPaneState) -> Vec<String> {
    let mut summary_lines = vec![
        "[ACT] r query | R refresh-page | [/] page | f freeze | a add | x delete.".to_string(),
        "[NAV] Up/Down/j/k move | Shift+Up/Down range | Home/End.".to_string(),
        "[EDIT] y pull | type value | Enter commit.".to_string(),
        format!(
            "[PAGE] {}/{} | size={} | results={}.",
            scan_results_pane_state.current_page_index,
            scan_results_pane_state.cached_last_page_index,
            scan_results_pane_state.results_per_page,
            scan_results_pane_state.total_result_count
        ),
        format!(
            "[SEL] index={} | selected={} | bytes={}.",
            option_to_compact_text(scan_results_pane_state.selected_result_index),
            scan_results_pane_state.selected_result_count(),
            scan_results_pane_state.total_size_in_bytes
        ),
        format!("[STAT] {}.", scan_results_pane_state.status_message),
        "[ROWS] top=5.".to_string(),
    ];

    if !scan_results_pane_state.pending_value_edit_text.is_empty() {
        summary_lines.insert(5, format!("[VAL] {}.", scan_results_pane_state.pending_value_edit_text));
    }

    summary_lines
}

fn option_to_compact_text<T: std::fmt::Display>(option_value: Option<T>) -> String {
    option_value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}
