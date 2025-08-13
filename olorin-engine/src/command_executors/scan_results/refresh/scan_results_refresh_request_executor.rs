use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use olorin_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use olorin_engine_api::structures::memory::pointer::Pointer;
use olorin_engine_api::structures::scan_results::scan_result::ScanResult;
use olorin_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use olorin_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use olorin_engine_memory::memory_reader::MemoryReader;
use olorin_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsRefreshRequest {
    type ResponseType = ScanResultsRefreshResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let symbol_registry = engine_privileged_state.get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return ScanResultsRefreshResponse::default();
            }
        };
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = match snapshot.read() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                log::error!("Failed to acquire read lock on Snapshot: {}", error);

                return ScanResultsRefreshResponse::default();
            }
        };
        let mut scan_results_list = vec![];

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        // Wrap each ScanResultBase with a full ScanResult that includes current values and module information.
        for scan_result_ref in self.scan_result_refs.clone().into_iter() {
            if let Some(scan_result) = snapshot_guard.get_scan_result(&symbol_registry, scan_result_ref.get_scan_result_index()) {
                let mut recently_read_value = None;
                let mut module_name = String::default();
                let address = scan_result.get_address();
                let mut module_offset = address;

                // Best-effort attempt to read the values for this scan result.
                if let Some(opened_process_info) = engine_privileged_state
                    .get_process_manager()
                    .get_opened_process()
                {
                    if let Some(mut data_value) = scan_result.get_current_value().clone() {
                        if MemoryReader::get_instance().read(&opened_process_info, address, &mut data_value) {
                            recently_read_value = Some(data_value);
                        }
                    }
                }

                // Check whether this scan result belongs to a module (ie check if the address is static).
                if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(address, &modules) {
                    module_name = found_module_name;
                    module_offset = address;
                }

                let pointer = Pointer::new(module_offset, vec![], module_name.clone());
                let is_frozen = if let Ok(freeze_list_registry) = engine_privileged_state.get_freeze_list_registry().read() {
                    freeze_list_registry.is_address_frozen(&pointer)
                } else {
                    false
                };

                let recently_read_display_values = if let Some(data_value) = recently_read_value.as_ref() {
                    Some(symbol_registry_guard.create_display_values(data_value.get_data_type_ref(), data_value.get_value_bytes()))
                } else {
                    None
                };

                scan_results_list.push(ScanResult::new(
                    scan_result,
                    module_name,
                    module_offset,
                    recently_read_value,
                    recently_read_display_values,
                    is_frozen,
                ));
            }
        }

        ScanResultsRefreshResponse {
            scan_results: scan_results_list,
        }
    }
}
