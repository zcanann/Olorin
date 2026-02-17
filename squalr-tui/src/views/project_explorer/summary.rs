use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;

pub fn build_project_explorer_summary_lines(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<String> {
    vec![
        "Mode: p project list, i project hierarchy.".to_string(),
        "Project list: r refresh, n create, Enter/o open, e rename, x delete, c close active.".to_string(),
        "Hierarchy: h refresh, j/k select, l expand, Left collapse, Space activate.".to_string(),
        "Hierarchy cont: n new folder, x delete(confirm), m stage move, b move here, [/] reorder, u cancel move.".to_string(),
        "Input mode: type, Backspace, Ctrl+u clear, Enter commit, Esc cancel.".to_string(),
        format!("focus_target={:?}", project_explorer_pane_state.focus_target),
        format!("list_count={}", project_explorer_pane_state.project_entries.len()),
        format!("selected_name={:?}", project_explorer_pane_state.selected_project_name),
        format!("active_project={:?}", project_explorer_pane_state.active_project_name),
        format!("active_directory={:?}", project_explorer_pane_state.active_project_directory_path),
        format!("selected_item={:?}", project_explorer_pane_state.selected_item_path),
        format!("visible_item_count={}", project_explorer_pane_state.project_item_visible_entries.len()),
        format!("expanded={}", project_explorer_pane_state.is_hierarchy_expanded),
        format!("input_mode={:?}", project_explorer_pane_state.input_mode),
        format!("pending_name={}", project_explorer_pane_state.pending_project_name_input),
        format!("pending_move_count={}", project_explorer_pane_state.pending_move_source_paths.len()),
        format!(
            "pending_delete_count={}",
            project_explorer_pane_state
                .pending_delete_confirmation_paths
                .len()
        ),
        format!("awaiting_list={}", project_explorer_pane_state.is_awaiting_project_list_response),
        format!("awaiting_item_list={}", project_explorer_pane_state.is_awaiting_project_item_list_response),
        format!("creating={}", project_explorer_pane_state.is_creating_project),
        format!("opening={}", project_explorer_pane_state.is_opening_project),
        format!("renaming={}", project_explorer_pane_state.is_renaming_project),
        format!("deleting={}", project_explorer_pane_state.is_deleting_project),
        format!("closing={}", project_explorer_pane_state.is_closing_project),
        format!("creating_item={}", project_explorer_pane_state.is_creating_project_item),
        format!("deleting_item={}", project_explorer_pane_state.is_deleting_project_item),
        format!("moving_item={}", project_explorer_pane_state.is_moving_project_item),
        format!("reordering_item={}", project_explorer_pane_state.is_reordering_project_item),
        format!("activating_item={}", project_explorer_pane_state.is_toggling_project_item_activation),
        format!("status={}", project_explorer_pane_state.status_message),
        "Projects (top 5) + Hierarchy (top 10).".to_string(),
    ]
}
