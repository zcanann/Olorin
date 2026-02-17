use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::scan_results::entry_rows::build_visible_scan_result_rows;
use crate::views::scan_results::summary::build_scan_results_summary_lines;
use squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use std::ops::RangeInclusive;

/// Stores pagination and selection state for scan results.
#[derive(Clone, Debug)]
pub struct ScanResultsPaneState {
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub results_per_page: u64,
    pub total_result_count: u64,
    pub total_size_in_bytes: u64,
    pub scan_results: Vec<ScanResult>,
    pub selected_result_index: Option<usize>,
    pub selected_range_end_index: Option<usize>,
    pub pending_value_edit_text: String,
    pub is_querying_scan_results: bool,
    pub is_refreshing_scan_results: bool,
    pub is_freezing_scan_results: bool,
    pub is_deleting_scan_results: bool,
    pub is_adding_scan_results_to_project: bool,
    pub is_committing_value_edit: bool,
    pub status_message: String,
}

impl ScanResultsPaneState {
    pub fn clear_results(&mut self) {
        self.current_page_index = 0;
        self.cached_last_page_index = 0;
        self.results_per_page = 50;
        self.total_result_count = 0;
        self.total_size_in_bytes = 0;
        self.scan_results.clear();
        self.selected_result_index = None;
        self.selected_range_end_index = None;
        self.pending_value_edit_text = "0".to_string();
        self.status_message = "Scan results cleared.".to_string();
        self.is_querying_scan_results = false;
        self.is_refreshing_scan_results = false;
        self.is_freezing_scan_results = false;
        self.is_deleting_scan_results = false;
        self.is_adding_scan_results_to_project = false;
        self.is_committing_value_edit = false;
    }

    pub fn apply_query_response(
        &mut self,
        scan_results_query_response: ScanResultsQueryResponse,
    ) {
        let selected_scan_result_global_index_before_refresh = self
            .selected_result_index
            .and_then(|selected_result_index| self.scan_results.get(selected_result_index))
            .map(|selected_scan_result| {
                selected_scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index()
            });
        let selected_range_end_scan_result_global_index_before_refresh = self
            .selected_range_end_index
            .and_then(|selected_range_end_index| self.scan_results.get(selected_range_end_index))
            .map(|selected_scan_result| {
                selected_scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index()
            });
        let selected_result_index_before_refresh = self.selected_result_index;
        let selected_range_end_index_before_refresh = self.selected_range_end_index;

        self.current_page_index = scan_results_query_response.page_index;
        self.cached_last_page_index = scan_results_query_response.last_page_index;
        self.results_per_page = scan_results_query_response.page_size;
        self.total_result_count = scan_results_query_response.result_count;
        self.total_size_in_bytes = scan_results_query_response.total_size_in_bytes;
        self.scan_results = scan_results_query_response.scan_results;
        self.selected_result_index = selected_scan_result_global_index_before_refresh
            .and_then(|selected_scan_result_global_index| {
                self.scan_results.iter().position(|scan_result| {
                    scan_result
                        .get_base_result()
                        .get_scan_result_ref()
                        .get_scan_result_global_index()
                        == selected_scan_result_global_index
                })
            })
            .or_else(|| selected_result_index_before_refresh.filter(|selected_result_index| *selected_result_index < self.scan_results.len()))
            .or_else(|| if self.scan_results.is_empty() { None } else { Some(0) });
        self.selected_range_end_index = selected_range_end_scan_result_global_index_before_refresh
            .and_then(|selected_range_end_scan_result_global_index| {
                self.scan_results.iter().position(|scan_result| {
                    scan_result
                        .get_base_result()
                        .get_scan_result_ref()
                        .get_scan_result_global_index()
                        == selected_range_end_scan_result_global_index
                })
            })
            .or_else(|| selected_range_end_index_before_refresh.filter(|selected_range_end_index| *selected_range_end_index < self.scan_results.len()));
        self.clamp_selection_to_bounds();
        self.sync_pending_value_edit_from_selection();
    }

    pub fn apply_refreshed_results(
        &mut self,
        refreshed_scan_results: Vec<ScanResult>,
    ) {
        self.scan_results = refreshed_scan_results;
        self.clamp_selection_to_bounds();
        self.sync_pending_value_edit_from_selection();
    }

    pub fn set_current_page_index(
        &mut self,
        requested_page_index: u64,
    ) -> bool {
        let clamped_page_index = requested_page_index.clamp(0, self.cached_last_page_index);
        if clamped_page_index == self.current_page_index {
            return false;
        }

        self.current_page_index = clamped_page_index;
        self.selected_result_index = None;
        self.selected_range_end_index = None;
        true
    }

    pub fn set_selected_range_end_to_current(&mut self) {
        if self.selected_result_index.is_some() {
            self.selected_range_end_index = self.selected_result_index;
        }
    }

    pub fn select_next_result(
        &mut self,
        keep_existing_range_anchor: bool,
    ) {
        if self.scan_results.is_empty() {
            self.selected_result_index = None;
            self.selected_range_end_index = None;
            return;
        }

        let selected_result_index = self.selected_result_index.unwrap_or(0);
        let next_result_index = (selected_result_index + 1) % self.scan_results.len();
        self.selected_result_index = Some(next_result_index);
        if !keep_existing_range_anchor {
            self.selected_range_end_index = None;
        }
        self.sync_pending_value_edit_from_selection();
    }

    pub fn select_previous_result(
        &mut self,
        keep_existing_range_anchor: bool,
    ) {
        if self.scan_results.is_empty() {
            self.selected_result_index = None;
            self.selected_range_end_index = None;
            return;
        }

        let selected_result_index = self.selected_result_index.unwrap_or(0);
        let previous_result_index = if selected_result_index == 0 {
            self.scan_results.len() - 1
        } else {
            selected_result_index - 1
        };
        self.selected_result_index = Some(previous_result_index);
        if !keep_existing_range_anchor {
            self.selected_range_end_index = None;
        }
        self.sync_pending_value_edit_from_selection();
    }

    pub fn select_first_result(
        &mut self,
        keep_existing_range_anchor: bool,
    ) {
        if self.scan_results.is_empty() {
            self.selected_result_index = None;
            self.selected_range_end_index = None;
            return;
        }

        self.selected_result_index = Some(0);
        if !keep_existing_range_anchor {
            self.selected_range_end_index = None;
        }
        self.sync_pending_value_edit_from_selection();
    }

    pub fn select_last_result(
        &mut self,
        keep_existing_range_anchor: bool,
    ) {
        if self.scan_results.is_empty() {
            self.selected_result_index = None;
            self.selected_range_end_index = None;
            return;
        }

        self.selected_result_index = Some(self.scan_results.len() - 1);
        if !keep_existing_range_anchor {
            self.selected_range_end_index = None;
        }
        self.sync_pending_value_edit_from_selection();
    }

    pub fn selected_scan_result_refs(&self) -> Vec<ScanResultRef> {
        let Some(selected_range) = self.selected_result_range() else {
            return Vec::new();
        };

        selected_range
            .filter_map(|selected_result_index| self.scan_results.get(selected_result_index))
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect()
    }

    pub fn selected_scan_results(&self) -> Vec<ScanResult> {
        let Some(selected_range) = self.selected_result_range() else {
            return Vec::new();
        };

        selected_range
            .filter_map(|selected_result_position| self.scan_results.get(selected_result_position))
            .cloned()
            .collect()
    }

    pub fn selected_result_count(&self) -> usize {
        let Some(selected_range) = self.selected_result_range() else {
            return 0;
        };

        selected_range.count()
    }

    pub fn any_selected_result_frozen(&self) -> bool {
        let Some(selected_range) = self.selected_result_range() else {
            return false;
        };

        selected_range
            .filter_map(|selected_result_index| self.scan_results.get(selected_result_index))
            .any(ScanResult::get_is_frozen)
    }

    pub fn selected_result_current_value_text(&self) -> Option<String> {
        let selected_result_index = self.selected_result_index?;
        let selected_result = self.scan_results.get(selected_result_index)?;

        if let Some(current_display_value) = selected_result.get_current_display_value(AnonymousValueStringFormat::Decimal) {
            return Some(current_display_value.get_anonymous_value_string().to_string());
        }

        if let Some(current_display_value) = selected_result.get_current_display_values().first() {
            return Some(current_display_value.get_anonymous_value_string().to_string());
        }

        if let Some(recently_read_display_value) = selected_result.get_recently_read_display_value(AnonymousValueStringFormat::Decimal) {
            return Some(
                recently_read_display_value
                    .get_anonymous_value_string()
                    .to_string(),
            );
        }

        selected_result
            .get_recently_read_display_values()
            .first()
            .map(|recently_read_display_value| {
                recently_read_display_value
                    .get_anonymous_value_string()
                    .to_string()
            })
    }

    pub fn sync_pending_value_edit_from_selection(&mut self) {
        if let Some(current_value_text) = self.selected_result_current_value_text() {
            self.pending_value_edit_text = current_value_text;
        } else if self.pending_value_edit_text.is_empty() {
            self.pending_value_edit_text = "0".to_string();
        }
    }

    pub fn append_pending_value_edit_character(
        &mut self,
        value_character: char,
    ) {
        if !Self::is_supported_value_edit_character(value_character) {
            return;
        }

        if self.pending_value_edit_text == "0" && value_character.is_ascii_digit() {
            self.pending_value_edit_text.clear();
        }

        self.pending_value_edit_text.push(value_character);
    }

    pub fn backspace_pending_value_edit(&mut self) {
        self.pending_value_edit_text.pop();

        if self.pending_value_edit_text.is_empty() {
            self.pending_value_edit_text = "0".to_string();
        }
    }

    pub fn clear_pending_value_edit(&mut self) {
        self.pending_value_edit_text = "0".to_string();
    }

    pub fn summary_lines(&self) -> Vec<String> {
        build_scan_results_summary_lines(self)
    }

    pub fn visible_scan_result_rows(&self) -> Vec<PaneEntryRow> {
        build_visible_scan_result_rows(self)
    }

    fn selected_result_range(&self) -> Option<RangeInclusive<usize>> {
        let selection_anchor_index = self.selected_result_index?;
        let selection_end_index = self.selected_range_end_index.unwrap_or(selection_anchor_index);
        let (range_low_index, range_high_index) = if selection_anchor_index <= selection_end_index {
            (selection_anchor_index, selection_end_index)
        } else {
            (selection_end_index, selection_anchor_index)
        };

        Some(range_low_index..=range_high_index)
    }

    fn clamp_selection_to_bounds(&mut self) {
        if self.scan_results.is_empty() {
            self.selected_result_index = None;
            self.selected_range_end_index = None;
            return;
        }

        if let Some(selected_result_index) = self.selected_result_index {
            self.selected_result_index = Some(selected_result_index.min(self.scan_results.len() - 1));
        }

        if let Some(selected_range_end_index) = self.selected_range_end_index {
            self.selected_range_end_index = Some(selected_range_end_index.min(self.scan_results.len() - 1));
        }
    }

    fn is_supported_value_edit_character(value_character: char) -> bool {
        value_character.is_ascii_digit() || value_character == '-' || value_character == '.'
    }
}

impl Default for ScanResultsPaneState {
    fn default() -> Self {
        Self {
            current_page_index: 0,
            cached_last_page_index: 0,
            results_per_page: 50,
            total_result_count: 0,
            total_size_in_bytes: 0,
            scan_results: Vec::new(),
            selected_result_index: None,
            selected_range_end_index: None,
            pending_value_edit_text: "0".to_string(),
            is_querying_scan_results: false,
            is_refreshing_scan_results: false,
            is_freezing_scan_results: false,
            is_deleting_scan_results: false,
            is_adding_scan_results_to_project: false,
            is_committing_value_edit: false,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::views::scan_results_pane_state::ScanResultsPaneState;
    use squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
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

    fn create_query_response(
        page_index: u64,
        scan_result_global_indices: &[u64],
    ) -> ScanResultsQueryResponse {
        ScanResultsQueryResponse {
            page_index,
            last_page_index: page_index,
            page_size: scan_result_global_indices.len() as u64,
            result_count: scan_result_global_indices.len() as u64,
            total_size_in_bytes: 0,
            scan_results: scan_result_global_indices
                .iter()
                .map(|scan_result_global_index| create_scan_result(*scan_result_global_index))
                .collect(),
        }
    }

    #[test]
    fn selected_scan_result_refs_uses_range_bounds() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.scan_results = [10, 11, 12, 13]
            .iter()
            .map(|scan_result_global_index| create_scan_result(*scan_result_global_index))
            .collect();
        scan_results_pane_state.selected_result_index = Some(1);
        scan_results_pane_state.selected_range_end_index = Some(2);

        let selected_scan_result_global_indices = scan_results_pane_state
            .selected_scan_result_refs()
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();

        assert_eq!(selected_scan_result_global_indices, vec![11, 12]);
    }

    #[test]
    fn set_current_page_index_clears_selection_when_page_changes() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.cached_last_page_index = 9;
        scan_results_pane_state.selected_result_index = Some(3);
        scan_results_pane_state.selected_range_end_index = Some(4);

        let did_change_page = scan_results_pane_state.set_current_page_index(5);

        assert!(did_change_page);
        assert_eq!(scan_results_pane_state.current_page_index, 5);
        assert!(scan_results_pane_state.selected_result_index.is_none());
        assert!(scan_results_pane_state.selected_range_end_index.is_none());
    }

    #[test]
    fn set_current_page_index_returns_false_when_index_unchanged() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.current_page_index = 2;
        scan_results_pane_state.cached_last_page_index = 9;

        let did_change_page = scan_results_pane_state.set_current_page_index(2);

        assert!(!did_change_page);
    }

    #[test]
    fn apply_query_response_restores_selected_result_by_scan_result_ref() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.scan_results = [10, 11, 12]
            .iter()
            .map(|scan_result_global_index| create_scan_result(*scan_result_global_index))
            .collect();
        scan_results_pane_state.selected_result_index = Some(1);
        scan_results_pane_state.selected_range_end_index = Some(2);

        scan_results_pane_state.apply_query_response(create_query_response(0, &[12, 10, 11]));

        assert_eq!(scan_results_pane_state.selected_result_index, Some(2));
        assert_eq!(scan_results_pane_state.selected_range_end_index, Some(0));
    }

    #[test]
    fn apply_query_response_defaults_to_first_result_when_previous_selection_missing() {
        let mut scan_results_pane_state = ScanResultsPaneState::default();
        scan_results_pane_state.scan_results = [10, 11]
            .iter()
            .map(|scan_result_global_index| create_scan_result(*scan_result_global_index))
            .collect();
        scan_results_pane_state.selected_result_index = Some(1);
        scan_results_pane_state.selected_range_end_index = Some(1);

        scan_results_pane_state.apply_query_response(create_query_response(0, &[99]));

        assert_eq!(scan_results_pane_state.selected_result_index, Some(0));
        assert_eq!(scan_results_pane_state.selected_range_end_index, None);
    }
}
