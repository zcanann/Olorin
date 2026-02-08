use crate::api::commands::stateless::project::ProjectSessionHandle;
use crate::structures::projects::project_info::ProjectInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

/// Stateless project-items command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProjectItemsRequest {
    Activate(ProjectItemsActivateRequest),
    List(ProjectItemsListRequest),
}

/// Stateless project-items command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProjectItemsResponse {
    Activate(ProjectItemsActivateResponse),
    List(ProjectItemsListResponse),
}

/// Stateless project-items activate request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsActivateRequest {
    pub session_handle: ProjectSessionHandle,
    pub project_item_paths: Vec<String>,
    pub is_activated: bool,
}

/// Stateless project-items activate response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectItemsActivateResponse {
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project-items list request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectItemsListRequest {
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project-items list response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsListResponse {
    pub session_handle: ProjectSessionHandle,
    pub opened_project_info: Option<ProjectInfo>,
    pub opened_project_root: Option<ProjectItem>,
}
