use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::structures::projects::project_info::ProjectInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectRenameResponse {
    pub renamed_project_info: Option<ProjectInfo>,
}

impl TypedEngineCommandResponse for ProjectRenameResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Project(ProjectResponse::Rename {
            project_rename_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Project(ProjectResponse::Rename { project_rename_response }) = response {
            Ok(project_rename_response)
        } else {
            Err(response)
        }
    }
}
