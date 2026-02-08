use crate::structures::projects::project_info::ProjectInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Explicit caller-owned project context used by stateless command flows.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProjectSessionHandle {
    pub session_id: Uuid,
}

/// Stateless project command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProjectRequest {
    Create(ProjectCreateRequest),
    Delete(ProjectDeleteRequest),
    Open(ProjectOpenRequest),
    Close(ProjectCloseRequest),
    Rename(ProjectRenameRequest),
    Save(ProjectSaveRequest),
    Export(ProjectExportRequest),
    List(ProjectListRequest),
}

/// Stateless project command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProjectResponse {
    Create(ProjectCreateResponse),
    Delete(ProjectDeleteResponse),
    Open(ProjectOpenResponse),
    Close(ProjectCloseResponse),
    Rename(ProjectRenameResponse),
    Save(ProjectSaveResponse),
    Export(ProjectExportResponse),
    List(ProjectListResponse),
}

/// Stateless project create request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectCreateRequest {
    pub project_directory_path: Option<PathBuf>,
    pub project_name: Option<String>,
}

/// Stateless project create response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectCreateResponse {
    pub success: bool,
    pub new_project_path: PathBuf,
}

/// Stateless project delete request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDeleteRequest {
    pub project_directory_path: Option<PathBuf>,
    pub project_name: Option<String>,
}

/// Stateless project delete response.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDeleteResponse {
    pub success: bool,
}

/// Stateless project open request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectOpenRequest {
    pub open_file_browser: bool,
    pub project_directory_path: Option<PathBuf>,
    pub project_name: Option<String>,
}

/// Stateless project open response carrying caller-owned context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectOpenResponse {
    pub success: bool,
    pub session_handle: Option<ProjectSessionHandle>,
}

/// Stateless project close request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCloseRequest {
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project close response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCloseResponse {
    pub success: bool,
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project rename request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRenameRequest {
    pub project_directory_path: PathBuf,
    pub new_project_name: String,
}

/// Stateless project rename response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRenameResponse {
    pub success: bool,
    pub new_project_path: PathBuf,
}

/// Stateless project save request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSaveRequest {
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project save response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSaveResponse {
    pub success: bool,
    pub session_handle: ProjectSessionHandle,
}

/// Stateless project export request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectExportRequest {
    pub project_directory_path: Option<PathBuf>,
    pub project_name: Option<String>,
    pub open_export_folder: bool,
}

/// Stateless project export response.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectExportResponse {
    pub success: bool,
}

/// Stateless project list request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectListRequest {}

/// Stateless project list response.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects_info: Vec<ProjectInfo>,
}
