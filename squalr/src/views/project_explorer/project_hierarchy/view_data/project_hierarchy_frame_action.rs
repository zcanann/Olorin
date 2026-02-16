use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyFrameAction {
    None,
    SelectProjectItem(PathBuf),
    ToggleDirectoryExpansion(PathBuf),
    SetProjectItemActivation(PathBuf, bool),
}
