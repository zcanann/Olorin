/// Stores state for viewing and editing selected structures.
#[derive(Clone, Debug, Default)]
pub struct StructViewerPaneState {
    pub selected_struct_name: Option<String>,
    pub selected_field_name: Option<String>,
    pub has_uncommitted_edit: bool,
}
