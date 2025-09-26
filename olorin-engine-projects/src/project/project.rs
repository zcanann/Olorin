use crate::project::project_tree::ProjectTree;
use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use olorin_engine_api::structures::{
    processes::process_icon::ProcessIcon,
    projects::{
        project_info::ProjectInfo,
        project_items::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item::ProjectItem},
        project_manifest::ProjectManifest,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Default)]
pub struct Project {
    project_info: ProjectInfo,

    // The tree structure for the project, with all parent relations.
    #[serde(skip)]
    project_tree_root: ProjectTree,

    // The full list of all project items. These are not arranged heirarchically, as the tree is responsible for that.
    #[serde(rename = "project_items")]
    project_items: Vec<ProjectItem>,
}

impl Project {
    pub const PROJECT_FILE: &'static str = "project.json";
    pub const PROJECT_DIR: &'static str = "project";

    pub fn new(
        project_info: ProjectInfo,
        project_tree_root: ProjectTree,
        project_items: Vec<ProjectItem>,
    ) -> Self {
        Self {
            project_info,
            project_tree_root,
            project_items,
        }
    }

    /// Creates a new project and writes it to disk.
    pub fn create_new_project(path: &Path) -> anyhow::Result<Self> {
        if path.exists() && path.read_dir()?.next().is_some() {
            anyhow::bail!("Cannot create project: directory already contains files.");
        }

        fs::create_dir_all(path)?;

        let project_info = ProjectInfo::new(path.to_path_buf(), None, ProjectManifest::default());
        let project_root = ProjectItemTypeDirectory::new_project_item(path);
        let project_tree_root = ProjectTree::new(project_root.get_ref(), true);

        let mut project = Self {
            project_info,
            project_tree_root,
            project_items: vec![project_root],
        };

        project.save_to_path(path, true)?;

        Ok(project)
    }

    pub fn save(
        &mut self,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        let project_path = self.project_info.get_path().to_owned();

        self.save_to_path(&project_path, save_even_if_unchanged)
    }

    pub fn get_name(&self) -> &str {
        self.project_info.get_name()
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_info.get_project_manifest()
    }

    pub fn get_project_info(&self) -> &ProjectInfo {
        &self.project_info
    }

    pub fn get_project_info_mut(&mut self) -> &mut ProjectInfo {
        &mut self.project_info
    }

    pub fn set_project_icon(
        &mut self,
        project_icon: Option<ProcessIcon>,
    ) {
        self.project_info.set_project_icon(project_icon);
    }

    /*
    pub fn get_project_tree(&self) -> &ProjectItem {
        &self.project_tree
    }

    pub fn get_project_tree_mut(&mut self) -> &mut ProjectItem {
        &mut self.project_tree
    }

    pub fn find_project_item_mut(
        &mut self,
        project_item_path: &Path,
    ) -> Option<&mut ProjectItem> {
        // Ensure the path is within the root.
        let root_path = self.project_tree.get_path();
        let relative_path = project_item_path.strip_prefix(root_path).ok()?;

        // Start at the root and search linearly.
        let mut current = &mut self.project_tree;

        // Iterate each path component (nested directories) of the relative path corresponding to the target project item.
        for component in relative_path.components() {
            let name = match component {
                Component::Normal(os_str) => os_str.to_str()?,
                _ => return None,
            };

            // Linear search for the next directory (or final node) among immediate children.
            current = current
                .get_children_mut()
                .iter_mut()
                .find(|child| child.get_file_or_directory_name() == name)?;
        }

        Some(current)
    }*/
}
