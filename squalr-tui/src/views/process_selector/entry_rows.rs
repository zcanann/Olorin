use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::process_selector_pane_state::ProcessSelectorPaneState;

pub fn build_visible_process_entry_rows(process_selector_pane_state: &ProcessSelectorPaneState) -> Vec<PaneEntryRow> {
    let visible_process_count = process_selector_pane_state.process_list_entries.len().min(5);
    let mut entry_rows = Vec::with_capacity(visible_process_count);

    for visible_process_position in 0..visible_process_count {
        if let Some(process_entry) = process_selector_pane_state
            .process_list_entries
            .get(visible_process_position)
        {
            let is_selected_process = process_selector_pane_state.selected_process_list_index == Some(visible_process_position);
            let is_opened_process = process_selector_pane_state.opened_process_identifier == Some(process_entry.get_process_id_raw());
            let marker_text = if is_opened_process { "*".to_string() } else { String::new() };
            let primary_text = process_entry.get_name().to_string();
            let secondary_text = Some(format!("pid={}", process_entry.get_process_id_raw()));

            if is_selected_process {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}
