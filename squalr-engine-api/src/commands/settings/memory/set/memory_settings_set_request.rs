use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct MemorySettingsSetRequest {
    #[structopt(short = "m_n", long)]
    pub memory_type_none: Option<bool>,
    #[structopt(short = "m_p", long)]
    pub memory_type_private: Option<bool>,
    #[structopt(short = "m_i", long)]
    pub memory_type_image: Option<bool>,
    #[structopt(short = "m_m", long)]
    pub memory_type_mapped: Option<bool>,
    #[structopt(short = "r_w", long)]
    pub required_write: Option<bool>,
    #[structopt(short = "r_e", long)]
    pub required_execute: Option<bool>,
    #[structopt(short = "r_cow", long)]
    pub required_copy_on_write: Option<bool>,
    #[structopt(short = "e_w", long)]
    pub excluded_write: Option<bool>,
    #[structopt(short = "e_e", long)]
    pub excluded_execute: Option<bool>,
    #[structopt(short = "e_cow", long)]
    pub excluded_copy_on_write: Option<bool>,
    #[structopt(short = "s_adr", long)]
    pub start_address: Option<u64>,
    #[structopt(short = "e_adr", long)]
    pub end_address: Option<u64>,
    #[structopt(short = "usr", long)]
    pub only_query_usermode: Option<bool>,
}

impl EngineCommandRequest for MemorySettingsSetRequest {
    type ResponseType = MemorySettingsSetResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::Set {
                memory_settings_set_request: self.clone(),
            },
        })
    }
}

impl From<MemorySettingsSetResponse> for MemorySettingsResponse {
    fn from(memory_settings_set_response: MemorySettingsSetResponse) -> Self {
        MemorySettingsResponse::Set { memory_settings_set_response }
    }
}
