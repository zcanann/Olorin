use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::process::open::process_open_response::ProcessOpenResponse;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;
use sysinfo::Pid;

impl EngineCommandRequestExecutor for ProcessOpenRequest {
    type ResponseType = ProcessOpenResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if self.process_id.is_none() && self.search_name.is_none() {
            log::error!("Error: Neither PID nor search name provided. Cannot open process.");
            return ProcessOpenResponse { opened_process_info: None };
        }

        log::info!("Opening process...");

        let options = ProcessQueryOptions {
            search_name: self.search_name.clone(),
            required_process_id: self.process_id.map(Pid::from_u32),
            require_windowed: false,
            match_case: self.match_case,
            fetch_icons: false,
            limit: Some(1),
        };

        let processes = ProcessQuery::get_processes(options);

        if let Some(process_info) = processes.first() {
            match ProcessQuery::open_process(&process_info) {
                Ok(opened_process_info) => {
                    engine_privileged_state
                        .get_process_manager()
                        .set_opened_process(opened_process_info.clone());

                    return ProcessOpenResponse {
                        opened_process_info: Some(opened_process_info),
                    };
                }
                Err(error) => {
                    log::info!("Failed to open process {}: {}", process_info.get_process_id_raw(), error);
                }
            }
        } else {
            log::error!("No matching process found.");
        }

        ProcessOpenResponse { opened_process_info: None }
    }
}
