use crate::views::scan_results::pane_state::ScanResultsPaneState;

pub fn build_scan_results_summary_lines(scan_results_pane_state: &ScanResultsPaneState) -> Vec<String> {
    vec![
        "[ACT] r query | R refresh-page | [/] page | f freeze | a add | x delete.".to_string(),
        "[NAV] Up/Down/j/k move | Shift+Up/Down range | Home/End bounds.".to_string(),
        "[EDIT] digits - . append | Backspace | Ctrl+u clear | Enter commit | y pull value.".to_string(),
        format!(
            "[PAGE] current={}/{}",
            scan_results_pane_state.current_page_index, scan_results_pane_state.cached_last_page_index
        ),
        format!(
            "[META] page_size={} | result_count={} | total_bytes={}.",
            scan_results_pane_state.results_per_page, scan_results_pane_state.total_result_count, scan_results_pane_state.total_size_in_bytes
        ),
        format!(
            "[SEL] index={} | selected_count={}.",
            option_to_compact_text(scan_results_pane_state.selected_result_index),
            scan_results_pane_state.selected_result_count()
        ),
        format!("[VAL] {}.", scan_results_pane_state.pending_value_edit_text),
        format!(
            "[OPS] query={} | refresh={} | freeze={} | delete={} | add={} | commit={}.",
            scan_results_pane_state.is_querying_scan_results,
            scan_results_pane_state.is_refreshing_scan_results,
            scan_results_pane_state.is_freezing_scan_results,
            scan_results_pane_state.is_deleting_scan_results,
            scan_results_pane_state.is_adding_scan_results_to_project,
            scan_results_pane_state.is_committing_value_edit
        ),
        format!("[STAT] {}.", scan_results_pane_state.status_message),
        "[ROWS] top=5.".to_string(),
    ]
}

fn option_to_compact_text<T: std::fmt::Display>(option_value: Option<T>) -> String {
    option_value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}
