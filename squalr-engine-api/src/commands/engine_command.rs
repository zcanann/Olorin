use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum EngineCommand {
    #[structopt(alias = "mem", alias = "m")]
    Memory(MemoryCommand),

    #[structopt(alias = "proc", alias = "pr")]
    Process(ProcessCommand),

    #[structopt(alias = "proj", alias = "p")]
    Project(ProjectCommand),

    #[structopt(alias = "proj_items", alias = "pi")]
    ProjectItems(ProjectItemsCommand),

    #[structopt(alias = "res", alias = "r")]
    Results(ScanResultsCommand),

    #[structopt(alias = "scan", alias = "s")]
    Scan(ScanCommand),

    #[structopt(alias = "set", alias = "st")]
    Settings(SettingsCommand),

    #[structopt(alias = "set", alias = "st")]
    TrackableTasks(TrackableTasksCommand),
}
