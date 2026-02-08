use serde_json::Value;
use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::scan::new::scan_new_request::ScanNewRequest as LegacyScanNewRequest;
use squalr_engine_api::api::commands::scan::new::scan_new_response::ScanNewResponse as LegacyScanNewResponse;
use squalr_engine_api::api::commands::scan::reset::scan_reset_request::ScanResetRequest as LegacyScanResetRequest;
use squalr_engine_api::api::commands::scan::reset::scan_reset_response::ScanResetResponse as LegacyScanResetResponse;
use squalr_engine_api::api::commands::stateless::process::ProcessSessionHandle;
use squalr_engine_api::api::commands::stateless::scan::{
    ScanNewRequest as StatelessScanNewRequest, ScanResetRequest as StatelessScanResetRequest, ScanSessionHandle, StatelessScanRequest, StatelessScanResponse,
};
use uuid::Uuid;

#[test]
fn stateless_scan_contract_json_round_trip_requests() {
    let session_handle = ScanSessionHandle {
        session_id: Uuid::from_u128(0xAAAABBBBCCCCDDDDEEEEFFFF00001111),
    };
    let new_request = StatelessScanRequest::New(StatelessScanNewRequest {
        process_session_handle: ProcessSessionHandle {
            session_id: Uuid::from_u128(0x99998888777766665555444433332222),
        },
    });
    let reset_request = StatelessScanRequest::Reset(StatelessScanResetRequest { session_handle });

    let serialized_new_request = serde_json::to_string(&new_request).expect("Stateless scan new request should serialize.");
    let serialized_reset_request = serde_json::to_string(&reset_request).expect("Stateless scan reset request should serialize.");

    let deserialized_new_request: StatelessScanRequest = serde_json::from_str(&serialized_new_request).expect("Stateless scan new request should deserialize.");
    let deserialized_reset_request: StatelessScanRequest =
        serde_json::from_str(&serialized_reset_request).expect("Stateless scan reset request should deserialize.");

    match deserialized_new_request {
        StatelessScanRequest::New(scan_new_request) => {
            assert_eq!(
                scan_new_request.process_session_handle.session_id,
                Uuid::from_u128(0x99998888777766665555444433332222)
            );
        }
        _ => panic!("Deserialized stateless scan request did not match new variant."),
    }

    match deserialized_reset_request {
        StatelessScanRequest::Reset(scan_reset_request) => {
            assert_eq!(
                scan_reset_request.session_handle.session_id,
                Uuid::from_u128(0xAAAABBBBCCCCDDDDEEEEFFFF00001111)
            );
        }
        _ => panic!("Deserialized stateless scan request did not match reset variant."),
    }
}

#[test]
fn stateless_scan_contract_json_round_trip_responses() {
    let new_response = StatelessScanResponse::New(squalr_engine_api::api::commands::stateless::scan::ScanNewResponse {
        session_handle: Some(ScanSessionHandle {
            session_id: Uuid::from_u128(0x0123456789ABCDEFFEDCBA9876543210),
        }),
    });
    let reset_response = StatelessScanResponse::Reset(squalr_engine_api::api::commands::stateless::scan::ScanResetResponse {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0x11112222333344445555666677778888),
        },
        success: true,
    });

    let serialized_new_response = serde_json::to_string(&new_response).expect("Stateless scan new response should serialize.");
    let serialized_reset_response = serde_json::to_string(&reset_response).expect("Stateless scan reset response should serialize.");

    let deserialized_new_response: StatelessScanResponse =
        serde_json::from_str(&serialized_new_response).expect("Stateless scan new response should deserialize.");
    let deserialized_reset_response: StatelessScanResponse =
        serde_json::from_str(&serialized_reset_response).expect("Stateless scan reset response should deserialize.");

    match deserialized_new_response {
        StatelessScanResponse::New(scan_new_response) => {
            assert_eq!(
                scan_new_response.session_handle.map(|handle| handle.session_id),
                Some(Uuid::from_u128(0x0123456789ABCDEFFEDCBA9876543210))
            );
        }
        _ => panic!("Deserialized stateless scan response did not match new variant."),
    }

    match deserialized_reset_response {
        StatelessScanResponse::Reset(scan_reset_response) => {
            assert!(scan_reset_response.success);
            assert_eq!(
                scan_reset_response.session_handle.session_id,
                Uuid::from_u128(0x11112222333344445555666677778888)
            );
        }
        _ => panic!("Deserialized stateless scan response did not match reset variant."),
    }
}

#[test]
fn scan_response_typed_mapping_round_trip_reset() {
    let typed_reset_response = LegacyScanResetResponse { success: true };
    let engine_response = typed_reset_response.to_engine_response();
    let remapped_typed_reset_response =
        LegacyScanResetResponse::from_engine_response(engine_response).expect("Typed scan reset response should round trip through privileged response.");

    assert!(remapped_typed_reset_response.success);
}

#[test]
fn scan_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = LegacyScanNewResponse {}.to_engine_response();
    let typed_result = LegacyScanResetResponse::from_engine_response(mismatched_engine_response);

    assert!(typed_result.is_err(), "Unexpectedly mapped a scan new response into a reset response.");
}

#[test]
fn stateless_scan_reset_request_contains_legacy_payload_fields() {
    let legacy_reset_request = LegacyScanResetRequest {};
    let stateless_reset_request = StatelessScanResetRequest {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0xF1F2F3F4F5F6F7F8F9FAFBFCFDFEFF00),
        },
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_reset_request).expect("Legacy scan reset request should serialize.");
    let mut stateless_serialized_request = serde_json::to_value(&stateless_reset_request).expect("Stateless scan reset request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless scan reset request should be a JSON object.")
        .remove("session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_scan_new_request_contains_legacy_payload_fields() {
    let legacy_new_request = LegacyScanNewRequest {};
    let stateless_new_request = StatelessScanNewRequest {
        process_session_handle: ProcessSessionHandle {
            session_id: Uuid::from_u128(0x0F0E0D0C0B0A09080706050403020100),
        },
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_new_request).expect("Legacy scan new request should serialize.");
    let mut stateless_serialized_request = serde_json::to_value(&stateless_new_request).expect("Stateless scan new request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless scan new request should be a JSON object.")
        .remove("process_session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_scan_request_variants_serialize_with_expected_variant_names() {
    let request = StatelessScanRequest::Reset(StatelessScanResetRequest {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0x12121212343434345656565678787878),
        },
    });
    let serialized_request = serde_json::to_value(&request).expect("Stateless scan request should serialize.");

    match serialized_request {
        Value::Object(serialized_request_object) => {
            assert!(
                serialized_request_object.contains_key("Reset"),
                "Serialized stateless scan request should contain the Reset variant key."
            );
        }
        _ => panic!("Serialized stateless scan request should be an object."),
    }
}
