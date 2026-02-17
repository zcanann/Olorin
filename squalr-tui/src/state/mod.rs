pub mod element_scanner_pane_state;
pub mod output_pane_state;
pub mod pane;
pub mod pane_layout_state;
pub mod process_selector_pane_state;
pub mod project_explorer_pane_state;
pub mod scan_results_pane_state;
pub mod settings_pane_state;
pub mod struct_viewer_pane_state;

use crate::state::element_scanner_pane_state::ElementScannerPaneState;
use crate::state::output_pane_state::OutputPaneState;
use crate::state::pane::TuiPane;
use crate::state::pane_layout_state::PaneLayoutState;
use crate::state::process_selector_pane_state::ProcessSelectorPaneState;
use crate::state::project_explorer_pane_state::ProjectExplorerPaneState;
use crate::state::scan_results_pane_state::ScanResultsPaneState;
use crate::state::settings_pane_state::SettingsPaneState;
use crate::state::struct_viewer_pane_state::StructViewerPaneState;

/// Root state container for TUI panes.
#[derive(Clone, Debug, Default)]
pub struct TuiAppState {
    pub pane_layout_state: PaneLayoutState,
    pub process_selector_pane_state: ProcessSelectorPaneState,
    pub element_scanner_pane_state: ElementScannerPaneState,
    pub scan_results_pane_state: ScanResultsPaneState,
    pub project_explorer_pane_state: ProjectExplorerPaneState,
    pub struct_viewer_pane_state: StructViewerPaneState,
    pub output_pane_state: OutputPaneState,
    pub settings_pane_state: SettingsPaneState,
}

impl TuiAppState {
    pub fn focused_pane(&self) -> TuiPane {
        self.pane_layout_state.focused_pane
    }

    pub fn is_pane_visible(
        &self,
        pane: TuiPane,
    ) -> bool {
        self.pane_layout_state.is_pane_visible(pane)
    }

    pub fn visible_panes_in_order(&self) -> Vec<TuiPane> {
        self.pane_layout_state.visible_panes_in_order()
    }

    pub fn set_focus_to_pane(
        &mut self,
        pane: TuiPane,
    ) {
        let pane_was_made_visible = self.pane_layout_state.set_pane_visibility(pane, true);
        if pane_was_made_visible {
            self.pane_layout_state.focused_pane = pane;
        }
    }

    pub fn cycle_focus_forward(&mut self) {
        self.cycle_focus(true);
    }

    pub fn cycle_focus_backward(&mut self) {
        self.cycle_focus(false);
    }

    pub fn toggle_focused_pane_visibility(&mut self) -> bool {
        self.pane_layout_state
            .toggle_pane_visibility(self.focused_pane())
    }

    pub fn toggle_pane_visibility(
        &mut self,
        pane: TuiPane,
    ) -> bool {
        self.pane_layout_state.toggle_pane_visibility(pane)
    }

    pub fn show_all_panes(&mut self) {
        self.pane_layout_state.show_all_panes();
    }

    pub fn pane_summary_lines(
        &self,
        pane: TuiPane,
    ) -> Vec<String> {
        match pane {
            TuiPane::ProcessSelector => self.process_selector_pane_state.summary_lines(),
            TuiPane::ElementScanner => self.element_scanner_pane_state.summary_lines(),
            TuiPane::ScanResults => {
                vec![
                    format!("page={}", self.scan_results_pane_state.current_page_index),
                    format!("page_size={}", self.scan_results_pane_state.results_per_page),
                    format!("selected_offset={:?}", self.scan_results_pane_state.selected_result_offset),
                ]
            }
            TuiPane::ProjectExplorer => {
                vec![
                    format!("active_project={:?}", self.project_explorer_pane_state.active_project_name),
                    format!("selected_item={:?}", self.project_explorer_pane_state.selected_item_path),
                    format!("expanded={}", self.project_explorer_pane_state.is_hierarchy_expanded),
                ]
            }
            TuiPane::StructViewer => {
                vec![
                    format!("selected_struct={:?}", self.struct_viewer_pane_state.selected_struct_name),
                    format!("selected_field={:?}", self.struct_viewer_pane_state.selected_field_name),
                    format!("uncommitted_edit={}", self.struct_viewer_pane_state.has_uncommitted_edit),
                ]
            }
            TuiPane::Output => {
                vec![
                    format!("log_line_count={}", self.output_pane_state.log_lines.len()),
                    format!("max_log_lines={}", self.output_pane_state.max_log_line_count),
                ]
            }
            TuiPane::Settings => {
                vec![
                    format!("category={:?}", self.settings_pane_state.selected_category),
                    format!("pending_changes={}", self.settings_pane_state.has_pending_changes),
                    format!("category_count={}", crate::state::settings_pane_state::SettingsCategory::all_categories().len()),
                ]
            }
        }
    }

    fn cycle_focus(
        &mut self,
        is_forward_direction: bool,
    ) {
        let visible_panes = self.visible_panes_in_order();
        if visible_panes.is_empty() {
            return;
        }

        let focused_pane = self.focused_pane();
        let focused_visible_index = visible_panes
            .iter()
            .position(|visible_pane| *visible_pane == focused_pane)
            .unwrap_or(0);

        let next_visible_index = if is_forward_direction {
            (focused_visible_index + 1) % visible_panes.len()
        } else if focused_visible_index == 0 {
            visible_panes.len() - 1
        } else {
            focused_visible_index - 1
        };

        self.pane_layout_state.focused_pane = visible_panes[next_visible_index];
    }
}

#[cfg(test)]
mod tests {
    use crate::state::TuiAppState;
    use crate::state::pane::TuiPane;

    #[test]
    fn cycle_focus_forward_skips_hidden_panes() {
        let mut app_state = TuiAppState::default();
        app_state.toggle_pane_visibility(TuiPane::ElementScanner);
        app_state.toggle_pane_visibility(TuiPane::ScanResults);
        app_state.set_focus_to_pane(TuiPane::ProcessSelector);

        app_state.cycle_focus_forward();

        assert_eq!(app_state.focused_pane(), TuiPane::ProjectExplorer);
    }

    #[test]
    fn hiding_last_visible_pane_is_rejected() {
        let mut app_state = TuiAppState::default();
        for pane in TuiPane::all_panes()
            .into_iter()
            .filter(|pane| *pane != TuiPane::Settings)
        {
            let toggle_succeeded = app_state.toggle_pane_visibility(pane);
            assert!(toggle_succeeded);
        }

        let final_toggle_succeeded = app_state.toggle_pane_visibility(TuiPane::Settings);

        assert!(!final_toggle_succeeded);
        assert!(app_state.is_pane_visible(TuiPane::Settings));
    }

    #[test]
    fn focusing_hidden_pane_makes_it_visible() {
        let mut app_state = TuiAppState::default();
        let hide_succeeded = app_state.toggle_pane_visibility(TuiPane::Output);
        assert!(hide_succeeded);
        assert!(!app_state.is_pane_visible(TuiPane::Output));

        app_state.set_focus_to_pane(TuiPane::Output);

        assert!(app_state.is_pane_visible(TuiPane::Output));
        assert_eq!(app_state.focused_pane(), TuiPane::Output);
    }
}
