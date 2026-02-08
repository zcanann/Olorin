use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest as LegacyGeneralSettingsSetRequest;
use squalr_engine_api::api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest as LegacyMemorySettingsSetRequest;
use squalr_engine_api::api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest as LegacyScanSettingsSetRequest;
use squalr_engine_api::api::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse as LegacyScanSettingsSetResponse;
use squalr_engine_api::api::commands::stateless::settings::{
    GeneralSettingsRequest, GeneralSettingsSetRequest as StatelessGeneralSettingsSetRequest, MemorySettingsRequest,
    MemorySettingsSetRequest as StatelessMemorySettingsSetRequest, ScanSettingsRequest, ScanSettingsSetRequest as StatelessScanSettingsSetRequest,
    StatelessSettingsRequest, StatelessSettingsResponse,
};
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;

#[test]
fn stateless_settings_contract_json_round_trip_requests() {
    let general_request = StatelessSettingsRequest::General(GeneralSettingsRequest::Set(StatelessGeneralSettingsSetRequest {
        engine_request_delay: Some(42),
    }));
    let memory_request = StatelessSettingsRequest::Memory(MemorySettingsRequest::Set(StatelessMemorySettingsSetRequest {
        memory_type_none: Some(false),
        memory_type_private: Some(true),
        memory_type_image: None,
        memory_type_mapped: None,
        required_write: Some(true),
        required_execute: None,
        required_copy_on_write: None,
        excluded_write: None,
        excluded_execute: Some(false),
        excluded_copy_on_write: None,
        start_address: Some(0x1000),
        end_address: Some(0x2000),
        only_query_usermode: Some(true),
    }));
    let scan_request = StatelessSettingsRequest::Scan(ScanSettingsRequest::Set(StatelessScanSettingsSetRequest {
        results_page_size: Some(256),
        results_read_interval_ms: Some(40),
        project_read_interval_ms: Some(50),
        freeze_interval_ms: Some(60),
        memory_alignment: Some(MemoryAlignment::Alignment4),
        memory_read_mode: Some(MemoryReadMode::ReadInterleavedWithScan),
        floating_point_tolerance: Some(FloatingPointTolerance::Tolerance10E3),
        is_single_threaded_scan: Some(false),
        debug_perform_validation_scan: Some(true),
    }));

    let serialized_general_request = serde_json::to_string(&general_request).expect("Stateless general-settings request should serialize.");
    let serialized_memory_request = serde_json::to_string(&memory_request).expect("Stateless memory-settings request should serialize.");
    let serialized_scan_request = serde_json::to_string(&scan_request).expect("Stateless scan-settings request should serialize.");

    let deserialized_general_request: StatelessSettingsRequest =
        serde_json::from_str(&serialized_general_request).expect("Stateless general-settings request should deserialize.");
    let deserialized_memory_request: StatelessSettingsRequest =
        serde_json::from_str(&serialized_memory_request).expect("Stateless memory-settings request should deserialize.");
    let deserialized_scan_request: StatelessSettingsRequest =
        serde_json::from_str(&serialized_scan_request).expect("Stateless scan-settings request should deserialize.");

    match deserialized_general_request {
        StatelessSettingsRequest::General(GeneralSettingsRequest::Set(general_settings_set_request)) => {
            assert_eq!(general_settings_set_request.engine_request_delay, Some(42));
        }
        _ => panic!("Deserialized stateless settings request did not match general set variant."),
    }

    match deserialized_memory_request {
        StatelessSettingsRequest::Memory(MemorySettingsRequest::Set(memory_settings_set_request)) => {
            assert_eq!(memory_settings_set_request.required_write, Some(true));
            assert_eq!(memory_settings_set_request.start_address, Some(0x1000));
            assert_eq!(memory_settings_set_request.end_address, Some(0x2000));
        }
        _ => panic!("Deserialized stateless settings request did not match memory set variant."),
    }

    match deserialized_scan_request {
        StatelessSettingsRequest::Scan(ScanSettingsRequest::Set(scan_settings_set_request)) => {
            assert_eq!(scan_settings_set_request.results_page_size, Some(256));
            assert_eq!(scan_settings_set_request.memory_read_mode, Some(MemoryReadMode::ReadInterleavedWithScan));
            assert_eq!(scan_settings_set_request.floating_point_tolerance, Some(FloatingPointTolerance::Tolerance10E3));
        }
        _ => panic!("Deserialized stateless settings request did not match scan set variant."),
    }
}

#[test]
fn stateless_settings_contract_json_round_trip_responses() {
    let response = StatelessSettingsResponse::Scan(squalr_engine_api::api::commands::stateless::settings::ScanSettingsResponse::Set(
        squalr_engine_api::api::commands::stateless::settings::ScanSettingsSetResponse {},
    ));
    let serialized_response = serde_json::to_string(&response).expect("Stateless settings response should serialize.");
    let deserialized_response: StatelessSettingsResponse = serde_json::from_str(&serialized_response).expect("Stateless settings response should deserialize.");

    match deserialized_response {
        StatelessSettingsResponse::Scan(squalr_engine_api::api::commands::stateless::settings::ScanSettingsResponse::Set(_)) => {}
        _ => panic!("Deserialized stateless settings response did not match scan set variant."),
    }
}

#[test]
fn settings_response_typed_mapping_round_trip_scan_set() {
    let typed_scan_set_response = LegacyScanSettingsSetResponse {};
    let engine_response = typed_scan_set_response.to_engine_response();
    let remapped_typed_scan_set_response = LegacyScanSettingsSetResponse::from_engine_response(engine_response)
        .expect("Typed scan-settings set response should round trip through privileged response.");

    let serialized_scan_settings_response =
        serde_json::to_string(&remapped_typed_scan_set_response).expect("Remapped typed scan-settings set response should serialize.");
    assert_eq!(serialized_scan_settings_response, "{}");
}

#[test]
fn stateless_general_settings_set_request_contains_legacy_payload_fields() {
    let legacy_request = LegacyGeneralSettingsSetRequest {
        engine_request_delay: Some(30),
    };
    let stateless_request = StatelessGeneralSettingsSetRequest {
        engine_request_delay: legacy_request.engine_request_delay,
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_request).expect("Legacy general-settings set request should serialize.");
    let stateless_serialized_request = serde_json::to_value(&stateless_request).expect("Stateless general-settings set request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_memory_settings_set_request_contains_legacy_payload_fields() {
    let legacy_request = LegacyMemorySettingsSetRequest {
        memory_type_none: Some(false),
        memory_type_private: Some(true),
        memory_type_image: Some(false),
        memory_type_mapped: Some(true),
        required_write: Some(true),
        required_execute: Some(false),
        required_copy_on_write: Some(false),
        excluded_write: Some(false),
        excluded_execute: Some(true),
        excluded_copy_on_write: Some(false),
        start_address: Some(0xABCD),
        end_address: Some(0xBCDE),
        only_query_usermode: Some(true),
    };
    let stateless_request = StatelessMemorySettingsSetRequest {
        memory_type_none: legacy_request.memory_type_none,
        memory_type_private: legacy_request.memory_type_private,
        memory_type_image: legacy_request.memory_type_image,
        memory_type_mapped: legacy_request.memory_type_mapped,
        required_write: legacy_request.required_write,
        required_execute: legacy_request.required_execute,
        required_copy_on_write: legacy_request.required_copy_on_write,
        excluded_write: legacy_request.excluded_write,
        excluded_execute: legacy_request.excluded_execute,
        excluded_copy_on_write: legacy_request.excluded_copy_on_write,
        start_address: legacy_request.start_address,
        end_address: legacy_request.end_address,
        only_query_usermode: legacy_request.only_query_usermode,
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_request).expect("Legacy memory-settings set request should serialize.");
    let stateless_serialized_request = serde_json::to_value(&stateless_request).expect("Stateless memory-settings set request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_scan_settings_set_request_contains_legacy_payload_fields() {
    let legacy_request = LegacyScanSettingsSetRequest {
        results_page_size: Some(512),
        results_read_interval_ms: Some(100),
        project_read_interval_ms: Some(150),
        freeze_interval_ms: Some(200),
        memory_alignment: Some(MemoryAlignment::Alignment8),
        memory_read_mode: Some(MemoryReadMode::ReadBeforeScan),
        floating_point_tolerance: Some(FloatingPointTolerance::Tolerance10E4),
        is_single_threaded_scan: Some(true),
        debug_perform_validation_scan: Some(false),
    };
    let stateless_request = StatelessScanSettingsSetRequest {
        results_page_size: legacy_request.results_page_size,
        results_read_interval_ms: legacy_request.results_read_interval_ms,
        project_read_interval_ms: legacy_request.project_read_interval_ms,
        freeze_interval_ms: legacy_request.freeze_interval_ms,
        memory_alignment: legacy_request.memory_alignment,
        memory_read_mode: legacy_request.memory_read_mode,
        floating_point_tolerance: legacy_request.floating_point_tolerance,
        is_single_threaded_scan: legacy_request.is_single_threaded_scan,
        debug_perform_validation_scan: legacy_request.debug_perform_validation_scan,
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_request).expect("Legacy scan-settings set request should serialize.");
    let stateless_serialized_request = serde_json::to_value(&stateless_request).expect("Stateless scan-settings set request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}
