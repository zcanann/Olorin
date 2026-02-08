use squalr_engine_api::api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest as LegacyProjectItemsActivateRequest;
use squalr_engine_api::api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse as LegacyProjectItemsActivateResponse;
use squalr_engine_api::api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest as LegacyProjectItemsListRequest;
use squalr_engine_api::api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse as LegacyProjectItemsListResponse;
use squalr_engine_api::api::commands::stateless::project::ProjectSessionHandle;
use squalr_engine_api::api::commands::stateless::project_items::{
    ProjectItemsActivateRequest as StatelessProjectItemsActivateRequest, ProjectItemsListRequest as StatelessProjectItemsListRequest,
    StatelessProjectItemsRequest, StatelessProjectItemsResponse,
};
use squalr_engine_api::api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use uuid::Uuid;

#[test]
fn stateless_project_items_contract_json_round_trip_requests() {
    let session_handle = ProjectSessionHandle {
        session_id: Uuid::from_u128(0x81828384858687889192939495969798),
    };
    let activate_request = StatelessProjectItemsRequest::Activate(StatelessProjectItemsActivateRequest {
        session_handle,
        project_item_paths: vec!["folder/value_a".to_string(), "folder/value_b".to_string()],
        is_activated: true,
    });
    let list_request = StatelessProjectItemsRequest::List(StatelessProjectItemsListRequest { session_handle });

    let serialized_activate_request = serde_json::to_string(&activate_request).expect("Stateless project-items activate request should serialize.");
    let serialized_list_request = serde_json::to_string(&list_request).expect("Stateless project-items list request should serialize.");

    let deserialized_activate_request: StatelessProjectItemsRequest =
        serde_json::from_str(&serialized_activate_request).expect("Stateless project-items activate request should deserialize.");
    let deserialized_list_request: StatelessProjectItemsRequest =
        serde_json::from_str(&serialized_list_request).expect("Stateless project-items list request should deserialize.");

    match deserialized_activate_request {
        StatelessProjectItemsRequest::Activate(project_items_activate_request) => {
            assert_eq!(
                project_items_activate_request.session_handle.session_id,
                Uuid::from_u128(0x81828384858687889192939495969798)
            );
            assert_eq!(project_items_activate_request.project_item_paths.len(), 2);
            assert!(project_items_activate_request.is_activated);
        }
        _ => panic!("Deserialized stateless project-items request did not match activate variant."),
    }

    match deserialized_list_request {
        StatelessProjectItemsRequest::List(project_items_list_request) => {
            assert_eq!(
                project_items_list_request.session_handle.session_id,
                Uuid::from_u128(0x81828384858687889192939495969798)
            );
        }
        _ => panic!("Deserialized stateless project-items request did not match list variant."),
    }
}

#[test]
fn stateless_project_items_contract_json_round_trip_responses() {
    let session_handle = ProjectSessionHandle {
        session_id: Uuid::from_u128(0x71727374757677778182838485868788),
    };
    let activate_response =
        StatelessProjectItemsResponse::Activate(squalr_engine_api::api::commands::stateless::project_items::ProjectItemsActivateResponse { session_handle });
    let list_response = StatelessProjectItemsResponse::List(squalr_engine_api::api::commands::stateless::project_items::ProjectItemsListResponse {
        session_handle,
        opened_project_info: None,
        opened_project_root: None,
    });

    let serialized_activate_response = serde_json::to_string(&activate_response).expect("Stateless project-items activate response should serialize.");
    let serialized_list_response = serde_json::to_string(&list_response).expect("Stateless project-items list response should serialize.");

    let deserialized_activate_response: StatelessProjectItemsResponse =
        serde_json::from_str(&serialized_activate_response).expect("Stateless project-items activate response should deserialize.");
    let deserialized_list_response: StatelessProjectItemsResponse =
        serde_json::from_str(&serialized_list_response).expect("Stateless project-items list response should deserialize.");

    match deserialized_activate_response {
        StatelessProjectItemsResponse::Activate(project_items_activate_response) => {
            assert_eq!(project_items_activate_response.session_handle.session_id, session_handle.session_id);
        }
        _ => panic!("Deserialized stateless project-items response did not match activate variant."),
    }

    match deserialized_list_response {
        StatelessProjectItemsResponse::List(project_items_list_response) => {
            assert_eq!(project_items_list_response.session_handle.session_id, session_handle.session_id);
            assert!(project_items_list_response.opened_project_info.is_none());
            assert!(project_items_list_response.opened_project_root.is_none());
        }
        _ => panic!("Deserialized stateless project-items response did not match list variant."),
    }
}

#[test]
fn project_items_response_typed_mapping_round_trip_list() {
    let typed_list_response = LegacyProjectItemsListResponse {
        opened_project_info: None,
        opened_project_root: None,
    };
    let engine_response = typed_list_response.to_engine_response();
    let remapped_typed_list_response = LegacyProjectItemsListResponse::from_engine_response(engine_response)
        .expect("Typed project-items list response should round trip through unprivileged response.");

    assert!(remapped_typed_list_response.opened_project_info.is_none());
    assert!(remapped_typed_list_response.opened_project_root.is_none());
}

#[test]
fn project_items_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = LegacyProjectItemsActivateResponse {}.to_engine_response();
    let typed_result = LegacyProjectItemsListResponse::from_engine_response(mismatched_engine_response);

    assert!(
        typed_result.is_err(),
        "Unexpectedly mapped a project-items activate response into a list response."
    );
}

#[test]
fn stateless_project_items_activate_request_contains_legacy_payload_fields() {
    let legacy_activate_request = LegacyProjectItemsActivateRequest {
        project_item_paths: vec!["alpha".to_string(), "beta".to_string()],
        is_activated: false,
    };
    let stateless_activate_request = StatelessProjectItemsActivateRequest {
        session_handle: ProjectSessionHandle {
            session_id: Uuid::from_u128(0x0102030405060708090A0B0C0D0E0F00),
        },
        project_item_paths: legacy_activate_request.project_item_paths.clone(),
        is_activated: legacy_activate_request.is_activated,
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_activate_request).expect("Legacy project-items activate request should serialize.");
    let mut stateless_serialized_request =
        serde_json::to_value(&stateless_activate_request).expect("Stateless project-items activate request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless project-items activate request should be a JSON object.")
        .remove("session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_project_items_list_request_contains_legacy_payload_fields() {
    let legacy_list_request = LegacyProjectItemsListRequest {};
    let stateless_list_request = StatelessProjectItemsListRequest {
        session_handle: ProjectSessionHandle {
            session_id: Uuid::from_u128(0x00112233445566778899AABBCCDDEEFF),
        },
    };

    let legacy_serialized_request = serde_json::to_value(&legacy_list_request).expect("Legacy project-items list request should serialize.");
    let mut stateless_serialized_request = serde_json::to_value(&stateless_list_request).expect("Stateless project-items list request should serialize.");
    stateless_serialized_request
        .as_object_mut()
        .expect("Serialized stateless project-items list request should be a JSON object.")
        .remove("session_handle");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}
