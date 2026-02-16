use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::view_data::{
    project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
    project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
};
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
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

    fn build_tree_entries(
        opened_project_info: Option<&ProjectInfo>,
        project_items: &[(ProjectItemRef, ProjectItem)],
        expanded_directory_paths: &HashSet<PathBuf>,
    ) -> Vec<ProjectHierarchyTreeEntry> {
        let project_info = match opened_project_info {
            Some(project_info) => project_info,
            None => return Vec::new(),
        };
        let project_directory_path = match project_info.get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => return Vec::new(),
        };
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
                .unwrap_or_default()
                .to_string();
            let preview_value = Self::build_preview_value(project_item);

            visible_tree_entries.push(ProjectHierarchyTreeEntry {
                project_item_ref: project_item_ref.clone(),
                project_item: project_item.clone(),
                project_item_path: child_path.clone(),
                display_name,
                preview_value,
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
            let module_name = Self::read_string_field(project_item, ProjectItemTypeAddress::PROPERTY_MODULE);
            let address = Self::read_u64_field(project_item, ProjectItemTypeAddress::PROPERTY_ADDRESS);

            if module_name.is_empty() {
                format!("0x{:X}", address)
            } else {
                format!("{}+0x{:X}", module_name, address)
            }
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

    fn read_u64_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> u64 {
        let data_value = match project_item
            .get_properties()
            .get_field(field_name)
            .and_then(|field| field.get_data_value())
        {
            Some(data_value) => data_value,
            None => return 0,
        };
        let value_bytes = data_value.get_value_bytes();

        if value_bytes.len() < std::mem::size_of::<u64>() {
            return 0;
        }

        let mut address_bytes = [0_u8; std::mem::size_of::<u64>()];

        address_bytes.copy_from_slice(&value_bytes[..std::mem::size_of::<u64>()]);

        u64::from_le_bytes(address_bytes)
    }
}
