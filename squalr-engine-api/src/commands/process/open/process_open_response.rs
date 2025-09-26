use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub opened_process_info: Option<OpenedProcessInfo>,
}

impl TypedEngineCommandResponse for ProcessOpenResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Process(ProcessResponse::Open {
            process_open_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Process(ProcessResponse::Open { process_open_response }) = response {
            Ok(process_open_response)
        } else {
            Err(response)
        }
    }
}
