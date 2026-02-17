use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::process_selector::entry_rows::build_visible_process_entry_rows;
use crate::views::process_selector::summary::build_process_selector_summary_lines;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;

/// Stores text input mode for process selector search workflows.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProcessSelectorInputMode {
    #[default]
    None,
    Search,
}

/// Stores UI state for process selection workflows.
#[derive(Clone, Debug)]
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
    pub is_process_selector_view_active: bool,
    pub input_mode: ProcessSelectorInputMode,
    pub pending_search_name_input: String,
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
                self.is_process_selector_view_active = false;
            }
            None => {
                self.opened_process_identifier = None;
                self.opened_process_name = None;
                self.is_process_selector_view_active = true;
            }
        }
    }

    pub fn activate_process_selector_view(&mut self) {
        self.is_process_selector_view_active = true;
    }

    pub fn begin_search_input(&mut self) {
        self.input_mode = ProcessSelectorInputMode::Search;
    }

    pub fn commit_search_input(&mut self) {
        self.input_mode = ProcessSelectorInputMode::None;
    }

    pub fn cancel_search_input(&mut self) {
        self.input_mode = ProcessSelectorInputMode::None;
        self.pending_search_name_input.clear();
    }

    pub fn append_pending_search_character(
        &mut self,
        pending_character: char,
    ) {
        if !Self::is_supported_search_character(pending_character) {
            return;
        }

        self.pending_search_name_input.push(pending_character);
    }

    pub fn backspace_pending_search_name(&mut self) {
        self.pending_search_name_input.pop();
    }

    pub fn clear_pending_search_name(&mut self) {
        self.pending_search_name_input.clear();
    }

    pub fn pending_search_name_trimmed(&self) -> Option<String> {
        let trimmed_search_name = self.pending_search_name_input.trim();
        if trimmed_search_name.is_empty() {
            None
        } else {
            Some(trimmed_search_name.to_string())
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        build_process_selector_summary_lines(self)
    }

    pub fn visible_process_entry_rows(
        &self,
        viewport_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        build_visible_process_entry_rows(self, viewport_capacity)
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

    fn is_supported_search_character(pending_character: char) -> bool {
        pending_character.is_ascii_graphic() || pending_character == ' '
    }
}

impl Default for ProcessSelectorPaneState {
    fn default() -> Self {
        Self {
            selected_process_identifier: None,
            selected_process_name: None,
            show_windowed_processes_only: false,
            process_list_entries: Vec::new(),
            selected_process_list_index: None,
            opened_process_identifier: None,
            opened_process_name: None,
            has_loaded_process_list_once: false,
            is_awaiting_process_list_response: false,
            is_opening_process: false,
            is_process_selector_view_active: true,
            input_mode: ProcessSelectorInputMode::None,
            pending_search_name_input: String::new(),
            status_message: "Ready.".to_string(),
        }
    }
}
