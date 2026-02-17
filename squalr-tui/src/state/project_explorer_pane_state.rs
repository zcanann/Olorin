use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::path::{Path, PathBuf};

/// Stores text input mode for project selector operations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectSelectorInputMode {
    #[default]
    None,
    CreatingProject,
    RenamingProject,
}

/// Stores state for browsing projects and project items.
#[derive(Clone, Debug)]
pub struct ProjectExplorerPaneState {
    pub project_entries: Vec<ProjectInfo>,
    pub selected_project_list_index: Option<usize>,
    pub selected_project_name: Option<String>,
    pub selected_project_directory_path: Option<PathBuf>,
    pub active_project_name: Option<String>,
    pub active_project_directory_path: Option<PathBuf>,
    pub selected_item_path: Option<String>,
    pub is_hierarchy_expanded: bool,
    pub input_mode: ProjectSelectorInputMode,
    pub pending_project_name_input: String,
    pub has_loaded_project_list_once: bool,
    pub is_awaiting_project_list_response: bool,
    pub is_creating_project: bool,
    pub is_opening_project: bool,
    pub is_renaming_project: bool,
    pub is_deleting_project: bool,
    pub is_closing_project: bool,
    pub status_message: String,
}

impl ProjectExplorerPaneState {
    pub fn apply_project_list(
        &mut self,
        project_entries: Vec<ProjectInfo>,
    ) {
        self.project_entries = project_entries;
        self.selected_project_list_index = if self.project_entries.is_empty() { None } else { Some(0) };
        self.update_selected_project_fields();
    }

    pub fn select_next_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        let selected_project_index = self.selected_project_list_index.unwrap_or(0);
        let next_project_index = (selected_project_index + 1) % self.project_entries.len();
        self.selected_project_list_index = Some(next_project_index);
        self.update_selected_project_fields();
    }

    pub fn select_previous_project(&mut self) {
        if self.project_entries.is_empty() {
            self.selected_project_list_index = None;
            self.update_selected_project_fields();
            return;
        }

        let selected_project_index = self.selected_project_list_index.unwrap_or(0);
        let previous_project_index = if selected_project_index == 0 {
            self.project_entries.len() - 1
        } else {
            selected_project_index - 1
        };
        self.selected_project_list_index = Some(previous_project_index);
        self.update_selected_project_fields();
    }

    pub fn select_project_by_directory_path(
        &mut self,
        project_directory_path: &Path,
    ) -> bool {
        let matching_project_index = self.project_entries.iter().position(|project_entry| {
            project_entry
                .get_project_directory()
                .as_deref()
                .is_some_and(|entry_directory| entry_directory == project_directory_path)
        });

        if let Some(matching_project_index) = matching_project_index {
            self.selected_project_list_index = Some(matching_project_index);
            self.update_selected_project_fields();
            return true;
        }

        false
    }

    pub fn selected_project_directory_path(&self) -> Option<PathBuf> {
        self.selected_project_directory_path.clone()
    }

    pub fn selected_project_name(&self) -> Option<String> {
        self.selected_project_name.clone()
    }

    pub fn begin_create_project_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::CreatingProject;
        self.pending_project_name_input = "NewProject".to_string();
    }

    pub fn begin_rename_selected_project_input(&mut self) -> bool {
        let Some(selected_project_name) = self.selected_project_name.clone() else {
            return false;
        };

        self.input_mode = ProjectSelectorInputMode::RenamingProject;
        self.pending_project_name_input = selected_project_name;
        true
    }

    pub fn cancel_project_name_input(&mut self) {
        self.input_mode = ProjectSelectorInputMode::None;
        self.pending_project_name_input.clear();
    }

    pub fn pending_project_name_trimmed(&self) -> Option<String> {
        let trimmed_project_name = self.pending_project_name_input.trim();
        if trimmed_project_name.is_empty() {
            None
        } else {
            Some(trimmed_project_name.to_string())
        }
    }

    pub fn append_pending_project_name_character(
        &mut self,
        pending_character: char,
    ) {
        if !Self::is_supported_project_name_character(pending_character) {
            return;
        }

        self.pending_project_name_input.push(pending_character);
    }

    pub fn backspace_pending_project_name(&mut self) {
        self.pending_project_name_input.pop();
    }

    pub fn clear_pending_project_name(&mut self) {
        self.pending_project_name_input.clear();
    }

    pub fn set_active_project(
        &mut self,
        active_project_name: Option<String>,
        active_project_directory_path: Option<PathBuf>,
    ) {
        self.active_project_name = active_project_name;
        self.active_project_directory_path = active_project_directory_path;
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut summary_lines = vec![
            "Actions: r refresh, n create, Enter/o open, e rename, x delete, c close active.".to_string(),
            "Selection: Up/Down/j/k move. Input: type, Backspace, Ctrl+u clear, Enter commit, Esc cancel.".to_string(),
            format!("list_count={}", self.project_entries.len()),
            format!("selected_name={:?}", self.selected_project_name),
            format!("active_project={:?}", self.active_project_name),
            format!("active_directory={:?}", self.active_project_directory_path),
            format!("selected_item={:?}", self.selected_item_path),
            format!("expanded={}", self.is_hierarchy_expanded),
            format!("input_mode={:?}", self.input_mode),
            format!("pending_name={}", self.pending_project_name_input),
            format!("awaiting_list={}", self.is_awaiting_project_list_response),
            format!("creating={}", self.is_creating_project),
            format!("opening={}", self.is_opening_project),
            format!("renaming={}", self.is_renaming_project),
            format!("deleting={}", self.is_deleting_project),
            format!("closing={}", self.is_closing_project),
            format!("status={}", self.status_message),
        ];

        let visible_entry_count = self.project_entries.len().min(5);
        for visible_project_index in 0..visible_entry_count {
            if let Some(project_entry) = self.project_entries.get(visible_project_index) {
                let selected_marker = if self.selected_project_list_index == Some(visible_project_index) {
                    ">"
                } else {
                    " "
                };
                let active_marker = if self
                    .active_project_directory_path
                    .as_ref()
                    .zip(project_entry.get_project_directory())
                    .is_some_and(|(active_project_directory, project_entry_directory)| *active_project_directory == project_entry_directory)
                {
                    "*"
                } else {
                    " "
                };
                let project_directory_display = project_entry
                    .get_project_directory()
                    .map(|project_directory| project_directory.display().to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());

                summary_lines.push(format!(
                    "{}{} {} ({})",
                    selected_marker,
                    active_marker,
                    project_entry.get_name(),
                    project_directory_display
                ));
            }
        }

        summary_lines
    }

    fn update_selected_project_fields(&mut self) {
        if let Some(selected_project_index) = self.selected_project_list_index {
            if let Some(selected_project_entry) = self.project_entries.get(selected_project_index) {
                self.selected_project_name = Some(selected_project_entry.get_name().to_string());
                self.selected_project_directory_path = selected_project_entry.get_project_directory();
                return;
            }
        }

        self.selected_project_name = None;
        self.selected_project_directory_path = None;
    }

    fn is_supported_project_name_character(pending_character: char) -> bool {
        pending_character.is_ascii_alphanumeric()
            || pending_character == ' '
            || pending_character == '_'
            || pending_character == '-'
            || pending_character == '.'
    }
}

impl Default for ProjectExplorerPaneState {
    fn default() -> Self {
        Self {
            project_entries: Vec::new(),
            selected_project_list_index: None,
            selected_project_name: None,
            selected_project_directory_path: None,
            active_project_name: None,
            active_project_directory_path: None,
            selected_item_path: None,
            is_hierarchy_expanded: true,
            input_mode: ProjectSelectorInputMode::None,
            pending_project_name_input: String::new(),
            has_loaded_project_list_once: false,
            is_awaiting_project_list_response: false,
            is_creating_project: false,
            is_opening_project: false,
            is_renaming_project: false,
            is_deleting_project: false,
            is_closing_project: false,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::project_explorer_pane_state::{ProjectExplorerPaneState, ProjectSelectorInputMode};
    use squalr_engine_api::structures::projects::{project_info::ProjectInfo, project_manifest::ProjectManifest};
    use std::path::PathBuf;

    #[test]
    fn apply_project_list_selects_first_project() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Alpha/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Beta/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
        ]);

        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(0));
        assert_eq!(project_explorer_pane_state.selected_project_name, Some("project".to_string()));
        assert_eq!(
            project_explorer_pane_state.selected_project_directory_path,
            Some(PathBuf::from("C:/Projects/Alpha/project"))
        );
    }

    #[test]
    fn project_selection_wraps_forward_and_backward() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Alpha/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Beta/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
        ]);

        project_explorer_pane_state.select_next_project();
        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(1));

        project_explorer_pane_state.select_next_project();
        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(0));

        project_explorer_pane_state.select_previous_project();
        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(1));
    }

    #[test]
    fn rename_input_requires_selected_project() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();

        assert!(!project_explorer_pane_state.begin_rename_selected_project_input());
        assert_eq!(project_explorer_pane_state.input_mode, ProjectSelectorInputMode::None);
    }
}
