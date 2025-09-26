use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::process::process_response::ProcessResponse;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineCommandResponse {
    Memory(MemoryResponse),
    Process(ProcessResponse),
    Results(ScanResultsResponse),
    Project(ProjectResponse),
    ProjectItems(ProjectItemsResponse),
    Scan(ScanResponse),
    Settings(SettingsResponse),
    TrackableTasks(TrackableTasksResponse),
}

pub trait TypedEngineCommandResponse: Sized {
    fn to_engine_response(&self) -> EngineCommandResponse;
    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse>;
}
