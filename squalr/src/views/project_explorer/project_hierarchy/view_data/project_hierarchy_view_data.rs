use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
    project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
};
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyViewData {
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
    pub project_items: Vec<(ProjectItemRef, ProjectItem)>,
    pub tree_entries: Vec<ProjectHierarchyTreeEntry>,
    pub selected_project_item_path: Option<PathBuf>,
    pub expanded_directory_paths: HashSet<PathBuf>,
    pub context_menu_project_item_path: Option<PathBuf>,
    pub dragged_project_item_path: Option<PathBuf>,
    pub take_over_state: ProjectHierarchyTakeOverState,
    pub pending_operation: ProjectHierarchyPendingOperation,
}

impl ProjectHierarchyViewData {
    pub fn new() -> Self {
        Self {
            opened_project_info: None,
            opened_project_root: None,
            project_items: Vec::new(),
            tree_entries: Vec::new(),
            selected_project_item_path: None,
            expanded_directory_paths: HashSet::new(),
            context_menu_project_item_path: None,
            dragged_project_item_path: None,
            take_over_state: ProjectHierarchyTakeOverState::None,
            pending_operation: ProjectHierarchyPendingOperation::None,
        }
    }

    pub fn refresh_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
    ) {
        if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy view data refresh request") {
            if project_hierarchy_view_data.pending_operation == ProjectHierarchyPendingOperation::Refreshing {
                return;
            }

            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Refreshing;
        }

        let project_items_list_request = ProjectItemsListRequest {};

        project_items_list_request.send(&app_context.engine_unprivileged_state, move |project_items_list_response| {
            let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy view data refresh response") {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };

            project_hierarchy_view_data.opened_project_info = project_items_list_response.opened_project_info;
            project_hierarchy_view_data.opened_project_root = project_items_list_response.opened_project_root;
            project_hierarchy_view_data.project_items = project_items_list_response.opened_project_items;

            if let Some(project_info) = &project_hierarchy_view_data.opened_project_info {
                if let Some(project_directory_path) = project_info.get_project_directory() {
                    project_hierarchy_view_data
                        .expanded_directory_paths
                        .insert(project_directory_path);
                }
            }

            project_hierarchy_view_data.tree_entries = Self::build_tree_entries(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &project_hierarchy_view_data.expanded_directory_paths,
            );

            if let Some(selected_project_item_path) = &project_hierarchy_view_data.selected_project_item_path {
                let is_valid_selection = project_hierarchy_view_data
                    .tree_entries
                    .iter()
                    .any(|tree_entry| &tree_entry.project_item_path == selected_project_item_path);

                if !is_valid_selection {
                    project_hierarchy_view_data.selected_project_item_path = None;
                }
            }

            if let Some(dragged_project_item_path) = &project_hierarchy_view_data.dragged_project_item_path {
                let is_valid_dragged_project_item = project_hierarchy_view_data
                    .tree_entries
                    .iter()
                    .any(|tree_entry| &tree_entry.project_item_path == dragged_project_item_path);

                if !is_valid_dragged_project_item {
                    project_hierarchy_view_data.dragged_project_item_path = None;
                }
            }

            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
        });
    }

    pub fn select_project_item(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy select project item") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.selected_project_item_path = Some(project_item_path);
    }

    pub fn toggle_directory_expansion(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy toggle directory expansion") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data
            .expanded_directory_paths
            .contains(&project_item_path)
        {
            project_hierarchy_view_data
                .expanded_directory_paths
                .remove(&project_item_path);
        } else {
            project_hierarchy_view_data
                .expanded_directory_paths
                .insert(project_item_path);
        }

        project_hierarchy_view_data.tree_entries = Self::build_tree_entries(
            project_hierarchy_view_data.opened_project_info.as_ref(),
            &project_hierarchy_view_data.project_items,
            &project_hierarchy_view_data.expanded_directory_paths,
        );
    }

    pub fn request_delete_confirmation_for_selected_project_item(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let selected_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy selected project item for delete request")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.selected_project_item_path.clone());

        if let Some(selected_project_item_path) = selected_project_item_path {
            Self::request_delete_confirmation(project_hierarchy_view_data, vec![selected_project_item_path]);
        }
    }

    pub fn request_delete_confirmation(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy request delete confirmation") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths };
    }

    pub fn cancel_take_over(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cancel take over") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
    }

    pub fn begin_reorder_drag(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_path: PathBuf,
    ) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy begin reorder drag") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
            return;
        }

        project_hierarchy_view_data.dragged_project_item_path = Some(project_item_path);
    }

    pub fn cancel_reorder_drag(project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>) {
        let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy cancel reorder drag") {
            Some(project_hierarchy_view_data) => project_hierarchy_view_data,
            None => return,
        };

        project_hierarchy_view_data.dragged_project_item_path = None;
    }

    pub fn commit_reorder_drop(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        target_project_item_path: PathBuf,
    ) {
        let reordered_project_item_paths = {
            let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy commit reorder drop") {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };
            let dragged_project_item_path = match project_hierarchy_view_data.dragged_project_item_path.clone() {
                Some(dragged_project_item_path) => dragged_project_item_path,
                None => return,
            };

            if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                project_hierarchy_view_data.dragged_project_item_path = None;
                return;
            }

            let reordered_project_item_paths = Self::build_reordered_project_item_paths(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &dragged_project_item_path,
                &target_project_item_path,
            );

            match reordered_project_item_paths {
                Some(reordered_project_item_paths) => {
                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Reordering;
                    project_hierarchy_view_data.dragged_project_item_path = None;
                    reordered_project_item_paths
                }
                None => {
                    project_hierarchy_view_data.dragged_project_item_path = None;
                    return;
                }
            }
        };

        let project_items_reorder_request = ProjectItemsReorderRequest {
            project_item_paths: reordered_project_item_paths,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_reorder_request.send(&app_context.engine_unprivileged_state, move |project_items_reorder_response| {
            if !project_items_reorder_response.success {
                log::error!(
                    "Failed to reorder project items. Reordered count: {}.",
                    project_items_reorder_response.reordered_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy reorder project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
            }

            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    pub fn delete_project_items(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
    ) {
        if project_item_paths.is_empty() {
            Self::cancel_take_over(project_hierarchy_view_data);

            return;
        }

        if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy begin delete project items") {
            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Deleting;
            project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
        }

        let project_items_delete_request = ProjectItemsDeleteRequest { project_item_paths };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_delete_request.send(&app_context.engine_unprivileged_state, move |project_items_delete_response| {
            if !project_items_delete_response.success {
                log::error!(
                    "Failed to delete one or more project items. Deleted count: {}.",
                    project_items_delete_response.deleted_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data_clone.write("Project hierarchy delete project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
            }

            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    pub fn set_project_item_activation(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
        project_item_path: PathBuf,
        is_activated: bool,
    ) {
        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: vec![project_item_path.to_string_lossy().into_owned()],
            is_activated,
        };
        let app_context_clone = app_context.clone();
        let project_hierarchy_view_data_clone = project_hierarchy_view_data.clone();

        project_items_activate_request.send(&app_context.engine_unprivileged_state, move |_project_items_activate_response| {
            Self::refresh_project_items(project_hierarchy_view_data_clone, app_context_clone);
        });
    }

    fn build_tree_entries(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        expanded_directory_paths: &HashSet<PathBuf>,
    ) -> Vec<ProjectHierarchyTreeEntry> {
        let project_info = match opened_project_info {
            Some(project_info) => project_info,
            None => return Vec::new(),
        };
        let (project_directory_path, project_item_map, child_paths_by_parent_path) = match Self::build_project_hierarchy_maps(project_info, project_items) {
            Some(project_hierarchy_maps) => project_hierarchy_maps,
            None => return Vec::new(),
        };

        let mut visible_tree_entries = Vec::new();

        Self::append_visible_entries(
            &mut visible_tree_entries,
            &project_directory_path,
            &child_paths_by_parent_path,
            &project_item_map,
            0,
            expanded_directory_paths,
        );

        visible_tree_entries
    }

    fn build_reordered_project_item_paths(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        dragged_project_item_path: &Path,
        target_project_item_path: &Path,
    ) -> Option<Vec<PathBuf>> {
        let project_info = opened_project_info?;
        let (project_directory_path, _project_item_map, mut child_paths_by_parent_path) = Self::build_project_hierarchy_maps(project_info, project_items)?;
        let dragged_parent_path = dragged_project_item_path.parent()?.to_path_buf();
        let target_parent_path = target_project_item_path.parent()?.to_path_buf();

        if dragged_project_item_path == target_project_item_path || dragged_parent_path != target_parent_path {
            return None;
        }

        let sibling_paths = child_paths_by_parent_path.get_mut(&dragged_parent_path)?;
        let dragged_sibling_index = sibling_paths
            .iter()
            .position(|project_item_path| project_item_path == dragged_project_item_path)?;
        let target_sibling_index = sibling_paths
            .iter()
            .position(|project_item_path| project_item_path == target_project_item_path)?;
        let dragged_path = sibling_paths.remove(dragged_sibling_index);
        let adjusted_target_sibling_index = if dragged_sibling_index < target_sibling_index {
            target_sibling_index.saturating_sub(1)
        } else {
            target_sibling_index
        };

        sibling_paths.insert(adjusted_target_sibling_index, dragged_path);

        let mut reordered_project_item_paths = Vec::new();
        Self::append_project_item_paths_in_order(&project_directory_path, &child_paths_by_parent_path, &mut reordered_project_item_paths);

        Some(reordered_project_item_paths)
    }

    fn append_visible_entries(
        visible_tree_entries: &mut Vec<ProjectHierarchyTreeEntry>,
        parent_path: &PathBuf,
        child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
        project_item_map: &HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
        depth: usize,
        expanded_directory_paths: &HashSet<PathBuf>,
    ) {
        let child_paths = match child_paths_by_parent_path.get(parent_path) {
            Some(child_paths) => child_paths,
            None => return,
        };

        for child_path in child_paths {
            let (project_item_ref, project_item) = match project_item_map.get(child_path) {
                Some(project_item_pair) => project_item_pair,
                None => continue,
            };
            let is_directory = Self::is_directory_project_item(project_item);
            let has_children = child_paths_by_parent_path
                .get(child_path)
                .map(|entries| !entries.is_empty())
                .unwrap_or(false);
            let is_expanded = expanded_directory_paths.contains(child_path);
            let display_name = child_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            let display_name_from_property = project_item.get_field_name();
            let display_name = if display_name_from_property.is_empty() {
                display_name.to_string()
            } else {
                display_name_from_property
            };
            let preview_value = Self::build_preview_value(project_item);

            visible_tree_entries.push(ProjectHierarchyTreeEntry {
                project_item_ref: project_item_ref.clone(),
                project_item: project_item.clone(),
                project_item_path: child_path.clone(),
                display_name,
                preview_value,
                is_activated: project_item.get_is_activated(),
                depth,
                is_directory,
                has_children,
                is_expanded,
            });

            if is_directory && is_expanded {
                Self::append_visible_entries(
                    visible_tree_entries,
                    child_path,
                    child_paths_by_parent_path,
                    project_item_map,
                    depth + 1,
                    expanded_directory_paths,
                );
            }
        }
    }

    fn build_sort_order_lookup(
        project_info: &ProjectInfo,
        project_directory_path: &Path,
    ) -> HashMap<PathBuf, usize> {
        project_info
            .get_project_manifest()
            .get_project_item_sort_order()
            .iter()
            .enumerate()
            .map(|(sort_order_index, relative_project_item_path)| (project_directory_path.join(relative_project_item_path), sort_order_index))
            .collect()
    }

    fn build_project_hierarchy_maps(
        project_info: &ProjectInfo,
        project_items: &[(ProjectItemRef, ProjectItem)],
    ) -> Option<(PathBuf, HashMap<PathBuf, (ProjectItemRef, ProjectItem)>, HashMap<PathBuf, Vec<PathBuf>>)> {
        let project_directory_path = project_info.get_project_directory()?;
        let project_item_map: HashMap<PathBuf, (ProjectItemRef, ProjectItem)> = project_items
            .iter()
            .map(|(project_item_ref, project_item)| {
                (
                    project_item_ref.get_project_item_path().clone(),
                    (project_item_ref.clone(), project_item.clone()),
                )
            })
            .collect();
        let sort_order_lookup = Self::build_sort_order_lookup(project_info, &project_directory_path);
        let mut child_paths_by_parent_path: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for project_item_path in project_item_map.keys() {
            if project_item_path == &project_directory_path {
                continue;
            }

            let parent_path = project_item_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| project_directory_path.clone());

            child_paths_by_parent_path
                .entry(parent_path)
                .or_default()
                .push(project_item_path.clone());
        }

        for child_paths in child_paths_by_parent_path.values_mut() {
            child_paths.sort_by(|left_path, right_path| {
                let left_order = sort_order_lookup.get(left_path).copied().unwrap_or(usize::MAX);
                let right_order = sort_order_lookup.get(right_path).copied().unwrap_or(usize::MAX);

                if left_order != right_order {
                    return left_order.cmp(&right_order);
                }

                let left_is_directory = Self::is_directory_path(left_path, &project_item_map);
                let right_is_directory = Self::is_directory_path(right_path, &project_item_map);

                if left_is_directory != right_is_directory {
                    return right_is_directory.cmp(&left_is_directory);
                }

                let left_name = left_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                let right_name = right_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();

                left_name.cmp(right_name)
            });
        }

        Some((project_directory_path, project_item_map, child_paths_by_parent_path))
    }

    fn append_project_item_paths_in_order(
        parent_path: &Path,
        child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
        reordered_project_item_paths: &mut Vec<PathBuf>,
    ) {
        let child_paths = match child_paths_by_parent_path.get(parent_path) {
            Some(child_paths) => child_paths,
            None => return,
        };

        for child_path in child_paths {
            reordered_project_item_paths.push(child_path.clone());
            Self::append_project_item_paths_in_order(child_path, child_paths_by_parent_path, reordered_project_item_paths);
        }
    }

    fn is_directory_path(
        project_item_path: &Path,
        project_item_map: &HashMap<PathBuf, (ProjectItemRef, ProjectItem)>,
    ) -> bool {
        project_item_map
            .get(project_item_path)
            .map(|(_, project_item)| Self::is_directory_project_item(project_item))
            .unwrap_or(false)
    }

    fn is_directory_project_item(project_item: &ProjectItem) -> bool {
        project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID
    }

    fn build_preview_value(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let preview_value = Self::read_string_field(project_item, ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE);

            if preview_value.is_empty() { "??".to_string() } else { preview_value }
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            "Pointer".to_string()
        } else {
            String::new()
        }
    }

    fn read_string_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> String {
        let data_value = match project_item
            .get_properties()
            .get_field(field_name)
            .and_then(|field| field.get_data_value())
        {
            Some(data_value) => data_value,
            None => return String::new(),
        };

        String::from_utf8(data_value.get_value_bytes().clone()).unwrap_or_default()
    }
}
