use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::project_explorer_pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState};

pub fn build_visible_project_entry_rows(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<PaneEntryRow> {
    let visible_project_count = project_explorer_pane_state.project_entries.len().min(5);
    let mut entry_rows = Vec::with_capacity(visible_project_count);

    for visible_project_position in 0..visible_project_count {
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

pub fn build_visible_project_item_entry_rows(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<PaneEntryRow> {
    let visible_project_item_count = project_explorer_pane_state
        .project_item_visible_entries
        .len()
        .min(10);
    let mut entry_rows = Vec::with_capacity(visible_project_item_count);

    for visible_project_item_position in 0..visible_project_item_count {
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
