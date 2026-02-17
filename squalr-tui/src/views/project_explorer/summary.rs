use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;

pub fn build_project_explorer_summary_lines(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<String> {
    vec![
        "[MARK] project=* active | hierarchy=col1 active(*) col2 dir(+/-).".to_string(),
        "[MODE] p project-list | i hierarchy.".to_string(),
        "[LIST] r refresh | n create | Enter/o open | e rename | x delete | c close.".to_string(),
        "[TREE] h refresh | j/k select | l/Right expand | Left collapse | Space activate.".to_string(),
        "[MOVE] n folder | x delete(confirm) | m stage | b move | [/] reorder | u clear-stage.".to_string(),
        "[INPUT] type | Backspace | Ctrl+u clear | Enter commit | Esc cancel.".to_string(),
        format!(
            "[META] focus={:?} | input_mode={:?} | expanded={}.",
            project_explorer_pane_state.focus_target, project_explorer_pane_state.input_mode, project_explorer_pane_state.is_hierarchy_expanded
        ),
        format!(
            "[PROJ] selected={} | active={} | dir={}.",
            option_to_compact_text(project_explorer_pane_state.selected_project_name.as_deref()),
            option_to_compact_text(project_explorer_pane_state.active_project_name.as_deref()),
            option_path_to_compact_text(
                project_explorer_pane_state
                    .active_project_directory_path
                    .as_deref()
            )
        ),
        format!(
            "[ITEM] selected={} | visible_count={} | pending_name={}.",
            option_to_compact_text(project_explorer_pane_state.selected_item_path.as_deref()),
            project_explorer_pane_state.project_item_visible_entries.len(),
            project_explorer_pane_state.pending_project_name_input
        ),
        format!(
            "[PEND] move_count={} | delete_count={}.",
            project_explorer_pane_state.pending_move_source_paths.len(),
            project_explorer_pane_state
                .pending_delete_confirmation_paths
                .len()
        ),
        format!(
            "[WAIT] project_list={} | item_list={}.",
            project_explorer_pane_state.is_awaiting_project_list_response, project_explorer_pane_state.is_awaiting_project_item_list_response
        ),
        format!(
            "[OPS] create={} | open={} | rename={} | delete={} | close={}.",
            project_explorer_pane_state.is_creating_project,
            project_explorer_pane_state.is_opening_project,
            project_explorer_pane_state.is_renaming_project,
            project_explorer_pane_state.is_deleting_project,
            project_explorer_pane_state.is_closing_project
        ),
        format!(
            "[ITEM_OPS] create={} | delete={} | move={} | reorder={} | activate={}.",
            project_explorer_pane_state.is_creating_project_item,
            project_explorer_pane_state.is_deleting_project_item,
            project_explorer_pane_state.is_moving_project_item,
            project_explorer_pane_state.is_reordering_project_item,
            project_explorer_pane_state.is_toggling_project_item_activation
        ),
        format!(
            "[COUNT] project_count={} | visible_item_count={}.",
            project_explorer_pane_state.project_entries.len(),
            project_explorer_pane_state.project_item_visible_entries.len()
        ),
        format!("[STAT] {}.", project_explorer_pane_state.status_message),
        "[ROWS] projects=5 | hierarchy=10.".to_string(),
    ]
}

fn option_to_compact_text(option_text: Option<&str>) -> String {
    option_text
        .map(|text| format!("\"{}\"", text))
        .unwrap_or_else(|| "none".to_string())
}

fn option_path_to_compact_text(option_path: Option<&std::path::Path>) -> String {
    option_path
        .map(|path| format!("\"{}\"", path.display()))
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(test)]
mod tests {
    use super::build_project_explorer_summary_lines;
    use crate::views::project_explorer::pane_state::ProjectExplorerPaneState;

    #[test]
    fn summary_uses_condensed_marker_group_lead_lines() {
        let project_explorer_pane_state = ProjectExplorerPaneState::default();
        let summary_lines = build_project_explorer_summary_lines(&project_explorer_pane_state);

        assert!(summary_lines[0].starts_with("[MARK]"));
        assert!(summary_lines[1].starts_with("[MODE]"));
        assert!(summary_lines[2].starts_with("[LIST]"));
        assert!(summary_lines[3].starts_with("[TREE]"));
        assert!(summary_lines[4].starts_with("[MOVE]"));
        assert!(summary_lines[5].starts_with("[INPUT]"));
    }
}
