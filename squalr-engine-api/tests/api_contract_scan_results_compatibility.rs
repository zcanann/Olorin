use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest as LegacyScanResultsDeleteRequest;
use squalr_engine_api::api::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse as LegacyScanResultsDeleteResponse;
use squalr_engine_api::api::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse as LegacyScanResultsFreezeResponse;
use squalr_engine_api::api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest as LegacyScanResultsListRequest;
use squalr_engine_api::api::commands::stateless::scan::ScanSessionHandle;
use squalr_engine_api::api::commands::stateless::scan_results::{
    ScanResultsDeleteRequest as StatelessScanResultsDeleteRequest, ScanResultsListRequest as StatelessScanResultsListRequest,
    ScanResultsRefreshRequest as StatelessScanResultsRefreshRequest, ScanResultsRefreshResponse as StatelessScanResultsRefreshResponse,
    StatelessScanResultsRequest, StatelessScanResultsResponse,
};
use squalr_engine_api::api::types::scan_results::scan_result_ref::ScanResultRef;
use uuid::Uuid;

#[test]
fn stateless_scan_results_contract_json_round_trip_requests() {
    let session_handle = ScanSessionHandle {
        session_id: Uuid::from_u128(0xABCABCABCABCABCABCABCABCABCABCAB),
    };
    let list_request = StatelessScanResultsRequest::List(StatelessScanResultsListRequest { session_handle, page_index: 5 });
    let refresh_request = StatelessScanResultsRequest::Refresh(StatelessScanResultsRefreshRequest {
        session_handle,
        scan_result_refs: vec![ScanResultRef::new(10), ScanResultRef::new(11)],
    });

    let serialized_list_request = serde_json::to_string(&list_request).expect("Stateless scan results list request should serialize.");
    let serialized_refresh_request = serde_json::to_string(&refresh_request).expect("Stateless scan results refresh request should serialize.");

    let deserialized_list_request: StatelessScanResultsRequest =
        serde_json::from_str(&serialized_list_request).expect("Stateless scan results list request should deserialize.");
    let deserialized_refresh_request: StatelessScanResultsRequest =
        serde_json::from_str(&serialized_refresh_request).expect("Stateless scan results refresh request should deserialize.");

    match deserialized_list_request {
        StatelessScanResultsRequest::List(scan_results_list_request) => {
            assert_eq!(scan_results_list_request.page_index, 5);
            assert_eq!(
                scan_results_list_request.session_handle.session_id,
                Uuid::from_u128(0xABCABCABCABCABCABCABCABCABCABCAB)
            );
        }
        _ => panic!("Deserialized stateless scan results request did not match list variant."),
    }

    match deserialized_refresh_request {
        StatelessScanResultsRequest::Refresh(scan_results_refresh_request) => {
            assert_eq!(scan_results_refresh_request.scan_result_refs.len(), 2);
            assert_eq!(
                scan_results_refresh_request
                    .scan_result_refs
                    .first()
                    .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index()),
                Some(10)
            );
        }
        _ => panic!("Deserialized stateless scan results request did not match refresh variant."),
    }
}

#[test]
fn stateless_scan_results_contract_json_round_trip_responses() {
    let response = StatelessScanResultsResponse::Refresh(StatelessScanResultsRefreshResponse {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0x55556666777788889999AAAABBBBCCCC),
        },
        scan_results: Vec::new(),
    });

    let serialized_response = serde_json::to_string(&response).expect("Stateless scan results response should serialize.");
    let deserialized_response: StatelessScanResultsResponse =
        serde_json::from_str(&serialized_response).expect("Stateless scan results response should deserialize.");

    match deserialized_response {
        StatelessScanResultsResponse::Refresh(scan_results_refresh_response) => {
            assert_eq!(
                scan_results_refresh_response.session_handle.session_id,
                Uuid::from_u128(0x55556666777788889999AAAABBBBCCCC)
            );
            assert!(scan_results_refresh_response.scan_results.is_empty());
        }
        _ => panic!("Deserialized stateless scan results response did not match refresh variant."),
    }
}

#[test]
fn scan_results_response_typed_mapping_round_trip_delete() {
    let typed_delete_response = LegacyScanResultsDeleteResponse {};
    let engine_response = typed_delete_response.to_engine_response();
    let remapped_typed_delete_response = LegacyScanResultsDeleteResponse::from_engine_response(engine_response)
        .expect("Typed scan results delete response should round trip through privileged response.");

    let serialized_remapped_delete_response =
        serde_json::to_string(&remapped_typed_delete_response).expect("Remapped typed scan results delete response should serialize.");

    assert_eq!(serialized_remapped_delete_response, "{}");
}

#[test]
fn scan_results_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = LegacyScanResultsFreezeResponse {
        failed_freeze_toggle_scan_result_refs: Vec::new(),
    }
    .to_engine_response();
    let typed_result = LegacyScanResultsDeleteResponse::from_engine_response(mismatched_engine_response);

    assert!(typed_result.is_err(), "Unexpectedly mapped a freeze response into a delete response.");
}

#[test]
fn stateless_scan_results_list_request_contains_legacy_payload_fields() {
    let legacy_list_request = LegacyScanResultsListRequest { page_index: 8 };
    let stateless_list_request = StatelessScanResultsListRequest {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0x20202020404040406060606080808080),
        },
        page_index: legacy_list_request.page_index,
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_list_request).expect("Legacy scan results list request should serialize.");
    let mut stateless_serialized_request = serde_json::to_value(&stateless_list_request).expect("Stateless scan results list request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless scan results list request should be a JSON object.")
        .remove("session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_scan_results_delete_request_contains_legacy_payload_fields() {
    let legacy_delete_request = LegacyScanResultsDeleteRequest {
        scan_result_refs: vec![ScanResultRef::new(3), ScanResultRef::new(4)],
    };
    let stateless_delete_request = StatelessScanResultsDeleteRequest {
        session_handle: ScanSessionHandle {
            session_id: Uuid::from_u128(0x30303030505050507070707090909090),
        },
        scan_result_refs: legacy_delete_request.scan_result_refs.clone(),
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_delete_request).expect("Legacy scan results delete request should serialize.");
    let mut stateless_serialized_request = serde_json::to_value(&stateless_delete_request).expect("Stateless scan results delete request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless scan results delete request should be a JSON object.")
        .remove("session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}
