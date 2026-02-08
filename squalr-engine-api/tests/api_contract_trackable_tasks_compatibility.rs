use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::stateless::trackable_tasks::{
    StatelessTrackableTasksRequest, StatelessTrackableTasksResponse, TrackableTasksCancelRequest as StatelessTrackableTasksCancelRequest,
    TrackableTasksCancelResponse as StatelessTrackableTasksCancelResponse, TrackableTasksListRequest as StatelessTrackableTasksListRequest,
    TrackableTasksListResponse as StatelessTrackableTasksListResponse,
};
use squalr_engine_api::api::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest;
use squalr_engine_api::api::commands::trackable_tasks::cancel::trackable_tasks_cancel_response::TrackableTasksCancelResponse;
use squalr_engine_api::api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use squalr_engine_api::api::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;

#[test]
fn stateless_trackable_tasks_contract_json_round_trip_requests() {
    let list_request = StatelessTrackableTasksRequest::List(StatelessTrackableTasksListRequest {});
    let cancel_request = StatelessTrackableTasksRequest::Cancel(StatelessTrackableTasksCancelRequest {
        task_id: "task-123".to_string(),
    });

    let serialized_list_request = serde_json::to_string(&list_request).expect("Stateless trackable tasks list request should serialize.");
    let serialized_cancel_request = serde_json::to_string(&cancel_request).expect("Stateless trackable tasks cancel request should serialize.");

    let deserialized_list_request: StatelessTrackableTasksRequest =
        serde_json::from_str(&serialized_list_request).expect("Stateless trackable tasks list request should deserialize.");
    let deserialized_cancel_request: StatelessTrackableTasksRequest =
        serde_json::from_str(&serialized_cancel_request).expect("Stateless trackable tasks cancel request should deserialize.");

    match deserialized_list_request {
        StatelessTrackableTasksRequest::List(_) => {}
        _ => panic!("Deserialized stateless trackable tasks request did not match list variant."),
    }

    match deserialized_cancel_request {
        StatelessTrackableTasksRequest::Cancel(trackable_tasks_cancel_request) => {
            assert_eq!(trackable_tasks_cancel_request.task_id, "task-123");
        }
        _ => panic!("Deserialized stateless trackable tasks request did not match cancel variant."),
    }
}

#[test]
fn stateless_trackable_tasks_contract_json_round_trip_responses() {
    let list_response = StatelessTrackableTasksResponse::List(StatelessTrackableTasksListResponse {});
    let cancel_response = StatelessTrackableTasksResponse::Cancel(StatelessTrackableTasksCancelResponse {});

    let serialized_list_response = serde_json::to_string(&list_response).expect("Stateless trackable tasks list response should serialize.");
    let serialized_cancel_response = serde_json::to_string(&cancel_response).expect("Stateless trackable tasks cancel response should serialize.");

    let deserialized_list_response: StatelessTrackableTasksResponse =
        serde_json::from_str(&serialized_list_response).expect("Stateless trackable tasks list response should deserialize.");
    let deserialized_cancel_response: StatelessTrackableTasksResponse =
        serde_json::from_str(&serialized_cancel_response).expect("Stateless trackable tasks cancel response should deserialize.");

    match deserialized_list_response {
        StatelessTrackableTasksResponse::List(_) => {}
        _ => panic!("Deserialized stateless trackable tasks response did not match list variant."),
    }

    match deserialized_cancel_response {
        StatelessTrackableTasksResponse::Cancel(_) => {}
        _ => panic!("Deserialized stateless trackable tasks response did not match cancel variant."),
    }
}

#[test]
fn trackable_tasks_response_typed_mapping_round_trip_cancel() {
    let typed_cancel_response = TrackableTasksCancelResponse {};
    let engine_response = typed_cancel_response.to_engine_response();
    let remapped_typed_cancel_response = TrackableTasksCancelResponse::from_engine_response(engine_response)
        .expect("Typed trackable tasks cancel response should round trip through privileged response.");

    let serialized_remapped_cancel_response =
        serde_json::to_string(&remapped_typed_cancel_response).expect("Remapped typed trackable tasks cancel response should serialize.");

    assert_eq!(serialized_remapped_cancel_response, "{}");
}

#[test]
fn trackable_tasks_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = TrackableTasksListResponse {}.to_engine_response();
    let typed_result = TrackableTasksCancelResponse::from_engine_response(mismatched_engine_response);

    assert!(typed_result.is_err(), "Unexpectedly mapped a list response into a cancel response.");
}

#[test]
fn stateless_and_legacy_trackable_task_cancel_requests_are_shape_compatible() {
    let legacy_cancel_request = TrackableTasksCancelRequest {
        task_id: "task-compatibility".to_string(),
    };
    let stateless_cancel_request = StatelessTrackableTasksCancelRequest {
        task_id: legacy_cancel_request.task_id.clone(),
    };
    let legacy_serialized_request = serde_json::to_string(&legacy_cancel_request).expect("Legacy trackable tasks cancel request should serialize.");
    let stateless_serialized_request = serde_json::to_string(&stateless_cancel_request).expect("Stateless trackable tasks cancel request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_and_legacy_trackable_task_list_requests_are_shape_compatible() {
    let legacy_list_request = TrackableTasksListRequest {};
    let stateless_list_request = StatelessTrackableTasksListRequest {};
    let legacy_serialized_request = serde_json::to_string(&legacy_list_request).expect("Legacy trackable tasks list request should serialize.");
    let stateless_serialized_request = serde_json::to_string(&stateless_list_request).expect("Stateless trackable tasks list request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}
