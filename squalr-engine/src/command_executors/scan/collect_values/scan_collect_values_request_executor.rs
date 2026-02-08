use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollector;
use std::sync::Arc;
use std::thread;

const TASK_NAME: &str = "Value Collector";

impl PrivilegedCommandRequestExecutor for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let task = TrackableTask::create(TASK_NAME.to_string(), None);
            let task_handle = task.get_task_handle();
            let progress_receiver = task.subscribe_to_progress_updates();
            let engine_privileged_state = engine_privileged_state.clone();
            let task_for_execution = task.clone();
            let cancellation_token = task_for_execution.get_cancellation_token();
            let progress_reporter: Arc<dyn Fn(f32) + Send + Sync> = {
                let task_for_progress = task_for_execution.clone();
                Arc::new(move |progress: f32| {
                    task_for_progress.set_progress(progress);
                })
            };

            engine_privileged_state
                .get_trackable_task_manager()
                .register_task(task.clone());

            // Spawn a thread to listen to progress updates
            thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    log::info!("Progress: {:.2}%", progress);
                }
            });

            thread::spawn(move || {
                let scan_execution_context = ScanExecutionContext::new(Some(cancellation_token), Some(progress_reporter));
                ValueCollector::collect_values(process_info.clone(), snapshot, true, &scan_execution_context);
                task_for_execution.complete();
            });

            thread::spawn(move || {
                task.wait_for_completion();
                engine_privileged_state
                    .get_trackable_task_manager()
                    .unregister_task(&task.get_task_identifier());
                engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });
            });

            ScanCollectValuesResponse {
                trackable_task_handle: Some(task_handle),
            }
        } else {
            log::error!("No opened process");
            ScanCollectValuesResponse { trackable_task_handle: None }
        }
    }
}
