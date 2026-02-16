use crate::{
    app_context::AppContext,
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        project_item_entry_view::ProjectItemEntryView,
        view_data::{project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_view_data::ProjectHierarchyViewData},
    },
};
use eframe::egui::{Align, Layout, Response, ScrollArea, TextureHandle, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectHierarchyView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
        }
    }
}

impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.refresh_if_project_changed();

        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };

                user_interface.add(self.project_hierarchy_toolbar_view);

                ScrollArea::vertical()
                    .id_salt("project_hierarchy")
                    .auto_shrink([false, false])
                    .show(user_interface, |user_interface| {
                        for tree_entry in &project_hierarchy_view_data.tree_entries {
                            let is_selected = project_hierarchy_view_data
                                .selected_project_item_path
                                .as_ref()
                                .map(|selected_project_item_path| selected_project_item_path == &tree_entry.project_item_path)
                                .unwrap_or(false);
                            let icon = Self::resolve_tree_entry_icon(
                                self.app_context.clone(),
                                tree_entry
                                    .project_item
                                    .get_item_type()
                                    .get_project_item_type_id(),
                            );

                            user_interface.add(ProjectItemEntryView::new(
                                self.app_context.clone(),
                                &tree_entry.project_item_path,
                                &tree_entry.display_name,
                                &tree_entry.preview_value,
                                tree_entry.depth,
                                icon,
                                is_selected,
                                tree_entry.is_directory,
                                tree_entry.has_children,
                                tree_entry.is_expanded,
                                &mut project_hierarchy_frame_action,
                            ));
                        }
                    });
            })
            .response;

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
            ProjectHierarchyFrameAction::SelectProjectItem(project_item_path) => {
                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
        }

        response
    }
}

impl ProjectHierarchyView {
    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        project_item_type_id: &str,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_data_type_blue_blocks_8.clone())
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_project_pointer_type.clone())
        } else {
            Some(icon_library.icon_handle_data_type_unknown.clone())
        }
    }

    fn refresh_if_project_changed(&self) {
        let opened_project_directory_path = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .and_then(|opened_project| opened_project.get_project_info().get_project_directory()),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                None
            }
        };
        let loaded_project_directory_path = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .and_then(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory())
            });

        if opened_project_directory_path != loaded_project_directory_path {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }
}
