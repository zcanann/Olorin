use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRow;
use crate::state::pane_layout_state::PaneLayoutState;
use crate::views::element_scanner_pane_state::ElementScannerPaneState;
use crate::views::output_pane_state::OutputPaneState;
use crate::views::process_selector::pane_state::ProcessSelectorPaneState;
use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;
use crate::views::scan_results::pane_state::ScanResultsPaneState;
use crate::views::settings_pane_state::SettingsPaneState;
use crate::views::struct_viewer_pane_state::StructViewerPaneState;

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
            TuiPane::ScanResults => self.scan_results_pane_state.summary_lines(),
            TuiPane::ProjectExplorer => self.project_explorer_pane_state.summary_lines(),
            TuiPane::StructViewer => self.struct_viewer_pane_state.summary_lines(),
            TuiPane::Output => self.output_pane_state.summary_lines(),
            TuiPane::Settings => self.settings_pane_state.summary_lines(),
        }
    }

    pub fn pane_entry_rows(
        &self,
        pane: TuiPane,
    ) -> Vec<PaneEntryRow> {
        match pane {
            TuiPane::ProcessSelector => self.process_selector_pane_state.visible_process_entry_rows(),
            TuiPane::ScanResults => self.scan_results_pane_state.visible_scan_result_rows(),
            TuiPane::ProjectExplorer => {
                let mut entry_rows = self.project_explorer_pane_state.visible_project_entry_rows();
                entry_rows.extend(
                    self.project_explorer_pane_state
                        .visible_project_item_entry_rows(),
                );
                entry_rows
            }
            _ => Vec::new(),
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
