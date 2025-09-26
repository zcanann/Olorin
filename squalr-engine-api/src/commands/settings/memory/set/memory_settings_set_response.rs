use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySettingsSetResponse {}

impl TypedEngineCommandResponse for MemorySettingsSetResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::Set {
                memory_settings_set_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::Set { memory_settings_set_response },
        }) = response
        {
            Ok(memory_settings_set_response)
        } else {
            Err(response)
        }
    }
}
