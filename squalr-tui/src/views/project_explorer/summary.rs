use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;

pub fn build_project_explorer_summary_lines(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<String> {
    vec![
        "[MODE] p project-list | i hierarchy.".to_string(),
        "[LIST] r refresh | n create | Enter/o open | e rename | x delete | c close.".to_string(),
        "[TREE] h refresh | j/k select | l/Right expand | Left collapse | Space activate.".to_string(),
        "[MOVE] n folder | x delete(confirm) | m stage | b move | [/] reorder | u clear-stage.".to_string(),
        "[INPUT] type | Backspace | Ctrl+u clear | Enter commit | Esc cancel.".to_string(),
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
        "[ROWS] projects=5 | hierarchy=10.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::build_project_explorer_summary_lines;
    use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;

    #[test]
    fn summary_uses_condensed_marker_group_lead_lines() {
        let project_explorer_pane_state = ProjectExplorerPaneState::default();
        let summary_lines = build_project_explorer_summary_lines(&project_explorer_pane_state);

        assert!(summary_lines[0].starts_with("[MODE]"));
        assert!(summary_lines[1].starts_with("[LIST]"));
        assert!(summary_lines[2].starts_with("[TREE]"));
        assert!(summary_lines[3].starts_with("[MOVE]"));
        assert!(summary_lines[4].starts_with("[INPUT]"));
    }
}
