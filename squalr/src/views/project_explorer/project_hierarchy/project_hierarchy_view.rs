use crate::{
    app_context::AppContext,
    ui::widgets::controls::button::Button,
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        project_item_entry_view::ProjectItemEntryView,
        view_data::{
            project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_pending_operation::ProjectHierarchyPendingOperation,
            project_hierarchy_take_over_state::ProjectHierarchyTakeOverState, project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
};
use eframe::egui::{Align, Layout, Response, ScrollArea, TextureHandle, Ui, Widget, vec2};
use epaint::Color32;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use std::collections::HashSet;
use std::path::PathBuf;
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
        let mut should_cancel_take_over = false;
        let mut delete_confirmation_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };
                let take_over_state = project_hierarchy_view_data.take_over_state.clone();
                let tree_entries = project_hierarchy_view_data.tree_entries.clone();
                let selected_project_item_path = project_hierarchy_view_data.selected_project_item_path.clone();
                let pending_operation = project_hierarchy_view_data.pending_operation.clone();

                user_interface.add(self.project_hierarchy_toolbar_view);

                if pending_operation == ProjectHierarchyPendingOperation::Deleting {
                    user_interface.label("Deleting project item(s)...");
                }

                match take_over_state {
                    ProjectHierarchyTakeOverState::None => {
                        ScrollArea::vertical()
                            .id_salt("project_hierarchy")
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                for tree_entry in &tree_entries {
                                    let is_selected = selected_project_item_path
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
                    }
                    ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => {
                        user_interface.label("Confirm deletion of selected project item(s).");

                        ScrollArea::vertical()
                            .id_salt("project_hierarchy_delete_confirmation")
                            .max_height(160.0)
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                for project_item_path in &project_item_paths {
                                    let project_item_name = project_item_path
                                        .file_name()
                                        .and_then(|value| value.to_str())
                                        .unwrap_or_default();
                                    user_interface.label(project_item_name);
                                }
                            });

                        user_interface.horizontal(|user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_cancel = user_interface.add_sized(
                                button_size,
                                Button::new_from_theme(&self.app_context.theme)
                                    .with_tooltip_text("Cancel project item deletion.")
                                    .background_color(Color32::TRANSPARENT),
                            );

                            if button_cancel.clicked() {
                                should_cancel_take_over = true;
                            }

                            let button_confirm_delete = user_interface.add_sized(
                                button_size,
                                Button::new_from_theme(&self.app_context.theme).with_tooltip_text("Permanently delete selected project item(s)."),
                            );

                            if button_confirm_delete.clicked() {
                                delete_confirmation_project_item_paths = Some(project_item_paths);
                            }
                        });
                    }
                }
            })
            .response;

        if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Delete)) {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if should_cancel_take_over {
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
        }

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
        let (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order) = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .map(|opened_project| {
                    let opened_project_directory_path = opened_project.get_project_info().get_project_directory();
                    let opened_project_item_paths = opened_project
                        .get_project_items()
                        .keys()
                        .map(|project_item_ref| project_item_ref.get_project_item_path().clone())
                        .collect::<HashSet<PathBuf>>();
                    let opened_project_sort_order = opened_project
                        .get_project_info()
                        .get_project_manifest()
                        .get_project_item_sort_order()
                        .iter()
                        .cloned()
                        .collect::<Vec<PathBuf>>();

                    (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order)
                })
                .unwrap_or((None, HashSet::new(), Vec::new())),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                (None, HashSet::new(), Vec::new())
            }
        };

        let (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order) = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .map(|project_hierarchy_view_data| {
                let loaded_project_directory_path = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory());
                let loaded_project_item_paths = project_hierarchy_view_data
                    .project_items
                    .iter()
                    .map(|(project_item_ref, _)| project_item_ref.get_project_item_path().clone())
                    .collect::<HashSet<PathBuf>>();
                let loaded_project_sort_order = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|project_info| {
                        project_info
                            .get_project_manifest()
                            .get_project_item_sort_order()
                            .iter()
                            .cloned()
                            .collect::<Vec<PathBuf>>()
                    })
                    .unwrap_or_default();

                (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order)
            })
            .unwrap_or((None, HashSet::new(), Vec::new()));

        let project_directory_changed = opened_project_directory_path != loaded_project_directory_path;
        let project_items_changed = opened_project_item_paths != loaded_project_item_paths;
        let sort_order_changed = opened_project_sort_order != loaded_project_sort_order;

        if project_directory_changed || project_items_changed || sort_order_changed {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }
}
