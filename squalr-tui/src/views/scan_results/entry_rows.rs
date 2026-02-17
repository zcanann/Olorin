use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::scan_results::pane_state::ScanResultsPaneState;
use std::ops::RangeInclusive;

pub fn build_visible_scan_result_rows(
    scan_results_pane_state: &ScanResultsPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let selected_result_range = build_selected_result_range(scan_results_pane_state);
    let visible_scan_result_range = build_selection_relative_viewport_range(
        scan_results_pane_state.scan_results.len(),
        scan_results_pane_state.selected_result_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_scan_result_range.len());

    for visible_scan_result_position in visible_scan_result_range {
        if let Some(scan_result) = scan_results_pane_state
            .scan_results
            .get(visible_scan_result_position)
        {
            let is_selected_scan_result = scan_results_pane_state.selected_result_index == Some(visible_scan_result_position);
            let is_in_selected_range = selected_result_range
                .as_ref()
                .map(|selected_range| selected_range.contains(&visible_scan_result_position))
                .unwrap_or(false);
            let freeze_marker = if scan_result.get_is_frozen() { "F" } else { " " };
            let value_preview = scan_result
                .get_current_display_values()
                .first()
                .map(|display_value| display_value.get_anonymous_value_string().to_string())
                .unwrap_or_else(|| "?".to_string());
            let marker_text = format!("{}{}", if is_in_selected_range { "*" } else { " " }, freeze_marker);
            let primary_text = format!(
                "idx={} global={} addr=0x{:X}",
                visible_scan_result_position,
                scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index(),
                scan_result.get_address()
            );
            let secondary_text = Some(format!("type={} value={}", scan_result.get_data_type_ref().get_data_type_id(), value_preview));

            if is_selected_scan_result {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else if value_preview == "?" {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

fn build_selected_result_range(scan_results_pane_state: &ScanResultsPaneState) -> Option<RangeInclusive<usize>> {
    let selection_anchor_position = scan_results_pane_state.selected_result_index?;
    let selection_end_position = scan_results_pane_state
        .selected_range_end_index
        .unwrap_or(selection_anchor_position);
    let (range_start_position, range_end_position) = if selection_anchor_position <= selection_end_position {
        (selection_anchor_position, selection_end_position)
    } else {
        (selection_end_position, selection_anchor_position)
    };

    Some(range_start_position..=range_end_position)
}

#[cfg(test)]
mod tests {
    use crate::views::scan_results::entry_rows::build_visible_scan_result_rows;
    use crate::views::scan_results::pane_state::ScanResultsPaneState;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
    use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
    use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;

    fn create_scan_result(scan_result_global_index: u64) -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            0x1000 + scan_result_global_index,
            DataTypeRef::new("u8"),
            String::new(),
            None,
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(scan_result_global_index),
        );

        ScanResult::new(scan_result_valued, String::new(), 0, None, Vec::new(), false)
    }

    #[test]
    fn scan_result_rows_window_tracks_selected_result_position() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.scan_results = (0..10)
            .map(|scan_result_position| create_scan_result(scan_result_position as u64))
            .collect();
        scan_results_pane_state.selected_result_index = Some(7);
        scan_results_pane_state.selected_range_end_index = Some(7);

        let entry_rows = build_visible_scan_result_rows(&scan_results_pane_state, 5);
        let entry_primary_text: Vec<&str> = entry_rows
            .iter()
            .map(|entry_row| entry_row.primary_text.as_str())
            .collect();

        assert_eq!(
            entry_primary_text,
            vec![
                "idx=5 global=5 addr=0x1005",
                "idx=6 global=6 addr=0x1006",
                "idx=7 global=7 addr=0x1007",
                "idx=8 global=8 addr=0x1008",
                "idx=9 global=9 addr=0x1009",
            ]
        );
    }
}
