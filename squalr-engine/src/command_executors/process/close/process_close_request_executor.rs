use crate::{command_executors::engine_request_executor::EngineCommandRequestExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::process::close::{process_close_request::ProcessCloseRequest, process_close_response::ProcessCloseResponse};
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProcessCloseRequest {
    type ResponseType = ProcessCloseResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            log::info!(
                "Closing process {} with handle {}",
                process_info.get_process_id_raw(),
                process_info.get_handle()
            );

            match ProcessQuery::close_process(process_info.get_handle()) {
                Ok(_) => {
                    engine_privileged_state
                        .get_process_manager()
                        .clear_opened_process();
                }
                Err(error) => {
                    log::error!("Failed to close process handle {}: {}", process_info.get_handle(), error);
                }
            }

            ProcessCloseResponse {
                process_info: Some(process_info),
            }
        } else {
            log::error!("No process to close");
            ProcessCloseResponse { process_info: None }
        }
    }
}
