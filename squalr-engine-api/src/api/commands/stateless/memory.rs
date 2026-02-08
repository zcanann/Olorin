use crate::api::commands::stateless::process::ProcessSessionHandle;
use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use crate::structures::structs::valued_struct::ValuedStruct;
use serde::{Deserialize, Serialize};

/// Stateless memory command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessMemoryRequest {
    Read(MemoryReadRequest),
    Write(MemoryWriteRequest),
}

/// Stateless memory command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessMemoryResponse {
    Read(MemoryReadResponse),
    Write(MemoryWriteResponse),
}

/// Stateless memory read request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryReadRequest {
    pub session_handle: ProcessSessionHandle,
    pub address: u64,
    pub module_name: String,
    pub symbolic_struct_definition: SymbolicStructDefinition,
}

/// Stateless memory read response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    pub session_handle: ProcessSessionHandle,
    pub valued_struct: ValuedStruct,
    pub address: u64,
    pub success: bool,
}

/// Stateless memory write request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    pub session_handle: ProcessSessionHandle,
    pub address: u64,
    pub module_name: String,
    pub value: Vec<u8>,
}

/// Stateless memory write response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryWriteResponse {
    pub session_handle: ProcessSessionHandle,
    pub success: bool,
}
