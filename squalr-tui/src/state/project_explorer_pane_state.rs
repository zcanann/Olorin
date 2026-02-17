use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Stores text input mode for project selector operations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectSelectorInputMode {
    #[default]
    None,
    CreatingProject,
    RenamingProject,
    CreatingProjectDirectory,
}

/// Stores current focus target for the project explorer pane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectExplorerFocusTarget {
    #[default]
    ProjectList,
    ProjectHierarchy,
}

/// Stores a visible hierarchy entry for a project item.
#[derive(Clone, Debug)]
pub struct ProjectHierarchyEntry {
    pub project_item_path: PathBuf,
    pub display_name: String,
    pub depth: usize,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub is_activated: bool,
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
    pub focus_target: ProjectExplorerFocusTarget,
    pub input_mode: ProjectSelectorInputMode,
    pub pending_project_name_input: String,
    pub has_loaded_project_list_once: bool,
    pub is_awaiting_project_list_response: bool,
    pub is_creating_project: bool,
    pub is_opening_project: bool,
    pub is_renaming_project: bool,
    pub is_deleting_project: bool,
    pub is_closing_project: bool,
    pub has_loaded_project_item_list_once: bool,
    pub is_awaiting_project_item_list_response: bool,
    pub is_creating_project_item: bool,
    pub is_deleting_project_item: bool,
    pub is_moving_project_item: bool,
    pub is_reordering_project_item: bool,
    pub is_toggling_project_item_activation: bool,
    pub project_item_visible_entries: Vec<ProjectHierarchyEntry>,
    pub selected_project_item_visible_index: Option<usize>,
    pub pending_move_source_paths: Vec<PathBuf>,
    pub pending_delete_confirmation_paths: Vec<PathBuf>,
    pub status_message: String,
    opened_project_item_map: HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>>,
    root_project_item_paths: Vec<PathBuf>,
    expanded_directory_paths: HashSet<PathBuf>,
}

impl ProjectExplorerPaneState {
    pub fn apply_project_list(
        &mut self,
        project_entries: Vec<ProjectInfo>,
    ) {
        let selected_project_directory_path_before_refresh = self.selected_project_directory_path.clone();
        self.project_entries = project_entries;
        self.selected_project_list_index = selected_project_directory_path_before_refresh
            .as_ref()
            .and_then(|selected_project_directory_path| {
                self.project_entries.iter().position(|project_entry| {
                    project_entry
                        .get_project_directory()
                        .as_deref()
                        .is_some_and(|project_directory| project_directory == selected_project_directory_path.as_path())
                })
            })
            .or_else(|| if self.project_entries.is_empty() { None } else { Some(0) });
        self.update_selected_project_fields();
    }

    pub fn apply_project_items_list(
        &mut self,
        opened_project_items: Vec<(ProjectItemRef, ProjectItem)>,
    ) {
        let selected_project_item_path_before_refresh = self.selected_project_item_path();

        self.opened_project_item_map.clear();
        self.child_paths_by_parent_path.clear();
        self.root_project_item_paths.clear();

        for (project_item_ref, project_item) in opened_project_items {
            self.opened_project_item_map
                .insert(project_item_ref.get_project_item_path().clone(), project_item);
        }

        let all_project_item_paths: HashSet<PathBuf> = self.opened_project_item_map.keys().cloned().collect();
        for project_item_path in &all_project_item_paths {
            let parent_directory_path = project_item_path.parent().map(Path::to_path_buf);
            if parent_directory_path
                .as_ref()
                .is_some_and(|candidate_parent_path| all_project_item_paths.contains(candidate_parent_path))
            {
                if let Some(parent_directory_path) = parent_directory_path {
                    self.child_paths_by_parent_path
                        .entry(parent_directory_path)
                        .or_default()
                        .push(project_item_path.clone());
                }
            } else {
                self.root_project_item_paths.push(project_item_path.clone());
            }
        }

        self.root_project_item_paths.sort();
        for child_paths in self.child_paths_by_parent_path.values_mut() {
            child_paths.sort();
        }

        let valid_project_item_paths: HashSet<PathBuf> = self.opened_project_item_map.keys().cloned().collect();
        let valid_directory_paths: HashSet<PathBuf> = self
            .opened_project_item_map
            .iter()
            .filter_map(|(project_item_path, project_item)| {
                if Self::is_directory_project_item(project_item) {
                    Some(project_item_path.clone())
                } else {
                    None
                }
            })
            .collect();
        self.expanded_directory_paths
            .retain(|expanded_directory_path| valid_directory_paths.contains(expanded_directory_path));
        self.pending_move_source_paths
            .retain(|pending_move_source_path| valid_project_item_paths.contains(pending_move_source_path));
        self.pending_delete_confirmation_paths
            .retain(|pending_delete_confirmation_path| valid_project_item_paths.contains(pending_delete_confirmation_path));

        self.rebuild_visible_hierarchy_entries();
        self.restore_selected_project_item_path(selected_project_item_path_before_refresh);
        self.has_loaded_project_item_list_once = true;
    }

    pub fn clear_project_items(&mut self) {
        self.opened_project_item_map.clear();
        self.child_paths_by_parent_path.clear();
        self.root_project_item_paths.clear();
        self.project_item_visible_entries.clear();
        self.selected_project_item_visible_index = None;
        self.pending_move_source_paths.clear();
        self.pending_delete_confirmation_paths.clear();
        self.selected_item_path = None;
        self.has_loaded_project_item_list_once = false;
    }

    pub fn select_next_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        let selected_project_item_visible_index = self.selected_project_item_visible_index.unwrap_or(0);
        let next_project_item_visible_index = (selected_project_item_visible_index + 1) % self.project_item_visible_entries.len();
        self.selected_project_item_visible_index = Some(next_project_item_visible_index);
        self.update_selected_item_path();
    }

    pub fn select_previous_project_item(&mut self) {
        if self.project_item_visible_entries.is_empty() {
            self.selected_project_item_visible_index = None;
            self.update_selected_item_path();
            return;
        }

        let selected_project_item_visible_index = self.selected_project_item_visible_index.unwrap_or(0);
        let previous_project_item_visible_index = if selected_project_item_visible_index == 0 {
            self.project_item_visible_entries.len() - 1
        } else {
            selected_project_item_visible_index - 1
        };

        self.selected_project_item_visible_index = Some(previous_project_item_visible_index);
        self.update_selected_item_path();
    }

    pub fn selected_project_item_path(&self) -> Option<PathBuf> {
        let selected_project_item_visible_index = self.selected_project_item_visible_index?;
        self.project_item_visible_entries
            .get(selected_project_item_visible_index)
            .map(|project_item_entry| project_item_entry.project_item_path.clone())
    }

    pub fn selected_project_items_for_struct_viewer(&self) -> Vec<(PathBuf, ProjectItem)> {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return Vec::new();
        };
        let Some(selected_project_item) = self.opened_project_item_map.get(&selected_project_item_path) else {
            return Vec::new();
        };

        vec![(selected_project_item_path, selected_project_item.clone())]
    }

    pub fn selected_project_item_directory_target_path(&self) -> Option<PathBuf> {
        let selected_project_item_path = self.selected_project_item_path()?;
        if self.is_directory_path(&selected_project_item_path) {
            Some(selected_project_item_path)
        } else {
            selected_project_item_path.parent().map(Path::to_path_buf)
        }
    }

    pub fn expand_selected_project_item_directory(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };

        if !self.is_directory_path(&selected_project_item_path) {
            return false;
        }

        let was_inserted = self
            .expanded_directory_paths
            .insert(selected_project_item_path.clone());
        if was_inserted {
            self.rebuild_visible_hierarchy_entries();
            self.restore_selected_project_item_path(Some(selected_project_item_path));
        }

        was_inserted
    }

    pub fn collapse_selected_project_item_directory_or_select_parent(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };

        if self
            .expanded_directory_paths
            .remove(&selected_project_item_path)
        {
            self.rebuild_visible_hierarchy_entries();
            self.restore_selected_project_item_path(Some(selected_project_item_path));
            return true;
        }

        let Some(parent_directory_path) = selected_project_item_path.parent().map(Path::to_path_buf) else {
            return false;
        };
        let parent_project_item_visible_index = self
            .project_item_visible_entries
            .iter()
            .position(|project_item_entry| project_item_entry.project_item_path == parent_directory_path);
        if let Some(parent_project_item_visible_index) = parent_project_item_visible_index {
            self.selected_project_item_visible_index = Some(parent_project_item_visible_index);
            self.update_selected_item_path();
            return true;
        }

        false
    }

    pub fn selected_project_item_is_activated(&self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.opened_project_item_map
            .get(&selected_project_item_path)
            .map(ProjectItem::get_is_activated)
            .unwrap_or(false)
    }

    pub fn stage_selected_project_item_for_move(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_move_source_paths = vec![selected_project_item_path];
        true
    }

    pub fn has_pending_move_source_paths(&self) -> bool {
        !self.pending_move_source_paths.is_empty()
    }

    pub fn pending_move_source_paths(&self) -> Vec<PathBuf> {
        self.pending_move_source_paths.clone()
    }

    pub fn clear_pending_move_source_paths(&mut self) {
        self.pending_move_source_paths.clear();
    }

    pub fn arm_delete_confirmation_for_selected_project_item(&mut self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_delete_confirmation_paths = vec![selected_project_item_path];
        true
    }

    pub fn has_pending_delete_confirmation_for_selected_project_item(&self) -> bool {
        let Some(selected_project_item_path) = self.selected_project_item_path() else {
            return false;
        };
        self.pending_delete_confirmation_paths == vec![selected_project_item_path]
    }

    pub fn take_pending_delete_confirmation_paths(&mut self) -> Vec<PathBuf> {
        let pending_delete_confirmation_paths = self.pending_delete_confirmation_paths.clone();
        self.pending_delete_confirmation_paths.clear();
        pending_delete_confirmation_paths
    }

    pub fn build_reorder_request_paths_for_selected_project_item(
        &self,
        move_toward_previous_position: bool,
    ) -> Option<Vec<PathBuf>> {
        let selected_project_item_path = self.selected_project_item_path()?;
        let parent_directory_path = selected_project_item_path.parent().map(Path::to_path_buf);
        let mut child_paths_by_parent_path = self.child_paths_by_parent_path.clone();
        let mut root_project_item_paths = self.root_project_item_paths.clone();

        if let Some(parent_directory_path) = parent_directory_path {
            if self
                .opened_project_item_map
                .contains_key(&parent_directory_path)
            {
                let sibling_paths = child_paths_by_parent_path.get_mut(&parent_directory_path)?;
                let selected_sibling_position = sibling_paths
                    .iter()
                    .position(|sibling_path| sibling_path == &selected_project_item_path)?;
                if move_toward_previous_position {
                    if selected_sibling_position == 0 {
                        return None;
                    }
                    sibling_paths.swap(selected_sibling_position, selected_sibling_position - 1);
                } else {
                    let next_sibling_position = selected_sibling_position + 1;
                    if next_sibling_position >= sibling_paths.len() {
                        return None;
                    }
                    sibling_paths.swap(selected_sibling_position, next_sibling_position);
                }
            } else {
                let selected_root_position = root_project_item_paths
                    .iter()
                    .position(|root_path| root_path == &selected_project_item_path)?;
                if move_toward_previous_position {
                    if selected_root_position == 0 {
                        return None;
                    }
                    root_project_item_paths.swap(selected_root_position, selected_root_position - 1);
                } else {
                    let next_root_position = selected_root_position + 1;
                    if next_root_position >= root_project_item_paths.len() {
                        return None;
                    }
                    root_project_item_paths.swap(selected_root_position, next_root_position);
                }
            }
        }

        let mut reordered_project_item_paths = Vec::new();
        for root_project_item_path in &root_project_item_paths {
            Self::append_project_item_paths_preorder(root_project_item_path, &child_paths_by_parent_path, &mut reordered_project_item_paths);
        }

        Some(
            reordered_project_item_paths
                .into_iter()
                .filter(|project_item_path| {
                    !project_item_path
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        .is_some_and(|file_name| file_name == Project::PROJECT_DIR)
                })
                .collect(),
        )
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

    pub fn begin_create_project_directory_input(&mut self) -> bool {
        if self.selected_project_item_directory_target_path().is_none() {
            return false;
        }

        self.input_mode = ProjectSelectorInputMode::CreatingProjectDirectory;
        self.pending_project_name_input = self.build_unique_new_directory_name();
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
            "Mode: p project list, i project hierarchy.".to_string(),
            "Project list: r refresh, n create, Enter/o open, e rename, x delete, c close active.".to_string(),
            "Hierarchy: h refresh, j/k select, l expand, Left collapse, Space activate.".to_string(),
            "Hierarchy cont: n new folder, x delete(confirm), m stage move, b move here, [/] reorder, u cancel move.".to_string(),
            "Input mode: type, Backspace, Ctrl+u clear, Enter commit, Esc cancel.".to_string(),
            format!("focus_target={:?}", self.focus_target),
            format!("list_count={}", self.project_entries.len()),
            format!("selected_name={:?}", self.selected_project_name),
            format!("active_project={:?}", self.active_project_name),
            format!("active_directory={:?}", self.active_project_directory_path),
            format!("selected_item={:?}", self.selected_item_path),
            format!("visible_item_count={}", self.project_item_visible_entries.len()),
            format!("expanded={}", self.is_hierarchy_expanded),
            format!("input_mode={:?}", self.input_mode),
            format!("pending_name={}", self.pending_project_name_input),
            format!("pending_move_count={}", self.pending_move_source_paths.len()),
            format!("pending_delete_count={}", self.pending_delete_confirmation_paths.len()),
            format!("awaiting_list={}", self.is_awaiting_project_list_response),
            format!("awaiting_item_list={}", self.is_awaiting_project_item_list_response),
            format!("creating={}", self.is_creating_project),
            format!("opening={}", self.is_opening_project),
            format!("renaming={}", self.is_renaming_project),
            format!("deleting={}", self.is_deleting_project),
            format!("closing={}", self.is_closing_project),
            format!("creating_item={}", self.is_creating_project_item),
            format!("deleting_item={}", self.is_deleting_project_item),
            format!("moving_item={}", self.is_moving_project_item),
            format!("reordering_item={}", self.is_reordering_project_item),
            format!("activating_item={}", self.is_toggling_project_item_activation),
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

        let visible_project_item_count = self.project_item_visible_entries.len().min(10);
        for visible_project_item_index in 0..visible_project_item_count {
            if let Some(project_item_entry) = self
                .project_item_visible_entries
                .get(visible_project_item_index)
            {
                let selected_marker = if self.selected_project_item_visible_index == Some(visible_project_item_index) {
                    ">"
                } else {
                    " "
                };
                let activation_marker = if project_item_entry.is_activated { "*" } else { " " };
                let directory_marker = if project_item_entry.is_directory {
                    if project_item_entry.is_expanded { "-" } else { "+" }
                } else {
                    " "
                };
                let indentation = " ".repeat(project_item_entry.depth.saturating_mul(2));

                summary_lines.push(format!(
                    "{}{}{} {}{}",
                    selected_marker, activation_marker, directory_marker, indentation, project_item_entry.display_name
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

    fn rebuild_visible_hierarchy_entries(&mut self) {
        self.project_item_visible_entries.clear();
        if !self.is_hierarchy_expanded {
            return;
        }

        let root_project_item_paths = self.root_project_item_paths.clone();
        for root_project_item_path in &root_project_item_paths {
            self.append_visible_hierarchy_entries(root_project_item_path, 0);
        }
    }

    fn append_visible_hierarchy_entries(
        &mut self,
        project_item_path: &Path,
        project_item_depth: usize,
    ) {
        let Some(project_item) = self.opened_project_item_map.get(project_item_path).cloned() else {
            return;
        };

        let is_directory = Self::is_directory_project_item(&project_item);
        let is_expanded = is_directory && self.expanded_directory_paths.contains(project_item_path);
        let child_paths = self
            .child_paths_by_parent_path
            .get(project_item_path)
            .cloned()
            .unwrap_or_default();

        let mut display_name = project_item.get_field_name();
        if display_name.is_empty() {
            display_name = project_item_path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .unwrap_or_default()
                .to_string();
        }

        self.project_item_visible_entries.push(ProjectHierarchyEntry {
            project_item_path: project_item_path.to_path_buf(),
            display_name,
            depth: project_item_depth,
            is_directory,
            is_expanded,
            is_activated: project_item.get_is_activated(),
        });

        if !is_expanded {
            return;
        }

        for child_path in &child_paths {
            self.append_visible_hierarchy_entries(child_path, project_item_depth + 1);
        }
    }

    fn restore_selected_project_item_path(
        &mut self,
        selected_project_item_path_before_refresh: Option<PathBuf>,
    ) {
        let selected_project_item_visible_index = selected_project_item_path_before_refresh
            .as_ref()
            .and_then(|selected_project_item_path| {
                self.project_item_visible_entries
                    .iter()
                    .position(|project_item_entry| &project_item_entry.project_item_path == selected_project_item_path)
            })
            .or_else(|| if self.project_item_visible_entries.is_empty() { None } else { Some(0) });

        self.selected_project_item_visible_index = selected_project_item_visible_index;
        self.update_selected_item_path();
    }

    fn update_selected_item_path(&mut self) {
        self.selected_item_path = self
            .selected_project_item_path()
            .map(|project_item_path| project_item_path.display().to_string());
    }

    fn is_directory_project_item(project_item: &ProjectItem) -> bool {
        project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
    }

    fn is_directory_path(
        &self,
        project_item_path: &Path,
    ) -> bool {
        self.opened_project_item_map
            .get(project_item_path)
            .map(Self::is_directory_project_item)
            .unwrap_or(false)
    }

    fn build_unique_new_directory_name(&self) -> String {
        const BASE_DIRECTORY_NAME: &str = "New Folder";
        let Some(parent_directory_path) = self.selected_project_item_directory_target_path() else {
            return BASE_DIRECTORY_NAME.to_string();
        };

        let existing_child_names: HashSet<String> = self
            .child_paths_by_parent_path
            .get(&parent_directory_path)
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|project_item_path| {
                project_item_path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map(str::to_string)
            })
            .collect();

        if !existing_child_names.contains(BASE_DIRECTORY_NAME) {
            return BASE_DIRECTORY_NAME.to_string();
        }

        let mut suffix_number = 2usize;
        loop {
            let candidate_directory_name = format!("{} {}", BASE_DIRECTORY_NAME, suffix_number);
            if !existing_child_names.contains(&candidate_directory_name) {
                return candidate_directory_name;
            }
            suffix_number += 1;
        }
    }

    fn append_project_item_paths_preorder(
        project_item_path: &Path,
        child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
        reordered_project_item_paths: &mut Vec<PathBuf>,
    ) {
        reordered_project_item_paths.push(project_item_path.to_path_buf());
        if let Some(child_paths) = child_paths_by_parent_path.get(project_item_path) {
            for child_path in child_paths {
                Self::append_project_item_paths_preorder(child_path, child_paths_by_parent_path, reordered_project_item_paths);
            }
        }
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
            focus_target: ProjectExplorerFocusTarget::ProjectList,
            input_mode: ProjectSelectorInputMode::None,
            pending_project_name_input: String::new(),
            has_loaded_project_list_once: false,
            is_awaiting_project_list_response: false,
            is_creating_project: false,
            is_opening_project: false,
            is_renaming_project: false,
            is_deleting_project: false,
            is_closing_project: false,
            has_loaded_project_item_list_once: false,
            is_awaiting_project_item_list_response: false,
            is_creating_project_item: false,
            is_deleting_project_item: false,
            is_moving_project_item: false,
            is_reordering_project_item: false,
            is_toggling_project_item_activation: false,
            project_item_visible_entries: Vec::new(),
            selected_project_item_visible_index: None,
            pending_move_source_paths: Vec::new(),
            pending_delete_confirmation_paths: Vec::new(),
            status_message: "Ready.".to_string(),
            opened_project_item_map: HashMap::new(),
            child_paths_by_parent_path: HashMap::new(),
            root_project_item_paths: Vec::new(),
            expanded_directory_paths: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::project_explorer_pane_state::{ProjectExplorerPaneState, ProjectSelectorInputMode};
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::{project_info::ProjectInfo, project_manifest::ProjectManifest};
    use std::path::PathBuf;

    fn create_directory_project_item_entry(
        project_item_path: PathBuf
    ) -> (
        ProjectItemRef,
        squalr_engine_api::structures::projects::project_items::project_item::ProjectItem,
    ) {
        let project_item_ref = ProjectItemRef::new(project_item_path);
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        (project_item_ref, project_item)
    }

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

    #[test]
    fn apply_project_list_preserves_selected_project_by_directory_path() {
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
        assert_eq!(
            project_explorer_pane_state.selected_project_directory_path,
            Some(PathBuf::from("C:/Projects/Beta/project"))
        );

        project_explorer_pane_state.apply_project_list(vec![
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Gamma/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
            ProjectInfo::new(
                PathBuf::from("C:/Projects/Beta/project/squalr-project.json"),
                None,
                ProjectManifest::new(Vec::new()),
            ),
        ]);

        assert_eq!(project_explorer_pane_state.selected_project_list_index, Some(1));
        assert_eq!(
            project_explorer_pane_state.selected_project_directory_path,
            Some(PathBuf::from("C:/Projects/Beta/project"))
        );
    }

    #[test]
    fn apply_project_items_list_preserves_selected_item_by_path() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        let first_directory_path = PathBuf::from("root/a");
        let second_directory_path = PathBuf::from("root/b");
        project_explorer_pane_state.apply_project_items_list(vec![
            create_directory_project_item_entry(first_directory_path.clone()),
            create_directory_project_item_entry(second_directory_path.clone()),
        ]);
        project_explorer_pane_state.select_next_project_item();
        assert_eq!(project_explorer_pane_state.selected_project_item_path(), Some(second_directory_path.clone()));

        project_explorer_pane_state.apply_project_items_list(vec![
            create_directory_project_item_entry(second_directory_path.clone()),
            create_directory_project_item_entry(PathBuf::from("root/c")),
        ]);

        assert_eq!(project_explorer_pane_state.selected_project_item_path(), Some(second_directory_path));
    }

    #[test]
    fn apply_project_items_list_prunes_stale_refresh_state() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        let retained_directory_path = PathBuf::from("root/retained");
        let removed_directory_path = PathBuf::from("root/removed");
        project_explorer_pane_state
            .expanded_directory_paths
            .insert(retained_directory_path.clone());
        project_explorer_pane_state
            .expanded_directory_paths
            .insert(removed_directory_path.clone());
        project_explorer_pane_state.pending_move_source_paths = vec![retained_directory_path.clone(), removed_directory_path.clone()];
        project_explorer_pane_state.pending_delete_confirmation_paths = vec![removed_directory_path.clone(), retained_directory_path.clone()];

        project_explorer_pane_state.apply_project_items_list(vec![create_directory_project_item_entry(
            retained_directory_path.clone(),
        )]);

        assert_eq!(
            project_explorer_pane_state
                .expanded_directory_paths
                .contains(&retained_directory_path),
            true
        );
        assert_eq!(
            project_explorer_pane_state
                .expanded_directory_paths
                .contains(&removed_directory_path),
            false
        );
        assert_eq!(project_explorer_pane_state.pending_move_source_paths, vec![retained_directory_path.clone()]);
        assert_eq!(project_explorer_pane_state.pending_delete_confirmation_paths, vec![retained_directory_path]);
    }
}
