use crate::commands::settings::memory::list::memory_settings_list_response::MemorySettingsListResponse;
use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemorySettingsListRequest {}

impl EngineCommandRequest for MemorySettingsListRequest {
    type ResponseType = MemorySettingsListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::List {
                memory_settings_list_request: self.clone(),
            },
        })
    }
}

impl From<MemorySettingsListResponse> for MemorySettingsResponse {
    fn from(memory_settings_list_response: MemorySettingsListResponse) -> Self {
        MemorySettingsResponse::List { memory_settings_list_response }
    }
}
