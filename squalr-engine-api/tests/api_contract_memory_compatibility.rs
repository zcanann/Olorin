use squalr_engine_api::api::commands::memory::read::memory_read_request::MemoryReadRequest as LegacyMemoryReadRequest;
use squalr_engine_api::api::commands::memory::read::memory_read_response::MemoryReadResponse as LegacyMemoryReadResponse;
use squalr_engine_api::api::commands::memory::write::memory_write_request::MemoryWriteRequest as LegacyMemoryWriteRequest;
use squalr_engine_api::api::commands::memory::write::memory_write_response::MemoryWriteResponse as LegacyMemoryWriteResponse;
use squalr_engine_api::api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::api::commands::stateless::memory::{
    MemoryReadRequest as StatelessMemoryReadRequest, MemoryReadResponse as StatelessMemoryReadResponse, MemoryWriteRequest as StatelessMemoryWriteRequest,
    MemoryWriteResponse as StatelessMemoryWriteResponse, StatelessMemoryRequest, StatelessMemoryResponse,
};
use squalr_engine_api::api::commands::stateless::process::ProcessSessionHandle;
use squalr_engine_api::api::types::structs::symbolic_struct_definition::SymbolicStructDefinition;
use squalr_engine_api::api::types::structs::valued_struct::ValuedStruct;
use uuid::Uuid;

#[test]
fn stateless_memory_contract_json_round_trip_requests() {
    let session_handle = ProcessSessionHandle {
        session_id: Uuid::from_u128(0xAABBCCDDEEFF00112233445566778899),
    };
    let read_request = StatelessMemoryRequest::Read(StatelessMemoryReadRequest {
        session_handle,
        address: 0x1A2B3C,
        module_name: "game.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
    });
    let write_request = StatelessMemoryRequest::Write(StatelessMemoryWriteRequest {
        session_handle,
        address: 0x1A2B40,
        module_name: "game.exe".to_string(),
        value: vec![0x10, 0x20, 0x30, 0x40],
    });

    let serialized_read_request = serde_json::to_string(&read_request).expect("Stateless memory read request should serialize.");
    let serialized_write_request = serde_json::to_string(&write_request).expect("Stateless memory write request should serialize.");

    let deserialized_read_request: StatelessMemoryRequest =
        serde_json::from_str(&serialized_read_request).expect("Stateless memory read request should deserialize.");
    let deserialized_write_request: StatelessMemoryRequest =
        serde_json::from_str(&serialized_write_request).expect("Stateless memory write request should deserialize.");

    match deserialized_read_request {
        StatelessMemoryRequest::Read(memory_read_request) => {
            assert_eq!(memory_read_request.session_handle.session_id, session_handle.session_id);
            assert_eq!(memory_read_request.address, 0x1A2B3C);
            assert_eq!(memory_read_request.module_name, "game.exe");
            assert_eq!(
                memory_read_request
                    .symbolic_struct_definition
                    .get_symbol_namespace(),
                ""
            );
        }
        _ => panic!("Deserialized stateless memory request did not match read variant."),
    }

    match deserialized_write_request {
        StatelessMemoryRequest::Write(memory_write_request) => {
            assert_eq!(memory_write_request.session_handle.session_id, session_handle.session_id);
            assert_eq!(memory_write_request.address, 0x1A2B40);
            assert_eq!(memory_write_request.module_name, "game.exe");
            assert_eq!(memory_write_request.value, vec![0x10, 0x20, 0x30, 0x40]);
        }
        _ => panic!("Deserialized stateless memory request did not match write variant."),
    }
}

#[test]
fn stateless_memory_contract_json_round_trip_responses() {
    let session_handle = ProcessSessionHandle {
        session_id: Uuid::from_u128(0x11223344556677889900AABBCCDDEEFF),
    };
    let read_response = StatelessMemoryResponse::Read(StatelessMemoryReadResponse {
        session_handle,
        valued_struct: ValuedStruct::default(),
        address: 0xCAFEBABE,
        success: true,
    });
    let write_response = StatelessMemoryResponse::Write(StatelessMemoryWriteResponse { session_handle, success: true });

    let serialized_read_response = serde_json::to_string(&read_response).expect("Stateless memory read response should serialize.");
    let serialized_write_response = serde_json::to_string(&write_response).expect("Stateless memory write response should serialize.");

    let deserialized_read_response: StatelessMemoryResponse =
        serde_json::from_str(&serialized_read_response).expect("Stateless memory read response should deserialize.");
    let deserialized_write_response: StatelessMemoryResponse =
        serde_json::from_str(&serialized_write_response).expect("Stateless memory write response should deserialize.");

    match deserialized_read_response {
        StatelessMemoryResponse::Read(memory_read_response) => {
            assert_eq!(memory_read_response.session_handle.session_id, session_handle.session_id);
            assert_eq!(memory_read_response.address, 0xCAFEBABE);
            assert!(memory_read_response.success);
        }
        _ => panic!("Deserialized stateless memory response did not match read variant."),
    }

    match deserialized_write_response {
        StatelessMemoryResponse::Write(memory_write_response) => {
            assert_eq!(memory_write_response.session_handle.session_id, session_handle.session_id);
            assert!(memory_write_response.success);
        }
        _ => panic!("Deserialized stateless memory response did not match write variant."),
    }
}

#[test]
fn memory_response_typed_mapping_round_trip_write() {
    let typed_write_response = LegacyMemoryWriteResponse { success: true };
    let engine_response = typed_write_response.to_engine_response();
    let remapped_typed_write_response =
        LegacyMemoryWriteResponse::from_engine_response(engine_response).expect("Typed memory write response should round trip through privileged response.");

    assert!(remapped_typed_write_response.success);
}

#[test]
fn memory_response_typed_mapping_rejects_incorrect_variant() {
    let mismatched_engine_response = LegacyMemoryReadResponse {
        valued_struct: ValuedStruct::default(),
        address: 0x1234,
        success: true,
    }
    .to_engine_response();
    let typed_result = LegacyMemoryWriteResponse::from_engine_response(mismatched_engine_response);

    assert!(
        typed_result.is_err(),
        "Unexpectedly mapped a memory read response into a memory write response."
    );
}

#[test]
fn stateless_and_legacy_memory_write_requests_preserve_payload_values() {
    let legacy_write_request = LegacyMemoryWriteRequest {
        address: 0x998877,
        module_name: "target.exe".to_string(),
        value: vec![0xAA, 0xBB, 0xCC],
    };
    let stateless_write_request = StatelessMemoryWriteRequest {
        session_handle: ProcessSessionHandle {
            session_id: Uuid::from_u128(0x887766554433221100FFEEDDCCBBAA99),
        },
        address: legacy_write_request.address,
        module_name: legacy_write_request.module_name.clone(),
        value: legacy_write_request.value.clone(),
    };

    assert_eq!(legacy_write_request.address, stateless_write_request.address);
    assert_eq!(legacy_write_request.module_name, stateless_write_request.module_name);
    assert_eq!(legacy_write_request.value, stateless_write_request.value);
}

#[test]
fn stateless_and_legacy_memory_read_requests_preserve_payload_values() {
    let legacy_read_request = LegacyMemoryReadRequest {
        address: 0x445566,
        module_name: "target.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
    };
    let stateless_read_request = StatelessMemoryReadRequest {
        session_handle: ProcessSessionHandle {
            session_id: Uuid::from_u128(0x12341234123412341234123412341234),
        },
        address: legacy_read_request.address,
        module_name: legacy_read_request.module_name.clone(),
        symbolic_struct_definition: legacy_read_request.symbolic_struct_definition.clone(),
    };

    assert_eq!(legacy_read_request.address, stateless_read_request.address);
    assert_eq!(legacy_read_request.module_name, stateless_read_request.module_name);
    assert_eq!(
        legacy_read_request
            .symbolic_struct_definition
            .get_symbol_namespace(),
        stateless_read_request
            .symbolic_struct_definition
            .get_symbol_namespace()
    );
}
