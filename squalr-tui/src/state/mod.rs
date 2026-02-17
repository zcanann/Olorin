pub mod element_scanner_pane_state;
pub mod output_pane_state;
pub mod process_selector_pane_state;
pub mod project_explorer_pane_state;
pub mod scan_results_pane_state;
pub mod settings_pane_state;
pub mod struct_viewer_pane_state;

use crate::state::element_scanner_pane_state::ElementScannerPaneState;
use crate::state::output_pane_state::OutputPaneState;
use crate::state::process_selector_pane_state::ProcessSelectorPaneState;
use crate::state::project_explorer_pane_state::ProjectExplorerPaneState;
use crate::state::scan_results_pane_state::ScanResultsPaneState;
use crate::state::settings_pane_state::SettingsPaneState;
use crate::state::struct_viewer_pane_state::StructViewerPaneState;

/// Root state container for TUI panes.
#[derive(Clone, Debug, Default)]
pub struct TuiAppState {
    pub process_selector_pane_state: ProcessSelectorPaneState,
    pub element_scanner_pane_state: ElementScannerPaneState,
    pub scan_results_pane_state: ScanResultsPaneState,
    pub project_explorer_pane_state: ProjectExplorerPaneState,
    pub struct_viewer_pane_state: StructViewerPaneState,
    pub output_pane_state: OutputPaneState,
    pub settings_pane_state: SettingsPaneState,
}

impl TuiAppState {
    pub fn pane_count(&self) -> usize {
        7
    }

    pub fn status_summary_lines(&self) -> Vec<String> {
        vec![
            format!(
                "Process selector: selected_id={:?}, selected_name={:?}, windowed_only={}.",
                self.process_selector_pane_state.selected_process_identifier,
                self.process_selector_pane_state.selected_process_name,
                self.process_selector_pane_state.show_windowed_processes_only
            ),
            format!(
                "Element scanner: data_type={:?}, constraints={}, pending_scan={}.",
                self.element_scanner_pane_state.selected_data_type_name,
                self.element_scanner_pane_state.active_constraint_count,
                self.element_scanner_pane_state.has_pending_scan_request
            ),
            format!(
                "Scan results: page={}, page_size={}, selected_offset={:?}.",
                self.scan_results_pane_state.current_page_index,
                self.scan_results_pane_state.results_per_page,
                self.scan_results_pane_state.selected_result_offset
            ),
            format!(
                "Project explorer: active_project={:?}, selected_item={:?}, hierarchy_expanded={}.",
                self.project_explorer_pane_state.active_project_name,
                self.project_explorer_pane_state.selected_item_path,
                self.project_explorer_pane_state.is_hierarchy_expanded
            ),
            format!(
                "Struct viewer: selected_struct={:?}, selected_field={:?}, uncommitted_edit={}.",
                self.struct_viewer_pane_state.selected_struct_name,
                self.struct_viewer_pane_state.selected_field_name,
                self.struct_viewer_pane_state.has_uncommitted_edit
            ),
            format!(
                "Output: log_line_count={}, max_log_lines={}.",
                self.output_pane_state.log_lines.len(),
                self.output_pane_state.max_log_line_count
            ),
            format!(
                "Settings: category={:?}, pending_changes={}, category_count={}.",
                self.settings_pane_state.selected_category,
                self.settings_pane_state.has_pending_changes,
                crate::state::settings_pane_state::SettingsCategory::all_categories().len()
            ),
        ]
    }
}
