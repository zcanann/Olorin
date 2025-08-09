use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use olorin_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use olorin_engine_api::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use olorin_engine_api::structures::data_types::built_in_types::bool32::data_type_bool32::DataTypeBool32;
use olorin_engine_api::structures::data_types::data_type::DataType;
use olorin_engine_api::structures::scan_results::scan_result::ScanResult;
use olorin_engine_memory::memory_writer::MemoryWriter;
use olorin_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsSetPropertyRequest {
    type ResponseType = ScanResultsSetPropertyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let symbol_registry = engine_privileged_state.get_registries().get_symbol_registry();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return ScanResultsSetPropertyResponse::default();
            }
        };
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = match snapshot.read() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                log::error!("Failed to acquire read lock on Snapshot: {}", error);

                return ScanResultsSetPropertyResponse::default();
            }
        };

        match self.field_namespace.as_str() {
            ScanResult::PROPERTY_NAME_VALUE => {
                for scan_result_ref in &self.scan_result_refs {
                    if let Some(scan_result) = snapshot_guard.get_scan_result(&symbol_registry, scan_result_ref.get_scan_result_index()) {
                        if let Ok(data_value) = symbol_registry_guard.deanonymize_value(scan_result.get_data_type_ref(), self.anonymous_value.get_value()) {
                            let value_bytes = data_value.get_value_bytes();
                            let address = scan_result.get_address();
                            if let Some(opened_process_info) = engine_privileged_state
                                .get_process_manager()
                                .get_opened_process()
                            {
                                // Best-effort attempt to write the property bytes.
                                let _ = MemoryWriter::get_instance().write_bytes(&opened_process_info, address, &value_bytes);
                            }
                        }
                    }
                }
            }
            ScanResult::PROPERTY_NAME_IS_FROZEN => {
                let data_type = DataTypeBool32 {};
                if let Ok(data_value) = data_type.deanonymize_value(self.anonymous_value.get_value()) {
                    let is_frozen = data_value.get_value_bytes().iter().any(|&byte| byte != 0);

                    // Fire an internal request to freeze.
                    let scan_results_freeze_request = ScanResultsFreezeRequest {
                        scan_result_refs: self.scan_result_refs.clone(),
                        is_frozen,
                    };

                    scan_results_freeze_request.execute(engine_privileged_state);
                }
            }
            ScanResult::PROPERTY_NAME_ADDRESS | ScanResult::PROPERTY_NAME_MODULE | ScanResult::PROPERTY_NAME_MODULE_OFFSET => {
                log::warn!("Cannot set read-only property {}", self.field_namespace);
            }
            _ => {
                log::warn!("Attempted to set unsupported property on scan result.");
            }
        }

        ScanResultsSetPropertyResponse::default()
    }
}
