use crate::views::process_selector::pane_state::ProcessSelectorPaneState;

pub fn build_process_selector_summary_lines(process_selector_pane_state: &ProcessSelectorPaneState) -> Vec<String> {
    vec![
        "Actions: r refresh, w windowed/full, Up/Down select, Enter open.".to_string(),
        format!("windowed_only={}", process_selector_pane_state.show_windowed_processes_only),
        format!("list_count={}", process_selector_pane_state.process_list_entries.len()),
        format!("selected_id={:?}", process_selector_pane_state.selected_process_identifier),
        format!("selected_name={:?}", process_selector_pane_state.selected_process_name),
        format!("opened_id={:?}", process_selector_pane_state.opened_process_identifier),
        format!("opened_name={:?}", process_selector_pane_state.opened_process_name),
        format!("loaded_once={}", process_selector_pane_state.has_loaded_process_list_once),
        format!("awaiting_list={}", process_selector_pane_state.is_awaiting_process_list_response),
        format!("opening_process={}", process_selector_pane_state.is_opening_process),
        format!("status={}", process_selector_pane_state.status_message),
        "Entries (top 5).".to_string(),
    ]
}
