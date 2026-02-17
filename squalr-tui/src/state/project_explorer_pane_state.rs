/// Stores state for browsing projects and project items.
#[derive(Clone, Debug, Default)]
pub struct ProjectExplorerPaneState {
    pub active_project_name: Option<String>,
    pub selected_item_path: Option<String>,
    pub is_hierarchy_expanded: bool,
}
