use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState};

pub fn build_visible_project_entry_rows(
    project_explorer_pane_state: &ProjectExplorerPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let visible_project_range = build_selection_relative_viewport_range(
        project_explorer_pane_state.project_entries.len(),
        project_explorer_pane_state.selected_project_list_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_project_range.len());

    for visible_project_position in visible_project_range {
        if let Some(project_entry) = project_explorer_pane_state
            .project_entries
            .get(visible_project_position)
        {
            let is_selected_project = project_explorer_pane_state.selected_project_list_index == Some(visible_project_position);
            let is_active_project = project_explorer_pane_state
                .active_project_directory_path
                .as_ref()
                .zip(project_entry.get_project_directory())
                .is_some_and(|(active_project_directory, project_entry_directory)| *active_project_directory == project_entry_directory);
            let project_directory_display = project_entry
                .get_project_directory()
                .map(|project_directory| project_directory.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            let marker_text = if is_active_project { "*".to_string() } else { String::new() };
            let primary_text = project_entry.get_name().to_string();
            let secondary_text = Some(project_directory_display);

            if project_explorer_pane_state.focus_target != ProjectExplorerFocusTarget::ProjectList {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else if is_selected_project {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

pub fn build_visible_project_item_entry_rows(
    project_explorer_pane_state: &ProjectExplorerPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let visible_project_item_range = build_selection_relative_viewport_range(
        project_explorer_pane_state.project_item_visible_entries.len(),
        project_explorer_pane_state.selected_project_item_visible_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_project_item_range.len());

    for visible_project_item_position in visible_project_item_range {
        if let Some(project_item_entry) = project_explorer_pane_state
            .project_item_visible_entries
            .get(visible_project_item_position)
        {
            let is_selected_project_item = project_explorer_pane_state.selected_project_item_visible_index == Some(visible_project_item_position);
            let activation_marker = if project_item_entry.is_activated { "*" } else { " " };
            let directory_marker = if project_item_entry.is_directory {
                if project_item_entry.is_expanded { "-" } else { "+" }
            } else {
                " "
            };
            let indentation = " ".repeat(project_item_entry.depth.saturating_mul(2));
            let marker_text = format!("{}{}", activation_marker, directory_marker);
            let primary_text = format!("{}{}", indentation, project_item_entry.display_name);
            let secondary_text = Some(project_item_entry.project_item_path.display().to_string());

            if project_explorer_pane_state.focus_target != ProjectExplorerFocusTarget::ProjectHierarchy {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else if is_selected_project_item {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

#[cfg(test)]
mod tests {
    use crate::views::project_explorer::entry_rows::{build_visible_project_entry_rows, build_visible_project_item_entry_rows};
    use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState, ProjectHierarchyEntry};
    use squalr_engine_api::structures::projects::{project_info::ProjectInfo, project_manifest::ProjectManifest};
    use std::path::PathBuf;

    #[test]
    fn project_rows_window_tracks_selected_project_position() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.project_entries = (0..10)
            .map(|project_position| {
                ProjectInfo::new(
                    PathBuf::from(format!("C:/Projects/Project{project_position}/project/squalr-project.json")),
                    None,
                    ProjectManifest::new(Vec::new()),
                )
            })
            .collect();
        project_explorer_pane_state.selected_project_list_index = Some(7);
        project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectList;

        let entry_rows = build_visible_project_entry_rows(&project_explorer_pane_state, 5);
        let entry_directories: Vec<String> = entry_rows
            .iter()
            .map(|entry_row| entry_row.secondary_text.clone().unwrap_or_default())
            .collect();

        assert_eq!(
            entry_directories,
            vec![
                "C:/Projects/Project5/project",
                "C:/Projects/Project6/project",
                "C:/Projects/Project7/project",
                "C:/Projects/Project8/project",
                "C:/Projects/Project9/project",
            ]
        );
    }

    #[test]
    fn project_item_rows_window_tracks_selected_item_position() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.project_item_visible_entries = (0..16)
            .map(|item_position| ProjectHierarchyEntry {
                project_item_path: PathBuf::from(format!("root/item-{item_position}.json")),
                display_name: format!("item-{item_position}"),
                depth: 0,
                is_directory: false,
                is_expanded: false,
                is_activated: false,
            })
            .collect();
        project_explorer_pane_state.selected_project_item_visible_index = Some(12);
        project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectHierarchy;

        let entry_rows = build_visible_project_item_entry_rows(&project_explorer_pane_state, 10);
        let entry_names: Vec<&str> = entry_rows
            .iter()
            .map(|entry_row| entry_row.primary_text.trim())
            .collect();

        assert_eq!(
            entry_names,
            vec![
                "item-6", "item-7", "item-8", "item-9", "item-10", "item-11", "item-12", "item-13", "item-14", "item-15",
            ]
        );
    }
}
