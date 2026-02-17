use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::process_selector::entry_rows::build_visible_process_entry_rows;
use crate::views::process_selector::summary::build_process_selector_summary_lines;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;

/// Stores UI state for process selection workflows.
#[derive(Clone, Debug, Default)]
pub struct ProcessSelectorPaneState {
    pub selected_process_identifier: Option<u32>,
    pub selected_process_name: Option<String>,
    pub show_windowed_processes_only: bool,
    pub process_list_entries: Vec<ProcessInfo>,
    pub selected_process_list_index: Option<usize>,
    pub opened_process_identifier: Option<u32>,
    pub opened_process_name: Option<String>,
    pub has_loaded_process_list_once: bool,
    pub is_awaiting_process_list_response: bool,
    pub is_opening_process: bool,
    pub status_message: String,
}

impl ProcessSelectorPaneState {
    pub fn set_windowed_filter(
        &mut self,
        show_windowed_processes_only: bool,
    ) {
        self.show_windowed_processes_only = show_windowed_processes_only;
    }

    pub fn apply_process_list(
        &mut self,
        process_entries: Vec<ProcessInfo>,
    ) {
        let selected_process_identifier_before_refresh = self.selected_process_identifier;
        self.process_list_entries = process_entries;
        self.selected_process_list_index = selected_process_identifier_before_refresh
            .and_then(|selected_process_identifier| {
                self.process_list_entries
                    .iter()
                    .position(|process_entry| process_entry.get_process_id_raw() == selected_process_identifier)
            })
            .or_else(|| if self.process_list_entries.is_empty() { None } else { Some(0) });
        self.update_selected_process_fields();
    }

    pub fn select_next_process(&mut self) {
        if self.process_list_entries.is_empty() {
            self.selected_process_list_index = None;
            self.update_selected_process_fields();
            return;
        }

        let selected_process_index = self.selected_process_list_index.unwrap_or(0);
        let next_process_index = (selected_process_index + 1) % self.process_list_entries.len();
        self.selected_process_list_index = Some(next_process_index);
        self.update_selected_process_fields();
    }

    pub fn select_previous_process(&mut self) {
        if self.process_list_entries.is_empty() {
            self.selected_process_list_index = None;
            self.update_selected_process_fields();
            return;
        }

        let selected_process_index = self.selected_process_list_index.unwrap_or(0);
        let previous_process_index = if selected_process_index == 0 {
            self.process_list_entries.len() - 1
        } else {
            selected_process_index - 1
        };
        self.selected_process_list_index = Some(previous_process_index);
        self.update_selected_process_fields();
    }

    pub fn selected_process_id(&self) -> Option<u32> {
        self.selected_process_list_index
            .and_then(|selected_process_index| self.process_list_entries.get(selected_process_index))
            .map(|process_entry| process_entry.get_process_id_raw())
    }

    pub fn set_opened_process(
        &mut self,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        match opened_process {
            Some(opened_process_info) => {
                self.opened_process_identifier = Some(opened_process_info.get_process_id_raw());
                self.opened_process_name = Some(opened_process_info.get_name().to_string());
            }
            None => {
                self.opened_process_identifier = None;
                self.opened_process_name = None;
            }
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        build_process_selector_summary_lines(self)
    }

    pub fn visible_process_entry_rows(&self) -> Vec<PaneEntryRow> {
        build_visible_process_entry_rows(self)
    }

    fn update_selected_process_fields(&mut self) {
        if let Some(selected_process_index) = self.selected_process_list_index {
            if let Some(selected_process_entry) = self.process_list_entries.get(selected_process_index) {
                self.selected_process_identifier = Some(selected_process_entry.get_process_id_raw());
                self.selected_process_name = Some(selected_process_entry.get_name().to_string());
                return;
            }
        }

        self.selected_process_identifier = None;
        self.selected_process_name = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::views::process_selector_pane_state::ProcessSelectorPaneState;
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;

    #[test]
    fn apply_process_list_selects_first_process() {
        let mut process_selector_pane_state = ProcessSelectorPaneState::default();
        process_selector_pane_state.apply_process_list(vec![
            ProcessInfo::new(101, "alpha.exe".to_string(), true, None),
            ProcessInfo::new(202, "beta.exe".to_string(), true, None),
        ]);

        assert_eq!(process_selector_pane_state.selected_process_list_index, Some(0));
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(101));
        assert_eq!(process_selector_pane_state.selected_process_name, Some("alpha.exe".to_string()));
    }

    #[test]
    fn process_selection_wraps_forward_and_backward() {
        let mut process_selector_pane_state = ProcessSelectorPaneState::default();
        process_selector_pane_state.apply_process_list(vec![
            ProcessInfo::new(101, "alpha.exe".to_string(), true, None),
            ProcessInfo::new(202, "beta.exe".to_string(), true, None),
        ]);

        process_selector_pane_state.select_next_process();
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(202));

        process_selector_pane_state.select_next_process();
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(101));

        process_selector_pane_state.select_previous_process();
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(202));
    }

    #[test]
    fn apply_process_list_preserves_selected_process_by_identifier() {
        let mut process_selector_pane_state = ProcessSelectorPaneState::default();
        process_selector_pane_state.apply_process_list(vec![
            ProcessInfo::new(101, "alpha.exe".to_string(), true, None),
            ProcessInfo::new(202, "beta.exe".to_string(), true, None),
        ]);
        process_selector_pane_state.select_next_process();
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(202));

        process_selector_pane_state.apply_process_list(vec![
            ProcessInfo::new(303, "gamma.exe".to_string(), true, None),
            ProcessInfo::new(202, "beta.exe".to_string(), true, None),
        ]);

        assert_eq!(process_selector_pane_state.selected_process_list_index, Some(1));
        assert_eq!(process_selector_pane_state.selected_process_identifier, Some(202));
    }
}
