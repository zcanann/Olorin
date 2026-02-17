/// Stores UI state for element scanner controls.
#[derive(Clone, Debug)]
pub struct ElementScannerPaneState {
    pub selected_data_type_name: Option<String>,
    pub active_constraint_count: u8,
    pub has_pending_scan_request: bool,
}

impl Default for ElementScannerPaneState {
    fn default() -> Self {
        Self {
            selected_data_type_name: None,
            active_constraint_count: 0,
            has_pending_scan_request: false,
        }
    }
}
