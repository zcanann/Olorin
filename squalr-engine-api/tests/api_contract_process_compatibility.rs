use squalr_engine_api::api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::process::close::process_close_response::ProcessCloseResponse;
use squalr_engine_api::api::commands::process::list::process_list_response::ProcessListResponse;
use squalr_engine_api::api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::api::commands::process::open::process_open_response::ProcessOpenResponse;
use squalr_engine_api::api::commands::process::process_command::ProcessCommand;
use squalr_engine_api::api::commands::stateless::process::{
    ProcessCloseRequest as StatelessProcessCloseRequest, ProcessListRequest as StatelessProcessListRequest, ProcessOpenRequest as StatelessProcessOpenRequest,
    ProcessSessionHandle, StatelessProcessRequest,
};
use squalr_engine_api::api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::api::events::process::changed::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::api::types::memory::bitness::Bitness;
use squalr_engine_api::api::types::processes::opened_process_info::OpenedProcessInfo;
use uuid::Uuid;

#[test]
fn privileged_process_command_json_round_trip_open_request() {
    let open_request = ProcessOpenRequest {
        process_id: Some(4242),
        search_name: Some("eldenring.exe".to_string()),
        match_case: false,
    };
    let command = PrivilegedCommand::Process(ProcessCommand::Open {
        process_open_request: open_request,
    });

    let serialized_command = serde_json::to_string(&command).expect("Privileged process open command should serialize.");
    let deserialized_command: PrivilegedCommand = serde_json::from_str(&serialized_command).expect("Privileged process open command should deserialize.");

    match deserialized_command {
        PrivilegedCommand::Process(ProcessCommand::Open { process_open_request }) => {
            assert_eq!(process_open_request.process_id, Some(4242));
            assert_eq!(process_open_request.search_name.as_deref(), Some("eldenring.exe"));
            assert!(!process_open_request.match_case);
        }
        _ => panic!("Deserialized command variant did not match process open."),
    }
}

#[test]
fn process_response_typed_mapping_round_trip_open() {
    let opened_process_info = OpenedProcessInfo::new(1001, "game.exe".to_string(), 0xDEADBEEF, Bitness::Bit64, None);
    let typed_response = ProcessOpenResponse {
        opened_process_info: Some(opened_process_info),
    };

    let engine_response = typed_response.to_engine_response();
    let remapped_typed_response =
        ProcessOpenResponse::from_engine_response(engine_response).expect("Typed process open response should round trip through privileged response.");

    assert_eq!(
        remapped_typed_response
            .opened_process_info
            .as_ref()
            .map(|process_info| process_info.get_process_id_raw()),
        Some(1001)
    );
}

#[test]
fn process_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = ProcessListResponse { processes: Vec::new() }.to_engine_response();

    let typed_result = ProcessCloseResponse::from_engine_response(mismatched_engine_response);

    assert!(typed_result.is_err(), "Unexpectedly mapped a list response into a close response.");
}

#[test]
fn process_event_json_round_trip() {
    let process_changed_event = ProcessChangedEvent {
        process_info: Some(OpenedProcessInfo::new(2002, "target.exe".to_string(), 0xABCDEF, Bitness::Bit32, None)),
    };
    let engine_event = process_changed_event.to_engine_event();

    let serialized_event = serde_json::to_string(&engine_event).expect("Engine event should serialize.");
    let deserialized_event: EngineEvent = serde_json::from_str(&serialized_event).expect("Engine event should deserialize.");

    match deserialized_event {
        EngineEvent::Process(_) => {}
        _ => panic!("Deserialized event variant did not match process event."),
    }
}

#[test]
fn stateless_process_contract_json_round_trip_close_request() {
    let close_request = StatelessProcessCloseRequest {
        session_handle: ProcessSessionHandle {
            session_id: Uuid::from_u128(0x1234567890ABCDEF1234567890ABCDEF),
        },
    };
    let request = StatelessProcessRequest::Close(close_request);

    let serialized_request = serde_json::to_string(&request).expect("Stateless process close request should serialize.");
    let deserialized_request: StatelessProcessRequest = serde_json::from_str(&serialized_request).expect("Stateless process close request should deserialize.");

    match deserialized_request {
        StatelessProcessRequest::Close(process_close_request) => {
            assert_eq!(
                process_close_request.session_handle.session_id,
                Uuid::from_u128(0x1234567890ABCDEF1234567890ABCDEF)
            );
        }
        _ => panic!("Deserialized stateless process request did not match close variant."),
    }
}

#[test]
fn stateless_process_contract_json_round_trip_open_and_list_requests() {
    let open_request = StatelessProcessRequest::Open(StatelessProcessOpenRequest {
        process_id: None,
        search_name: Some("squalr.exe".to_string()),
        match_case: true,
    });
    let list_request = StatelessProcessRequest::List(StatelessProcessListRequest {
        require_windowed: true,
        search_name: Some("squalr".to_string()),
        match_case: false,
        limit: Some(25),
        fetch_icons: true,
    });

    let serialized_open_request = serde_json::to_string(&open_request).expect("Stateless open request should serialize.");
    let serialized_list_request = serde_json::to_string(&list_request).expect("Stateless list request should serialize.");

    let deserialized_open_request: StatelessProcessRequest =
        serde_json::from_str(&serialized_open_request).expect("Stateless open request should deserialize.");
    let deserialized_list_request: StatelessProcessRequest =
        serde_json::from_str(&serialized_list_request).expect("Stateless list request should deserialize.");

    match deserialized_open_request {
        StatelessProcessRequest::Open(process_open_request) => {
            assert!(process_open_request.process_id.is_none());
            assert_eq!(process_open_request.search_name.as_deref(), Some("squalr.exe"));
            assert!(process_open_request.match_case);
        }
        _ => panic!("Deserialized stateless process request did not match open variant."),
    }

    match deserialized_list_request {
        StatelessProcessRequest::List(process_list_request) => {
            assert!(process_list_request.require_windowed);
            assert_eq!(process_list_request.search_name.as_deref(), Some("squalr"));
            assert!(!process_list_request.match_case);
            assert_eq!(process_list_request.limit, Some(25));
            assert!(process_list_request.fetch_icons);
        }
        _ => panic!("Deserialized stateless process request did not match list variant."),
    }
}
