/// Stores UI state for process selection workflows.
#[derive(Clone, Debug, Default)]
pub struct ProcessSelectorPaneState {
    pub selected_process_identifier: Option<u32>,
    pub selected_process_name: Option<String>,
    pub show_windowed_processes_only: bool,
}
