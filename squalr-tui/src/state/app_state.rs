use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRow;
use crate::state::pane_layout_state::PaneLayoutState;
use crate::views::element_scanner::pane_state::ElementScannerPaneState;
use crate::views::output::pane_state::OutputPaneState;
use crate::views::output::summary::OUTPUT_FIXED_SUMMARY_LINE_COUNT;
use crate::views::process_selector::pane_state::ProcessSelectorPaneState;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState};
use crate::views::scan_results::pane_state::ScanResultsPaneState;
use crate::views::settings::pane_state::SettingsPaneState;
use crate::views::struct_viewer::pane_state::StructViewerPaneState;
use crate::views::struct_viewer::summary::STRUCT_VIEWER_FIXED_SUMMARY_LINE_COUNT;

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
        pane_content_height: usize,
    ) -> Vec<String> {
        match pane {
            TuiPane::ProcessSelector => self.process_selector_pane_state.summary_lines(),
            TuiPane::ElementScanner => self
                .element_scanner_pane_state
                .summary_lines_with_capacity(pane_content_height),
            TuiPane::ScanResults => self.scan_results_pane_state.summary_lines(),
            TuiPane::ProjectExplorer => self.project_explorer_pane_state.summary_lines(),
            TuiPane::StructViewer => self
                .struct_viewer_pane_state
                .summary_lines(pane_content_height.saturating_sub(STRUCT_VIEWER_FIXED_SUMMARY_LINE_COUNT)),
            TuiPane::Output => self
                .output_pane_state
                .summary_lines(pane_content_height.saturating_sub(OUTPUT_FIXED_SUMMARY_LINE_COUNT)),
            TuiPane::Settings => self
                .settings_pane_state
                .summary_lines_with_capacity(pane_content_height),
        }
    }

    pub fn pane_entry_rows(
        &self,
        pane: TuiPane,
        pane_entry_row_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        match pane {
            TuiPane::ProcessSelector => self
                .process_selector_pane_state
                .visible_process_entry_rows(pane_entry_row_capacity),
            TuiPane::ScanResults => self
                .scan_results_pane_state
                .visible_scan_result_rows(pane_entry_row_capacity),
            TuiPane::ProjectExplorer => {
                let (project_entry_row_capacity, project_item_entry_row_capacity) = self.project_explorer_entry_row_capacities(pane_entry_row_capacity);
                let mut entry_rows = self
                    .project_explorer_pane_state
                    .visible_project_entry_rows(project_entry_row_capacity);
                entry_rows.extend(
                    self.project_explorer_pane_state
                        .visible_project_item_entry_rows(project_item_entry_row_capacity),
                );
                entry_rows
            }
            _ => Vec::new(),
        }
    }

    pub fn pane_row_telemetry_line(
        &self,
        pane: TuiPane,
        pane_entry_row_capacity: usize,
    ) -> Option<String> {
        match pane {
            TuiPane::ProcessSelector | TuiPane::ScanResults => Some(format!("[ROWS] visible={}.", pane_entry_row_capacity)),
            TuiPane::ProjectExplorer => {
                let (project_entry_row_capacity, project_item_entry_row_capacity) = self.project_explorer_entry_row_capacities(pane_entry_row_capacity);
                Some(format!(
                    "[ROWS] projects={} | hierarchy={}.",
                    project_entry_row_capacity, project_item_entry_row_capacity
                ))
            }
            _ => None,
        }
    }

    fn project_explorer_entry_row_capacities(
        &self,
        total_entry_row_capacity: usize,
    ) -> (usize, usize) {
        if total_entry_row_capacity == 0 {
            return (0, 0);
        }

        let project_entry_count = self.project_explorer_pane_state.project_entries.len();
        let project_item_entry_count = self
            .project_explorer_pane_state
            .project_item_visible_entries
            .len();
        if project_entry_count == 0 {
            return (0, total_entry_row_capacity);
        }

        if project_item_entry_count == 0 {
            return (total_entry_row_capacity, 0);
        }

        if total_entry_row_capacity == 1 {
            return match self.project_explorer_pane_state.focus_target {
                ProjectExplorerFocusTarget::ProjectList => (1, 0),
                ProjectExplorerFocusTarget::ProjectHierarchy => (0, 1),
            };
        }

        let mut project_entry_row_capacity = ((total_entry_row_capacity as f32) * 0.33).round() as usize;
        project_entry_row_capacity = project_entry_row_capacity.clamp(1, total_entry_row_capacity.saturating_sub(1));
        let project_item_entry_row_capacity = total_entry_row_capacity.saturating_sub(project_entry_row_capacity);

        (project_entry_row_capacity, project_item_entry_row_capacity)
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
    use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectHierarchyEntry};
    use squalr_engine_api::structures::projects::{project_info::ProjectInfo, project_manifest::ProjectManifest};
    use std::path::PathBuf;

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

    #[test]
    fn project_explorer_entry_rows_respect_total_capacity() {
        let mut app_state = TuiAppState::default();
        app_state.project_explorer_pane_state.project_entries = vec![
            ProjectInfo::new(
                PathBuf::from("C:/Projects/ProjectA/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
            ProjectInfo::new(
                PathBuf::from("C:/Projects/ProjectB/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
        ];
        app_state
            .project_explorer_pane_state
            .project_item_visible_entries = vec![
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("root/item-a.json"),
                display_name: "item-a".to_string(),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            },
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("root/item-b.json"),
                display_name: "item-b".to_string(),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            },
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("root/item-c.json"),
                display_name: "item-c".to_string(),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            },
        ];

        let entry_rows = app_state.pane_entry_rows(TuiPane::ProjectExplorer, 4);

        assert_eq!(entry_rows.len(), 4);
    }

    #[test]
    fn project_explorer_single_row_capacity_prefers_focused_target() {
        let mut app_state = TuiAppState::default();
        app_state.project_explorer_pane_state.project_entries = vec![ProjectInfo::new(
            PathBuf::from("C:/Projects/ProjectA/project/squalr-project.json"),
            None,
            ProjectManifest::new(Vec::new()),
        )];
        app_state
            .project_explorer_pane_state
            .project_item_visible_entries = vec![ProjectHierarchyEntry {
            project_item_path: PathBuf::from("root/item-a.json"),
            display_name: "item-a".to_string(),
            depth: 0,
            is_directory: false,
            is_expanded: false,
            is_activated: false,
        }];
        app_state.project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectHierarchy;

        let entry_rows = app_state.pane_entry_rows(TuiPane::ProjectExplorer, 1);

        assert_eq!(entry_rows.len(), 1);
        assert_eq!(entry_rows[0].primary_text, "item-a");
    }

    #[test]
    fn row_telemetry_uses_split_project_explorer_capacities() {
        let mut app_state = TuiAppState::default();
        app_state.project_explorer_pane_state.project_entries = vec![ProjectInfo::new(
            PathBuf::from("C:/Projects/ProjectA/project/squalr-project.json"),
            None,
            ProjectManifest::new(Vec::new()),
        )];
        app_state
            .project_explorer_pane_state
            .project_item_visible_entries = vec![
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("root/item-a.json"),
                display_name: "item-a".to_string(),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            },
            ProjectHierarchyEntry {
                project_item_path: PathBuf::from("root/item-b.json"),
                display_name: "item-b".to_string(),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            },
        ];

        let row_telemetry_line = app_state.pane_row_telemetry_line(TuiPane::ProjectExplorer, 6);

        assert_eq!(row_telemetry_line, Some("[ROWS] projects=2 | hierarchy=4.".to_string()));
    }
}
