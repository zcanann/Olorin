/// Stores pagination and selection state for scan results.
#[derive(Clone, Debug)]
pub struct ScanResultsPaneState {
    pub current_page_index: u64,
    pub results_per_page: u64,
    pub selected_result_offset: Option<usize>,
}

impl Default for ScanResultsPaneState {
    fn default() -> Self {
        Self {
            current_page_index: 0,
            results_per_page: 50,
            selected_result_offset: None,
        }
    }
}
