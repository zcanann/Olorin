use squalr_engine_api::api::commands::project::create::project_create_request::ProjectCreateRequest as LegacyProjectCreateRequest;
use squalr_engine_api::api::commands::project::open::project_open_request::ProjectOpenRequest as LegacyProjectOpenRequest;
use squalr_engine_api::api::commands::project::open::project_open_response::ProjectOpenResponse as LegacyProjectOpenResponse;
use squalr_engine_api::api::commands::project::save::project_save_response::ProjectSaveResponse as LegacyProjectSaveResponse;
use squalr_engine_api::api::commands::stateless::project::{
    ProjectCloseRequest as StatelessProjectCloseRequest, ProjectListRequest as StatelessProjectListRequest, ProjectOpenRequest as StatelessProjectOpenRequest,
    ProjectSaveRequest as StatelessProjectSaveRequest, ProjectSessionHandle, StatelessProjectRequest, StatelessProjectResponse,
};
use squalr_engine_api::api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn stateless_project_contract_json_round_trip_requests() {
    let close_request = StatelessProjectRequest::Close(StatelessProjectCloseRequest {
        session_handle: ProjectSessionHandle {
            session_id: Uuid::from_u128(0xA1A2A3A4A5A6A7A8B1B2B3B4B5B6B7B8),
        },
    });
    let open_request = StatelessProjectRequest::Open(StatelessProjectOpenRequest {
        open_file_browser: false,
        project_directory_path: Some(PathBuf::from("C:\\Projects\\Samples")),
        project_name: Some("example_project".to_string()),
    });
    let list_request = StatelessProjectRequest::List(StatelessProjectListRequest {});

    let serialized_close_request = serde_json::to_string(&close_request).expect("Stateless project close request should serialize.");
    let serialized_open_request = serde_json::to_string(&open_request).expect("Stateless project open request should serialize.");
    let serialized_list_request = serde_json::to_string(&list_request).expect("Stateless project list request should serialize.");

    let deserialized_close_request: StatelessProjectRequest =
        serde_json::from_str(&serialized_close_request).expect("Stateless project close request should deserialize.");
    let deserialized_open_request: StatelessProjectRequest =
        serde_json::from_str(&serialized_open_request).expect("Stateless project open request should deserialize.");
    let deserialized_list_request: StatelessProjectRequest =
        serde_json::from_str(&serialized_list_request).expect("Stateless project list request should deserialize.");

    match deserialized_close_request {
        StatelessProjectRequest::Close(project_close_request) => {
            assert_eq!(
                project_close_request.session_handle.session_id,
                Uuid::from_u128(0xA1A2A3A4A5A6A7A8B1B2B3B4B5B6B7B8)
            );
        }
        _ => panic!("Deserialized stateless project request did not match close variant."),
    }

    match deserialized_open_request {
        StatelessProjectRequest::Open(project_open_request) => {
            assert_eq!(project_open_request.project_name.as_deref(), Some("example_project"));
            assert_eq!(project_open_request.project_directory_path, Some(PathBuf::from("C:\\Projects\\Samples")));
        }
        _ => panic!("Deserialized stateless project request did not match open variant."),
    }

    match deserialized_list_request {
        StatelessProjectRequest::List(_) => {}
        _ => panic!("Deserialized stateless project request did not match list variant."),
    }
}

#[test]
fn stateless_project_contract_json_round_trip_responses() {
    let open_response = StatelessProjectResponse::Open(squalr_engine_api::api::commands::stateless::project::ProjectOpenResponse {
        success: true,
        session_handle: Some(ProjectSessionHandle {
            session_id: Uuid::from_u128(0x00112233445566778899AABBCCDDEEFF),
        }),
    });
    let save_response = StatelessProjectResponse::Save(squalr_engine_api::api::commands::stateless::project::ProjectSaveResponse {
        success: true,
        session_handle: ProjectSessionHandle {
            session_id: Uuid::from_u128(0x11111111222222223333333344444444),
        },
    });

    let serialized_open_response = serde_json::to_string(&open_response).expect("Stateless project open response should serialize.");
    let serialized_save_response = serde_json::to_string(&save_response).expect("Stateless project save response should serialize.");

    let deserialized_open_response: StatelessProjectResponse =
        serde_json::from_str(&serialized_open_response).expect("Stateless project open response should deserialize.");
    let deserialized_save_response: StatelessProjectResponse =
        serde_json::from_str(&serialized_save_response).expect("Stateless project save response should deserialize.");

    match deserialized_open_response {
        StatelessProjectResponse::Open(project_open_response) => {
            assert!(project_open_response.success);
            assert_eq!(
                project_open_response
                    .session_handle
                    .map(|handle| handle.session_id),
                Some(Uuid::from_u128(0x00112233445566778899AABBCCDDEEFF))
            );
        }
        _ => panic!("Deserialized stateless project response did not match open variant."),
    }

    match deserialized_save_response {
        StatelessProjectResponse::Save(project_save_response) => {
            assert!(project_save_response.success);
            assert_eq!(
                project_save_response.session_handle.session_id,
                Uuid::from_u128(0x11111111222222223333333344444444)
            );
        }
        _ => panic!("Deserialized stateless project response did not match save variant."),
    }
}

#[test]
fn project_response_typed_mapping_round_trip_save() {
    let typed_save_response = LegacyProjectSaveResponse { success: true };
    let engine_response = typed_save_response.to_engine_response();
    let remapped_typed_save_response =
        LegacyProjectSaveResponse::from_engine_response(engine_response).expect("Typed project save response should round trip through unprivileged response.");

    assert!(remapped_typed_save_response.success);
}

#[test]
fn project_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = LegacyProjectOpenResponse { success: true }.to_engine_response();
    let typed_result = LegacyProjectSaveResponse::from_engine_response(mismatched_engine_response);

    assert!(typed_result.is_err(), "Unexpectedly mapped a project open response into a save response.");
}

#[test]
fn stateless_and_legacy_project_open_requests_are_shape_compatible() {
    let legacy_open_request = LegacyProjectOpenRequest {
        open_file_browser: true,
        project_directory_path: Some(PathBuf::from("C:\\Projects\\Compatibility")),
        project_name: Some("compat_project".to_string()),
    };
    let stateless_open_request = StatelessProjectOpenRequest {
        open_file_browser: legacy_open_request.open_file_browser,
        project_directory_path: legacy_open_request.project_directory_path.clone(),
        project_name: legacy_open_request.project_name.clone(),
    };
    let legacy_serialized_request = serde_json::to_string(&legacy_open_request).expect("Legacy project open request should serialize.");
    let stateless_serialized_request = serde_json::to_string(&stateless_open_request).expect("Stateless project open request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_and_legacy_project_create_requests_are_shape_compatible() {
    let legacy_create_request = LegacyProjectCreateRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects\\Compatibility")),
        project_name: Some("created_project".to_string()),
    };
    let stateless_create_request = squalr_engine_api::api::commands::stateless::project::ProjectCreateRequest {
        project_directory_path: legacy_create_request.project_directory_path.clone(),
        project_name: legacy_create_request.project_name.clone(),
    };
    let legacy_serialized_request = serde_json::to_string(&legacy_create_request).expect("Legacy project create request should serialize.");
    let stateless_serialized_request = serde_json::to_string(&stateless_create_request).expect("Stateless project create request should serialize.");

    assert_eq!(legacy_serialized_request, stateless_serialized_request);
}

#[test]
fn stateless_project_save_request_round_trip_contains_session_context() {
    let save_request = StatelessProjectSaveRequest {
        session_handle: ProjectSessionHandle {
            session_id: Uuid::from_u128(0xABCDEFABCDEFABCDEFABCDEFABCDEFAB),
        },
    };
    let serialized_save_request = serde_json::to_string(&save_request).expect("Stateless project save request should serialize.");
    let deserialized_save_request: StatelessProjectSaveRequest =
        serde_json::from_str(&serialized_save_request).expect("Stateless project save request should deserialize.");

    assert_eq!(
        deserialized_save_request.session_handle.session_id,
        Uuid::from_u128(0xABCDEFABCDEFABCDEFABCDEFABCDEFAB)
    );
}
