use crate::views::process_selector::pane_state::ProcessSelectorPaneState;

pub fn build_process_selector_summary_lines(process_selector_pane_state: &ProcessSelectorPaneState) -> Vec<String> {
    vec![
        "[ACT] r refresh | w windowed/full | Enter/o open.".to_string(),
        "[NAV] Up/Down or j/k select.".to_string(),
        format!(
            "[MODE] windowed_only={} | loaded_once={}.",
            process_selector_pane_state.show_windowed_processes_only, process_selector_pane_state.has_loaded_process_list_once
        ),
        format!(
            "[LIST] count={} | awaiting={} | opening={}.",
            process_selector_pane_state.process_list_entries.len(),
            process_selector_pane_state.is_awaiting_process_list_response,
            process_selector_pane_state.is_opening_process
        ),
        format!(
            "[SEL] id={} | name={}.",
            option_to_compact_text(process_selector_pane_state.selected_process_identifier),
            option_string_to_compact_text(process_selector_pane_state.selected_process_name.as_deref())
        ),
        format!(
            "[OPEN] id={} | name={}.",
            option_to_compact_text(process_selector_pane_state.opened_process_identifier),
            option_string_to_compact_text(process_selector_pane_state.opened_process_name.as_deref())
        ),
        format!("[STAT] {}.", process_selector_pane_state.status_message),
        "[ROWS] top=5.".to_string(),
    ]
}

fn option_to_compact_text<T: std::fmt::Display>(option_value: Option<T>) -> String {
    option_value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn option_string_to_compact_text(option_text: Option<&str>) -> String {
    option_text
        .map(|text| format!("\"{}\"", text))
        .unwrap_or_else(|| "none".to_string())
}
