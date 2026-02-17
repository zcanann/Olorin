use crate::views::struct_viewer::pane_state::StructViewerPaneState;

pub fn build_struct_viewer_summary_lines(struct_viewer_pane_state: &StructViewerPaneState) -> Vec<String> {
    let selected_field_display_format = struct_viewer_pane_state.selected_field_active_display_format();
    let selected_field_display_format_progress = struct_viewer_pane_state.selected_field_display_format_progress();
    let selected_field_edit_state = struct_viewer_pane_state.selected_field_edit_state_label();
    let mut summary_lines = vec![
        "Actions: r refresh source, Up/Down or j/k select field, Enter commit field edit.".to_string(),
        "Display format: [ previous, ] next. Disabled while an uncommitted edit exists.".to_string(),
        "Edit mode: type, Backspace, Ctrl+u clear. Value fields only.".to_string(),
        format!("source={:?}", struct_viewer_pane_state.source),
        format!("selected_struct={:?}", struct_viewer_pane_state.selected_struct_name),
        format!("field_count={}", struct_viewer_pane_state.focused_field_count()),
        format!("selected_field={:?}", struct_viewer_pane_state.selected_field_name),
        format!("selected_field_edit_state={}", selected_field_edit_state),
        format!(
            "selected_field_format={}",
            selected_field_display_format
                .map(|active_display_format| active_display_format.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "selected_field_format_index={}",
            selected_field_display_format_progress
                .map(|(active_display_value_index, display_value_count)| { format!("{}/{}", active_display_value_index + 1, display_value_count) })
                .unwrap_or_else(|| "0/0".to_string())
        ),
        format!("pending_edit={}", struct_viewer_pane_state.pending_edit_text),
        format!("uncommitted_edit={}", struct_viewer_pane_state.has_uncommitted_edit),
        format!("selected_scan_results={}", struct_viewer_pane_state.selected_scan_result_refs.len()),
        format!("selected_project_items={}", struct_viewer_pane_state.selected_project_item_paths.len()),
        format!("committing={}", struct_viewer_pane_state.is_committing_edit),
        format!("status={}", struct_viewer_pane_state.status_message),
    ];

    let visible_field_count = struct_viewer_pane_state.focused_field_count().min(5);
    for field_position in 0..visible_field_count {
        if let Some(focused_field) = struct_viewer_pane_state
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| focused_struct.get_fields().get(field_position))
        {
            let selected_marker = if struct_viewer_pane_state.selected_field_position == Some(field_position) {
                ">"
            } else {
                " "
            };
            let field_kind_marker = StructViewerPaneState::field_kind_marker(focused_field);
            let editability_marker = StructViewerPaneState::field_editability_marker(focused_field);
            let field_name = focused_field.get_name();
            let format_suffix = struct_viewer_pane_state
                .active_display_value_for_field(field_name)
                .map(|active_display_value| format!(" ({})", active_display_value.get_anonymous_value_string_format()))
                .unwrap_or_else(String::new);
            let value_preview = struct_viewer_pane_state
                .active_display_value_for_field(field_name)
                .map(|active_display_value| active_display_value.get_anonymous_value_string().to_string())
                .unwrap_or_else(|| "<nested>".to_string());
            summary_lines.push(format!(
                "{} [{}|{}] {}{} = {}",
                selected_marker, field_kind_marker, editability_marker, field_name, format_suffix, value_preview
            ));
        }
    }

    summary_lines
}
