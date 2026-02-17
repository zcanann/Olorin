use crate::views::scan_results::pane_state::ScanResultsPaneState;

pub fn build_scan_results_summary_lines(scan_results_pane_state: &ScanResultsPaneState) -> Vec<String> {
    vec![
        "[ACT] r query | R refresh-page | [/] page | f freeze | a add | x delete.".to_string(),
        "[NAV] Up/Down/j/k move | Shift+Up/Down range | Home/End bounds.".to_string(),
        "[EDIT] digits - . append | Backspace | Ctrl+u clear | Enter commit | y pull value.".to_string(),
        format!(
            "page={}/{}",
            scan_results_pane_state.current_page_index, scan_results_pane_state.cached_last_page_index
        ),
        format!("page_size={}", scan_results_pane_state.results_per_page),
        format!("result_count={}", scan_results_pane_state.total_result_count),
        format!("total_size_bytes={}", scan_results_pane_state.total_size_in_bytes),
        format!(
            "selected_index={:?} selected_count={}",
            scan_results_pane_state.selected_result_index,
            scan_results_pane_state.selected_result_count()
        ),
        format!("edit_value={}", scan_results_pane_state.pending_value_edit_text),
        format!("querying={}", scan_results_pane_state.is_querying_scan_results),
        format!("refreshing={}", scan_results_pane_state.is_refreshing_scan_results),
        format!("freezing={}", scan_results_pane_state.is_freezing_scan_results),
        format!("deleting={}", scan_results_pane_state.is_deleting_scan_results),
        format!("adding={}", scan_results_pane_state.is_adding_scan_results_to_project),
        format!("committing={}", scan_results_pane_state.is_committing_value_edit),
        format!("status={}", scan_results_pane_state.status_message),
        "[ROWS] top=5.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::build_scan_results_summary_lines;
    use crate::views::scan_results::pane_state::ScanResultsPaneState;

    #[test]
    fn summary_uses_condensed_marker_group_lead_lines() {
        let scan_results_pane_state = ScanResultsPaneState::default();
        let summary_lines = build_scan_results_summary_lines(&scan_results_pane_state);

        assert!(summary_lines[0].starts_with("[ACT]"));
        assert!(summary_lines[1].starts_with("[NAV]"));
        assert!(summary_lines[2].starts_with("[EDIT]"));
    }
}
